# Clean vault example

A minimal, correctly written Soroban contract used as a **negative** reference:
the scanner must report no findings for it.

It authorizes the caller with `require_auth`, extends the TTL after persistent
writes, avoids panics, and uses no hardcoded addresses.

```bash
cargo check --manifest-path examples/clean-contract/Cargo.toml
soroban-scan scan examples/clean-contract --fail-on high
```

Expected result: `0 finding(s)`.
