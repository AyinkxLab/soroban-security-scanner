//! Rule-quality guardrails: metadata, documentation, and per-rule compatibility.

use std::collections::BTreeSet;
use std::path::PathBuf;

use soroban_scan_core::config::ScanConfig;
use soroban_scan_core::engine;
use soroban_scan_core::registry::RuleRegistry;
use soroban_scan_core::rules::validation::{validate_docs, validate_registry};

fn docs_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs")
}

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/corpus")
}

#[test]
fn built_in_rule_metadata_is_valid() {
    let registry = RuleRegistry::with_default_rules();
    let issues = validate_registry(&registry);
    assert!(issues.is_empty(), "rule metadata issues: {issues:#?}");
}

#[test]
fn every_rule_has_documentation() {
    let registry = RuleRegistry::with_default_rules();
    let issues = validate_docs(&registry, &docs_root());
    assert!(issues.is_empty(), "documentation issues: {issues:#?}");
}

#[test]
fn every_rule_has_a_corpus_directory() {
    let registry = RuleRegistry::with_default_rules();
    let ids: BTreeSet<String> = registry
        .metadata_sorted()
        .iter()
        .map(|m| m.id.to_string())
        .collect();
    for id in &ids {
        let dir = corpus_dir().join(id);
        assert!(
            dir.join("positive.rs").exists(),
            "{id}: missing positive.rs"
        );
        assert!(
            dir.join("negative.rs").exists(),
            "{id}: missing negative.rs"
        );
    }
}

#[test]
fn rules_can_be_selected_individually() {
    let registry = RuleRegistry::with_default_rules();
    let base = ScanConfig::default();

    for meta in registry.metadata_sorted() {
        let src = std::fs::read_to_string(corpus_dir().join(meta.id).join("positive.rs")).unwrap();
        let config = ScanConfig {
            rules: soroban_scan_core::config::RulesConfig {
                enabled: vec![meta.id.to_string()],
                ..Default::default()
            },
            ..base.clone()
        };
        let outcome = engine::scan_source_str(&src, "positive.rs", &config, &registry);
        assert!(
            outcome.findings.iter().all(|f| f.rule_id == meta.id),
            "{}: selection leaked other rules",
            meta.id
        );
        assert!(
            !outcome.findings.is_empty(),
            "{}: rule could not be selected",
            meta.id
        );
    }
}

#[test]
fn metadata_is_deterministic_across_registries() {
    let a = RuleRegistry::with_default_rules();
    let b = RuleRegistry::with_default_rules();
    let ja = serde_json::to_string(
        &a.metadata_sorted()
            .iter()
            .map(|m| (m.id, m.default_severity, m.default_confidence))
            .collect::<Vec<_>>(),
    )
    .unwrap();
    let jb = serde_json::to_string(
        &b.metadata_sorted()
            .iter()
            .map(|m| (m.id, m.default_severity, m.default_confidence))
            .collect::<Vec<_>>(),
    )
    .unwrap();
    assert_eq!(ja, jb);
}
