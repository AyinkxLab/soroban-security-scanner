# Installation

## Supported platforms

- Linux, macOS, and Windows.
- A recent stable Rust toolchain. CI builds and tests on stable.

## From source

```bash
git clone https://github.com/AyinkxLab/soroban-security-scanner
cd soroban-security-scanner
cargo build --release -p soroban-scan-cli
```

The binary is written to `target/release/soroban-scan` (`.exe` on Windows).

## Install with Cargo

```bash
cargo install --path crates/soroban-scan-cli
```

This places `soroban-scan` on your Cargo bin path (usually `~/.cargo/bin`).

## Prebuilt binaries

Prebuilt binaries are added when the first tagged release is published. Until
then, build from source.

## Verify the installation

```bash
soroban-scan version
soroban-scan rules
```

## Windows notes

Either the MSVC or GNU toolchain works. If building with MSVC fails with
`linker link.exe not found`, install the Visual Studio C++ Build Tools, or use
the GNU toolchain with MinGW:

```powershell
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup override set stable-x86_64-pc-windows-gnu
```

See [Troubleshooting](troubleshooting.md) if you hit other issues.

## Uninstall

- From source: delete the checkout.
- From Cargo: `cargo uninstall soroban-scan-cli`.
