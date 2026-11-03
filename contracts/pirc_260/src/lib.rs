#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, String};

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
        if env.storage().instance().has(&DataKey::Admin) {
            return;
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn register_asset(env: Env, caller: Address, asset_id: BytesN<32>, metadata: String) -> bool {
        caller.require_auth();
        let asset_key = DataKey::Asset(asset_id.clone());
        let owner_key = DataKey::Owner(asset_id.clone());
        if env.storage().persistent().has(&asset_key) || env.storage().persistent().has(&owner_key) {
            return false;
        }
        env.storage().persistent().set(&asset_key, &metadata);
        env.storage().persistent().set(&owner_key, &caller);
        true
    }

    pub fn transfer_asset(env: Env, from: Address, to: Address, asset_id: BytesN<32>) -> bool {
        from.require_auth();
        let owner_key = DataKey::Owner(asset_id);
        let owner: Option<Address> = env.storage().persistent().get(&owner_key);
        let Some(owner) = owner else {
            return false;
        };
        if owner != from {
            return false;
        }
        if from == to {
            return true;
        }
        env.storage().persistent().set(&owner_key, &to);
        true
    }

    pub fn get_owner(env: Env, asset_id: BytesN<32>) -> Option<Address> {
        env.storage().persistent().get(&DataKey::Owner(asset_id))
    }

    pub fn ping(env: Env) -> String {
        String::from_str(&env, "pirc-rwa-ok")
    }
}

#[cfg(test)]
mod test {
    use super::{Contract, ContractClient};
    use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

    #[test]
    fn initialization_is_idempotent_and_asset_registration_is_unique() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register(Contract, ());
        let client = ContractClient::new(&env, &id);
        let admin = Address::generate(&env);
        let replacement_admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let asset_id = BytesN::from_array(&env, &[7u8; 32]);
        let metadata = String::from_str(&env, "{\"name\":\"Example Asset\",\"schema_version\":1}");

        client.initialize(&admin);
        client.initialize(&replacement_admin);
        assert!(client.register_asset(&issuer, &asset_id, &metadata));
        assert!(!client.register_asset(&issuer, &asset_id, &metadata));
        assert_eq!(client.get_owner(&asset_id), Some(issuer));
    }

    #[test]
    fn missing_asset_transfer_fails_without_trapping() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register(Contract, ());
        let client = ContractClient::new(&env, &id);
        let from = Address::generate(&env);
        let to = Address::generate(&env);
        let missing_asset = BytesN::from_array(&env, &[9u8; 32]);

        assert!(!client.transfer_asset(&from, &to, &missing_asset));
        assert_eq!(client.get_owner(&missing_asset), None);
    }

    #[test]
    fn only_current_owner_can_transfer_and_self_transfer_is_noop() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register(Contract, ());
        let client = ContractClient::new(&env, &id);
        let issuer = Address::generate(&env);
        let other = Address::generate(&env);
        let asset_id = BytesN::from_array(&env, &[3u8; 32]);
        let metadata = String::from_str(&env, "{\"name\":\"Transfer Test\",\"schema_version\":1}");

        assert!(client.register_asset(&issuer, &asset_id, &metadata));
        assert!(!client.transfer_asset(&other, &issuer, &asset_id));
        assert!(client.transfer_asset(&issuer, &issuer, &asset_id));
        assert_eq!(client.get_owner(&asset_id), Some(issuer.clone()));
        assert!(client.transfer_asset(&issuer, &other, &asset_id));
        assert_eq!(client.get_owner(&asset_id), Some(other));
    }
}
