//! Integration tests for project discovery and Soroban classification.

use std::path::PathBuf;

use soroban_scan_core::discovery::DiscoveryOptions;
use soroban_scan_core::model::ProjectKind;
use soroban_scan_core::project;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/projects")
        .join(name)
}

#[test]
fn generic_rust_is_not_classified_as_soroban() {
    let loaded = project::load(&fixture("generic-rust"), &DiscoveryOptions::default()).unwrap();
    assert_eq!(loaded.project.kind, ProjectKind::GenericRust);
    assert!(!loaded.project.soroban.has_sdk_dependency);
    assert_eq!(loaded.project.source_files, 1);
}

#[test]
fn soroban_dependency_without_contract_is_likely() {
    let loaded = project::load(&fixture("soroban-likely"), &DiscoveryOptions::default()).unwrap();
    assert_eq!(loaded.project.kind, ProjectKind::LikelySoroban);
    assert!(loaded.project.soroban.has_sdk_dependency);
    assert!(!loaded.project.soroban.contract_macros_found);
}

#[test]
fn contract_definition_confirms_soroban_and_discovers_items() {
    let loaded =
        project::load(&fixture("soroban-confirmed"), &DiscoveryOptions::default()).unwrap();
    assert_eq!(loaded.project.kind, ProjectKind::ConfirmedSoroban);
    assert!(loaded.project.soroban.contract_macros_found);
    assert_eq!(loaded.sources.len(), 2);

    let lib = loaded
        .sources
        .iter()
        .find(|s| s.path.ends_with("lib.rs"))
        .unwrap();
    let entry_points: Vec<_> = lib.contract_entry_points().collect();
    assert_eq!(entry_points.len(), 1);
    assert_eq!(entry_points[0].name, "hello");
}

#[test]
fn metadata_loading_is_deterministic() {
    let opts = DiscoveryOptions::default();
    let a = project::load_project(&fixture("soroban-confirmed"), &opts).unwrap();
    let b = project::load_project(&fixture("soroban-confirmed"), &opts).unwrap();
    let ja = serde_json::to_string(&a).unwrap();
    let jb = serde_json::to_string(&b).unwrap();
    assert_eq!(
        ja, jb,
        "metadata output must be byte-for-byte deterministic"
    );
}
