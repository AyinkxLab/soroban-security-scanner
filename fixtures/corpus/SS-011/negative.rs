// SS-011 negative: the initializer guards against re-initialization by reading
// storage before writing.

#![no_std]

use soroban_sdk::{contractimpl, Address, Env};

pub struct C;

#[contractimpl]
impl C {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            return;
        }
        env.storage().persistent().set(&DataKey::Admin, &admin);
    }
}
