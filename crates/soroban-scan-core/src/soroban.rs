//! Soroban detection and classification.
//!
//! Classification is evidence-based and deliberately conservative: a plain Rust
//! project is never labeled Soroban without evidence.
//!
//! * [`ProjectKind::ConfirmedSoroban`] requires concrete contract definitions
//!   (`#[contract]` / `#[contractimpl]`).
//! * [`ProjectKind::LikelySoroban`] means a Soroban dependency is present but no
//!   contract definition was found (for example, a shared library).
//! * Everything else is [`ProjectKind::GenericRust`].

use crate::model::{Manifest, ProjectKind, SorobanInfo};

/// Canonical Soroban SDK crate name.
pub const SOROBAN_SDK: &str = "soroban-sdk";

/// Returns true if `name` looks like a Soroban-related crate.
pub fn is_soroban_dependency(name: &str) -> bool {
    let normalized = name.replace('_', "-");
    normalized == SOROBAN_SDK
        || normalized.starts_with("soroban-")
        || normalized.starts_with("stellar-")
}

/// Strength of a Soroban attribute found in source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractAttrStrength {
    /// Defines a contract or its implementation (`contract`, `contractimpl`).
    Definition,
    /// Soroban-related but not a contract definition (`contracttype`,
    /// `contractclient`, `contracterror`, `contractstorage`).
    Related,
}

/// Classifies a Soroban attribute path.
pub fn contract_attribute_strength(path: &str) -> Option<ContractAttrStrength> {
    match path {
        "contract" | "contractimpl" => Some(ContractAttrStrength::Definition),
        "contracttype" | "contractclient" | "contracterror" | "contractstorage" => {
            Some(ContractAttrStrength::Related)
        }
        _ => None,
    }
}

/// Collects Soroban evidence from parsed manifests.
pub fn analyze_manifests(manifests: &[Manifest]) -> SorobanInfo {
    let mut info = SorobanInfo::default();
    for manifest in manifests {
        for dep in &manifest.dependencies {
            if is_soroban_dependency(&dep.name) {
                info.has_sdk_dependency = true;
                if let Some(version) = &dep.version {
                    info.sdk_versions.push(version.clone());
                }
                info.evidence.push(format!(
                    "dependency `{}` declared in {}",
                    dep.name,
                    manifest.path.display()
                ));
            }
        }
    }
    info.sdk_versions.sort();
    info.sdk_versions.dedup();
    info.evidence.sort();
    info.evidence.dedup();
    info
}

/// Derives the project classification from collected evidence.
pub fn classify(info: &SorobanInfo) -> ProjectKind {
    if info.contract_macros_found {
        ProjectKind::ConfirmedSoroban
    } else if info.has_sdk_dependency || !info.evidence.is_empty() {
        ProjectKind::LikelySoroban
    } else {
        ProjectKind::GenericRust
    }
}

/// Records that a contract definition attribute was observed.
pub fn note_contract_definition(info: &mut SorobanInfo, location: &str) {
    if !info.contract_macros_found {
        info.contract_macros_found = true;
    }
    let evidence = format!("contract definition at {location}");
    if !info.evidence.contains(&evidence) {
        info.evidence.push(evidence);
        info.evidence.sort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::parse_manifest;
    use std::path::Path;

    fn ws_manifest(toml: &str) -> Manifest {
        parse_manifest(Path::new("Cargo.toml"), toml).unwrap()
    }

    #[test]
    fn detects_soroban_dependency() {
        let m = ws_manifest("[dependencies]\nsoroban-sdk = \"21.0.0\"\n");
        let info = analyze_manifests(&[m]);
        assert!(info.has_sdk_dependency);
        assert_eq!(info.sdk_versions, vec!["21.0.0"]);
        assert_eq!(classify(&info), ProjectKind::LikelySoroban);
    }

    #[test]
    fn plain_rust_is_generic() {
        let m = ws_manifest("[dependencies]\nserde = \"1\"\n");
        let info = analyze_manifests(&[m]);
        assert!(!info.has_sdk_dependency);
        assert_eq!(classify(&info), ProjectKind::GenericRust);
    }

    #[test]
    fn contract_macro_confirms_soroban() {
        let mut info = analyze_manifests(&[ws_manifest("[dependencies]\nserde = \"1\"\n")]);
        note_contract_definition(&mut info, "src/lib.rs:1");
        assert_eq!(classify(&info), ProjectKind::ConfirmedSoroban);
    }

    #[test]
    fn attribute_classification() {
        assert_eq!(
            contract_attribute_strength("contractimpl"),
            Some(ContractAttrStrength::Definition)
        );
        assert_eq!(
            contract_attribute_strength("contracttype"),
            Some(ContractAttrStrength::Related)
        );
        assert_eq!(contract_attribute_strength("derive"), None);
    }

    #[test]
    fn soroban_dependency_name_matching() {
        assert!(is_soroban_dependency("soroban-sdk"));
        assert!(is_soroban_dependency("soroban_token_sdk"));
        assert!(is_soroban_dependency("soroban-token-sdk"));
        assert!(!is_soroban_dependency("serde"));
    }
}
