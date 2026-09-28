# Benchmarks

The scanner's performance is measured with a dependency-free in-process
benchmark. This keeps the measurement reproducible and avoids adding a benchmark
framework before it is needed.

## Running

```bash
cargo run --release -p soroban-scan-core --example benchmark
```

The benchmark generates synthetic `#[contractimpl]` sources with 100, 1,000,
5,000, and 10,000 entry points, and reports throughput.

## Methodology

- The source is generated in memory, so the numbers isolate parsing and detector
  cost from filesystem I/O.
- The scanner runs all built-in rules.
- A warm-up scan precedes each measurement.

## Baseline (illustrative)

Measured on a developer Windows machine with Rust 1.97.1. Treat these as
order-of-magnitude guidance, not guarantees.

| Entry points | Source size | Release time | Release rate |
| -----------: | ----------: | -----------: | -----------: |
| 100 | 24.6 KB | 39.5 ms | ~2,530/s |
| 1,000 | 246.9 KB | 404.7 ms | ~2,470/s |
| 5,000 | 1.24 MB | 1.83 s | ~2,730/s |
| 10,000 | 2.48 MB | 3.61 s | ~2,770/s |

Debug builds are roughly 5-8× slower; always benchmark with `--release`.

## End-to-end (CLI) benchmark

The in-process example measures parsing and detection on synthetic sources. To
measure the **combined** system — project discovery, parsing, scanning and
report generation — time the real release binary:

```bash
cargo build --release -p soroban-scan-cli
python tools/benchmark.py --target examples/vulnerable-contract --runs 10
```

The script performs one warm-up run, then reports min/median/mean/max over the
requested runs. It uses only the Python standard library. Compare before/after on
the same machine, the same target and the same profile, and always benchmark a
`--release` build.

## Pathological-input regression

`crates/soroban-scan-core/tests/performance.rs` asserts generous upper time
bounds for adversarial inputs (deeply chained helpers, many entry points with
cross-calls). These guard against algorithmic regressions — such as quadratic
helper-summary expansion — and are not precise benchmarks.

## Interpreting results

- Cost scales linearly with source size, which is expected for a
  parse-and-visit pipeline.
- The dominant cost is Rust parsing (`syn`). Detector evaluation is negligible
  by comparison.
- Optimize parsing and traversal before micro-optimizing individual rules.

## Adding measurements

If a change plausibly affects performance:

1. Record the "before" numbers from a clean checkout.
2. Apply the change.
3. Record the "after" numbers on the same machine, same profile.
4. Include both in the pull request.

Do not optimize speculatively: measure first.
