//! `Cargo.toml` parsing.
//!
//! Parsing is purely syntactic: manifests are treated as untrusted data and are
//! never used to execute or fetch anything.

use std::path::{Path, PathBuf};

use toml::Value;

use crate::error::ScanError;
use crate::model::{DepKind, DepSource, Dependency, Manifest, PackageInfo, WorkspaceInfo};

/// Parses a `Cargo.toml` from its raw text.
pub fn parse_manifest(path: &Path, content: &str) -> Result<Manifest, ScanError> {
    let value: Value = content
        .parse::<Value>()
        .map_err(|e| ScanError::parse(path, format!("invalid Cargo.toml: {e}")))?;

    let table = value
        .as_table()
        .ok_or_else(|| ScanError::parse(path, "Cargo.toml root is not a table"))?;

    let package = table.get("package").and_then(parse_package);
    let workspace = table.get("workspace").and_then(parse_workspace);

    let mut dependencies = Vec::new();
    collect_dependencies(table, "dependencies", DepKind::Normal, &mut dependencies);
    collect_dependencies(table, "dev-dependencies", DepKind::Dev, &mut dependencies);
    collect_dependencies(
        table,
        "build-dependencies",
        DepKind::Build,
        &mut dependencies,
    );

    // `[workspace.dependencies]` declares versions inherited by members.
    if let Some(ws) = table.get("workspace").and_then(Value::as_table) {
        if let Some(deps) = ws.get("dependencies").and_then(Value::as_table) {
            for (name, spec) in deps {
                dependencies.push(parse_dependency(name, spec, DepKind::Normal));
            }
        }
    }

    // `[target.<cfg>.dependencies]`
    if let Some(targets) = table.get("target").and_then(Value::as_table) {
        for target in targets.values() {
            if let Some(t) = target.as_table() {
                collect_dependencies(t, "dependencies", DepKind::Normal, &mut dependencies);
                collect_dependencies(t, "dev-dependencies", DepKind::Dev, &mut dependencies);
                collect_dependencies(t, "build-dependencies", DepKind::Build, &mut dependencies);
            }
        }
    }

    dependencies.sort_by(|a, b| a.name.cmp(&b.name));
    dependencies.dedup_by(|a, b| a.name == b.name && a.kind == b.kind);

    Ok(Manifest {
        path: path.to_path_buf(),
        package,
        workspace,
        dependencies,
    })
}

/// Reads and parses a `Cargo.toml` from disk.
pub fn load_manifest(path: &Path) -> Result<Manifest, ScanError> {
    let content = std::fs::read_to_string(path).map_err(|e| ScanError::io(path, e))?;
    parse_manifest(path, &content)
}

fn parse_package(value: &Value) -> Option<PackageInfo> {
    let table = value.as_table()?;
    let name = table.get("name")?.as_str()?.to_string();
    Some(PackageInfo {
        name,
        version: table
            .get("version")
            .and_then(Value::as_str)
            .map(str::to_string),
        edition: table
            .get("edition")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

fn parse_workspace(value: &Value) -> Option<WorkspaceInfo> {
    let table = value.as_table()?;
    let members = table
        .get("members")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let resolver = table.get("resolver").and_then(|v| {
        v.as_str()
            .map(str::to_string)
            .or_else(|| v.as_integer().map(|i| i.to_string()))
    });
    Some(WorkspaceInfo { members, resolver })
}

fn collect_dependencies(
    table: &toml::map::Map<String, Value>,
    key: &str,
    kind: DepKind,
    out: &mut Vec<Dependency>,
) {
    if let Some(deps) = table.get(key).and_then(Value::as_table) {
        for (name, spec) in deps {
            out.push(parse_dependency(name, spec, kind));
        }
    }
}

fn parse_dependency(name: &str, spec: &Value, kind: DepKind) -> Dependency {
    match spec {
        Value::String(version) => Dependency {
            name: name.to_string(),
            version: Some(version.clone()),
            source: DepSource::Registry { registry: None },
            kind,
            optional: false,
        },
        Value::Table(t) => {
            let version = t.get("version").and_then(Value::as_str).map(str::to_string);
            let optional = t.get("optional").and_then(Value::as_bool).unwrap_or(false);
            let source = if t.get("workspace").and_then(Value::as_bool) == Some(true) {
                DepSource::Workspace
            } else if let Some(path) = t.get("path").and_then(Value::as_str) {
                DepSource::Path {
                    path: path.to_string(),
                }
            } else if let Some(url) = t.get("git").and_then(Value::as_str) {
                DepSource::Git {
                    url: url.to_string(),
                }
            } else if let Some(registry) = t.get("registry").and_then(Value::as_str) {
                DepSource::Registry {
                    registry: Some(registry.to_string()),
                }
            } else {
                DepSource::Registry { registry: None }
            };
            Dependency {
                name: name.to_string(),
                version,
                source,
                kind,
                optional,
            }
        }
        _ => Dependency {
            name: name.to_string(),
            version: None,
            source: DepSource::Unknown,
            kind,
            optional: false,
        },
    }
}

/// Returns the conventional manifest path for a directory.
pub fn manifest_path(dir: &Path) -> PathBuf {
    dir.join("Cargo.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_package_and_dependencies() {
        let toml = r#"
[package]
name = "demo"
version = "0.1.0"
edition = "2021"

[dependencies]
soroban-sdk = "21.0.0"
serde = { version = "1", features = ["derive"], optional = true }

[dev-dependencies]
pretty_assertions = "1"
"#;
        let m = parse_manifest(Path::new("Cargo.toml"), toml).unwrap();
        assert_eq!(m.package.as_ref().unwrap().name, "demo");
        assert_eq!(
            m.dependency("soroban-sdk").unwrap().version.as_deref(),
            Some("21.0.0")
        );
        assert!(m.dependency("serde").unwrap().optional);
        assert_eq!(
            m.dependency("pretty_assertions").unwrap().kind,
            DepKind::Dev
        );
    }

    #[test]
    fn parses_workspace_and_members() {
        let toml = r#"
[workspace]
resolver = "2"
members = ["contracts/*", "sdk"]

[workspace.dependencies]
soroban-sdk = "21.0.0"
"#;
        let m = parse_manifest(Path::new("Cargo.toml"), toml).unwrap();
        let ws = m.workspace.as_ref().unwrap();
        assert_eq!(ws.resolver.as_deref(), Some("2"));
        assert_eq!(ws.members, vec!["contracts/*", "sdk"]);
        assert!(m.dependency("soroban-sdk").is_some());
        assert!(m.is_workspace_root());
    }

    #[test]
    fn parses_path_and_git_sources() {
        let toml = r#"
[dependencies]
local = { path = "../local" }
remote = { git = "https://example.com/x.git" }
"#;
        let m = parse_manifest(Path::new("Cargo.toml"), toml).unwrap();
        assert!(matches!(
            m.dependency("local").unwrap().source,
            DepSource::Path { .. }
        ));
        assert!(matches!(
            m.dependency("remote").unwrap().source,
            DepSource::Git { .. }
        ));
    }

    #[test]
    fn rejects_invalid_toml() {
        let err = parse_manifest(Path::new("Cargo.toml"), "this is not toml = =").unwrap_err();
        assert!(matches!(err, ScanError::Parse { .. }));
    }
}
