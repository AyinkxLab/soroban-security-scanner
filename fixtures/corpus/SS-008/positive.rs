// SS-008 positive: unchecked arithmetic in a contract entry point.
//
// The vault decrements a balance with a raw `-`, which underflows silently in
// release builds instead of failing the transaction.

#![no_std]

use soroban_sdk::{contractimpl, Address, Env};

pub struct Vault;

#[contractimpl]
impl Vault {
    pub fn withdraw(_env: Env, from: Address, amount: i128) -> i128 {
        from.require_auth();
        let balance = 100i128;
        balance - amount
    }
}
