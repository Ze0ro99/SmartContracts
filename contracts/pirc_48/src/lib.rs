#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, BytesN};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Identity(Address),
}

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn register_identity(env: Env, caller: Address, did: String, metadata_hash: BytesN<32>) -> bool {
        caller.require_auth();
        env.storage().persistent().set(&DataKey::Identity(caller.clone()), &(did, metadata_hash));
        true
    }

    pub fn get_identity(env: Env, address: Address) -> Option<(String, BytesN<32>)> {
        env.storage().persistent().get(&DataKey::Identity(address))
    }

    pub fn ping(env: Env) -> String {
        String::from_str(&env, "pirc-identity-ok")
    }
}
