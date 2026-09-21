//! Rust source parsing and item inventory.
//!
//! Parsing uses [`syn`] and never executes code. Source spans are captured with
//! `proc-macro2` span locations so findings can cite exact lines and columns.
//! A file that fails to parse is recorded as a diagnostic rather than aborting
//! the scan.

use std::path::{Path, PathBuf};

use quote::ToTokens;
use syn::spanned::Spanned;

use crate::error::ScanError;
use crate::model::{Diagnostic, Project, SorobanInfo};
use crate::soroban::{self, ContractAttrStrength};

/// Maximum source file size that will be read (8 MiB).
pub const MAX_SOURCE_BYTES: u64 = 8 * 1024 * 1024;

/// Kinds of top-level items the inventory recognizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    /// `fn`
    Function,
    /// `impl`
    Impl,
    /// `trait`
    Trait,
    /// `struct`
    Struct,
    /// `enum`
    Enum,
    /// `union`
    Union,
    /// `const`
    Const,
    /// `static`
    Static,
    /// `mod`
    Module,
    /// A macro invocation item.
    Macro,
    /// `type`
    TypeAlias,
    /// `use`
    Use,
    /// `extern` block.
    ForeignMod,
    /// Anything else.
    Other,
}

impl ItemKind {
    /// Stable lowercase name.
    pub fn as_str(self) -> &'static str {
        match self {
            ItemKind::Function => "function",
            ItemKind::Impl => "impl",
            ItemKind::Trait => "trait",
            ItemKind::Struct => "struct",
            ItemKind::Enum => "enum",
            ItemKind::Union => "union",
            ItemKind::Const => "const",
            ItemKind::Static => "static",
            ItemKind::Module => "module",
            ItemKind::Macro => "macro",
            ItemKind::TypeAlias => "type_alias",
            ItemKind::Use => "use",
            ItemKind::ForeignMod => "foreign_mod",
            ItemKind::Other => "other",
        }
    }
}

/// A function parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamSummary {
    /// Parameter binding name (or rendered pattern).
    pub name: String,
    /// Rendered parameter type.
    pub ty: String,
}

/// Summary of a function definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSummary {
    /// Unqualified function name.
    pub name: String,
    /// Name qualified by any enclosing module and impl/trait.
    pub qualified_name: String,
    /// 1-based line of the function.
    pub line: usize,
    /// 1-based column of the function.
    pub column: usize,
    /// 1-based end line of the function.
    pub end_line: usize,
    /// Whether the function is declared `pub`.
    pub is_public: bool,
    /// Whether the function is `async`.
    pub is_async: bool,
    /// Attribute path strings on the function.
    pub attributes: Vec<String>,
    /// Parameters.
    pub params: Vec<ParamSummary>,
    /// Rendered return type, if not `()`.
    pub return_type: Option<String>,
    /// Enclosing impl/trait name, if any.
    pub in_impl: Option<String>,
    /// Attribute path strings on the enclosing impl block.
    pub impl_attributes: Vec<String>,
    /// True when the function is an exported contract entry point
    /// (`pub fn` inside a `#[contractimpl]` block).
    pub is_contract_entry_point: bool,
}

/// Summary of a non-function item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemSummary {
    /// Item kind.
    pub kind: ItemKind,
    /// Item name (may be a rendered type for impls).
    pub name: String,
    /// 1-based line.
    pub line: usize,
    /// 1-based column.
    pub column: usize,
    /// 1-based end line.
    pub end_line: usize,
    /// Whether the item is `pub`.
    pub is_public: bool,
    /// Attribute path strings.
    pub attributes: Vec<String>,
}

/// A parsed Rust source file.
pub struct ParsedSource {
    /// File path.
    pub path: PathBuf,
    /// Logical module path (best effort).
    pub module_path: String,
    /// Parsed syntax tree, if parsing succeeded.
    pub syntax: Option<syn::File>,
    /// Parse error message, if parsing failed.
    pub parse_error: Option<String>,
    /// Non-function items.
    pub items: Vec<ItemSummary>,
    /// Function definitions.
    pub functions: Vec<FunctionSummary>,
    /// Locations of contract definitions, formatted as `path:line`.
    pub contract_definitions: Vec<String>,
}

