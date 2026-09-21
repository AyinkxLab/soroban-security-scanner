//! Storage helpers for the confirmed Soroban fixture.

use soroban_sdk::{symbol_short, Env};

pub fn counter_key(env: &Env) -> soroban_sdk::Symbol {
    let _ = env;
    symbol_short!("COUNTER")
}
