# Compatibility

## Rust toolchain

- **Policy:** a recent stable Rust toolchain. CI builds and tests on stable for
  Linux, Windows, and macOS.
- **Verified with:** Rust 1.97.1.
- **MSRV:** not currently guaranteed. Historical toolchains could not be
  verified in the development environment, so no `rust-version` is declared. A
  pinned, tested MSRV is tracked as future work. Do not rely on a specific
  minimum until it is declared and CI-tested.

## Operating systems

| OS | Status |
| -- | ------ |
| Linux | Supported (CI tested) |
| macOS | Supported (CI tested) |
| Windows | Supported (CI tested; MSVC and GNU toolchains) |

## Soroban SDK

- **Detection:** version-agnostic. Any `soroban-sdk` (or `soroban-*` /
  `stellar-*`) dependency is recognized, and `#[contract]` / `#[contractimpl]`
  are used to confirm contract projects.
- **Detector targeting:** the built-in detectors model the modern Soroban SDK
  storage and authorization APIs (`env.storage().persistent()/instance()/temporary()`,
  `Address::require_auth`, generated `*Client` constructors). These correspond to
  SDK v20 and later.
- **Verified against fixtures using** `soroban-sdk = "21.0.0"`.

## Machine-readable formats

| Format | Version | Stability |
| ------ | ------- | --------- |
| JSON report | tool version | Stable shape; additive changes only within a minor line. |
| SARIF | 2.1.0 | Stable. |
| Baseline file | `version: 1` | Versioned; unsupported versions are rejected. |
| Configuration | schema v0 | Unknown fields are rejected; additive changes are announced. |
| Rule ids | `SS-###` | Permanent and never reused. |

## Exit codes

Stable; see [CLI Reference](cli.md#exit-codes).
