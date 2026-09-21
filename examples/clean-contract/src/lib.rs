//! A clean Soroban contract used as an example and as a CI gate.
//!
//! It intentionally avoids the patterns the scanner flags: it authorizes the
//! caller, extends TTL after persistent writes, avoids panics, and uses no
//! hardcoded addresses.

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct CleanVault;

#[contractimpl]
impl CleanVault {
    pub fn set_admin(env: Env, admin: Address, new_admin: Address) {
        admin.require_auth();
        let key = symbol_short!("ADMIN");
        env.storage().persistent().set(&key, &new_admin);
        env.storage().persistent().extend_ttl(&key, 100, 1000);
    }

    pub fn balance(env: Env) -> i128 {
        env.storage()
            .persistent()
            .get(&symbol_short!("BAL"))
            .unwrap_or(0)
    }
}
