#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidPrice = 4,
    StaleSequence = 5,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Reporter(Address),
    Feed(String),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceRecord {
    pub price: u128,
    pub sequence: u64,
    pub updated_at: u64,
    pub reporter: Address,
    pub unit: String,
}

#[contract]
pub struct Oracle;

impl Oracle {
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
impl Oracle {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) { return Err(Error::AlreadyInitialized); }
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    pub fn set_reporter(env: Env, admin: Address, reporter: Address, enabled: bool) -> Result<(), Error> {
        Self::require_admin(&env, &admin)?;
        env.storage().persistent().set(&DataKey::Reporter(reporter.clone()), &enabled);
        env.events().publish((symbol_short!("oracle"), symbol_short!("reporter")), (reporter, enabled));
        Ok(())
    }

    pub fn update_price(
        env: Env,
        reporter: Address,
        feed_id: String,
        price: u128,
        sequence: u64,
        unit: String,
    ) -> Result<(), Error> {
        reporter.require_auth();
        let enabled: bool = env.storage().persistent().get(&DataKey::Reporter(reporter.clone())).unwrap_or(false);
        if !enabled { return Err(Error::Unauthorized); }
        if price == 0 { return Err(Error::InvalidPrice); }
        let key = DataKey::Feed(feed_id.clone());
        let previous: Option<PriceRecord> = env.storage().persistent().get(&key);
        if let Some(old) = previous {
            if sequence <= old.sequence { return Err(Error::StaleSequence); }
        }
        let record = PriceRecord {
            price,
            sequence,
            updated_at: env.ledger().timestamp(),
            reporter: reporter.clone(),
            unit,
        };
        env.storage().persistent().set(&key, &record);
        env.events().publish((symbol_short!("oracle"), symbol_short!("price")), (feed_id, price, sequence, reporter));
        Ok(())
    }

    pub fn get_latest(env: Env, feed_id: String) -> Option<PriceRecord> {
        env.storage().persistent().get(&DataKey::Feed(feed_id))
    }
}

#[cfg(test)]
mod test {
    use super::{Oracle, OracleClient};
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    #[test]
    fn authorized_reporter_updates_feed_and_sequence_is_monotonic() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register(Oracle, ());
        let client = OracleClient::new(&env, &id);
        let admin = Address::generate(&env);
        let reporter = Address::generate(&env);
        let feed = String::from_str(&env, "PI/USD");
        let unit = String::from_str(&env, "USD_PER_PI");
        client.initialize(&admin);
        assert!(matches!(client.try_set_reporter(&Address::generate(&env), &reporter, &true), Ok(Err(_))));
        client.set_reporter(&admin, &reporter, &true);
        client.update_price(&reporter, &feed, &123_000_000, &1, &unit);
        assert_eq!(client.get_latest(&feed).unwrap().price, 123_000_000);
        assert!(matches!(client.try_update_price(&reporter, &feed, &124_000_000, &1, &unit), Ok(Err(_))));
    }
}
