//! Inline suppression directives.
//!
//! A directive is a comment of the form
//!
//! ```text
//! // soroban-scan: ignore SS-001, SS-002 -- documented reason
//! ```
//!
//! The pattern is intentionally small and stable so that suppressions stay
//! auditable:
//!
//! * the marker is the literal `soroban-scan:` (matched case-insensitively);
//! * the keyword is `ignore`;
//! * the targets are one or more rule ids (`SS-XXX`, comma or whitespace
//!   separated) or the literal `all`;
//! * an optional reason follows a `--` separator.
//!
//! Detection is purely lexical and never executes the analyzed code:
//!
//! * directives inside string literals, character literals, raw strings, or
//!   block comments that are not on a single line are **not** recognized;
//! * a directive applies to the code line it is anchored to: the line it
//!   trails, or the first code line below it, allowing at most
//!   [`MAX_GAP_LINES`] intervening lines of blanks and non-directive comments;
//! * when the directive is anchored to the first line of an item (or the line
//!   directly above it, which is where attributes live), it applies to the
//!   whole item; otherwise it applies to a single finding location.
//!
//! Suppressions are never silent: the engine reports every suppressed finding
//! together with the directive that suppressed it and the stated reason.

use std::path::Path;

/// Marker that introduces a directive inside a comment.
pub const DIRECTIVE_MARKER: &str = "soroban-scan:";
/// Keyword that suppresses findings.
pub const IGNORE_KEYWORD: &str = "ignore";
/// Literal that suppresses every rule.
pub const ALL_TARGETS: &str = "all";
/// Token that separates rule targets from the optional reason.
pub const REASON_SEPARATOR: &str = "--";
/// Maximum number of blank/comment lines allowed between a directive and the
/// code line it applies to.
pub const MAX_GAP_LINES: usize = 3;

/// Rule targets of a directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Targets {
    /// Every rule is suppressed for the targeted location.
    All,
    /// Only these rule ids are suppressed for the targeted location.
    Rules(Vec<String>),
}

impl Targets {
    /// Returns true if the target set covers `rule_id`.
    pub fn matches(&self, rule_id: &str) -> bool {
        match self {
            Targets::All => true,
            Targets::Rules(ids) => ids.iter().any(|id| id == rule_id),
        }
    }

    /// Stable, comma-separated label used in reports.
    pub fn label(&self) -> String {
        match self {
            Targets::All => ALL_TARGETS.to_string(),
            Targets::Rules(ids) => ids.join(","),
        }
    }
}

/// A directive resolved to the source region it applies to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suppression {
    /// File containing the directive, as parsed (relativized by the engine).
    pub file: std::path::PathBuf,
    /// 1-based line of the directive comment.
    pub line: usize,
    /// Rule targets of the directive.
    pub targets: Targets,
    /// Optional reason supplied by the directive author.
    pub reason: Option<String>,
    /// 1-based first line the directive applies to.
    pub start_line: usize,
    /// 1-based last line the directive applies to (inclusive).
    pub end_line: usize,
}

impl Suppression {
    /// Returns a copy of this directive attributed to another file.
    pub fn with_file(mut self, file: impl Into<std::path::PathBuf>) -> Self {
        self.file = file.into();
        self
    }

    /// Root-relative, forward-slash file label used in reports.
    pub fn file_label(&self) -> String {
        normalize_rel_path(&self.file)
    }

    /// Returns true when the directive suppresses `rule_id` at `file:line`.
    pub fn applies_to(&self, file: &str, line: usize, rule_id: &str) -> bool {
        self.targets.matches(rule_id)
            && self.file_label() == file
            && line >= self.start_line
            && line <= self.end_line
    }
}

/// Normalizes a path to forward slashes without a leading `./`.
pub fn normalize_rel_path(path: &Path) -> String {
    let s = path.to_string_lossy().replace('\\', "/");
    s.strip_prefix("./").unwrap_or(&s).to_string()
}

/// Returns true if `token` looks like a rule id (`SS-` followed by 3 digits).
fn is_rule_id(token: &str) -> bool {
    let upper = token.to_ascii_uppercase();
    upper.len() == 6 && upper.starts_with("SS-") && upper[3..].chars().all(|c| c.is_ascii_digit())
}

