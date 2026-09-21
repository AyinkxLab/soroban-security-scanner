# Classic Stellar tooling example

A host-side library (not a Soroban contract) that validates classic Stellar
strkeys using the official `stellar-strkey` crate. It shows how to classify and
validate account (`G...`) and contract (`C...`) addresses before using them to
build transactions or construct clients.

Because it has no Soroban dependency, the scanner classifies it as
`generic_rust`, not a Soroban project.

```bash
cargo check --manifest-path examples/stellar-classic/Cargo.toml
cargo test --manifest-path examples/stellar-classic/Cargo.toml
soroban-scan scan examples/stellar-classic --fail-on high
```
