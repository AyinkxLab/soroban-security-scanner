// SS-007 negative: no address-shaped literals.
// This file is a security fixture, not production code.

use soroban_sdk::{contractimpl, Env};

#[contractimpl]
impl Config {
    pub fn greeting(env: Env) -> &'static str {
        let _ = env;
        "hello from soroban"
    }
}
