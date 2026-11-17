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
    fn require_initialized(env: &Env) -> Result<(), Error> {
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
        env.storage()
            .instance()
            .get(&DataKey::Reserve(id))
            .ok_or(Error::UnknownReserve)
    }
    fn write_reserve(env: &Env, record: &ReserveRecord) {
        env.storage()
            .instance()
            .set(&DataKey::Reserve(record.id), record);
    }
    fn read_liability(env: &Env, id: u64) -> Result<LiabilityRecord, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Liability(id))
            .ok_or(Error::UnknownLiability)
    }
    fn write_liability(env: &Env, record: &LiabilityRecord) {
        env.storage()
            .instance()
            .set(&DataKey::Liability(record.id), record);
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
        if env.storage().instance().has(&evidence_key) {
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
        env.storage().instance().set(&evidence_key, &true);
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
        if env.storage().instance().has(&evidence_key) {
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
        env.storage().instance().set(&evidence_key, &true);
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
        if env.storage().instance().has(&evidence_key) {
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
        env.storage().instance().set(&evidence_key, &true);
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
    }
}