/// Parses the target list of a directive.
fn parse_targets(text: &str) -> Option<Targets> {
    let mut ids: Vec<String> = Vec::new();
    for token in text.replace(',', " ").split_whitespace() {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        if token.eq_ignore_ascii_case(ALL_TARGETS) {
            return Some(Targets::All);
        }
        if !is_rule_id(token) {
            return None;
        }
        let id = token.to_ascii_uppercase();
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    if ids.is_empty() {
        None
    } else {
        Some(Targets::Rules(ids))
    }
}

/// Parses the text of a comment as a directive, if it is one.
fn parse_directive(comment: &str) -> Option<(Targets, Option<String>)> {
    let text = comment.trim_start_matches(['/', '!']).trim();
    let prefix = text.get(..DIRECTIVE_MARKER.len())?;
    if !prefix.eq_ignore_ascii_case(DIRECTIVE_MARKER) {
        return None;
    }
    let rest = text[DIRECTIVE_MARKER.len()..].trim_start();
    let mut parts = rest.splitn(2, char::is_whitespace);
    let keyword = parts.next().unwrap_or_default();
    if !keyword.eq_ignore_ascii_case(IGNORE_KEYWORD) {
        return None;
    }
    let args = parts.next().unwrap_or_default().trim();
    let (targets_text, reason) = match args.split_once(REASON_SEPARATOR) {
        Some((targets, reason)) => (targets.trim(), Some(reason.trim().to_string())),
        None => (args, None),
    };
    let reason = reason.filter(|r| !r.is_empty());
    let targets = parse_targets(targets_text)?;
    Some((targets, reason))
}

/// Per-line lexical facts needed to resolve directives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LineInfo {
    /// True when the line contains code outside strings and comments.
    has_code: bool,
}

fn is_char_boundary(chars: &[char], index: usize, needle: char) -> bool {
    chars.get(index) == Some(&needle)
}

/// Returns the number of `#` characters of a raw-string opener at `chars[0]`.
fn raw_string_opener(chars: &[char]) -> Option<usize> {
    if chars.first() != Some(&'r') {
        return None;
    }
    let mut hashes = 0usize;
    let mut index = 1usize;
    while chars.get(index) == Some(&'#') {
        hashes += 1;
        index += 1;
    }
    (chars.get(index) == Some(&'"')).then_some(hashes)
}

/// Finds the index of the closing quote of a raw string in `chars[from..]`.
fn raw_string_close(chars: &[char], from: usize, hashes: usize) -> Option<usize> {
    let mut index = from;
    while index < chars.len() {
        if chars[index] == '"' {
            let terminated = (0..hashes).all(|offset| chars.get(index + 1 + offset) == Some(&'#'));
            if terminated {
                return Some(index);
            }
        }
        index += 1;
    }
    None
}

/// Finds the index of the closing `*/` of a block comment in `chars`.
fn block_comment_close(chars: &[char]) -> Option<usize> {
    let mut index = 0usize;
    while index + 1 < chars.len() {
        if chars[index] == '*' && chars[index + 1] == '/' {
            return Some(index);
        }
        index += 1;
    }
    None
}

/// Scans the raw text and returns per-line facts and comment texts.
///
/// The scanner is intentionally conservative: when a construct cannot be
/// terminated on the current line (a block comment, a raw string, or a string
/// literal) the remaining lines are treated as non-code until it terminates, so
/// a directive inside a string is never treated as a directive.
fn line_infos(source: &str) -> (Vec<LineInfo>, Vec<Option<String>>) {
    let mut infos = Vec::new();
    let mut comments = Vec::new();
    let mut in_block_comment = false;
    let mut in_string = false;
    let mut raw_hashes: Option<usize> = None;

    for line in source.lines() {
        let chars: Vec<char> = line.chars().collect();
        let mut has_code = false;
        let mut comment: Option<String> = None;
        let mut index = 0usize;

        while index < chars.len() {
            if let Some(hashes) = raw_hashes {
                match raw_string_close(&chars, index, hashes) {
                    Some(close) => {
                        index = close + 1 + hashes;
                        raw_hashes = None;
                    }
                    None => index = chars.len(),
                }
                continue;
            }
            if in_string {
                if chars[index] == '\\' {
                    index += 2;
                    continue;
                }
                if chars[index] == '"' {
                    in_string = false;
                }
                index += 1;
                continue;
            }
            if in_block_comment {
                if is_char_boundary(&chars, index, '*') && is_char_boundary(&chars, index + 1, '/')
                {
                    in_block_comment = false;
                    index += 2;
                    continue;
                }
                index += 1;
                continue;
            }

            match chars[index] {
                '/' if is_char_boundary(&chars, index + 1, '/') => {
                    comment = Some(chars[index + 2..].iter().collect());
                    break;
                }
                '/' if is_char_boundary(&chars, index + 1, '*') => {
                    let body = &chars[index + 2..];
                    match block_comment_close(body) {
                        Some(close) => {
                            comment = Some(body[..close].iter().collect());
                            index += 2 + close + 2;
                        }
                        None => {
                            in_block_comment = true;
                            index += 2;
                        }
                    }
                }
                '"' => {
                    has_code = true;
                    in_string = true;
                    index += 1;
                }
                'r' if raw_string_opener(&chars[index..]).is_some() => {
                    let hashes = raw_string_opener(&chars[index..]).unwrap_or_default();
                    has_code = true;
                    let from = index + 1 + hashes + 1;
                    match raw_string_close(&chars, from, hashes) {
                        Some(close) => index = close + 1 + hashes,
                        None => {
                            raw_hashes = Some(hashes);
                            index = chars.len();
                        }
                    }
                }
                c if !c.is_whitespace() => {
                    has_code = true;
                    index += 1;
                }
                _ => index += 1,
            }
        }

        infos.push(LineInfo { has_code });
        comments.push(comment);
    }

    (infos, comments)
}

