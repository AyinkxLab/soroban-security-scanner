# Testing Guide

Testing is a first-class requirement. Security tooling that is not tested is
worse than no tooling, because it creates false confidence.

## Test layers

| Layer | Location | Purpose |
| ----- | -------- | ------- |
| Unit | `#[cfg(test)]` modules next to code | Individual functions and rules |
| Integration | `crates/*/tests/` | End-to-end crate behavior |
| Fixtures | `fixtures/` | Positive/negative source samples |
| Regression | `fixtures/corpus/` | Pins expected findings to prevent regressions |
| Adversarial | `fixtures/` + parser tests | Malformed, hostile, and pathological input |

## Running tests

```bash
cargo test --all
```

Run a single test:

```bash
cargo test -p soroban-scan-core -- --nocapture some_test_name
```

## Rules for security-sensitive tests

- Every rule ships with at least one **positive** fixture (must trigger) and one
  **negative** fixture (must not trigger).
- False-positive tests are as important as true-positive tests.
- Parser features require malformed-input tests.
- Any future network/CI integration requires permission and error-path tests.

## Fixture conventions

Fixtures live under `fixtures/` and are grouped by purpose:

```
fixtures/
  projects/   # whole mini-projects (manifests + source)
  corpus/     # regression corpus fixtures
```

Each fixture directory should be self-describing, for example:

```
fixtures/corpus/authorization/missing-auth-positive/
  Cargo.toml
  src/lib.rs
  expected.json      # expected findings (rule ids, lines)
```

Exact formats are defined in Phase 5. Until then, fixtures are plain source
samples.

## Determinism

Tests assert that scanning the same fixture twice produces byte-identical
machine-readable output.
