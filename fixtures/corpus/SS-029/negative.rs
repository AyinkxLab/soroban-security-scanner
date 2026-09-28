// SS-029 negative: ordinary entry point names.
//
// `transfer` and `latest` contain no test/debug/dev name segment, so they must
// not be flagged.

#![no_std]

use soroban_sdk::{contractimpl, Env};

pub struct C;

#[contractimpl]
impl C {
    pub fn transfer(_env: Env, amount: i128) -> i128 {
        amount
    }

    pub fn latest(_env: Env) -> u32 {
        1
    }
}
