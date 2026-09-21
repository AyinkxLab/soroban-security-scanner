# Project Status

This file is the authoritative progress tracker for the Soroban Security Scanner.
It is updated after every completed task so that a future session can resume
without ambiguity.

**Last updated:** 2026-09-21

**Repository:** https://github.com/AyinkxLab/soroban-security-scanner

**Default branch:** `main`

**Current phase:** Phase 3 — Security Analysis Engine

---

## Phase overview

| Phase | Title | Status |
| ----- | ----- | ------ |
| 1 | Foundation and Architecture | Complete |
| 2 | Source Parsing and Project Intelligence | Complete |
| 3 | Security Analysis Engine | In progress |
| 4 | Initial Soroban Security Detectors | Not started |
| 5 | Rule Quality and Research Infrastructure | Not started |
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

---

## Verified environment

- OS: Windows (win32), shell PowerShell 5.1.
- Rust: 1.97.1, toolchain `stable-x86_64-pc-windows-gnu` (local directory override).
- crates.io reachable.
- `cargo build`, `cargo test`, `cargo fmt --check`, `cargo clippy -D warnings` pass.

## Test summary

- Unit tests: 33
- Integration tests: 4
- Total: 37 (all passing)

## Known limitations

- No security rules or finding output yet.
- The CLI exposes only `version`.
- Source parsing is purely syntactic (no type resolution).

## Architecture decisions

- [ADR-0001: Rust core with Python supporting tooling](architecture/adr/0001-rust-core-python-tooling.md)

## Next task

Phase 3: implement the finding model, rule trait, rule registry, analysis
context, scanner pipeline, and diagnostics.
