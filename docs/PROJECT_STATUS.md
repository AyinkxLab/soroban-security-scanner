# Project Status

This file is the authoritative progress tracker for the Soroban Security Scanner.
It is updated after every completed task so that a future session can resume
without ambiguity.

**Last updated:** 2026-09-21

**Repository:** https://github.com/AyinkxLab/soroban-security-scanner

**Default branch:** `main`

**Current phase:** Phase 1 — Foundation and Architecture (in progress)

---

## Phase overview

| Phase | Title | Status |
| ----- | ----- | ------ |
| 1 | Foundation and Architecture | In progress |
| 2 | Source Parsing and Project Intelligence | Not started |
| 3 | Security Analysis Engine | Not started |
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

### Phase 1 (partial)

- [x] Created repository `AyinkxLab/soroban-security-scanner` under the org.
- [x] Cargo workspace with `soroban-scan-core` and `soroban-scan-cli`.
- [x] Core `ScanError` type and version helpers (with unit tests).
- [x] CLI bootstrap exposing `soroban-scan version`.
- [x] `.gitignore`, `.gitattributes`, `.editorconfig`, `rustfmt.toml`, `clippy.toml`.
- [x] `README.md`, `LICENSE`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`,
      `SECURITY.md`, `CHANGELOG.md`.
- [x] Architecture docs and ADR-0001.
- [x] Security model and threat model docs.
- [x] CI workflow and GitHub templates.

---

## Verified environment

- OS: Windows (win32), shell PowerShell 5.1.
- Rust: 1.97.1, toolchain `stable-x86_64-pc-windows-gnu` (local directory override).
- crates.io reachable.
- `cargo build`, `cargo test`, `cargo fmt --check`, `cargo clippy -D warnings` all pass.

## Known limitations

- No functional scanning yet; only the foundation exists.
- The CLI exposes only `version`.

## Architecture decisions

- [ADR-0001: Rust core with Python supporting tooling](architecture/adr/0001-rust-core-python-tooling.md)

## Next task

Continue Phase 1: finish development docs and tooling configuration, then begin
Phase 2 (source parsing and Soroban project intelligence).
