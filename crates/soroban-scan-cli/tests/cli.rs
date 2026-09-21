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
