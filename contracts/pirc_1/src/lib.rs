#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, BytesN, String, symbol_short};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&symbol_short!("admin"), &admin);
    }

    pub fn execute(env: Env, caller: Address, action: String, payload: BytesN<32>) -> bool {
        caller.require_auth();
        true
    }

    pub fn ping(env: Env) -> String {
        String::from_str(&env, "ok")
    }
}
