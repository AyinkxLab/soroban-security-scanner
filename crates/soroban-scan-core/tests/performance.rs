//! Bounded-performance regression tests for pathological inputs.
//!
//! These guard against algorithmic regressions (for example quadratic helper
//! expansion) and non-termination on adversarial source. They assert generous
//! upper bounds so they are not flaky on slow CI machines.

use std::time::{Duration, Instant};

use soroban_scan_core::config::ScanConfig;
use soroban_scan_core::engine;
use soroban_scan_core::registry::RuleRegistry;

fn scan(src: &str) -> engine::ScanOutcome {
    let registry = RuleRegistry::with_default_rules();
    engine::scan_source_str(src, "performance.rs", &ScanConfig::default(), &registry)
}

#[test]
fn deeply_chained_helpers_terminate_quickly() {
    // A long call chain exercises intra-file helper summaries. The walk is
    // bounded by depth and a visited set, so this must be fast and terminate.
    let n = 1_000;
    let mut src = String::from("#[contractimpl]\nimpl C {\n");
    src.push_str("    pub fn entry(env: Env) { Self::h0(&env); }\n");
    for i in 0..n {
        if i + 1 < n {
            src.push_str(&format!(
                "    fn h{i}(env: &Env) {{ Self::h{}(env); }}\n",
                i + 1
            ));
        } else {
            src.push_str(&format!("    fn h{i}(_env: &Env) {{ }}\n"));
        }
    }
    src.push_str("}\n");

    let start = Instant::now();
    let _ = scan(&src);
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_secs(20),
        "chained-helper scan took too long: {elapsed:?}"
    );
}

#[test]
fn many_functions_with_cross_calls_scale() {
    // Guards against recomputing the whole-file summary for every entry point.
    let n = 1_500;
    let mut src = String::from("#[contractimpl]\nimpl C {\n");
    for i in 0..n {
        src.push_str(&format!(
            "    pub fn f{i}(env: Env) {{ Self::helper_{i}(&env); env.storage().persistent().set(&K{i}, &{i}); }}\n"
        ));
        src.push_str(&format!(
            "    fn helper_{i}(env: &Env) {{ let _ = env; }}\n"
        ));
    }
    src.push_str("}\n");

    let start = Instant::now();
    let _ = scan(&src);
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_secs(30),
        "many-function scan took too long: {elapsed:?}"
    );
}
