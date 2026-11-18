#![no_std]
//! Test-only OUSD reserve/liability evidence controller.
//! Records role-attested evidence; it does not custody USD, inspect bank accounts,
//! prove off-chain documents, mint/burn OUSD, or execute payouts.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidRoleConfiguration = 4,
    InvalidAmount = 5,
    UnknownReserve = 6,
    UnknownLiability = 7,
    InsufficientReserve = 8,
    InvalidLiabilityState = 9,
    BeneficiaryMismatch = 10,
    PayoutAmountMismatch = 11,
    ArithmeticError = 12,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Issuer,
    ReserveVerifier,
    PayoutVerifier,
    NextReserveId,
    NextLiabilityId,
    OutstandingUsdMicros,
    BurnableUsdMicros,
    Reserve(u64),
    Liability(u64),
    ReserveEvidence(BytesN<32>),
    LiabilityEvidence(BytesN<32>),
    PayoutEvidence(BytesN<32>),
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LiabilityState {
    Issued,
    RedemptionPending,
    Paid,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReserveRecord {
    pub id: u64,
    pub attestor: Address,
    pub evidence_hash: BytesN<32>,
    pub attested_usd_micros: i128,
    pub available_usd_micros: i128,
    pub allocated_usd_micros: i128,
    pub paid_out_usd_micros: i128,
    pub attested_ledger: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiabilityRecord {
    pub id: u64,
    pub reserve_id: u64,
    pub beneficiary: Address,
    pub evidence_hash: BytesN<32>,
    pub amount_usd_micros: i128,
    pub state: LiabilityState,
    pub payout_evidence_hash: Option<BytesN<32>>,
    pub payout_ledger: Option<u32>,
}

#[contract]
pub struct OusdReserveController;

impl OusdReserveController {
    // Keep unbounded records and replay indexes out of the size-limited instance entry.
    // Active persistent entries are extended to the network maximum whenever accessed.
    // Expired entries require Soroban footprint restoration before the next contract call.
    const TTL_THRESHOLD: u32 = 100_000;

    fn extend_persistent_ttl<K>(env: &Env, key: &K)
    where
        K: soroban_sdk::IntoVal<Env, soroban_sdk::Val>,
    {
        env.storage().persistent().extend_ttl(
            key,
            Self::TTL_THRESHOLD,
            env.storage().max_ttl(),
        );
    }

    fn has_persistent<K>(env: &Env, key: &K) -> bool
    where
        K: soroban_sdk::IntoVal<Env, soroban_sdk::Val>,
    {
        let storage = env.storage().persistent();
        let exists = storage.has(key);
        if exists {
            Self::extend_persistent_ttl(env, key);
        }
        exists
    }

    fn set_persistent_marker<K>(env: &Env, key: &K)
    where
        K: soroban_sdk::IntoVal<Env, soroban_sdk::Val>,
    {
        env.storage().persistent().set(key, &true);
        Self::extend_persistent_ttl(env, key);
    }

    fn require_initialized(env: &Env) -> Result<(), Error> {
        env.storage().instance().extend_ttl(Self::TTL_THRESHOLD, env.storage().max_ttl());
        if env.storage().instance().has(&DataKey::Admin) {
            Ok(())
        } else {
            Err(Error::NotInitialized)
        }
    }
    fn require_role(env: &Env, supplied: &Address, key: DataKey) -> Result<(), Error> {
        supplied.require_auth();
        let stored: Address = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(Error::NotInitialized)?;
        if stored == *supplied {
            Ok(())
        } else {
            Err(Error::Unauthorized)
        }
    }
    fn read_reserve(env: &Env, id: u64) -> Result<ReserveRecord, Error> {
        let key = DataKey::Reserve(id);
        let record = env.storage().persistent().get(&key).ok_or(Error::UnknownReserve)?;
        Self::extend_persistent_ttl(env, &key);
        Ok(record)
    }
    fn write_reserve(env: &Env, record: &ReserveRecord) {
        let key = DataKey::Reserve(record.id);
        env.storage().persistent().set(&key, record);
        Self::extend_persistent_ttl(env, &key);
    }
    fn read_liability(env: &Env, id: u64) -> Result<LiabilityRecord, Error> {
        let key = DataKey::Liability(id);
        let record = env.storage().persistent().get(&key).ok_or(Error::UnknownLiability)?;
        Self::extend_persistent_ttl(env, &key);
        Ok(record)
    }
    fn write_liability(env: &Env, record: &LiabilityRecord) {
        let key = DataKey::Liability(record.id);
        env.storage().persistent().set(&key, record);
        Self::extend_persistent_ttl(env, &key);
    }
}

#[contractimpl]
impl OusdReserveController {
    /// Administrator, issuer, reserve verifier and payout verifier must be distinct.
    pub fn initialize(
        env: Env,
        admin: Address,
        issuer: Address,
        reserve_verifier: Address,
        payout_verifier: Address,
    ) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        if admin == issuer
            || admin == reserve_verifier
            || admin == payout_verifier
            || issuer == reserve_verifier
            || issuer == payout_verifier
            || reserve_verifier == payout_verifier
        {
            return Err(Error::InvalidRoleConfiguration);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Issuer, &issuer);
        env.storage()
            .instance()
            .set(&DataKey::ReserveVerifier, &reserve_verifier);
        env.storage()
            .instance()
            .set(&DataKey::PayoutVerifier, &payout_verifier);
        env.storage().instance().set(&DataKey::NextReserveId, &1u64);
        env.storage()
            .instance()
            .set(&DataKey::NextLiabilityId, &1u64);
        env.storage()
            .instance()
            .set(&DataKey::OutstandingUsdMicros, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::BurnableUsdMicros, &0i128);
        env.storage().instance().extend_ttl(Self::TTL_THRESHOLD, env.storage().max_ttl());
        env.events()
            .publish((symbol_short!("ousd"), symbol_short!("init")), admin);
        Ok(())
    }

    /// Records a settled-USD reserve attestation. The evidence hash is an audit reference,
    /// not proof of bank funds; the verifier is a trust boundary.
    pub fn attest_settled_reserve(
        env: Env,
        verifier: Address,
        evidence_hash: BytesN<32>,
        amount_usd_micros: i128,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env)?;
        Self::require_role(&env, &verifier, DataKey::ReserveVerifier)?;
        if amount_usd_micros <= 0 {
            return Err(Error::InvalidAmount);
        }
        let evidence_key = DataKey::ReserveEvidence(evidence_hash.clone());
        if Self::has_persistent(&env, &evidence_key) {
            return Err(Error::InvalidLiabilityState);
        }
        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextReserveId)
            .unwrap_or(1);
        let next = id.checked_add(1).ok_or(Error::ArithmeticError)?;
        let record = ReserveRecord {
            id,
            attestor: verifier.clone(),
            evidence_hash,
            attested_usd_micros: amount_usd_micros,
            available_usd_micros: amount_usd_micros,
            allocated_usd_micros: 0,
            paid_out_usd_micros: 0,
            attested_ledger: env.ledger().sequence(),
        };
        env.storage().instance().set(&DataKey::NextReserveId, &next);
        Self::set_persistent_marker(&env, &evidence_key);
        Self::write_reserve(&env, &record);
        env.events().publish(
            (symbol_short!("ousd"), symbol_short!("reserve")),
            (id, amount_usd_micros, verifier),
        );
        Ok(id)
    }

    /// Creates an off-chain liability record against a reserve allocation. Does NOT mint OUSD.
    pub fn issue_liability(
        env: Env,
        issuer: Address,
        reserve_id: u64,
        beneficiary: Address,
        evidence_hash: BytesN<32>,
        amount_usd_micros: i128,
    ) -> Result<u64, Error> {
        Self::require_initialized(&env)?;
        Self::require_role(&env, &issuer, DataKey::Issuer)?;
        if amount_usd_micros <= 0 {
            return Err(Error::InvalidAmount);
        }
        let evidence_key = DataKey::LiabilityEvidence(evidence_hash.clone());
        if Self::has_persistent(&env, &evidence_key) {
            return Err(Error::InvalidLiabilityState);
        }
        let mut reserve = Self::read_reserve(&env, reserve_id)?;
        if reserve.available_usd_micros < amount_usd_micros {
            return Err(Error::InsufficientReserve);
        }
        let outstanding: i128 = env
            .storage()
            .instance()
            .get(&DataKey::OutstandingUsdMicros)
            .unwrap_or(0);
        let next_outstanding = outstanding
            .checked_add(amount_usd_micros)
            .ok_or(Error::ArithmeticError)?;
        reserve.available_usd_micros = reserve
            .available_usd_micros
            .checked_sub(amount_usd_micros)
            .ok_or(Error::ArithmeticError)?;
        reserve.allocated_usd_micros = reserve
            .allocated_usd_micros
            .checked_add(amount_usd_micros)
            .ok_or(Error::ArithmeticError)?;
        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextLiabilityId)
            .unwrap_or(1);
        let next_id = id.checked_add(1).ok_or(Error::ArithmeticError)?;
        let record = LiabilityRecord {
            id,
            reserve_id,
            beneficiary: beneficiary.clone(),
            evidence_hash,
            amount_usd_micros,
            state: LiabilityState::Issued,
            payout_evidence_hash: None,
            payout_ledger: None,
        };
        env.storage()
            .instance()
            .set(&DataKey::NextLiabilityId, &next_id);
        env.storage()
            .instance()
            .set(&DataKey::OutstandingUsdMicros, &next_outstanding);
        Self::set_persistent_marker(&env, &evidence_key);
        Self::write_reserve(&env, &reserve);
        Self::write_liability(&env, &record);
        env.events().publish(
            (symbol_short!("ousd"), symbol_short!("issue")),
            (id, reserve_id, beneficiary, amount_usd_micros),
        );
        Ok(id)
    }

    /// Beneficiary-bound request; no payment is made here.
    pub fn request_redemption(
        env: Env,
        beneficiary: Address,
        liability_id: u64,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        beneficiary.require_auth();
        let mut record = Self::read_liability(&env, liability_id)?;
        if record.beneficiary != beneficiary {
            return Err(Error::BeneficiaryMismatch);
        }
        if record.state != LiabilityState::Issued {
            return Err(Error::InvalidLiabilityState);
        }
        record.state = LiabilityState::RedemptionPending;
        Self::write_liability(&env, &record);
        env.events().publish(
            (symbol_short!("ousd"), symbol_short!("redeem")),
            (liability_id, beneficiary),
        );
        Ok(())
    }

    /// Records payout only after a distinct verifier attests exact-amount settled USD.
    /// This contract cannot inspect a bank or payment rail.
    pub fn confirm_settled_payout(
        env: Env,
        verifier: Address,
        liability_id: u64,
        payout_evidence_hash: BytesN<32>,
        settled_amount_usd_micros: i128,
    ) -> Result<(), Error> {
        Self::require_initialized(&env)?;
        Self::require_role(&env, &verifier, DataKey::PayoutVerifier)?;
        let mut liability = Self::read_liability(&env, liability_id)?;
        if liability.state != LiabilityState::RedemptionPending {
            return Err(Error::InvalidLiabilityState);
        }
        if settled_amount_usd_micros != liability.amount_usd_micros {
            return Err(Error::PayoutAmountMismatch);
        }
        let evidence_key = DataKey::PayoutEvidence(payout_evidence_hash.clone());
        if Self::has_persistent(&env, &evidence_key) {
            return Err(Error::InvalidLiabilityState);
        }
        let mut reserve = Self::read_reserve(&env, liability.reserve_id)?;
        reserve.allocated_usd_micros = reserve
            .allocated_usd_micros
            .checked_sub(liability.amount_usd_micros)
            .ok_or(Error::ArithmeticError)?;
        reserve.paid_out_usd_micros = reserve
            .paid_out_usd_micros
            .checked_add(liability.amount_usd_micros)
            .ok_or(Error::ArithmeticError)?;
        let outstanding: i128 = env
            .storage()
            .instance()
            .get(&DataKey::OutstandingUsdMicros)
            .unwrap_or(0);
        let next_outstanding = outstanding
            .checked_sub(liability.amount_usd_micros)
            .ok_or(Error::ArithmeticError)?;
        let burnable: i128 = env
            .storage()
            .instance()
            .get(&DataKey::BurnableUsdMicros)
            .unwrap_or(0);
        let next_burnable = burnable
            .checked_add(liability.amount_usd_micros)
            .ok_or(Error::ArithmeticError)?;
        liability.state = LiabilityState::Paid;
        liability.payout_evidence_hash = Some(payout_evidence_hash);
        liability.payout_ledger = Some(env.ledger().sequence());
        env.storage()
            .instance()
            .set(&DataKey::OutstandingUsdMicros, &next_outstanding);
        env.storage()
            .instance()
            .set(&DataKey::BurnableUsdMicros, &next_burnable);
        Self::set_persistent_marker(&env, &evidence_key);
        Self::write_reserve(&env, &reserve);
        Self::write_liability(&env, &liability);
        env.events().publish(
            (symbol_short!("ousd"), symbol_short!("paid")),
            (liability_id, liability.amount_usd_micros, verifier),
        );
        Ok(())
    }

    pub fn get_reserve(env: Env, reserve_id: u64) -> Result<ReserveRecord, Error> {
        Self::require_initialized(&env)?;
        Self::read_reserve(&env, reserve_id)
    }
    pub fn get_liability(env: Env, liability_id: u64) -> Result<LiabilityRecord, Error> {
        Self::require_initialized(&env)?;
        Self::read_liability(&env, liability_id)
    }
    pub fn outstanding_usd_micros(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::OutstandingUsdMicros)
            .unwrap_or(0)
    }
    /// Paid liabilities are eligible for a separate token-burn workflow; no token is burned here.
    pub fn burnable_usd_micros(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::BurnableUsdMicros)
            .unwrap_or(0)
    }
    pub fn is_burn_authorized(env: Env, liability_id: u64) -> Result<bool, Error> {
        Ok(Self::read_liability(&env, liability_id)?.state == LiabilityState::Paid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn reserve_liability_redemption_and_burn_gate() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(OusdReserveController, ());
        let client = OusdReserveControllerClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let reserve_verifier = Address::generate(&env);
        let payout_verifier = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let reserve_hash = BytesN::from_array(&env, &[1u8; 32]);
        let liability_hash = BytesN::from_array(&env, &[2u8; 32]);
        let payout_hash = BytesN::from_array(&env, &[3u8; 32]);

        client.initialize(&admin, &issuer, &reserve_verifier, &payout_verifier);
        let reserve_id =
            client.attest_settled_reserve(&reserve_verifier, &reserve_hash, &1_000_000);
        let liability_id = client.issue_liability(
            &issuer,
            &reserve_id,
            &beneficiary,
            &liability_hash,
            &400_000,
        );
        assert_eq!(client.outstanding_usd_micros(), 400_000);
        assert_eq!(
            client.get_reserve(&reserve_id).available_usd_micros,
            600_000
        );
        assert!(!client.is_burn_authorized(&liability_id));

        client.request_redemption(&beneficiary, &liability_id);
        client.confirm_settled_payout(&payout_verifier, &liability_id, &payout_hash, &400_000);
        assert_eq!(client.outstanding_usd_micros(), 0);
        assert_eq!(client.burnable_usd_micros(), 400_000);
        assert!(client.is_burn_authorized(&liability_id));
        let final_reserve = client.get_reserve(&reserve_id);
        assert_eq!(final_reserve.available_usd_micros, 600_000);
        assert_eq!(final_reserve.allocated_usd_micros, 0);
        assert_eq!(final_reserve.paid_out_usd_micros, 400_000);

        // The same reserve evidence cannot be counted twice.
        assert!(client
            .try_attest_settled_reserve(&reserve_verifier, &reserve_hash, &1_000_000)
            .is_err());

        // The same liability evidence cannot create a second allocation.
        assert!(client
            .try_issue_liability(
                &issuer,
                &reserve_id,
                &beneficiary,
                &liability_hash,
                &100_000,
            )
            .is_err());

        // A payout evidence hash cannot be replayed for a different liability.
        let second_liability_hash = BytesN::from_array(&env, &[4u8; 32]);
        let second_liability = client.issue_liability(
            &issuer,
            &reserve_id,
            &beneficiary,
            &second_liability_hash,
            &100_000,
        );
        client.request_redemption(&beneficiary, &second_liability);
        assert!(
            client
                .try_confirm_settled_payout(
                    &payout_verifier,
                    &second_liability,
                    &payout_hash,
                    &100_000,
                )
                .is_err()
        );
    }
    #[test]
    fn rejects_wrong_beneficiary_and_mismatched_or_repeated_payout() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(OusdReserveController, ());
        let client = OusdReserveControllerClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let reserve_verifier = Address::generate(&env);
        let payout_verifier = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        let wrong_beneficiary = Address::generate(&env);
        client.initialize(&admin, &issuer, &reserve_verifier, &payout_verifier);
        let reserve_id = client.attest_settled_reserve(
            &reserve_verifier, &BytesN::from_array(&env, &[21u8; 32]), &1_000_000,
        );
        let liability_id = client.issue_liability(
            &issuer, &reserve_id, &beneficiary, &BytesN::from_array(&env, &[22u8; 32]), &400_000,
        );

        assert!(client.try_request_redemption(&wrong_beneficiary, &liability_id).is_err());
        assert_eq!(client.get_liability(&liability_id).state, LiabilityState::Issued);

        client.request_redemption(&beneficiary, &liability_id);
        let payout_hash = BytesN::from_array(&env, &[23u8; 32]);
        assert!(client.try_confirm_settled_payout(
            &payout_verifier, &liability_id, &payout_hash, &399_999,
        ).is_err());
        assert_eq!(client.get_liability(&liability_id).state, LiabilityState::RedemptionPending);
        assert_eq!(client.outstanding_usd_micros(), 400_000);
        assert!(client.try_confirm_settled_payout(
            &payout_verifier, &liability_id, &payout_hash, &400_000,
        ).is_ok());
        assert!(client.try_confirm_settled_payout(
            &payout_verifier, &liability_id, &payout_hash, &400_000,
        ).is_err());
        assert_eq!(client.outstanding_usd_micros(), 0);
        assert_eq!(client.burnable_usd_micros(), 400_000);
    }

    #[test]
    fn rejects_wrong_roles_and_non_positive_amounts() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(OusdReserveController, ());
        let client = OusdReserveControllerClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let reserve_verifier = Address::generate(&env);
        let payout_verifier = Address::generate(&env);
        let stranger = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        client.initialize(&admin, &issuer, &reserve_verifier, &payout_verifier);

        assert!(client.try_attest_settled_reserve(
            &stranger, &BytesN::from_array(&env, &[31u8; 32]), &100,
        ).is_err());
        assert!(client.try_issue_liability(
            &stranger, &1, &beneficiary, &BytesN::from_array(&env, &[32u8; 32]), &1,
        ).is_err());
        assert!(client.try_attest_settled_reserve(
            &reserve_verifier, &BytesN::from_array(&env, &[33u8; 32]), &0,
        ).is_err());
        let reserve_id = client.attest_settled_reserve(
            &reserve_verifier, &BytesN::from_array(&env, &[34u8; 32]), &100,
        );
        assert!(client.try_issue_liability(
            &issuer, &reserve_id, &beneficiary, &BytesN::from_array(&env, &[35u8; 32]), &0,
        ).is_err());
    }

    #[test]
    fn insufficient_reserve_and_overflow_leave_accounting_unchanged() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(OusdReserveController, ());
        let client = OusdReserveControllerClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let reserve_verifier = Address::generate(&env);
        let payout_verifier = Address::generate(&env);
        let beneficiary = Address::generate(&env);
        client.initialize(&admin, &issuer, &reserve_verifier, &payout_verifier);

        let small_reserve = client.attest_settled_reserve(
            &reserve_verifier, &BytesN::from_array(&env, &[41u8; 32]), &10,
        );
        assert!(client.try_issue_liability(
            &issuer, &small_reserve, &beneficiary, &BytesN::from_array(&env, &[42u8; 32]), &11,
        ).is_err());
        assert_eq!(client.get_reserve(&small_reserve).available_usd_micros, 10);

        let max_reserve = client.attest_settled_reserve(
            &reserve_verifier, &BytesN::from_array(&env, &[43u8; 32]), &i128::MAX,
        );
        client.issue_liability(
            &issuer, &max_reserve, &beneficiary, &BytesN::from_array(&env, &[44u8; 32]), &i128::MAX,
        );
        let final_reserve = client.attest_settled_reserve(
            &reserve_verifier, &BytesN::from_array(&env, &[45u8; 32]), &1,
        );
        assert!(client.try_issue_liability(
            &issuer, &final_reserve, &beneficiary, &BytesN::from_array(&env, &[46u8; 32]), &1,
        ).is_err());
        assert_eq!(client.outstanding_usd_micros(), i128::MAX);
        assert_eq!(client.get_reserve(&final_reserve).available_usd_micros, 1);
    }

}
