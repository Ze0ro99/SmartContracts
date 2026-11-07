#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidPrice = 4,
    StaleSequence = 5,
    InvalidPolicy = 6,
    UnitMismatch = 7,
    InsufficientQuorum = 8,
    ArithmeticError = 9,
    InvalidObservation = 10,
    InternalError = 11,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Reporters,
    ReporterEnabled(Address),
    Policy(String),
    Observation(String, Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeedPolicy {
    pub quorum: u32,
    pub max_age_seconds: u64,
    pub max_deviation_bps: u32,
    pub unit: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceObservation {
    pub price: u128,
    pub sequence: u64,
    pub updated_at: u64,
    pub reporter: Address,
    pub unit: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Consensus {
    pub median_price: u128,
    pub accepted_count: u32,
    pub rejected_count: u32,
    pub observed_at: u64,
    pub unit: String,
}

#[contract]
pub struct OracleQuorum;

impl OracleQuorum {
    fn require_admin(env: &Env, admin: &Address) -> Result<(), Error> {
        admin.require_auth();
        let stored: Option<Address> = env.storage().instance().get(&DataKey::Admin);
        match stored {
            Some(value) if value == admin.clone() => Ok(()),
            Some(_) => Err(Error::Unauthorized),
            None => Err(Error::NotInitialized),
        }
    }

    fn reporters(env: &Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::Reporters)
            .unwrap_or(Vec::new(env))
    }

    fn median(env: &Env, mut values: Vec<u128>) -> Result<u128, Error> {
        let len = values.len();
        if len == 0 {
            return Err(Error::InsufficientQuorum);
        }

        // Bounded insertion sort. Reporter count is constrained by feed policy
        // and the deployment's resource budget; no floating-point arithmetic.
        for i in 1..len {
            let key = values.get(i).ok_or(Error::InternalError)?;
            let mut j = i;
            while j > 0 {
                let previous = values.get(j - 1).ok_or(Error::InternalError)?;
                if previous <= key {
                    break;
                }
                values.set(j, previous);
                j -= 1;
            }
            values.set(j, key);
        }

        if len % 2 == 1 {
            values.get(len / 2).ok_or(Error::InternalError)
        } else {
            let lower = values.get(len / 2 - 1).ok_or(Error::InternalError)?;
            let upper = values.get(len / 2).ok_or(Error::InternalError)?;
            // Overflow-safe floor((lower + upper) / 2).
            Ok(lower / 2 + upper / 2 + ((lower % 2 + upper % 2) / 2))
        }
    }

    fn within_deviation(price: u128, reference: u128, max_bps: u32) -> Result<bool, Error> {
        if reference == 0 {
            return Err(Error::InvalidPrice);
        }
        let difference = if price >= reference {
            price - reference
        } else {
            reference - price
        };
        let scaled = difference
            .checked_mul(10_000)
            .ok_or(Error::ArithmeticError)?;
        let allowed = reference
            .checked_mul(max_bps as u128)
            .ok_or(Error::ArithmeticError)?;
        Ok(scaled <= allowed)
    }
}

#[contractimpl]
impl OracleQuorum {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::Reporters, &Vec::<Address>::new(&env));
        Ok(())
    }

    pub fn set_reporter(
        env: Env,
        admin: Address,
        reporter: Address,
        enabled: bool,
    ) -> Result<(), Error> {
        Self::require_admin(&env, &admin)?;
        let mut reporters = Self::reporters(&env);
        let mut found = false;
        for index in 0..reporters.len() {
            if reporters.get(index).as_ref() == Some(&reporter) {
                found = true;
                break;
            }
        }
        if enabled && !found {
            if reporters.len() >= 32 {
                return Err(Error::InvalidPolicy);
            }
            reporters.push_back(reporter.clone());
            env.storage()
                .instance()
                .set(&DataKey::Reporters, &reporters);
        }
        env.storage()
            .persistent()
            .set(&DataKey::ReporterEnabled(reporter.clone()), &enabled);
        env.events().publish(
            (symbol_short!("oracle"), symbol_short!("reporter")),
            (reporter, enabled),
        );
        Ok(())
    }

    pub fn configure_feed(
        env: Env,
        admin: Address,
        feed_id: String,
        quorum: u32,
        max_age_seconds: u64,
        max_deviation_bps: u32,
        unit: String,
    ) -> Result<(), Error> {
        Self::require_admin(&env, &admin)?;
        if quorum == 0
            || quorum > 32
            || max_age_seconds == 0
            || max_deviation_bps > 10_000
            || unit.is_empty()
        {
            return Err(Error::InvalidPolicy);
        }
        let policy = FeedPolicy {
            quorum,
            max_age_seconds,
            max_deviation_bps,
            unit,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Policy(feed_id.clone()), &policy);
        env.events().publish(
            (symbol_short!("oracle"), symbol_short!("policy")),
            (feed_id, quorum, max_age_seconds, max_deviation_bps),
        );
        Ok(())
    }

    pub fn submit_price(
        env: Env,
        reporter: Address,
        feed_id: String,
        price: u128,
        sequence: u64,
        unit: String,
    ) -> Result<(), Error> {
        reporter.require_auth();
        if price == 0 {
            return Err(Error::InvalidPrice);
        }
        let enabled: bool = env
            .storage()
            .persistent()
            .get(&DataKey::ReporterEnabled(reporter.clone()))
            .unwrap_or(false);
        if !enabled {
            return Err(Error::Unauthorized);
        }
        let policy: FeedPolicy = env
            .storage()
            .persistent()
            .get(&DataKey::Policy(feed_id.clone()))
            .ok_or(Error::InvalidPolicy)?;
        if unit != policy.unit {
            return Err(Error::UnitMismatch);
        }
        let key = DataKey::Observation(feed_id.clone(), reporter.clone());
        let previous: Option<PriceObservation> = env.storage().persistent().get(&key);
        if let Some(old) = previous {
            if sequence <= old.sequence {
                return Err(Error::StaleSequence);
            }
        }
        let observation = PriceObservation {
            price,
            sequence,
            updated_at: env.ledger().timestamp(),
            reporter: reporter.clone(),
            unit,
        };
        env.storage().persistent().set(&key, &observation);
        env.events().publish(
            (symbol_short!("oracle"), symbol_short!("price")),
            (feed_id, price, sequence, reporter),
        );
        Ok(())
    }

    pub fn get_consensus(env: Env, feed_id: String) -> Result<Consensus, Error> {
        let policy: FeedPolicy = env
            .storage()
            .persistent()
            .get(&DataKey::Policy(feed_id.clone()))
            .ok_or(Error::InvalidPolicy)?;
        let reporters = Self::reporters(&env);
        let now = env.ledger().timestamp();
        let mut fresh = Vec::<u128>::new(&env);
        let mut oldest = now;
        let mut rejected = 0u32;

        for index in 0..reporters.len() {
            let reporter = reporters.get(index).ok_or(Error::InternalError)?;
            let enabled: bool = env
                .storage()
                .persistent()
                .get(&DataKey::ReporterEnabled(reporter.clone()))
                .unwrap_or(false);
            if !enabled {
                continue;
            }
            let observation: Option<PriceObservation> = env
                .storage()
                .persistent()
                .get(&DataKey::Observation(feed_id.clone(), reporter));
            if let Some(value) = observation {
                if value.unit != policy.unit
                    || value.updated_at > now
                    || now - value.updated_at > policy.max_age_seconds
                {
                    rejected += 1;
                    continue;
                }
                oldest = oldest.min(value.updated_at);
                fresh.push_back(value.price);
            }
        }

        if fresh.len() < policy.quorum {
            return Err(Error::InsufficientQuorum);
        }
        let initial_median = Self::median(&env, fresh.clone())?;
        let mut accepted = Vec::<u128>::new(&env);
        for index in 0..fresh.len() {
            let price = fresh.get(index).ok_or(Error::InternalError)?;
            if Self::within_deviation(price, initial_median, policy.max_deviation_bps)? {
                accepted.push_back(price);
            } else {
                rejected += 1;
            }
        }
        if accepted.len() < policy.quorum {
            return Err(Error::InsufficientQuorum);
        }
        let median_price = Self::median(&env, accepted.clone())?;
        Ok(Consensus {
            median_price,
            accepted_count: accepted.len(),
            rejected_count: rejected,
            observed_at: oldest,
            unit: policy.unit,
        })
    }
}

#[cfg(test)]
mod test {
    use super::{OracleQuorum, OracleQuorumClient};
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    #[test]
    fn consensus_requires_quorum_and_uses_median() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register(OracleQuorum, ());
        let client = OracleQuorumClient::new(&env, &id);
        let admin = Address::generate(&env);
        let r1 = Address::generate(&env);
        let r2 = Address::generate(&env);
        let r3 = Address::generate(&env);
        let feed = String::from_str(&env, "PI/USD");
        let unit = String::from_str(&env, "USD_PER_PI");

        client.initialize(&admin);
        client.set_reporter(&admin, &r1, &true);
        client.set_reporter(&admin, &r2, &true);
        client.set_reporter(&admin, &r3, &true);
        client
            .configure_feed(&admin, &feed, &2, &300, &500, &unit);

        client
            .submit_price(&r1, &feed, &100_000, &1, &unit);
        client
            .submit_price(&r2, &feed, &101_000, &1, &unit);
        client
            .submit_price(&r3, &feed, &900_000, &1, &unit);

        let result = client.get_consensus(&feed);
        assert_eq!(result.median_price, 100_500);
        assert_eq!(result.accepted_count, 2);
        assert_eq!(result.rejected_count, 1);
    }
}
