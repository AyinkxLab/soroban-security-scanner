// SS-001 negative: authorization is checked before state changes.
// This file is a security fixture, not production code.

use soroban_sdk::{contractimpl, symbol_short, Address, Env};

#[contractimpl]
impl Vault {
    pub fn set_admin(env: Env, admin: Address, new_admin: Address) {
        admin.require_auth();
        env.storage()
            .persistent()
            .set(&symbol_short!("ADMIN"), &new_admin);
    }

    pub fn balance(env: Env) -> i128 {
        env.storage()
            .persistent()
            .get(&symbol_short!("BAL"))
            .unwrap_or(0)
    }
}