impl ParsedSource {
    /// Returns true if the file parsed successfully.
    pub fn is_parsed(&self) -> bool {
        self.syntax.is_some()
    }

    /// Returns functions that are Soroban contract entry points.
    pub fn contract_entry_points(&self) -> impl Iterator<Item = &FunctionSummary> {
        self.functions.iter().filter(|f| f.is_contract_entry_point)
    }
}

/// Extracts the attribute path (for example `contractimpl`) as a string.
pub fn attribute_path(attr: &syn::Attribute) -> String {
    attr.path()
        .segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn attribute_paths(attrs: &[syn::Attribute]) -> Vec<String> {
    attrs.iter().map(attribute_path).collect()
}

fn vis_is_public(vis: &syn::Visibility) -> bool {
    matches!(vis, syn::Visibility::Public(_))
}

fn span_start<T: Spanned>(node: &T) -> (usize, usize) {
    let start = node.span().start();
    (start.line, start.column + 1)
}

fn span_end_line<T: Spanned>(node: &T) -> usize {
    node.span().end().line
}

fn pat_name(pat: &syn::Pat) -> String {
    match pat {
        syn::Pat::Ident(pi) => pi.ident.to_string(),
        other => other.to_token_stream().to_string(),
    }
}

fn params_of(sig: &syn::Signature) -> Vec<ParamSummary> {
    sig.inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Receiver(_) => None,
            syn::FnArg::Typed(pt) => Some(ParamSummary {
                name: pat_name(&pt.pat),
                ty: pt.ty.to_token_stream().to_string(),
            }),
        })
        .collect()
}

fn return_of(sig: &syn::Signature) -> Option<String> {
    match &sig.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => Some(ty.to_token_stream().to_string()),
    }
}

fn record_contract_attrs(
    attrs: &[syn::Attribute],
    path: &Path,
    line: usize,
    out: &mut Vec<String>,
) {
    for attr in attrs {
        let attr_path = attribute_path(attr);
        if let Some(ContractAttrStrength::Definition) =
            soroban::contract_attribute_strength(&attr_path)
        {
            let location = format!("{}:{}", path.display(), line);
            if !out.contains(&location) {
                out.push(location);
            }
        }
    }
}

struct ImplContext {
    name: String,
    attributes: Vec<String>,
}

fn make_function(
    sig: &syn::Signature,
    attrs: &[syn::Attribute],
    vis_public: bool,
    prefix: &str,
    impl_ctx: Option<&ImplContext>,
) -> FunctionSummary {
    let name = sig.ident.to_string();
    let qualified_name = match impl_ctx {
        Some(ctx) if prefix.is_empty() => format!("{}::{}", ctx.name, name),
        Some(ctx) => format!("{}::{}::{}", prefix, ctx.name, name),
        None if prefix.is_empty() => name.clone(),
        None => format!("{prefix}::{name}"),
    };
    let impl_attributes = impl_ctx.map(|c| c.attributes.clone()).unwrap_or_default();
    let is_contract_entry_point = vis_public
        && impl_attributes.iter().any(|a| {
            matches!(
                soroban::contract_attribute_strength(a),
                Some(ContractAttrStrength::Definition)
            )
        });
    let (line, column) = span_start(sig);
    FunctionSummary {
        name,
        qualified_name,
        line,
        column,
        end_line: span_end_line(sig),
        is_public: vis_public,
        is_async: sig.asyncness.is_some(),
        attributes: attribute_paths(attrs),
        params: params_of(sig),
        return_type: return_of(sig),
        in_impl: impl_ctx.map(|c| c.name.clone()),
        impl_attributes,
        is_contract_entry_point,
    }
}

