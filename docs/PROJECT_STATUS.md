# Project Status

This file is the authoritative progress tracker for the Soroban Security Scanner.
It is updated after every completed task so that a future session can resume
without ambiguity.

**Last updated:** 2026-09-21

**Repository:** https://github.com/AyinkxLab/soroban-security-scanner

**Default branch:** `main`

**Current phase:** Phase 9 — Developer Experience

---

## Phase overview

| Phase | Title | Status |
| ----- | ----- | ------ |
| 1 | Foundation and Architecture | Complete |
| 2 | Source Parsing and Project Intelligence | Complete |
| 3 | Security Analysis Engine | Complete |
| 4 | Initial Soroban Security Detectors | Complete |
| 5 | Rule Quality and Research Infrastructure | Complete |
| 6 | CLI and Reporting | Complete |
| 7 | Baselines, Incremental Scanning and CI | Complete |
| 8 | GitHub and Developer Integrations | Complete |
| 9 | Developer Experience | In progress |
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

### Phase 5 — Rule Quality and Research Infrastructure (complete)

- [x] Rule metadata validation (`rules::validation`): id shape, description
      length, remediation, references, and rejection of `critical` + `high`
      combinations for heuristics.
- [x] Registry validation and documentation validation (every rule must have
      `docs/rules/detectors/<ID>.md`).
- [x] Rule-quality integration tests: metadata validity, docs presence, corpus
      presence, per-rule selection isolation, deterministic metadata.
- [x] Adversarial test suite: empty/whitespace/comment-only sources, deep
      nesting, very long lines, unicode identifiers, 2,000 entry points, and
      repeated-scan byte-identity.
- [x] **Security hardening:** added a pre-parse delimiter-nesting guard after
      discovering that deeply nested input could overflow the parser stack
      (a denial-of-service risk on untrusted input).
- [x] Dependency-free benchmark example with documented methodology and
      baseline (`docs/development/benchmarks.md`).
- [x] Rule template (`docs/rules/TEMPLATE.md`) and contributor guide
      (`docs/contributors/writing-rules.md`).

### Phase 6 — CLI and Reporting (complete)

- [x] Reporting layer in core: terminal (full + compact), JSON, SARIF 2.1.0, and
      Markdown renderers, with tests validating structure.
- [x] `scan` command: directory or single-file scanning, format selection,
      file output, min severity/confidence, excludes, rule enable/disable,
      `--fail-on` threshold, quiet/verbose.
- [x] `rules` command with category and severity filters.
- [x] `explain <RULE-ID>` command (terminal/JSON/Markdown).
- [x] `init` command writing a documented config template.
- [x] `config show` and `config validate` commands.
- [x] Strict config loading with CLI override support and unknown-rule detection.
- [x] Stable, documented exit codes (0/1/2/3).
- [x] [`docs/cli.md`](cli.md) reference and README usage update.
- [x] 10 black-box CLI integration tests.

### Phase 7 — Baselines, Incremental Scanning and CI (complete)

- [x] Baseline format (`baseline.rs`): line-independent fingerprints, sorted
      deterministic entries, version validation, and untrusted-input parsing.
- [x] Baseline comparison: new, existing, and resolved findings.
- [x] CLI baseline flags: `--baseline`, `--write-baseline`, `--new-only`,
      `--incremental`. The failure gate considers only new findings when a
      baseline is supplied.
- [x] Incremental scanning via single-file/subset scans with resolved reporting
      disabled.
- [x] `examples/clean-contract` (passes the gate) and
      `examples/vulnerable-contract` (intentional findings).
- [x] GitHub Actions `security-scan.yml` workflow with SARIF upload.
- [x] Cross-platform machine output: paths serialize with forward slashes.
- [x] [`docs/baselines.md`](baselines.md) and CLI reference updates.
- [x] Baseline CLI integration tests.

### Phase 8 — GitHub and Developer Integrations (complete)

- [x] Composite GitHub Action (`action.yml`) with inputs for path, format,
      output, fail-on, baseline, files-from, and extra args.
- [x] `--files-from` CLI option for changed-file analysis, with traversal
      rejection.
- [x] `pr-scan.yml` workflow: changed-file analysis, high/critical annotations,
      SARIF generation, guarded code-scanning upload, and opt-in PR comments.
- [x] SARIF upload compatibility exercised structurally; code-scanning upload
      guarded for fork pull requests.
- [x] Documented permissions, fork-PR safety, baseline synchronization, and an
      opt-in issue-creation pattern in [`docs/github-integration.md`](github-integration.md).
- [x] CLI integration tests for `--files-from` and traversal rejection.

---

## Verified environment

- OS: Windows (win32), shell PowerShell 5.1.
- Rust: 1.97.1, toolchain `stable-x86_64-pc-windows-gnu` (local directory override).
- crates.io reachable.
- `cargo build`, `cargo test`, `cargo fmt --check`, `cargo clippy -D warnings` pass.

## Test summary

- Unit tests: 102
- Integration tests: 32 (CLI 14, corpus 3, rule quality 5, adversarial 6,
  project detection 4)
- Total: 134 (all passing)

## CI status

- GitHub Actions `CI` workflow has passed on every push (format, clippy, tests
  on Linux/Windows/macOS, and `cargo-deny`).
- GitHub Actions `Security Scan` workflow passed, including SARIF upload to
  GitHub code scanning.
- `PR Security Scan` is authored and will run on the next pull request.

## Performance

Release throughput on a developer machine: ~2,500-2,800 entry points/s
(~700 KB/s of source). See `docs/development/benchmarks.md`.

## Known limitations

- Detectors are syntactic; no type resolution or cross-function data flow.
- SS-002, SS-003, SS-006, SS-007 are heuristic and low/medium confidence.
- The `PR Security Scan` workflow has not yet run (no pull request has been
  opened); its underlying commands are covered by local tests.
- Only seven detectors exist; the roadmap covers many more.

## Architecture decisions

- [ADR-0001: Rust core with Python supporting tooling](architecture/adr/0001-rust-core-python-tooling.md)

## Next task

Phase 9: developer experience — interactive explanations, richer reports,
configuration generator, examples, and the full developer documentation set.
