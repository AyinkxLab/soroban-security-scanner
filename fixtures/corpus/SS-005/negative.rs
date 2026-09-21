// SS-005 negative: safe Rust only.
// This file is a security fixture, not production code.

use soroban_sdk::{contractimpl, Env};

#[contractimpl]
impl Safe {
    pub fn add(env: Env, a: u32, b: u32) -> u32 {
        let _ = env;
        a + b
    }
}