#[allow(clippy::too_many_arguments)]
fn walk_items(
    items: &[syn::Item],
    prefix: &str,
    impl_ctx: Option<&ImplContext>,
    file: &Path,
    out_items: &mut Vec<ItemSummary>,
    out_functions: &mut Vec<FunctionSummary>,
    out_contract: &mut Vec<String>,
) {
    for item in items {
        let attrs = item_attrs(item);
        let (line, column) = span_start(item);
        let end_line = span_end_line(item);
        record_contract_attrs(&attrs, file, line, out_contract);

        match item {
            syn::Item::Fn(f) => {
                out_items.push(item_summary(
                    ItemKind::Function,
                    f.sig.ident.to_string(),
                    &attrs,
                    vis_is_public(&f.vis),
                    (line, column, end_line),
                ));
                out_functions.push(make_function(
                    &f.sig,
                    &attrs,
                    vis_is_public(&f.vis),
                    prefix,
                    impl_ctx,
                ));
            }
            syn::Item::Impl(imp) => {
                let name = imp.self_ty.to_token_stream().to_string();
                out_items.push(item_summary(
                    ItemKind::Impl,
                    name.clone(),
                    &attrs,
                    false,
                    (line, column, end_line),
                ));
                let ctx = ImplContext {
                    name,
                    attributes: attribute_paths(&attrs),
                };
                for impl_item in &imp.items {
                    if let syn::ImplItem::Fn(f) = impl_item {
                        let f_attrs = f.attrs.clone();
                        let (fl, fc) = span_start(&f.sig);
                        let fe = span_end_line(&f.sig);
                        out_items.push(item_summary(
                            ItemKind::Function,
                            f.sig.ident.to_string(),
                            &f_attrs,
                            vis_is_public(&f.vis),
                            (fl, fc, fe),
                        ));
                        out_functions.push(make_function(
                            &f.sig,
                            &f_attrs,
                            vis_is_public(&f.vis),
                            prefix,
                            Some(&ctx),
                        ));
                    }
                }
            }
            syn::Item::Trait(tr) => {
                let name = tr.ident.to_string();
                out_items.push(item_summary(
                    ItemKind::Trait,
                    name.clone(),
                    &attrs,
                    vis_is_public(&tr.vis),
                    (line, column, end_line),
                ));
                let ctx = ImplContext {
                    name,
                    attributes: attribute_paths(&attrs),
                };
                for trait_item in &tr.items {
                    if let syn::TraitItem::Fn(f) = trait_item {
                        let f_attrs = f.attrs.clone();
                        out_functions.push(make_function(
                            &f.sig,
                            &f_attrs,
                            false,
                            prefix,
                            Some(&ctx),
                        ));
                    }
                }
            }
            syn::Item::Mod(m) => {
                let name = m.ident.to_string();
                out_items.push(item_summary(
                    ItemKind::Module,
                    name.clone(),
                    &attrs,
                    vis_is_public(&m.vis),
                    (line, column, end_line),
                ));
                if let Some((_, sub_items)) = &m.content {
                    let child_prefix = if prefix.is_empty() {
                        name
                    } else {
                        format!("{prefix}::{name}")
                    };
                    walk_items(
                        sub_items,
                        &child_prefix,
                        None,
                        file,
                        out_items,
                        out_functions,
                        out_contract,
                    );
                }
            }
            syn::Item::Struct(s) => out_items.push(item_summary(
                ItemKind::Struct,
                s.ident.to_string(),
                &attrs,
                vis_is_public(&s.vis),
                (line, column, end_line),
            )),
            syn::Item::Enum(e) => out_items.push(item_summary(
                ItemKind::Enum,
                e.ident.to_string(),
                &attrs,
                vis_is_public(&e.vis),
                (line, column, end_line),
            )),
            syn::Item::Union(u) => out_items.push(item_summary(
                ItemKind::Union,
                u.ident.to_string(),
                &attrs,
                vis_is_public(&u.vis),
                (line, column, end_line),
            )),
            syn::Item::Const(c) => out_items.push(item_summary(
                ItemKind::Const,
                c.ident.to_string(),
                &attrs,
                vis_is_public(&c.vis),
                (line, column, end_line),
            )),
            syn::Item::Static(s) => out_items.push(item_summary(
                ItemKind::Static,
                s.ident.to_string(),
                &attrs,
                vis_is_public(&s.vis),
                (line, column, end_line),
            )),
            syn::Item::Macro(m) => out_items.push(item_summary(
                ItemKind::Macro,
                m.mac
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::"),
                &attrs,
                false,
                (line, column, end_line),
            )),
            syn::Item::Type(t) => out_items.push(item_summary(
                ItemKind::TypeAlias,
                t.ident.to_string(),
                &attrs,
                vis_is_public(&t.vis),
                (line, column, end_line),
            )),
            syn::Item::Use(_) => out_items.push(item_summary(
                ItemKind::Use,
                String::new(),
                &attrs,
                false,
                (line, column, end_line),
            )),
            syn::Item::ForeignMod(_) => out_items.push(item_summary(
                ItemKind::ForeignMod,
                String::new(),
                &attrs,
                false,
                (line, column, end_line),
            )),
            other => out_items.push(item_summary(
                ItemKind::Other,
                other
                    .to_token_stream()
                    .to_string()
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .to_string(),
                &attrs,
                false,
                (line, column, end_line),
            )),
        }
    }
}

