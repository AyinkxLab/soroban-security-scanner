// SS-006 negative: persistent write extends the entry's TTL.
// This file is a security fixture, not production code.

use soroban_sdk::{contractimpl, symbol_short, Address, Env};

#[contractimpl]
impl Vault {
    pub fn set_admin(env: Env, admin: Address, new_admin: Address) {
        admin.require_auth();
        let key = symbol_short!("ADMIN");
        env.storage().persistent().set(&key, &new_admin);
        env.storage().persistent().extend_ttl(&key, 100, 1000);
    }
}
