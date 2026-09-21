//! A minimal Soroban contract fixture used for project-detection tests.
//! This file intentionally contains no vulnerability analysis fixtures; it only
//! exists to exercise Soroban classification and item discovery.

use soroban_sdk::{contract, contractimpl, Env};

pub mod storage;

#[contract]
pub struct HelloContract;

#[contractimpl]
impl HelloContract {
    pub fn hello(env: Env, to: soroban_sdk::Symbol) -> soroban_sdk::Symbol {
        let _ = env;
        to
    }
}
