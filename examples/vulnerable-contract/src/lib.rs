//! Intentionally vulnerable Soroban contract.
//!
//! DO NOT USE THIS CODE. Every violation below is deliberate and exists to
//! demonstrate scanner findings. The contract still compiles so it can be built
//! in CI alongside the clean examples.
//!
//! Expected findings:
//! - SS-001 state-changing entry point without authorization (`set_admin`)
//! - SS-002 cross-contract call without authorization (`payout`)
//! - SS-004 panic-prone construct (`balance`)
//! - SS-006 persistent write without TTL management (`set_admin`)
//! - SS-007 hardcoded Stellar address (`treasury`)

use soroban_sdk::{contract, contractimpl, symbol_short, token, Address, Env, String};

#[contract]
pub struct VulnerableVault;

#[contractimpl]
impl VulnerableVault {
    /// Missing authorization, and no TTL extension after the persistent write.
    pub fn set_admin(env: Env, new_admin: Address) {
        env.storage()
            .persistent()
            .set(&symbol_short!("ADMIN"), &new_admin);
    }

    /// Panics when the key is absent.
    pub fn balance(env: Env) -> i128 {
        env.storage()
            .persistent()
            .get(&symbol_short!("BAL"))
            .unwrap()
    }

    /// Hardcoded address literal.
    pub fn treasury(env: Env) -> String {
        String::from_str(
            &env,
            "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        )
    }

    /// Cross-contract call with no caller authorization.
    pub fn payout(env: Env, token_id: Address, to: Address, amount: i128) {
        let client = token::Client::new(&env, &token_id);
        client.transfer(&env.current_contract_address(), &to, &amount);
    }
}
