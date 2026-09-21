# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-09-21

First MVP release. Deterministic, evidence-based security analysis for Stellar
Soroban smart contracts and Soroban-related Rust projects.

### Added

**Core engine**

- Cargo workspace with `soroban-scan-core` (library) and `soroban-scan-cli`
  (binary).
- Deterministic filesystem discovery with symlink safety, ignored directories,
  and exclude globs.
- `Cargo.toml` parsing (package, workspace, dependency tables, workspace and
  target dependencies).
- Evidence-based Soroban classification: `generic_rust`, `likely_soroban`,
  `confirmed_soroban`.
- `syn`-based source parsing with source locations, item/function inventory, and
  contract entry-point detection.
- Pre-parse delimiter-nesting guard that prevents parser stack overflow on
  hostile input.
- Finding model with severity, confidence, category, location, evidence,
  remediation, references, and line-independent fingerprints.
- Rule trait, registry with id validation, and an immutable analysis context.
- Scan pipeline that validates every finding location against the scan root and
  analyzed file set, rejecting fabricated locations and evidence-less findings.
- Scan configuration (rule enable/disable, severity/confidence overrides,
  thresholds, excludes) with strict TOML parsing.

**Detectors**

- SS-001 State-changing entry point without caller authorization.
- SS-002 Cross-contract call without caller authorization.
- SS-003 Unbounded storage iteration in an entry point.
- SS-004 Panic-prone construct in a contract entry point.
- SS-005 Unsafe code in a Soroban contract.
- SS-006 Persistent storage write without TTL management.
- SS-007 Hardcoded Stellar account or contract address.

**CLI and reporting**

- `scan`, `rules`, `explain`, `version`, `init`, and `config show/validate`.
- Terminal, JSON, SARIF 2.1.0, and Markdown output.
- Stable exit codes (0/1/2/3).
- Single-file, directory, and changed-file (`--files-from`) scanning.
- Baselines: `--baseline`, `--write-baseline`, `--new-only`, `--incremental`.

**Integrations**

- Composite GitHub Action.
- Security Scan workflow with SARIF upload to GitHub code scanning.
- PR changed-file workflow with annotations and opt-in comments.

**Assistance**

- Optional, clearly labeled, offline guidance (`--guidance`). Disabled by
  default and fails closed when a provider is unavailable.

**Quality and infrastructure**

- Security regression corpus with positive/negative fixtures per rule.
- Rule-quality validation (metadata and documentation).
- Adversarial test suite (malformed, deep, large, unicode, scale).
- Dependency-free benchmark example with documented baselines.
- CI: formatting, clippy (`-D warnings`), tests on Linux/Windows/macOS, and
  `cargo-deny`.
- Comprehensive documentation: getting started, installation, CLI,
  configuration, rules, baselines, GitHub integration, security model, threat
  model, compatibility, release, upgrading, and audit.

[Unreleased]: https://github.com/AyinkxLab/soroban-security-scanner/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/AyinkxLab/soroban-security-scanner/releases/tag/v0.1.0
