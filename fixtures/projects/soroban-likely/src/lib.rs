//! A library that depends on the Soroban SDK but defines no contract.
//! This fixture must be classified as "likely Soroban", not confirmed.

use soroban_sdk::{Env, Symbol};

pub fn key(env: &Env, name: &str) -> Symbol {
    let _ = env;
    Symbol::new(env, name)
}
