// SS-003 negative: iteration is over a bounded, caller-supplied collection.
// This file is a security fixture, not production code.

use soroban_sdk::{contractimpl, Env, Vec};

#[contractimpl]
impl Registry {
    pub fn sum(env: Env, values: Vec<u32>) -> u32 {
        let _ = env;
        let mut total = 0;
        for value in values.iter() {
            total += value;
        }
        total
    }
}