/// Resolves the code line a directive is anchored to.
fn resolve_target(line: usize, infos: &[LineInfo]) -> Option<usize> {
    if infos.get(line.checked_sub(1)?)?.has_code {
        return Some(line);
    }
    // Allow at most `MAX_GAP_LINES` blank/comment-only lines between the
    // directive and the code it applies to.
    for offset in 1..=MAX_GAP_LINES + 1 {
        let index = line
            .checked_add(offset)
            .and_then(|target| target.checked_sub(1))?;
        match infos.get(index) {
            Some(info) if info.has_code => return Some(line + offset),
            Some(_) => {}
            None => return None,
        }
    }
    None
}

/// Resolves the line range a directive covers for a given anchor line.
///
/// A directive anchored to the first line of an item (or the line directly
/// above it, where attributes are written) covers the whole item. Everything
/// else covers the single anchored line.
fn resolve_span(target: usize, spans: &[(usize, usize)]) -> (usize, usize) {
    let mut best: Option<(usize, usize)> = None;
    for &(start, end) in spans {
        if end < start || (start != target && start != target + 1) {
            continue;
        }
        let better = match best {
            Some((best_start, best_end)) => (end - start) < (best_end - best_start),
            None => true,
        };
        if better {
            best = Some((start, end));
        }
    }
    best.unwrap_or((target, target))
}

