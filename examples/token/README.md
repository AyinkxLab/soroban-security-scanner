# Token example

A clean, minimal fungible token contract for Soroban: initialize, mint,
transfer, and balance, with a typed error enum, role-restricted minting,
authorization on every state change, TTL extension, and `#[contractevent]`
events.

```bash
cargo check --manifest-path examples/token/Cargo.toml
soroban-scan scan examples/token --fail-on high
```

Expected result: `0 finding(s)`.
