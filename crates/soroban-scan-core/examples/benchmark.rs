//! Lightweight in-process benchmark for detector throughput.
//!
//! Run with:
//!
//! ```text
//! cargo run --release -p soroban-scan-core --example benchmark
//! ```
//!
//! This is intentionally dependency-free and is not part of the test gate. It
//! exists so contributors can measure the effect of detector changes.

use std::time::Instant;

use soroban_scan_core::config::ScanConfig;
use soroban_scan_core::engine;
use soroban_scan_core::registry::RuleRegistry;

fn generate(entry_points: usize) -> String {
    let mut src = String::from("#[contractimpl]\nimpl Bench {\n");
    for i in 0..entry_points {
        src.push_str(&format!(
            "    pub fn f{i}(env: Env, who: Address, amount: i128) {{\n\
             \x20       who.require_auth();\n\
             \x20       let key = symbol_short!(\"K\");\n\
             \x20       env.storage().persistent().set(&key, &amount);\n\
             \x20       env.storage().persistent().extend_ttl(&key, 100, 1000);\n\
             \x20   }}\n"
        ));
    }
    src.push_str("}\n");
    src
}

fn main() {
    let registry = RuleRegistry::with_default_rules();
    let config = ScanConfig::default();

    println!(
        "{:>12} {:>10} {:>10} {:>10} {:>12}",
        "entry_pts", "bytes", "findings", "files/s", "time"
    );

    for entry_points in [100usize, 1_000, 5_000, 10_000] {
        let src = generate(entry_points);

        // Warm-up.
        let _ = engine::scan_source_str(&src, "bench.rs", &config, &registry);

        let start = Instant::now();
        let outcome = engine::scan_source_str(&src, "bench.rs", &config, &registry);
        let elapsed = start.elapsed();

        let per_second = entry_points as f64 / elapsed.as_secs_f64().max(f64::EPSILON);
        println!(
            "{:>12} {:>10} {:>10} {:>10.0} {:>12?}",
            entry_points,
            src.len(),
            outcome.findings.len(),
            per_second,
            elapsed
        );
    }
}
