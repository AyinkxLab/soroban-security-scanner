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

    pub fn set_limit(env: Env, admin: Address, limit: i128) {
        admin.require_auth_for_args((limit,).into_val(&env));
        env.storage()
            .persistent()
            .set(&symbol_short!("LIM"), &limit);
    }
}

// Helper-based authorization: require_auth is delegated to a same-file helper,
// which must also suppress SS-001.
#[contractimpl]
impl Escrow {
    pub fn release(env: Env, admin: Address, to: Address, amount: i128) {
        Self::authorize(&admin);
        env.storage().persistent().set(&symbol_short!("TO"), &to);
        let _ = amount;
    }

    fn authorize(admin: &Address) {
        admin.require_auth();
    }
}
