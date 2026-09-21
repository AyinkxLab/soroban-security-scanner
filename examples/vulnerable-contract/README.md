# Vulnerable contract example

An intentionally vulnerable Soroban contract used as a **positive** reference.
**Do not deploy or copy it.** It still compiles so it can be built in CI.

It deliberately contains:

- SS-001 state-changing entry point without authorization (`set_admin`)
- SS-002 cross-contract call without authorization (`payout`)
- SS-004 panic-prone construct (`balance`)
- SS-006 persistent write without TTL management (`set_admin`)
- SS-007 hardcoded Stellar address (`treasury`)

```bash
cargo check --manifest-path examples/vulnerable-contract/Cargo.toml
soroban-scan scan examples/vulnerable-contract --fail-on none
```