fn item_attrs(item: &syn::Item) -> Vec<syn::Attribute> {
    use syn::Item;
    match item {
        Item::Const(i) => i.attrs.clone(),
        Item::Enum(i) => i.attrs.clone(),
        Item::ExternCrate(i) => i.attrs.clone(),
        Item::Fn(i) => i.attrs.clone(),
        Item::ForeignMod(i) => i.attrs.clone(),
        Item::Impl(i) => i.attrs.clone(),
        Item::Macro(i) => i.attrs.clone(),
        Item::Mod(i) => i.attrs.clone(),
        Item::Static(i) => i.attrs.clone(),
        Item::Struct(i) => i.attrs.clone(),
        Item::Trait(i) => i.attrs.clone(),
        Item::TraitAlias(i) => i.attrs.clone(),
        Item::Type(i) => i.attrs.clone(),
        Item::Union(i) => i.attrs.clone(),
        Item::Use(i) => i.attrs.clone(),
        _ => Vec::new(),
    }
}

fn item_summary(
    kind: ItemKind,
    name: String,
    attrs: &[syn::Attribute],
    is_public: bool,
    span: (usize, usize, usize),
) -> ItemSummary {
    ItemSummary {
        kind,
        name,
        line: span.0,
        column: span.1,
        end_line: span.2,
        is_public,
        attributes: attribute_paths(attrs),
    }
}

/// Computes a best-effort logical module path for a source file.
pub fn module_path_for(path: &Path, crate_root: &Path) -> String {
    let rel = path.strip_prefix(crate_root).unwrap_or(path);
    let comps: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect();

    let start = comps.iter().position(|c| c == "src");
    let mut parts: Vec<String> = match start {
        Some(idx) => comps[idx + 1..].to_vec(),
        None => comps.clone(),
    };

    if let Some(last) = parts.last_mut() {
        let stem = Path::new(last.as_str())
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        *last = stem;
    }
    if parts.last().map(String::as_str) == Some("mod") {
        parts.pop();
    }
    if parts.last().map(String::as_str) == Some("lib")
        || parts.last().map(String::as_str) == Some("main")
    {
        parts.pop();
    }

    if parts.is_empty() {
        "crate".to_string()
    } else {
        format!("crate::{}", parts.join("::"))
    }
}

/// Maximum delimiter nesting depth accepted before parsing is skipped.
///
/// Recursive-descent parsers can overflow the stack on pathologically nested
/// input. Since scanned repositories are untrusted, we reject deeply nested
/// sources before handing them to the parser.
pub const MAX_NESTING_DEPTH: usize = 192;

