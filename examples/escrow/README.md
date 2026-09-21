# Escrow example

A clean two-party escrow contract. The buyer funds the escrow; the buyer or the
arbiter releases it to the seller. It demonstrates authorized cross-contract
token transfers, typed errors, TTL management, and `#[contractevent]` events.

```bash
cargo check --manifest-path examples/escrow/Cargo.toml
soroban-scan scan examples/escrow --fail-on high
```

Expected result: `0 finding(s)`.
