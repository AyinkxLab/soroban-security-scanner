# ADR-0001: Rust core with Python supporting tooling

- **Status:** Accepted
- **Date:** 2026-09-21
- **Deciders:** AyinkxLab

## Context

We are building a static security scanner for Stellar Soroban smart contracts.
Soroban contracts are written in Rust using the `soroban-sdk`. We need a core
analyzer that:

- Parses Rust source accurately, including attributes, macros, and spans.
- Runs deterministically and fast enough for CI on real repositories.
- Ships as a single, dependency-light binary that developers can install easily.
- Is safe to run on untrusted code with no network access.

We evaluated the surrounding ecosystem:

- **Stellar/Soroban tooling** — `stellar-cli` (formerly `soroban-cli`), the
  `soroban-sdk`, and Rust-based contract templates. The whole contract ecosystem
  is Rust.
- **Existing security tooling** — general Rust linters (`clippy`) and manual
  audit practice. General-purpose analyzers do not model Soroban's authorization,
  storage, or resource semantics.
- **SARIF tooling** — SARIF 2.1.0 is the interchange format understood by
  GitHub code scanning and many IDEs.

## Decision

1. **Core analyzer in Rust.** Parsing Rust is most reliable with `syn`,
   Rust's canonical parser. Rust also gives predictable performance, a single
   static binary, and alignment with the Soroban ecosystem our users already
   have installed.
2. **Python for supporting tooling only.** Python is used for out-of-band
   automation (for example, generating and validating the contributor backlog,
   maintaining the fixture corpus, and release scripts). Python is never on the
   critical scanning path.
3. **Minimal, justified dependencies.** Each dependency must earn its place.
   Parsing uses `syn`/`proc-macro2`; serialization uses `serde`; CLI uses
   `clap`. Lockfiles are committed.

## Alternatives considered

- **Python core analyzer.** Familiar and fast to prototype, but weak for
  accurate Rust AST parsing, slower on large repositories, and harder to
  distribute as a single binary. Rejected for the core; retained for tooling.
- **Node/TypeScript core.** No strong advantage for parsing Rust. Rejected.
- **Building on an existing analyzer.** Existing tools either target other
  languages or do not model Soroban semantics. We may integrate outputs later,
  but not as the core engine.

## Consequences

- Contributors need a Rust toolchain; this matches the target audience.
- We own parsing and rule implementation, giving full control over determinism
  and evidence quality.
- Supporting automation can evolve independently in Python without destabilizing
  the scanner.
- We must be disciplined about dependency growth; `deny.toml` and review enforce
  this.
