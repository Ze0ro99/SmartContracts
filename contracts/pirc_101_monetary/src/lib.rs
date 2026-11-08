#![no_std]
//! PiRC-101 live-tick IPPR and governed adaptive-QWF reference controller.
//! This contract publishes a market-linked reference and Phi metric only.
//! It does not mint REF, alter native Pi, authorize payments, or hold collateral.

use soroban_sdk::{
    contract, contractclient, contracterror, contractimpl, contracttype, symbol_short,
    Address, BytesN, Env, String,
};

const IPPR_SUPPLY_MULTIPLIER: u128 = 10_000_000;
const PRICE_SCALE: u128 = 1_000_000_000; // nano-USD per USD
const MAX_QWF_ADJUSTMENT_BPS: u32 = 1_000; // governance step limit: 10%

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum OracleError {
    InsufficientQuorum = 8,
    InvalidPolicy = 6,
    Other = 11,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleConsensus {
    pub median_price: u128,
    pub accepted_count: u32,
    pub rejected_count: u32,
    pub observed_at: u64,
    pub unit: String,
}

#[contractclient(name = "OracleQuorumClient")]
pub trait OracleQuorumInterface {
    fn get_consensus(env: Env, feed_id: String) -> Result<OracleConsensus, OracleError>;
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidPolicy = 4,
    InvalidQwf = 5,
    InvalidAdjustment = 6,
    InvalidMetrics = 7,
    OracleUnavailable = 8,
    OracleUnitMismatch = 9,
    ArithmeticOverflow = 10,
    StalePolicyVersion = 11,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Oracle,
    FeedId,
    ExpectedUnit,
    Qwf,
    MinQwf,
    MaxQwf,
    VelocityPpm,
    TvlNanoUsd,
    MetricsObservedAt,
    MetricsMaxAgeSeconds,
    PolicyVersion,
    EvidenceHash,
    QwfUpdatedAt,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EconomicSnapshot {
    pub external_price_nano_usd_per_pi: u128,
    pub ippr_supply_multiplier: u128,
    pub ippr_nano_usd_per_pi: u128,
    pub qwf_current: u128,
    pub network_velocity_ppm: Option<u128>,
    pub tvl_nano_usd: Option<u128>,
    pub phi_ppm: Option<u128>,
    pub oracle_observed_at: u64,
    pub metrics_observed_at: Option<u64>,
    pub qwf_updated_at: u64,
    pub policy_version: u32,
    pub evidence_hash: Option<BytesN<32>>,
}

#[contract]
pub struct PiRc101Monetary;

impl PiRc101Monetary {
    fn require_admin(env: &Env, admin: &Address) -> Result<(), Error> {
        admin.require_auth();
        let stored: Option<Address> = env.storage().instance().get(&DataKey::Admin);
        match stored {
            Some(value) if value == admin.clone() => Ok(()),
            Some(_) => Err(Error::Unauthorized),
            None => Err(Error::NotInitialized),
        }
    }

    fn clamp(value: u128, min: u128, max: u128) -> u128 {
        value.max(min).min(max)
    }
}

#[contractimpl]
impl PiRc101Monetary {
    /// One-time setup. expected_unit must match the oracle's integer quote scale,
    /// for example NANO_USD_PER_PI. QWF is independent from the fixed IPPR multiplier.
    pub fn initialize(
        env: Env,
        admin: Address,
        oracle: Address,
        feed_id: String,
        expected_unit: String,
        qwf_initial: u128,
        qwf_min: u128,
        qwf_max: u128,
        metrics_max_age_seconds: u64,
    ) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        if feed_id.is_empty()
            || expected_unit.is_empty()
            || qwf_min == 0
            || qwf_min > qwf_initial
            || qwf_initial > qwf_max
            || metrics_max_age_seconds == 0
        {
            return Err(Error::InvalidPolicy);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Oracle, &oracle);
        env.storage().instance().set(&DataKey::FeedId, &feed_id);
        env.storage().instance().set(&DataKey::ExpectedUnit, &expected_unit);
        env.storage().instance().set(&DataKey::Qwf, &qwf_initial);
        env.storage().instance().set(&DataKey::MinQwf, &qwf_min);
        env.storage().instance().set(&DataKey::MaxQwf, &qwf_max);
        env.storage().instance().set(&DataKey::MetricsMaxAgeSeconds, &metrics_max_age_seconds);
        env.storage().instance().set(&DataKey::PolicyVersion, &0u32);
        env.storage().instance().set(&DataKey::QwfUpdatedAt, &env.ledger().timestamp());
        env.events().publish(
            (symbol_short!("pirc101"), symbol_short!("init")),
            (qwf_initial, qwf_min, qwf_max),
        );
        Ok(())
    }

    /// Change QWF safety bounds under an incremented policy version.
    /// Current QWF is clamped into the approved range; IPPR is not modified.
    pub fn set_qwf_bounds(
        env: Env,
        admin: Address,
        qwf_min: u128,
        qwf_max: u128,
        policy_version: u32,
        evidence_hash: BytesN<32>,
    ) -> Result<u128, Error> {
        Self::require_admin(&env, &admin)?;
        let previous_version: u32 = env.storage().instance().get(&DataKey::PolicyVersion).unwrap_or(0);
        if policy_version <= previous_version || qwf_min == 0 || qwf_min > qwf_max {
            return Err(Error::InvalidPolicy);
        }
        let current: u128 = env.storage().instance().get(&DataKey::Qwf).ok_or(Error::NotInitialized)?;
        let next = Self::clamp(current, qwf_min, qwf_max);
        env.storage().instance().set(&DataKey::MinQwf, &qwf_min);
        env.storage().instance().set(&DataKey::MaxQwf, &qwf_max);
        env.storage().instance().set(&DataKey::Qwf, &next);
        env.storage().instance().set(&DataKey::PolicyVersion, &policy_version);
        env.storage().instance().set(&DataKey::EvidenceHash, &evidence_hash);
        env.storage().instance().set(&DataKey::QwfUpdatedAt, &env.ledger().timestamp());
        env.events().publish(
            (symbol_short!("pirc101"), symbol_short!("bounds")),
            (qwf_min, qwf_max, next, policy_version, evidence_hash),
        );
        Ok(next)
    }

    /// Apply a governance-approved, bounded QWF adjustment using published velocity
    /// and TVL inputs. The caller must provide the evidence digest and a new policy
    /// version. This method does not autonomously infer an adjustment rate.
    pub fn apply_qwf_adjustment(
        env: Env,
        admin: Address,
        adjustment_bps: i32,
        network_velocity_ppm: u128,
        tvl_nano_usd: u128,
        policy_version: u32,
        evidence_hash: BytesN<32>,
    ) -> Result<u128, Error> {
        Self::require_admin(&env, &admin)?;
        let previous_version: u32 = env.storage().instance().get(&DataKey::PolicyVersion).unwrap_or(0);
        if policy_version <= previous_version {
            return Err(Error::StalePolicyVersion);
        }
        if adjustment_bps.unsigned_abs() > MAX_QWF_ADJUSTMENT_BPS
            || network_velocity_ppm > 100_000_000
            || tvl_nano_usd == 0
        {
            return Err(Error::InvalidMetrics);
        }
        let current: u128 = env.storage().instance().get(&DataKey::Qwf).ok_or(Error::NotInitialized)?;
        let min: u128 = env.storage().instance().get(&DataKey::MinQwf).ok_or(Error::NotInitialized)?;
        let max: u128 = env.storage().instance().get(&DataKey::MaxQwf).ok_or(Error::NotInitialized)?;
        let factor_bps: u128 = if adjustment_bps >= 0 {
            10_000u128 + adjustment_bps as u128
        } else {
            10_000u128 - adjustment_bps.unsigned_abs() as u128
        };
        let proposed = current
            .checked_mul(factor_bps)
            .ok_or(Error::ArithmeticOverflow)?
            / 10_000u128;
        let next = Self::clamp(proposed, min, max);

        env.storage().instance().set(&DataKey::Qwf, &next);
        env.storage().instance().set(&DataKey::VelocityPpm, &network_velocity_ppm);
        env.storage().instance().set(&DataKey::TvlNanoUsd, &tvl_nano_usd);
        env.storage().instance().set(&DataKey::MetricsObservedAt, &env.ledger().timestamp());
        env.storage().instance().set(&DataKey::PolicyVersion, &policy_version);
        env.storage().instance().set(&DataKey::EvidenceHash, &evidence_hash);
        env.storage().instance().set(&DataKey::QwfUpdatedAt, &env.ledger().timestamp());
        env.events().publish(
            (symbol_short!("pirc101"), symbol_short!("qwf")),
            (current, proposed, next, adjustment_bps, network_velocity_ppm, tvl_nano_usd, policy_version, evidence_hash),
        );
        Ok(next)
    }

    /// Read live market-linked IPPR and separate adaptive QWF.
    /// Phi is withheld when the approved velocity metrics are absent or stale.
    pub fn get_snapshot(env: Env) -> Result<EconomicSnapshot, Error> {
        let oracle: Address = env.storage().instance().get(&DataKey::Oracle).ok_or(Error::NotInitialized)?;
        let feed_id: String = env.storage().instance().get(&DataKey::FeedId).ok_or(Error::NotInitialized)?;
        let expected_unit: String = env.storage().instance().get(&DataKey::ExpectedUnit).ok_or(Error::NotInitialized)?;
        let consensus = match OracleQuorumClient::new(&env, &oracle).try_get_consensus(&feed_id) {
            Ok(Ok(value)) => value,
            _ => return Err(Error::OracleUnavailable),
        };
        if consensus.median_price == 0 {
            return Err(Error::OracleUnavailable);
        }
        if consensus.unit != expected_unit {
            return Err(Error::OracleUnitMismatch);
        }
        let qwf: u128 = env.storage().instance().get(&DataKey::Qwf).ok_or(Error::NotInitialized)?;
        let ippr = consensus.median_price
            .checked_mul(IPPR_SUPPLY_MULTIPLIER)
            .ok_or(Error::ArithmeticOverflow)?;
        let now = env.ledger().timestamp();
        let max_age: u64 = env.storage().instance().get(&DataKey::MetricsMaxAgeSeconds).unwrap_or(0);
        let metrics_at: Option<u64> = env.storage().instance().get(&DataKey::MetricsObservedAt);
        let metrics_fresh = metrics_at.map(|at| at <= now && now - at <= max_age).unwrap_or(false);
        let velocity_raw: Option<u128> = env.storage().instance().get(&DataKey::VelocityPpm);
        let tvl_raw: Option<u128> = env.storage().instance().get(&DataKey::TvlNanoUsd);
        let velocity = if metrics_fresh { velocity_raw } else { None };
        let tvl = if metrics_fresh { tvl_raw } else { None };
        let phi = if let Some(v) = velocity {
            Some(
                v.checked_mul(ippr)
                    .ok_or(Error::ArithmeticOverflow)?
                    / (PRICE_SCALE.checked_mul(qwf).ok_or(Error::ArithmeticOverflow)?),
            )
        } else {
            None
        };
        let qwf_updated_at: u64 = env.storage().instance().get(&DataKey::QwfUpdatedAt).unwrap_or(0);
        let policy_version: u32 = env.storage().instance().get(&DataKey::PolicyVersion).unwrap_or(0);
        let evidence_hash: Option<BytesN<32>> = env.storage().instance().get(&DataKey::EvidenceHash);
        Ok(EconomicSnapshot {
            external_price_nano_usd_per_pi: consensus.median_price,
            ippr_supply_multiplier: IPPR_SUPPLY_MULTIPLIER,
            ippr_nano_usd_per_pi: ippr,
            qwf_current: qwf,
            network_velocity_ppm: velocity,
            tvl_nano_usd: tvl,
            phi_ppm: phi,
            oracle_observed_at: consensus.observed_at,
            metrics_observed_at: if metrics_fresh { metrics_at } else { None },
            qwf_updated_at,
            policy_version,
            evidence_hash,
        })
    }

    pub fn get_qwf(env: Env) -> Result<u128, Error> {
        env.storage().instance().get(&DataKey::Qwf).ok_or(Error::NotInitialized)
    }
}

#[cfg(test)]
mod test {
    use super::{
        Error, OracleConsensus, OracleError, OracleQuorumInterface, PiRc101Monetary,
        PiRc101MonetaryClient,
    };
    use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, BytesN, Env, String};

    #[contract]
    pub struct MockOracle;

    #[contractimpl]
    impl OracleQuorumInterface for MockOracle {
        fn get_consensus(env: Env, _feed_id: String) -> Result<OracleConsensus, OracleError> {
            Ok(OracleConsensus {
                median_price: 72_955_500,
                accepted_count: 4,
                rejected_count: 0,
                observed_at: env.ledger().timestamp(),
                unit: String::from_str(&env, "NANO_USD_PER_PI"),
            })
        }
    }

    fn setup(env: &Env) -> (PiRc101MonetaryClient<'_>, Address) {
        env.mock_all_auths();
        let oracle = env.register(MockOracle, ());
        let contract_id = env.register(PiRc101Monetary, ());
        let client = PiRc101MonetaryClient::new(env, &contract_id);
        let admin = Address::generate(env);
        client.initialize(
            &admin,
            &oracle,
            &String::from_str(env, "PI/USD"),
            &String::from_str(env, "NANO_USD_PER_PI"),
            &10_072_123,
            &1_000_000,
            &11_000_000,
            &120,
        );
        (client, admin)
    }

    #[test]
    fn ippr_rebases_from_oracle_and_keeps_qwf_separate() {
        let env = Env::default();
        let (client, _) = setup(&env);
        let snap = client.get_snapshot();
        assert_eq!(snap.external_price_nano_usd_per_pi, 72_955_500);
        assert_eq!(snap.ippr_supply_multiplier, 10_000_000);
        assert_eq!(snap.ippr_nano_usd_per_pi, 729_555_000_000_000);
        assert_eq!(snap.qwf_current, 10_072_123);
        assert_eq!(snap.phi_ppm, None);
    }

    #[test]
    fn governed_qwf_update_changes_phi_not_ippr() {
        let env = Env::default();
        let (client, admin) = setup(&env);
        let digest = BytesN::from_array(&env, &[7u8; 32]);
        let qwf = client.apply_qwf_adjustment(
            &admin,
            &100,
            &511_500,
            &50_000_000_000,
            &1,
            &digest,
        );
        assert_eq!(qwf, 10_172_844);
        let snap = client.get_snapshot();
        assert_eq!(snap.ippr_nano_usd_per_pi, 729_555_000_000_000);
        assert_eq!(snap.network_velocity_ppm, Some(511_500));
        assert_eq!(snap.phi_ppm, Some(36_682));
    }

    #[test]
    fn qwf_adjustment_is_clamped_to_governed_bounds() {
        let env = Env::default();
        let (client, admin) = setup(&env);
        let digest = BytesN::from_array(&env, &[8u8; 32]);
        let qwf = client.apply_qwf_adjustment(
            &admin,
            &1_000,
            &500_000,
            &75_000_000_000,
            &1,
            &digest,
        );
        assert_eq!(qwf, 11_000_000);
    }

    #[test]
    fn invalid_adjustment_and_missing_metrics_are_rejected() {
        let env = Env::default();
        let (client, admin) = setup(&env);
        let digest = BytesN::from_array(&env, &[9u8; 32]);
        let result = client.apply_qwf_adjustment(
            &admin,
            &2_000,
            &500_000,
            &75_000_000_000,
            &1,
            &digest,
        );
        assert_eq!(result, Err(Error::InvalidMetrics));
    }
}
