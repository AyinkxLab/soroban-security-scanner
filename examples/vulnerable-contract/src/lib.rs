//! Intentionally vulnerable Soroban contract.
//!
//! DO NOT USE THIS CODE. It exists only to demonstrate scanner findings in
//! examples and CI. Every violation below is deliberate.

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct VulnerableVault;

#[contractimpl]
impl VulnerableVault {
    /// Missing authorization: anyone can replace the admin.
    pub fn set_admin(env: Env, new_admin: Address) {
        env.storage()
            .persistent()
            .set(&symbol_short!("ADMIN"), &new_admin);
    }

    /// Panics when the balance is absent.
    pub fn balance(env: Env) -> i128 {
        env.storage()
            .persistent()
            .get(&symbol_short!("BAL"))
            .unwrap()
    }

    /// Hardcoded address literal.
    pub fn treasury(env: Env) -> &'static str {
        let _ = env;
        "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
    }
}
