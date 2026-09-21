//! Adversarial and pathological input tests.
//!
//! The scanner must never panic, hang, or fabricate findings on hostile input.

use std::time::{Duration, Instant};

use soroban_scan_core::config::ScanConfig;
use soroban_scan_core::engine;
use soroban_scan_core::registry::RuleRegistry;

fn scan(src: &str) -> engine::ScanOutcome {
    let registry = RuleRegistry::with_default_rules();
    engine::scan_source_str(src, "adversarial.rs", &ScanConfig::default(), &registry)
}

#[test]
fn empty_and_whitespace_sources_are_safe() {
    for src in ["", "   \n\t  ", "// just a comment\n", "/* block */\n"] {
        let outcome = scan(src);
        assert!(outcome.findings.is_empty());
    }
}

#[test]
fn deeply_nested_structure_does_not_panic() {
    let depth = 200;
    let src = format!("fn f() {{ {} 0 {} }}", "(".repeat(depth), ")".repeat(depth));
    let outcome = scan(&src);
    assert!(outcome.findings.is_empty());
}

#[test]
fn very_long_line_does_not_panic() {
    let src = format!("fn f() {{ let x = \"{}\"; }}", "a".repeat(100_000));
    let outcome = scan(&src);
    assert!(outcome.findings.is_empty());
}

#[test]
fn unicode_identifiers_do_not_panic() {
    let src = "#[contractimpl]\nimpl C { pub fn \u{03b2}(\u{03b1}: u32) -> u32 { \u{03b1} } }\n";
    let _ = scan(src);
}

#[test]
fn many_entry_points_complete_quickly() {
    let mut src = String::from("#[contractimpl]\nimpl C {\n");
    for i in 0..2_000 {
        src.push_str(&format!(
            "    pub fn f{i}(env: Env) {{ env.storage().persistent().set(&K{i}, &{i}); }}\n"
        ));
    }
    src.push_str("}\n");

    let start = Instant::now();
    let outcome = scan(&src);
    let elapsed = start.elapsed();

    assert!(!outcome.findings.is_empty());
    assert!(
        elapsed < Duration::from_secs(30),
        "scan took too long: {elapsed:?}"
    );
}

#[test]
fn repeated_scans_are_byte_identical() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/corpus/SS-001/positive.rs"
    ))
    .unwrap();
    let registry = RuleRegistry::with_default_rules();
    let config = ScanConfig::default();
    let first = serde_json::to_string(
        &engine::scan_source_str(&src, "positive.rs", &config, &registry).findings,
    )
    .unwrap();
    for _ in 0..5 {
        let again = serde_json::to_string(
            &engine::scan_source_str(&src, "positive.rs", &config, &registry).findings,
        )
        .unwrap();
        assert_eq!(first, again);
    }
}
