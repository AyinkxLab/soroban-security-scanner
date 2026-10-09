//! Black-box CLI integration tests.

use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_soroban-scan"))
}

fn fixture(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(relative)
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

struct Output {
    stdout: String,
    stderr: String,
    code: i32,
}

fn run(args: &[&str]) -> Output {
    let result = Command::new(bin())
        .args(args)
        .output()
        .expect("failed to run soroban-scan");
    Output {
        stdout: String::from_utf8_lossy(&result.stdout).to_string(),
        stderr: String::from_utf8_lossy(&result.stderr).to_string(),
        code: result.status.code().unwrap_or(-1),
    }
}

#[test]
fn version_exits_zero_and_prints_name() {
    let out = run(&["version"]);
    assert_eq!(out.code, 0);
    assert!(out.stdout.contains("soroban-scan"));
    assert!(out.stdout.contains(soroban_scan_core::VERSION));
}

#[test]
fn rules_lists_built_in_rules() {
    let out = run(&["rules"]);
    assert_eq!(out.code, 0);
    assert!(out.stdout.contains("SS-001"));
    assert!(out.stdout.contains("SS-007"));
}

#[test]
fn explain_known_rule_succeeds() {
    let out = run(&["explain", "SS-001"]);
    assert_eq!(out.code, 0);
    assert!(out.stdout.contains("SS-001"));
    assert!(out.stdout.contains("Remediation"));
}

#[test]
fn explain_unknown_rule_is_usage_error() {
    let out = run(&["explain", "SS-999"]);
    assert_eq!(out.code, 2);
    assert!(out.stderr.contains("unknown rule id"));
}

#[test]
fn scan_positive_fixture_fails_the_gate() {
    let out = run(&[
        "scan",
        fixture("corpus/SS-001").to_str().unwrap(),
        "--quiet",
    ]);
    assert_eq!(out.code, 1);
    assert!(out.stdout.contains("SS-001"));
}

#[test]
fn scan_with_fail_on_none_exits_zero() {
    let out = run(&[
        "scan",
        fixture("corpus/SS-001").to_str().unwrap(),
        "--quiet",
        "--fail-on",
        "none",
    ]);
    assert_eq!(out.code, 0);
}

#[test]
fn scan_json_output_is_machine_readable() {
    let out = run(&[
        "scan",
        fixture("corpus/SS-001").to_str().unwrap(),
        "--format",
        "json",
        "--fail-on",
        "none",
    ]);
    assert_eq!(out.code, 0);
    let value: serde_json::Value = serde_json::from_str(&out.stdout).unwrap();
    assert_eq!(value["tool"]["name"], "soroban-scan");
    assert!(!value["findings"].as_array().unwrap().is_empty());
}

#[test]
fn scan_sarif_output_is_valid() {
    let out = run(&[
        "scan",
        fixture("corpus/SS-001").to_str().unwrap(),
        "--format",
        "sarif",
        "--fail-on",
        "none",
    ]);
    assert_eq!(out.code, 0);
    let value: serde_json::Value = serde_json::from_str(&out.stdout).unwrap();
    assert_eq!(value["version"], "2.1.0");
    assert_eq!(value["runs"][0]["tool"]["driver"]["name"], "soroban-scan");
}

#[test]
fn scan_missing_path_is_runtime_error() {
    let out = run(&["scan", "definitely/not/a/real/path/xyz"]);
    assert_eq!(out.code, 3);
}

#[test]
fn init_creates_then_refuses_to_overwrite() {
    let dir = std::env::temp_dir().join(format!("soroban-scan-cli-init-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let first = run(&["init", dir.to_str().unwrap()]);
    assert_eq!(first.code, 0);
    assert!(dir.join("soroban-scan.toml").exists());

    let second = run(&["init", dir.to_str().unwrap()]);
    assert_eq!(second.code, 2);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn baseline_suppresses_existing_findings() {
    let dir = std::env::temp_dir().join(format!("soroban-scan-cli-bl-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let baseline = dir.join("baseline.json");

    let write = run(&[
        "scan",
        fixture("corpus/SS-001").to_str().unwrap(),
        "--write-baseline",
        baseline.to_str().unwrap(),
        "--quiet",
    ]);
    assert_eq!(write.code, 0);
    assert!(baseline.exists());

    // Known findings no longer fail the high gate.
    let known = run(&[
        "scan",
        fixture("corpus/SS-001").to_str().unwrap(),
        "--baseline",
        baseline.to_str().unwrap(),
        "--fail-on",
        "high",
        "--quiet",
    ]);
    assert_eq!(known.code, 0);

    // new-only output is empty because everything is baselined.
    let new_only = run(&[
        "scan",
        fixture("corpus/SS-001").to_str().unwrap(),
        "--baseline",
        baseline.to_str().unwrap(),
        "--new-only",
        "--fail-on",
        "none",
        "--quiet",
    ]);
    assert_eq!(new_only.code, 0);
    assert!(new_only.stdout.contains("0 finding(s)"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn baseline_from_clean_project_flags_all_findings_as_new() {
    let dir = std::env::temp_dir().join(format!("soroban-scan-cli-bl2-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let baseline = dir.join("baseline.json");

    let write = run(&[
        "scan",
        fixture("projects/soroban-confirmed").to_str().unwrap(),
        "--write-baseline",
        baseline.to_str().unwrap(),
        "--quiet",
    ]);
    assert_eq!(write.code, 0);

    let out = run(&[
        "scan",
        fixture("corpus/SS-001").to_str().unwrap(),
        "--baseline",
        baseline.to_str().unwrap(),
        "--fail-on",
        "high",
        "--quiet",
    ]);
    assert_eq!(out.code, 1);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn files_from_scans_only_listed_files() {
    let dir = std::env::temp_dir().join(format!("soroban-scan-cli-ff-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let list = dir.join("changed.txt");
    std::fs::write(&list, "fixtures/corpus/SS-001/positive.rs\n").unwrap();

    let out = run(&[
        "scan",
        repo_root().to_str().unwrap(),
        "--files-from",
        list.to_str().unwrap(),
        "--quiet",
        "--fail-on",
        "high",
    ]);
    assert_eq!(out.code, 1);
    assert!(out.stdout.contains("positive.rs"));
    assert!(!out.stdout.contains("negative.rs"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn rules_search_filters_results() {
    let out = run(&["rules", "--search", "ttl"]);
    assert_eq!(out.code, 0);
    assert!(out.stdout.contains("SS-006"));
    assert!(!out.stdout.contains("SS-001"));
}

#[test]
fn rules_unknown_category_is_usage_error() {
    let out = run(&["rules", "--category", "nonsense"]);
    assert_eq!(out.code, 2);
    assert!(out.stderr.contains("unknown category"));
}

#[test]
fn guidance_is_offline_and_labeled() {
    let out = run(&[
        "scan",
        fixture("corpus/SS-001").to_str().unwrap(),
        "--fail-on",
        "none",
        "--quiet",
        "--guidance",
    ]);
    assert_eq!(out.code, 0);
    assert!(out.stderr.contains("GUIDANCE (offline, rule-based)"));
    assert!(out.stderr.contains("deterministic findings"));
}

#[test]
fn enabling_ai_without_a_provider_fails_closed() {
    let dir = std::env::temp_dir().join(format!("soroban-scan-cli-ai-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let config = dir.join("ai.toml");
    std::fs::write(&config, "[ai]\nenabled = true\nprovider = \"openai\"\n").unwrap();

    let out = run(&[
        "scan",
        fixture("corpus/SS-001").to_str().unwrap(),
        "--config",
        config.to_str().unwrap(),
        "--fail-on",
        "none",
        "--quiet",
        "--guidance",
    ]);
    assert_eq!(out.code, 2);
    assert!(out.stderr.contains("assistance is unavailable"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn files_from_rejects_path_traversal() {
    let dir = std::env::temp_dir().join(format!("soroban-scan-cli-ff2-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let list = dir.join("changed.txt");
    std::fs::write(&list, "../outside.rs\n").unwrap();

    let out = run(&[
        "scan",
        repo_root().to_str().unwrap(),
        "--files-from",
        list.to_str().unwrap(),
    ]);
    assert_eq!(out.code, 2);
    assert!(out.stderr.contains("escapes the scan root"));

    let _ = std::fs::remove_dir_all(&dir);
}

/// Writes a minimal Soroban project whose single entry point lacks
/// authorization. With `directive` set, an inline suppression comment sits above
/// the entry point.
fn write_suppression_project(dir: &std::path::Path, directive: bool) {
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"vault\"\nversion = \"0.1.0\"\n\n[dependencies]\nsoroban-sdk = \"21.0.0\"\n",
    )
    .unwrap();

    let mut src = String::new();
    src.push_str("use soroban_sdk::{contractimpl, symbol_short, Address, Env};\n\n");
    src.push_str("#[contractimpl]\n");
    src.push_str("impl Vault {\n");
    if directive {
        src.push_str("    // soroban-scan: ignore SS-001 -- reviewed by the security team\n");
    }
    src.push_str("    pub fn set_admin(env: Env, new_admin: Address) {\n");
    src.push_str(
        "        env.storage().persistent().set(&symbol_short!(\"ADMIN\"), &new_admin);\n",
    );
    src.push_str("    }\n");
    src.push_str("}\n");
    std::fs::write(dir.join("src/lib.rs"), src).unwrap();
}

fn rule_ids(value: &serde_json::Value) -> Vec<String> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| finding["rule_id"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn inline_suppressions_are_applied_and_reported() {
    let dir = std::env::temp_dir().join(format!(
        "soroban-scan-cli-suppression-{}",
        std::process::id()
    ));

    // Without a directive the finding is reported and fails the gate.
    write_suppression_project(&dir, false);
    let plain = run(&[
        "scan",
        dir.to_str().unwrap(),
        "--format",
        "json",
        "--rule",
        "SS-001",
        "--fail-on",
        "high",
    ]);
    assert_eq!(plain.code, 1, "an unsuppressed SS-001 must fail the gate");
    let value: serde_json::Value = serde_json::from_str(&plain.stdout).unwrap();
    assert!(rule_ids(&value["findings"]).contains(&"SS-001".to_string()));
    assert_eq!(value["stats"]["findings_suppressed"], 0);

    // With the directive the finding leaves the gate but is still reported.
    write_suppression_project(&dir, true);
    let suppressed = run(&[
        "scan",
        dir.to_str().unwrap(),
        "--format",
        "json",
        "--rule",
        "SS-001",
        "--fail-on",
        "high",
    ]);
    assert_eq!(
        suppressed.code, 0,
        "suppressed findings must not fail the gate: {}",
        suppressed.stdout
    );
    let value: serde_json::Value = serde_json::from_str(&suppressed.stdout).unwrap();
    assert_eq!(value["stats"]["suppressions_detected"], 1);
    assert_eq!(value["stats"]["findings_suppressed"], 1);
    assert!(!rule_ids(&value["findings"]).contains(&"SS-001".to_string()));
    assert_eq!(value["suppressed"][0]["rule_id"], "SS-001");
    assert_eq!(value["suppressed"][0]["suppression"]["file"], "src/lib.rs");
    assert_eq!(value["suppressed"][0]["suppression"]["rules"], "SS-001");
    assert_eq!(
        value["suppressed"][0]["suppression"]["reason"],
        "reviewed by the security team"
    );
    assert_eq!(value["suppressions"][0]["applied"], true);

    // The terminal report surfaces the count, the directive and the reason.
    let terminal = run(&[
        "scan",
        dir.to_str().unwrap(),
        "--rule",
        "SS-001",
        "--fail-on",
        "high",
    ]);
    assert_eq!(terminal.code, 0);
    assert!(
        terminal.stdout.contains("1 suppressed finding(s)"),
        "{}",
        terminal.stdout
    );
    assert!(terminal.stdout.contains("reviewed by the security team"));

    // SARIF keeps the result and marks it as an in-source suppression.
    let sarif = run(&[
        "scan",
        dir.to_str().unwrap(),
        "--format",
        "sarif",
        "--rule",
        "SS-001",
        "--fail-on",
        "high",
    ]);
    let value: serde_json::Value = serde_json::from_str(&sarif.stdout).unwrap();
    let results = value["runs"][0]["results"].as_array().unwrap();
    let marked = results
        .iter()
        .find(|result| result["ruleId"] == "SS-001")
        .expect("SS-001 result stays in SARIF");
    assert_eq!(marked["suppressions"][0]["kind"], "inSource");
    assert_eq!(marked["suppressions"][0]["status"], "accepted");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn no_config_ignores_discovered_configuration() {
    let dir = std::env::temp_dir().join(format!("soroban-scan-cli-noconf-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);

    // A minimal project with an SS-001 finding plus a repository-provided
    // configuration that would hide it.
    write_suppression_project(&dir, false);
    let repo_config = dir.join("soroban-scan.toml");
    std::fs::write(&repo_config, "min_severity = \"critical\"\n").unwrap();

    // The discovered configuration filters the high-severity finding out.
    let discovered = run(&[
        "scan",
        dir.to_str().unwrap(),
        "--format",
        "json",
        "--rule",
        "SS-001",
        "--fail-on",
        "high",
    ]);
    assert_eq!(discovered.code, 0, "{}", discovered.stdout);
    let value: serde_json::Value = serde_json::from_str(&discovered.stdout).unwrap();
    assert!(
        value["findings"].as_array().unwrap().is_empty(),
        "the repository configuration must be applied by default: {}",
        discovered.stdout
    );

    // --no-config ignores it: the finding is reported and fails the gate.
    let ignored = run(&[
        "scan",
        dir.to_str().unwrap(),
        "--no-config",
        "--format",
        "json",
        "--rule",
        "SS-001",
        "--fail-on",
        "high",
    ]);
    assert_eq!(ignored.code, 1, "{}", ignored.stdout);
    let value: serde_json::Value = serde_json::from_str(&ignored.stdout).unwrap();
    assert!(rule_ids(&value["findings"]).contains(&"SS-001".to_string()));

    // It cannot be combined with --config.
    let both = run(&[
        "scan",
        dir.to_str().unwrap(),
        "--no-config",
        "--config",
        repo_config.to_str().unwrap(),
    ]);
    assert_eq!(both.code, 2, "{}", both.stderr);
    assert!(both.stderr.contains("--no-config"));

    let _ = std::fs::remove_dir_all(&dir);
}
