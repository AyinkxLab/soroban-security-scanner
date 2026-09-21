# Vault example

A clean, pausable, access-controlled vault. It demonstrates role-based access
control (admin and operator), a pause switch, authorized deposits and
withdrawals against a token, and correct TTL handling.

```bash
cargo check --manifest-path examples/vault/Cargo.toml
soroban-scan scan examples/vault --fail-on high
```

Expected result: `0 finding(s)`.
