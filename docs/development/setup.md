# Development Setup

## Prerequisites

- **Rust** (recent stable; CI builds and tests on stable).
- **Git**.
- Optional: **Python 3.10+** for supporting tooling under `tools/`.

Check your toolchain:

```bash
rustc --version
cargo --version
```

### Windows note

Either the MSVC or the GNU target works. If `link.exe` is unavailable, install
the Visual Studio C++ Build Tools, or use the GNU toolchain with MinGW:

```powershell
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup component add rustfmt clippy --toolchain stable-x86_64-pc-windows-gnu
rustup override set stable-x86_64-pc-windows-gnu
```

Using a directory override keeps the choice local to your checkout and out of
the committed configuration.

## Clone and build

```bash
git clone https://github.com/AyinkxLab/soroban-security-scanner
cd soroban-security-scanner
cargo build
```

## Run the CLI

```bash
cargo run -p soroban-scan-cli -- --help
cargo run -p soroban-scan-cli -- version
```

## Local quality gate

Run everything CI runs:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

## Formatting

```bash
cargo fmt --all
```

## Linting

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

## Dependency audit

Install [`cargo-deny`](https://github.com/EmbarkStudios/cargo-deny):

```bash
cargo install cargo-deny --locked
cargo deny check
```

## Pre-commit (optional)

Install [`pre-commit`](https://pre-commit.com/) and enable the hooks:

```bash
pre-commit install
```

## Troubleshooting

- **`linker link.exe not found`** — you are on Windows without MSVC build tools;
  see the Windows note above.
- **Slow first build** — the first build downloads and compiles dependencies.
