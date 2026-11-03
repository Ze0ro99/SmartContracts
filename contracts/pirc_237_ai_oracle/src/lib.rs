#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env, String};

const SCORE_SCALE: u128 = 1_000_000;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    ScoreOutOfRange = 4,
    StaleSequence = 5,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Updater(Address),
    Feed(String),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AIModelRecord {
    pub volatility_index: u128,
    pub sentiment_score: u128,
    pub sequence: u64,
    pub updated_at: u64,
    pub proof_hash: BytesN<32>,
    pub updater: Address,
    pub model_version: String,
}

#[contract]
pub struct AIOracle;

impl AIOracle {
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
impl AIOracle {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) { return Err(Error::AlreadyInitialized); }
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    pub fn set_updater(env: Env, admin: Address, updater: Address, enabled: bool) -> Result<(), Error> {
        Self::require_admin(&env, &admin)?;
        env.storage().persistent().set(&DataKey::Updater(updater.clone()), &enabled);
        env.events().publish((symbol_short!("ai"), symbol_short!("updater")), (updater, enabled));
        Ok(())
    }

    pub fn submit(
        env: Env,
        updater: Address,
        asset_id: String,
        volatility_index: u128,
        sentiment_score: u128,
        sequence: u64,
        proof_hash: BytesN<32>,
        model_version: String,
    ) -> Result<(), Error> {
        updater.require_auth();
        let enabled: bool = env.storage().persistent().get(&DataKey::Updater(updater.clone())).unwrap_or(false);
        if !enabled { return Err(Error::Unauthorized); }
        if volatility_index > SCORE_SCALE || sentiment_score > SCORE_SCALE {
            return Err(Error::ScoreOutOfRange);
        }
        let key = DataKey::Feed(asset_id.clone());
        let previous: Option<AIModelRecord> = env.storage().persistent().get(&key);
        if let Some(old) = previous {
            if sequence <= old.sequence { return Err(Error::StaleSequence); }
        }
        let record = AIModelRecord {
            volatility_index,
            sentiment_score,
            sequence,
            updated_at: env.ledger().timestamp(),
            proof_hash,
            updater: updater.clone(),
            model_version,
        };
        env.storage().persistent().set(&key, &record);
        env.events().publish((symbol_short!("ai"), symbol_short!("updated")), (asset_id, sequence, updater));
        Ok(())
    }

    pub fn get_latest(env: Env, asset_id: String) -> Option<AIModelRecord> {
        env.storage().persistent().get(&DataKey::Feed(asset_id))
    }
}

#[cfg(test)]
mod test {
    use super::{AIOracle, AIOracleClient};
    use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

    #[test]
    fn allowlisted_updater_submits_bounded_monotonic_record() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register(AIOracle, ());
        let client = AIOracleClient::new(&env, &id);
        let admin = Address::generate(&env);
        let updater = Address::generate(&env);
        let asset = String::from_str(&env, "PI/USD");
        let proof = BytesN::from_array(&env, &[5u8; 32]);
        let model = String::from_str(&env, "model-1");
        client.initialize(&admin).unwrap();
        client.set_updater(&admin, &updater, &true).unwrap();
        client.submit(&updater, &asset, &500_000, &700_000, &1, &proof, &model).unwrap();
        assert_eq!(client.get_latest(&asset).unwrap().sentiment_score, 700_000);
        assert!(client.submit(&updater, &asset, &500_000, &700_000, &1, &proof, &model).is_err());
    }
}
