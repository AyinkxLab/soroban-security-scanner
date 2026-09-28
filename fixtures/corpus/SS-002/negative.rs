// SS-002 negative: caller is authorized before the cross-contract call.
// This file is a security fixture, not production code.

use soroban_sdk::{contractimpl, Address, Env};

#[contractimpl]
impl Paymaster {
    pub fn payout(env: Env, from: Address, token_id: Address, to: Address, amount: i128) {
        from.require_auth();
        let client = TokenClient::new(&env, &token_id);
        client.transfer(&from, &to, &amount);
    }
}

// Helper-based authorization before a cross-contract call.
#[contractimpl]
impl Router {
    pub fn route(env: Env, from: Address, token_id: Address, to: Address, amount: i128) {
        Self::authorize(&from);
        let client = TokenClient::new(&env, &token_id);
        client.transfer(&from, &to, &amount);
    }

    fn authorize(from: &Address) {
        from.require_auth();
    }
}
