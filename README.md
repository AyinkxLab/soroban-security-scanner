# Soroban Security Scanner

**Deterministic, evidence-based security analysis for Stellar Soroban smart contracts and Soroban-related Rust projects.**

`soroban-scan` statically analyzes Soroban/Rust source code and reports security
findings with stable rule identifiers, severity and confidence ratings, precise
source locations, evidence, and remediation guidance.

The core scanner is **deterministic** and **AI-independent**. It never executes,
deploys, signs, or mutates the code it analyzes.

> **Project status:** early development. See [`docs/PROJECT_STATUS.md`](docs/PROJECT_STATUS.md)
> for an honest, continuously updated description of what exists today.

---

## Why

Soroban smart contracts handle real value. Security review is still largely manual
and ad hoc. Generic Rust linters (`clippy`) do not understand Soroban semantics,
and general-purpose static analyzers do not model Soroban's authorization,
storage, and resource model.

`soroban-scan` fills that gap with focused, Soroban-aware detectors, a
deterministic rule engine, and CI-friendly machine-readable output (JSON, SARIF).

## Goals

- Detect real Soroban security problems (authorization, arithmetic, storage,
  resource usage, unsafe patterns, and more).
- Produce reproducible, evidence-based findings — never fabricated ones.
- Integrate cleanly with CI and GitHub code scanning via SARIF.
- Stay useful with no network access and no AI.
- Be safe to run on untrusted repositories.

## Non-goals

- The scanner does not replace professional security audits.
- The scanner does not execute or deploy contracts.
- The scanner does not require private keys, wallets, or network access.
- AI is never the authoritative security engine.

## Installation

> Not yet published. Build from source:

```bash
git clone https://github.com/AyinkxLab/soroban-security-scanner
cd soroban-security-scanner
cargo build --release
./target/release/soroban-scan --help
```

Requires a stable Rust toolchain. See [`docs/development/setup.md`](docs/development/setup.md).

## Usage

> The CLI is under active development. Current bootstrap command:

```bash
soroban-scan version
```

Scanning, rules listing, and reporting commands are implemented in later phases.
See [`docs/PROJECT_STATUS.md`](docs/PROJECT_STATUS.md).

## Architecture

```
crates/
  soroban-scan-core   # deterministic analysis engine (library)
  soroban-scan-cli    # `soroban-scan` binary
tools/                # supporting tooling (Python)
fixtures/             # positive/negative security fixtures and corpus
docs/                 # architecture, development, security, rules, contributors
```

See [`docs/architecture/overview.md`](docs/architecture/overview.md) and the
[architecture decision records](docs/architecture/adr/).

## Security model

The scanner treats every scanned repository as **hostile input**. It does not
require or store keys, does not sign or submit transactions, does not execute
scanned code, and makes no network requests by default. See
[`SECURITY.md`](SECURITY.md) and [`docs/security/threat-model.md`](docs/security/threat-model.md).

## Contributing

Contributions are welcome. Start with [`CONTRIBUTING.md`](CONTRIBUTING.md) and the
[contributor roadmap](docs/contributors/roadmap.md).

## License

[MIT](LICENSE) © 2026 AyinkxLab
