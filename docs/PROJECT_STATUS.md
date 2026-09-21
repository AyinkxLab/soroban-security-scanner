# Project Status

This file is the authoritative progress tracker for the Soroban Security Scanner.
It is updated after every completed task so that a future session can resume
without ambiguity.

**Last updated:** 2026-09-21

**Repository:** https://github.com/AyinkxLab/soroban-security-scanner

**Default branch:** `main`

**Current phase:** Phase 5 — Rule Quality and Research Infrastructure

---

## Phase overview

| Phase | Title | Status |
| ----- | ----- | ------ |
| 1 | Foundation and Architecture | Complete |
| 2 | Source Parsing and Project Intelligence | Complete |
| 3 | Security Analysis Engine | Complete |
| 4 | Initial Soroban Security Detectors | Complete |
| 5 | Rule Quality and Research Infrastructure | In progress |
| 6 | CLI and Reporting | Not started |
| 7 | Baselines, Incremental Scanning and CI | Not started |
| 8 | GitHub and Developer Integrations | Not started |
| 9 | Developer Experience | Not started |
| 10 | Advanced Analysis and Optional AI | Not started |
| 11 | Production Hardening, Documentation and Release | Not started |

---

## Completed tasks

### Phase 1 — Foundation (complete)

- [x] Repository created under `AyinkxLab`.
- [x] Cargo workspace: `soroban-scan-core` (lib), `soroban-scan-cli` (bin).
- [x] Core `ScanError` type and version helpers.
- [x] CLI bootstrap exposing `soroban-scan version`.
- [x] Governance docs: README, LICENSE, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY,
      CHANGELOG.
- [x] Architecture overview and ADR-0001.
- [x] Security model and threat model.
- [x] Rule documentation and contributor roadmap.
- [x] CI workflow, issue templates, PR template, pre-commit, cargo-deny.

### Phase 2 — Source Parsing and Project Intelligence (complete)

- [x] Serializable project metadata model (`Project`, `Manifest`, `Dependency`,
      `Diagnostic`, `ProjectKind`).
- [x] Deterministic filesystem discovery with symlink safety and exclude globs.
- [x] `Cargo.toml` parsing (package, workspace, dependencies, workspace and
      target-specific dependencies).
- [x] Evidence-based Soroban classification: `generic_rust`, `likely_soroban`,
      `confirmed_soroban`.
- [x] `syn`-based source parsing with span locations, function/impl/trait/item
      inventory, module paths, and contract entry-point detection.
- [x] Graceful degradation: malformed manifests and unparseable sources become
      diagnostics, not crashes.
- [x] Fixtures: `generic-rust`, `soroban-likely`, `soroban-confirmed`.
- [x] Integration test asserting deterministic metadata output.

### Phase 3 — Security Analysis Engine (complete)

- [x] Severity model (`info`..`critical`) with stable ordering and parsing.
- [x] Confidence model (`low`, `medium`, `high`).
- [x] Category model (authorization, storage, arithmetic, ...).
- [x] `SourceLocation` and `Finding` models with stable, line-independent
      fingerprints for baselines.
- [x] `RuleMetadata` + `Rule` trait; rules observe an immutable context.
- [x] `RuleRegistry` with id validation and duplicate rejection.
- [x] `AnalysisContext` exposing project, sources, functions, and contract entry
      points.
- [x] `ScanConfig`/`RulesConfig` (enable/disable, severity/confidence overrides,
      minimum thresholds, excludes) with strict TOML parsing.
- [x] Engine pipeline: run enabled rules, relativize and validate every location
      against the scan root and analyzed file set, reject rule-fabricated
      locations and evidence-less findings, apply overrides and filters,
      de-duplicate, and sort deterministically.

### Phase 4 — Initial Soroban Security Detectors (complete)

Seven detectors, each with metadata, remediation, positive/negative fixtures,
tests, and documentation under `docs/rules/detectors/`:

- [x] SS-001 State-changing entry point without caller authorization (high/medium).
- [x] SS-002 Cross-contract call without caller authorization (high/low).
- [x] SS-003 Unbounded storage iteration in an entry point (medium/low).
- [x] SS-004 Panic-prone construct in a contract entry point (low/medium).
- [x] SS-005 Unsafe code in a Soroban contract (medium/high).
- [x] SS-006 Persistent storage write without TTL management (low/low).
- [x] SS-007 Hardcoded Stellar account or contract address (medium/low).

Supporting work:

- [x] AST utility layer (`rules::util`) extracting calls, macros, strings, and
      `unsafe` facts; contract entry-point discovery.
- [x] Public in-memory scanning API (`engine::scan_source_str`,
      `engine::synthetic_loaded`).
- [x] Security regression corpus under `fixtures/corpus/` with a data-driven
      integration test over all rule ids.
- [x] Adversarial test: malformed source produces diagnostics, not panics.

---

## Verified environment

- OS: Windows (win32), shell PowerShell 5.1.
- Rust: 1.97.1, toolchain `stable-x86_64-pc-windows-gnu` (local directory override).
- crates.io reachable.
- `cargo build`, `cargo test`, `cargo fmt --check`, `cargo clippy -D warnings` pass.

## Test summary

- Unit tests: 83
- Corpus / integration tests: 7
- Total: 90 (all passing)

## Known limitations

- The CLI exposes only `version`; reporting is implemented in Phase 6.
- Detectors are syntactic; no type resolution or cross-function data flow.
- SS-002, SS-003, SS-006, SS-007 are heuristic and low/medium confidence.

## Architecture decisions

- [ADR-0001: Rust core with Python supporting tooling](architecture/adr/0001-rust-core-python-tooling.md)

## Next task

Phase 5: rule metadata validation, documentation validation, a regression runner,
benchmark fixtures, determinism tests, and a contributor template for new rules.
