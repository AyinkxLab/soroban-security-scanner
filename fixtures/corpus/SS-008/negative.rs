// SS-008 negative: overflow-aware arithmetic and non-entry helpers.
//
// The entry point uses `checked_sub` and handles the `None` case, and the
// arithmetic in the private helper is not reachable as a contract entry point.

#![no_std]

use soroban_sdk::{contractimpl, Address, Env};

pub struct Vault;

#[contractimpl]
impl Vault {
    pub fn withdraw(_env: Env, from: Address, amount: i128) -> i128 {
        from.require_auth();
        let balance = 100i128;
        balance.checked_sub(amount).unwrap_or(0)
    }

    fn preview(a: i128, b: i128) -> i128 {
        a - b
    }
}
