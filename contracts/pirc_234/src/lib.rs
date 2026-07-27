#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, BytesN};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Asset(BytesN<32>),
    Owner(BytesN<32>),
}

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn register_asset(env: Env, caller: Address, asset_id: BytesN<32>, metadata: String) -> bool {
        caller.require_auth();
        env.storage().persistent().set(&DataKey::Asset(asset_id.clone()), &metadata);
        env.storage().persistent().set(&DataKey::Owner(asset_id), &caller);
        true
    }

    pub fn transfer_asset(env: Env, from: Address, to: Address, asset_id: BytesN<32>) -> bool {
        from.require_auth();
        let owner: Address = env.storage().persistent().get(&DataKey::Owner(asset_id.clone())).unwrap();
        if owner != from { return false; }
        env.storage().persistent().set(&DataKey::Owner(asset_id), &to);
        true
    }

    pub fn get_owner(env: Env, asset_id: BytesN<32>) -> Option<Address> {
        env.storage().persistent().get(&DataKey::Owner(asset_id))
    }

    pub fn ping(env: Env) -> String {
        String::from_str(&env, "pirc-rwa-ok")
    }
}
