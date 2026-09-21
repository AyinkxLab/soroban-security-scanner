//! Project loading: discovery plus manifest intelligence.

use std::path::Path;

use crate::discovery::{self, DiscoveryOptions};
use crate::error::ScanError;
use crate::manifest;
use crate::model::{Diagnostic, Project};
use crate::soroban;

/// Loads project metadata from a root directory.
///
/// Malformed manifests are reported as diagnostics rather than aborting the
/// scan, so that one bad file does not hide an entire project. The root's own
/// manifest, if present and unreadable, is an error.
pub fn load_project(root: &Path, opts: &DiscoveryOptions) -> Result<Project, ScanError> {
    let discovered = discovery::discover(root, opts)?;
    let mut manifests = Vec::new();
    let mut diagnostics = Vec::new();

    for path in &discovered.manifests {
        match manifest::load_manifest(path) {
            Ok(manifest) => manifests.push(manifest),
            Err(err) => {
                let is_root_manifest = path.parent() == Some(root);
                if is_root_manifest {
                    return Err(err);
                }
                diagnostics.push(Diagnostic::warning(
                    format!("skipping unreadable manifest: {err}"),
                    Some(path.clone()),
                ));
            }
        }
    }

    let soroban = soroban::analyze_manifests(&manifests);
    let kind = soroban::classify(&soroban);

    Ok(Project {
        root: root.to_path_buf(),
        kind,
        manifests,
        soroban,
        source_files: discovered.rust_files.len(),
        diagnostics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ProjectKind;
    use std::path::PathBuf;

    fn tmpdir(name: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "soroban-scan-project-{}-{}",
            name,
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        base
    }

    fn write(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }

    #[test]
    fn classifies_soroban_project() {
        let root = tmpdir("soroban");
        write(
            &root.join("Cargo.toml"),
            "[package]\nname = \"c\"\nversion = \"0.1.0\"\n[dependencies]\nsoroban-sdk = \"21.0.0\"\n",
        );
        write(&root.join("src/lib.rs"), "fn f() {}");
        let project = load_project(&root, &DiscoveryOptions::default()).unwrap();
        assert_eq!(project.kind, ProjectKind::LikelySoroban);
        assert!(project.soroban.has_sdk_dependency);
        assert_eq!(project.source_files, 1);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn classifies_generic_rust_project() {
        let root = tmpdir("generic");
        write(
            &root.join("Cargo.toml"),
            "[package]\nname = \"g\"\nversion = \"0.1.0\"\n[dependencies]\nserde = \"1\"\n",
        );
        let project = load_project(&root, &DiscoveryOptions::default()).unwrap();
        assert_eq!(project.kind, ProjectKind::GenericRust);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn invalid_root_manifest_is_fatal() {
        let root = tmpdir("badroot");
        write(&root.join("Cargo.toml"), "not = = toml");
        assert!(load_project(&root, &DiscoveryOptions::default()).is_err());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn invalid_nested_manifest_is_diagnostic() {
        let root = tmpdir("badnested");
        write(
            &root.join("Cargo.toml"),
            "[package]\nname = \"root\"\nversion = \"0.1.0\"\n",
        );
        write(&root.join("nested/Cargo.toml"), "not = = toml");
        let project = load_project(&root, &DiscoveryOptions::default()).unwrap();
        assert_eq!(project.manifests.len(), 1);
        assert_eq!(project.diagnostics.len(), 1);
        let _ = std::fs::remove_dir_all(&root);
    }
}
