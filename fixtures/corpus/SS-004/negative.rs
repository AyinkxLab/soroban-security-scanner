// SS-004 negative: fallible operations use safe fallbacks.
// This file is a security fixture, not production code.

use soroban_sdk::{contractimpl, Env};

#[contractimpl]
impl Store {
    pub fn read(env: Env) -> u32 {
        env.storage().persistent().get(&K).unwrap_or(0)
    }
}