fn skip_string(bytes: &[u8], mut i: usize) -> usize {
    let n = bytes.len();
    while i < n {
        match bytes[i] {
            b'\\' => i += 2,
            b'"' => return i + 1,
            _ => i += 1,
        }
    }
    n
}

fn skip_char_literal(bytes: &[u8], i: usize) -> usize {
    let n = bytes.len();
    if i >= n {
        return i;
    }
    if bytes[i] == b'\\' {
        let mut j = i + 1;
        if j < n && bytes[j] == b'u' {
            j += 1;
            if j < n && bytes[j] == b'{' {
                while j < n && bytes[j] != b'}' {
                    j += 1;
                }
                if j < n {
                    j += 1;
                }
            }
        } else {
            j += 1;
        }
        return if j < n && bytes[j] == b'\'' { j + 1 } else { j };
    }
    // `'x'` is a char literal; `'x` is a lifetime.
    if i + 1 < n && bytes[i + 1] == b'\'' {
        return i + 2;
    }
    i
}

/// Computes the maximum delimiter nesting depth, ignoring strings and comments.
pub fn max_delimiter_depth(src: &str) -> usize {
    let bytes = src.as_bytes();
    let n = bytes.len();
    let mut i = 0usize;
    let mut depth = 0usize;
    let mut max_depth = 0usize;

    while i < n {
        match bytes[i] {
            b'/' if i + 1 < n && bytes[i + 1] == b'/' => {
                i += 2;
                while i < n && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < n && bytes[i + 1] == b'*' => {
                i += 2;
                let mut level = 1usize;
                while i < n && level > 0 {
                    if i + 1 < n && bytes[i] == b'/' && bytes[i + 1] == b'*' {
                        level += 1;
                        i += 2;
                    } else if i + 1 < n && bytes[i] == b'*' && bytes[i + 1] == b'/' {
                        level -= 1;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
            }
            b'"' => i = skip_string(bytes, i + 1),
            b'\'' => i = skip_char_literal(bytes, i + 1),
            b'r' | b'b' => {
                let mut j = i;
                if bytes[j] == b'b' {
                    j += 1;
                    if j >= n || bytes[j] != b'r' {
                        i += 1;
                        continue;
                    }
                }
                j += 1;
                let mut hashes = 0;
                while j < n && bytes[j] == b'#' {
                    hashes += 1;
                    j += 1;
                }
                if j < n && bytes[j] == b'"' {
                    j += 1;
                    loop {
                        if j >= n {
                            break;
                        }
                        if bytes[j] == b'"' {
                            let mut k = j + 1;
                            let mut h = 0;
                            while h < hashes && k < n && bytes[k] == b'#' {
                                h += 1;
                                k += 1;
                            }
                            if h == hashes {
                                j = k;
                                break;
                            }
                        }
                        j += 1;
                    }
                    i = j;
                } else {
                    i += 1;
                }
            }
            b'(' | b'[' | b'{' => {
                depth += 1;
                max_depth = max_depth.max(depth);
                i += 1;
            }
            b')' | b']' | b'}' => {
                depth = depth.saturating_sub(1);
                i += 1;
            }
            _ => i += 1,
        }
    }

    max_depth
}

/// Parses a single source file's content.
pub fn parse_source(path: &Path, crate_root: &Path, content: &str) -> ParsedSource {
    let module_path = module_path_for(path, crate_root);

    if max_delimiter_depth(content) > MAX_NESTING_DEPTH {
        return ParsedSource {
            path: path.to_path_buf(),
            module_path,
            syntax: None,
            parse_error: Some(format!(
                "source nesting exceeds the {MAX_NESTING_DEPTH}-level safety limit"
            )),
            items: Vec::new(),
            functions: Vec::new(),
            contract_definitions: Vec::new(),
        };
    }

    match syn::parse_file(content) {
        Ok(file) => {
            let mut items = Vec::new();
            let mut functions = Vec::new();
            let mut contract_definitions = Vec::new();
            walk_items(
                &file.items,
                "",
                None,
                path,
                &mut items,
                &mut functions,
                &mut contract_definitions,
            );
            ParsedSource {
                path: path.to_path_buf(),
                module_path,
                syntax: Some(file),
                parse_error: None,
                items,
                functions,
                contract_definitions,
            }
        }
        Err(err) => ParsedSource {
            path: path.to_path_buf(),
            module_path,
            syntax: None,
            parse_error: Some(err.to_string()),
            items: Vec::new(),
            functions: Vec::new(),
            contract_definitions: Vec::new(),
        },
    }
}

fn nearest_crate_root(path: &Path, crate_roots: &[PathBuf]) -> Option<PathBuf> {
    crate_roots
        .iter()
        .filter(|root| path.starts_with(root))
        .max_by_key(|root| root.components().count())
        .cloned()
}

/// Loads and parses a set of Rust source files.
///
/// Unreadable or oversized files produce diagnostics and are skipped. Every
/// other file is parsed; parse failures are recorded on the [`ParsedSource`].
pub fn load_sources(
    files: &[PathBuf],
    manifests: &[crate::model::Manifest],
    root: &Path,
) -> (Vec<ParsedSource>, Vec<Diagnostic>) {
    let crate_roots: Vec<PathBuf> = manifests
        .iter()
        .filter_map(|m| m.path.parent().map(Path::to_path_buf))
        .collect();

    let mut sources = Vec::new();
    let mut diagnostics = Vec::new();

    for path in files {
        match std::fs::metadata(path) {
            Ok(meta) if meta.len() > MAX_SOURCE_BYTES => {
                diagnostics.push(Diagnostic::warning(
                    format!(
                        "skipping source file larger than {} bytes",
                        MAX_SOURCE_BYTES
                    ),
                    Some(path.clone()),
                ));
                continue;
            }
            Ok(_) => {}
            Err(err) => {
                diagnostics.push(Diagnostic::warning(
                    format!("skipping unreadable source file: {err}"),
                    Some(path.clone()),
                ));
                continue;
            }
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(err) => {
                diagnostics.push(Diagnostic::warning(
                    format!("skipping unreadable source file: {err}"),
                    Some(path.clone()),
                ));
                continue;
            }
        };

        let crate_root =
            nearest_crate_root(path, &crate_roots).unwrap_or_else(|| root.to_path_buf());
        let source = parse_source(path, &crate_root, &content);
        if let Some(err) = &source.parse_error {
            diagnostics.push(Diagnostic::warning(
                format!("failed to parse source: {err}"),
                Some(path.clone()),
            ));
        }
        sources.push(source);
    }

    (sources, diagnostics)
}

/// Applies contract-definition evidence from parsed sources to a project.
pub fn apply_source_evidence(project: &mut Project, sources: &[ParsedSource]) {
    for source in sources {
        for location in &source.contract_definitions {
            soroban::note_contract_definition(&mut project.soroban, location);
        }
    }
    project.kind = soroban::classify(&project.soroban);
}

/// Convenience wrapper returning updated [`SorobanInfo`] for tests.
pub fn soroban_evidence(sources: &[ParsedSource]) -> SorobanInfo {
    let mut info = SorobanInfo::default();
    for source in sources {
        for location in &source.contract_definitions {
            soroban::note_contract_definition(&mut info, location);
        }
    }
    info
}

/// Reads and parses a single file from disk (test/helper convenience).
pub fn parse_file(path: &Path, crate_root: &Path) -> Result<ParsedSource, ScanError> {
    let content = std::fs::read_to_string(path).map_err(|e| ScanError::io(path, e))?;
    Ok(parse_source(path, crate_root, &content))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> ParsedSource {
        parse_source(Path::new("src/lib.rs"), Path::new("."), src)
    }

    #[test]
    fn captures_function_spans() {
        let s = parse("fn a() {}\n\nfn b() {}\n");
        assert!(s.is_parsed());
        assert_eq!(s.functions.len(), 2);
        assert_eq!(s.functions[0].name, "a");
        assert_eq!(s.functions[0].line, 1);
        assert_eq!(s.functions[1].name, "b");
        assert_eq!(s.functions[1].line, 3);
    }

    #[test]
    fn identifies_contract_entry_points() {
        let src = r#"
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct C;

#[contractimpl]
impl C {
    pub fn transfer(env: Env, amount: i128) {}
}
"#;
        let s = parse(src);
        assert_eq!(s.contract_definitions.len(), 2);
        assert!(s.contract_definitions[0].contains("src/lib.rs:"));
        let entry: Vec<_> = s.contract_entry_points().collect();
        assert_eq!(entry.len(), 1);
        assert_eq!(entry[0].name, "transfer");
        assert_eq!(entry[0].params.len(), 2);
        assert_eq!(entry[0].params[1].name, "amount");
        assert_eq!(entry[0].in_impl.as_deref(), Some("C"));
    }

    #[test]
    fn plain_impl_is_not_contract() {
        let src = "struct C;\nimpl C { pub fn f(&self) {} }\n";
        let s = parse(src);
        assert!(s.contract_definitions.is_empty());
        assert_eq!(s.contract_entry_points().count(), 0);
        assert_eq!(s.functions.len(), 1);
        assert!(!s.functions[0].is_contract_entry_point);
        assert_eq!(s.functions[0].params.len(), 0);
    }

    #[test]
    fn walks_nested_modules() {
        let src = "mod inner { pub fn g() {} }\n";
        let s = parse(src);
        let f = s.functions.iter().find(|f| f.name == "g").unwrap();
        assert_eq!(f.qualified_name, "inner::g");
        assert!(s
            .items
            .iter()
            .any(|i| i.kind == ItemKind::Module && i.name == "inner"));
    }

    #[test]
    fn parse_error_is_recorded_not_panicked() {
        let s = parse("fn broken( {");
        assert!(!s.is_parsed());
        assert!(s.parse_error.is_some());
        assert!(s.functions.is_empty());
    }

    #[test]
    fn module_path_is_computed() {
        let root = Path::new("/proj");
        assert_eq!(
            module_path_for(Path::new("/proj/src/lib.rs"), root),
            "crate"
        );
        assert_eq!(
            module_path_for(Path::new("/proj/src/foo.rs"), root),
            "crate::foo"
        );
        assert_eq!(
            module_path_for(Path::new("/proj/src/foo/mod.rs"), root),
            "crate::foo"
        );
    }

    #[test]
    fn classifies_contract_evidence() {
        let src = "#[contractimpl]\nimpl C { pub fn f() {} }\n";
        let s = parse(src);
        let info = soroban_evidence(&[s]);
        assert!(info.contract_macros_found);
    }

    #[test]
    fn measures_delimiter_depth() {
        assert_eq!(max_delimiter_depth("fn f() { (()) }"), 3);
        assert_eq!(max_delimiter_depth("fn f() {}"), 1);
    }

    #[test]
    fn ignores_delimiters_in_strings_and_comments() {
        assert_eq!(max_delimiter_depth("let s = \"((((((\";"), 0);
        assert_eq!(max_delimiter_depth("// ((((((\n"), 0);
        assert_eq!(max_delimiter_depth("/* (((((( */"), 0);
        assert_eq!(max_delimiter_depth("let r = r#\"(((((\"#;"), 0);
        assert_eq!(max_delimiter_depth("let c = '(';"), 0);
    }

    #[test]
    fn rejects_pathologically_nested_source() {
        let depth = MAX_NESTING_DEPTH + 50;
        let src = format!("fn f() {{ {}0{} }}", "(".repeat(depth), ")".repeat(depth));
        let parsed = parse(&src);
        assert!(parsed.parse_error.is_some());
        assert!(!parsed.is_parsed());
    }
}
