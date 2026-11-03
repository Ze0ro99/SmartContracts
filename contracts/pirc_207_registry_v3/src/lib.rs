#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    AssetAlreadyRegistered = 4,
    AssetNotFound = 5,
    InvalidDecimals = 6,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Asset(BytesN<32>),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetRecord {
    pub issuer: Address,
    pub metadata_hash: BytesN<32>,
    pub decimals: u32,
    pub active: bool,
}

#[contract]
pub struct Registry;

impl Registry {
    fn require_admin(env: &Env, admin: &Address) -> Result<(), Error> {
        admin.require_auth();
        let stored: Option<Address> = env.storage().instance().get(&DataKey::Admin);
        match stored {
            Some(value) if value == admin.clone() => Ok(()),
            Some(_) => Err(Error::Unauthorized),
            None => Err(Error::NotInitialized),
        }
    }
}

#[contractimpl]
impl Registry {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    pub fn register_asset(
        env: Env,
        admin: Address,
        asset_id: BytesN<32>,
        issuer: Address,
        metadata_hash: BytesN<32>,
        decimals: u32,
    ) -> Result<(), Error> {
        Self::require_admin(&env, &admin)?;
        if decimals > 18 {
            return Err(Error::InvalidDecimals);
        }
        let key = DataKey::Asset(asset_id.clone());
        if env.storage().persistent().has(&key) {
            return Err(Error::AssetAlreadyRegistered);
        }
        let record = AssetRecord { issuer, metadata_hash, decimals, active: true };
        env.storage().persistent().set(&key, &record);
        env.events().publish((symbol_short!("asset"), symbol_short!("reg")), asset_id);
        Ok(())
    }

    pub fn set_asset_active(
        env: Env,
        admin: Address,
        asset_id: BytesN<32>,
        active: bool,
    ) -> Result<(), Error> {
        Self::require_admin(&env, &admin)?;
        let key = DataKey::Asset(asset_id.clone());
        let mut record: AssetRecord = env.storage().persistent().get(&key).ok_or(Error::AssetNotFound)?;
        record.active = active;
        env.storage().persistent().set(&key, &record);
        env.events().publish((symbol_short!("asset"), symbol_short!("state")), (asset_id, active));
        Ok(())
    }

    pub fn get_asset(env: Env, asset_id: BytesN<32>) -> Option<AssetRecord> {
        env.storage().persistent().get(&DataKey::Asset(asset_id))
    }

    pub fn admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }
}

#[cfg(test)]
mod test {
    use super::{AssetRecord, Registry, RegistryClient};
    use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

    #[test]
    fn admin_registers_and_deactivates_asset() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register(Registry, ());
        let client = RegistryClient::new(&env, &id);
        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let asset = BytesN::from_array(&env, &[7u8; 32]);
        let hash = BytesN::from_array(&env, &[9u8; 32]);
        client.initialize(&admin).unwrap();
        assert!(client.register_asset(&Address::generate(&env), &asset, &issuer, &hash, &7).is_err());
        client.register_asset(&admin, &asset, &issuer, &hash, &7).unwrap();
        let record: AssetRecord = client.get_asset(&asset).unwrap();
        assert!(record.active);
        assert_eq!(record.issuer, issuer);
        client.set_asset_active(&admin, &asset, &false).unwrap();
        assert!(!client.get_asset(&asset).unwrap().active);
    }

    #[test]
    fn duplicate_registration_is_rejected() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register(Registry, ());
        let client = RegistryClient::new(&env, &id);
        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let asset = BytesN::from_array(&env, &[1u8; 32]);
        let hash = BytesN::from_array(&env, &[2u8; 32]);
        client.initialize(&admin).unwrap();
        client.register_asset(&admin, &asset, &issuer, &hash, &7).unwrap();
        assert!(client.register_asset(&admin, &asset, &issuer, &hash, &7).is_err());
    }
}