/// Detects and resolves inline suppression directives in a source file.
///
/// `spans` are the `(first_line, last_line)` bounds of the items (functions and
/// other items) in the file; they are used to widen a directive to the item it
/// is anchored to.
pub fn detect(source: &str, file: &Path, spans: &[(usize, usize)]) -> Vec<Suppression> {
    let (infos, comments) = line_infos(source);
    let mut suppressions = Vec::new();
    for (index, comment) in comments.iter().enumerate() {
        let Some(comment) = comment else { continue };
        let Some((targets, reason)) = parse_directive(comment) else {
            continue;
        };
        let line = index + 1;
        let Some(target) = resolve_target(line, &infos) else {
            continue;
        };
        let (start_line, end_line) = resolve_span(target, spans);
        suppressions.push(Suppression {
            file: file.to_path_buf(),
            line,
            targets,
            reason,
            start_line,
            end_line,
        });
    }
    suppressions
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn file() -> PathBuf {
        PathBuf::from("src/lib.rs")
    }

    fn detect_str(src: &str, spans: &[(usize, usize)]) -> Vec<Suppression> {
        detect(src, &file(), spans)
    }

    #[test]
    fn parses_a_directive_with_reason_and_multiple_rules() {
        let src = "// soroban-scan: ignore SS-001, SS-002 -- documented trade-off\nfn f() {}\n";
        let found = detect_str(src, &[]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].line, 1);
        assert_eq!(found[0].targets.label(), "SS-001,SS-002");
        assert_eq!(found[0].reason.as_deref(), Some("documented trade-off"));
        assert_eq!((found[0].start_line, found[0].end_line), (2, 2));
    }

    #[test]
    fn targets_are_case_insensitive_and_deduplicated() {
        let src = "// soroban-scan: ignore ss-001 SS-001\nfn f() {}\n";
        let found = detect_str(src, &[(1, 2)]);
        assert_eq!(found[0].targets.label(), "SS-001");
    }

    #[test]
    fn parses_all_and_rule_agnostic_matching() {
        let src = "// soroban-scan: ignore all\nfn f() {}\n";
        let found = detect_str(src, &[(1, 2)]);
        assert_eq!(found[0].targets, Targets::All);
        assert!(found[0].targets.matches("SS-029"));
        assert_eq!(found[0].reason, None);
    }

    #[test]
    fn doc_comments_and_trailing_comments_are_supported() {
        let src = "/// soroban-scan: ignore SS-001\nfn f() {}\nlet x = 1; // soroban-scan: ignore SS-002\n";
        let found = detect_str(src, &[(1, 2)]);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].line, 1);
        assert_eq!(found[1].line, 3);
        assert_eq!((found[1].start_line, found[1].end_line), (3, 3));
    }

    #[test]
    fn single_line_block_comment_directive_is_supported() {
        let src = "/* soroban-scan: ignore SS-001 */\nfn f() {}\n";
        let found = detect_str(src, &[(1, 2)]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].targets.label(), "SS-001");
    }

    #[test]
    fn invalid_directives_are_ignored() {
        for src in [
            "// soroban-scan: ignore\nfn f() {}\n",
            "// soroban-scan: ignore SS-1\nfn f() {}\n",
            "// soroban-scan: ignore BANANA\nfn f() {}\n",
            "// soroban-scan: allow SS-001\nfn f() {}\n",
            "// not-a-directive: ignore SS-001\nfn f() {}\n",
        ] {
            assert!(
                detect_str(src, &[(1, 2)]).is_empty(),
                "unexpected directive in {src:?}"
            );
        }
    }

    #[test]
    fn directives_inside_strings_are_ignored() {
        let src = "fn f() {\n    let s = \"// soroban-scan: ignore SS-001\";\n}\n";
        assert!(detect_str(src, &[(1, 3)]).is_empty());

        let src = "fn f() {\n    let s = r\"// soroban-scan: ignore SS-001\";\n}\n";
        assert!(detect_str(src, &[(1, 3)]).is_empty());

        let src = "fn f() {\n    let s = r#\"\n// soroban-scan: ignore SS-001\n\"#;\n}\n";
        assert!(detect_str(src, &[(1, 5)]).is_empty());

        let src = "fn f() {\n    let s = b\"// soroban-scan: ignore SS-001\";\n}\n";
        assert!(detect_str(src, &[(1, 3)]).is_empty());
    }

    #[test]
    fn multi_line_block_comments_are_not_directives() {
        let src = "/*\n// soroban-scan: ignore SS-001\n*/\nfn f() {}\n";
        assert!(detect_str(src, &[(4, 4)]).is_empty());
    }

    #[test]
    fn non_adjacent_directive_is_ignored() {
        let src = "// soroban-scan: ignore SS-001\n\n\n\n\nfn f() {}\n";
        assert!(detect_str(src, &[(6, 6)]).is_empty());

        let src = "// soroban-scan: ignore SS-001\n\n\nfn f() {}\n";
        assert_eq!(detect_str(src, &[(4, 4)]).len(), 1);
    }

    #[test]
    fn directive_above_attributes_covers_the_item() {
        let src =
            "// soroban-scan: ignore SS-001\n#[contractimpl]\nimpl C {\n    pub fn f() {}\n}\n";
        let found = detect_str(src, &[(3, 5)]);
        assert_eq!(found.len(), 1);
        assert_eq!((found[0].start_line, found[0].end_line), (3, 5));
        assert!(found[0].applies_to("src/lib.rs", 4, "SS-001"));
        assert!(!found[0].applies_to("src/lib.rs", 4, "SS-002"));
        assert!(!found[0].applies_to("src/other.rs", 4, "SS-001"));
    }

    #[test]
    fn directive_inside_an_item_covers_only_the_anchored_line() {
        let src = "fn f() {\n    // soroban-scan: ignore SS-001\n    let x = 1;\n}\n";
        let found = detect_str(src, &[(1, 4)]);
        assert_eq!(found.len(), 1);
        assert_eq!((found[0].start_line, found[0].end_line), (3, 3));
    }

    #[test]
    fn file_labels_are_normalized() {
        let found = detect(
            "// soroban-scan: ignore SS-001\nfn f() {}\n",
            Path::new("./a\\b.rs"),
            &[],
        );
        assert_eq!(found[0].file_label(), "a/b.rs");
    }

    #[test]
    fn detection_is_deterministic() {
        let src =
            "// soroban-scan: ignore SS-001\nfn a() {}\n// soroban-scan: ignore SS-002\nfn b() {}\n";
        let a = detect_str(src, &[(2, 2), (4, 4)]);
        let b = detect_str(src, &[(2, 2), (4, 4)]);
        assert_eq!(a, b);
    }
}
