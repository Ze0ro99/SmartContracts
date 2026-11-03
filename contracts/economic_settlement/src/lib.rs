#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    String,
};

const MAX_PERIOD_AMOUNT: i128 = i128::MAX;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
    DuplicatePeriod = 5,
    PeriodNotFound = 6,
    InvalidState = 7,
    UnderfundedPeriod = 8,
    DuplicateAllocation = 9,
    AllocationExceedsBudget = 10,
    AllocationIncomplete = 11,
    AlreadySettled = 12,
    InvalidDigest = 13,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PeriodState {
    Proposed,
    Approved,
    Settled,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Period {
    pub asset_id: String,
    pub collected_revenue: i128,
    pub deductions: i128,
    pub distributable: i128,
    pub policy_version: u32,
    pub input_digest: BytesN<32>,
    pub state: PeriodState,
    pub allocated_total: i128,
    pub settlement_digest: Option<BytesN<32>>,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Operator(Address),
    Period(BytesN<32>),
    Allocation(BytesN<32>, Address),
}

#[contract]
pub struct EconomicSettlement;

#[contractimpl]
impl EconomicSettlement {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    pub fn set_operator(
        env: Env,
        operator: Address,
        enabled: bool,
    ) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();
        env.storage()
            .persistent()
            .set(&DataKey::Operator(operator), &enabled);
        Ok(())
    }

    pub fn propose_period(
        env: Env,
        operator: Address,
        period_id: BytesN<32>,
        asset_id: String,
        collected_revenue: i128,
        deductions: i128,
        policy_version: u32,
        input_digest: BytesN<32>,
    ) -> Result<i128, Error> {
        Self::require_operator(&env, &operator)?;
        if asset_id.len() == 0 || policy_version == 0 {
            return Err(Error::InvalidDigest);
        }
        if collected_revenue < 0 || deductions < 0 || collected_revenue > MAX_PERIOD_AMOUNT {
            return Err(Error::InvalidAmount);
        }
        if deductions > collected_revenue {
            return Err(Error::UnderfundedPeriod);
        }
        if env.storage().persistent().has(&DataKey::Period(period_id.clone())) {
            return Err(Error::DuplicatePeriod);
        }
        let distributable = collected_revenue
            .checked_sub(deductions)
            .ok_or(Error::InvalidAmount)?;
        let period = Period {
            asset_id,
            collected_revenue,
            deductions,
            distributable,
            policy_version,
            input_digest,
            state: PeriodState::Proposed,
            allocated_total: 0,
            settlement_digest: None,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Period(period_id.clone()), &period);
        env.events().publish(
            (symbol_short!("period"), symbol_short!("proposed")),
            (period_id, distributable),
        );
        Ok(distributable)
    }

    pub fn approve_period(
        env: Env,
        approver: Address,
        period_id: BytesN<32>,
    ) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if approver != admin {
            return Err(Error::Unauthorized);
        }
        approver.require_auth();
        let mut period = Self::load_period(&env, &period_id)?;
        if period.state != PeriodState::Proposed {
            return Err(Error::InvalidState);
        }
        period.state = PeriodState::Approved;
        env.storage()
            .persistent()
            .set(&DataKey::Period(period_id.clone()), &period);
        env.events().publish((symbol_short!("period"), symbol_short!("approved")), period_id);
        Ok(())
    }

    pub fn record_allocation(
        env: Env,
        operator: Address,
        period_id: BytesN<32>,
        participant: Address,
        amount: i128,
        evidence_hash: BytesN<32>,
    ) -> Result<i128, Error> {
        Self::require_operator(&env, &operator)?;
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        let mut period = Self::load_period(&env, &period_id)?;
        if period.state != PeriodState::Approved {
            return Err(Error::InvalidState);
        }
        let key = DataKey::Allocation(period_id.clone(), participant.clone());
        if env.storage().persistent().has(&key) {
            return Err(Error::DuplicateAllocation);
        }
        let next_total = period
            .allocated_total
            .checked_add(amount)
            .ok_or(Error::InvalidAmount)?;
        if next_total > period.distributable {
            return Err(Error::AllocationExceedsBudget);
        }
        period.allocated_total = next_total;
        env.storage().persistent().set(&key, &(amount, evidence_hash));
        env.storage()
            .persistent()
            .set(&DataKey::Period(period_id.clone()), &period);
        env.events().publish(
            (symbol_short!("alloc"), symbol_short!("recorded")),
            (period_id, participant, amount),
        );
        Ok(next_total)
    }

    pub fn settle_period(
        env: Env,
        operator: Address,
        period_id: BytesN<32>,
        settlement_digest: BytesN<32>,
    ) -> Result<(), Error> {
        Self::require_operator(&env, &operator)?;
        let mut period = Self::load_period(&env, &period_id)?;
        if period.state == PeriodState::Settled {
            return Err(Error::AlreadySettled);
        }
        if period.state != PeriodState::Approved {
            return Err(Error::InvalidState);
        }
        if period.allocated_total != period.distributable {
            return Err(Error::AllocationIncomplete);
        }
        period.state = PeriodState::Settled;
        period.settlement_digest = Some(settlement_digest.clone());
        env.storage()
            .persistent()
            .set(&DataKey::Period(period_id.clone()), &period);
        env.events().publish(
            (symbol_short!("period"), symbol_short!("settled")),
            (period_id, settlement_digest),
        );
        Ok(())
    }

    pub fn get_period(env: Env, period_id: BytesN<32>) -> Result<Period, Error> {
        Self::load_period(&env, &period_id)
    }

    pub fn get_allocation(
        env: Env,
        period_id: BytesN<32>,
        participant: Address,
    ) -> Option<(i128, BytesN<32>)> {
        env.storage()
            .persistent()
            .get(&DataKey::Allocation(period_id, participant))
    }

    fn require_operator(env: &Env, operator: &Address) -> Result<(), Error> {
        operator.require_auth();
        let enabled: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Operator(operator.clone()))
            .unwrap_or(false);
        if !enabled {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }

    fn load_period(env: &Env, period_id: &BytesN<32>) -> Result<Period, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Period(period_id.clone()))
            .ok_or(Error::PeriodNotFound)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn period_requires_approval_and_conserves_allocation() {
        let env = Env::default();
        let contract_id = env.register(EconomicSettlement, ());
        let client = EconomicSettlementClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let operator = Address::generate(&env);
        let participant_a = Address::generate(&env);
        let participant_b = Address::generate(&env);
        let period_id = BytesN::from_array(&env, &[7u8; 32]);
        let input_digest = BytesN::from_array(&env, &[8u8; 32]);
        let evidence_a = BytesN::from_array(&env, &[9u8; 32]);
        let evidence_b = BytesN::from_array(&env, &[10u8; 32]);
        let settlement_digest = BytesN::from_array(&env, &[11u8; 32]);

        env.mock_all_auths();
        client.initialize(&admin);
        client.set_operator(&operator, &true);
        assert_eq!(
            client.propose_period(
                &operator,
                &period_id,
                &String::from_str(&env, "PI-TEST"),
                &100,
                &20,
                &1,
                &input_digest
            ),
            80
        );
        assert_eq!(client.get_period(&period_id).state, PeriodState::Proposed);
        client.approve_period(&admin, &period_id);
        client.record_allocation(&operator, &period_id, &participant_a, &30, &evidence_a);
        client.record_allocation(&operator, &period_id, &participant_b, &50, &evidence_b);
        client.settle_period(&operator, &period_id, &settlement_digest);
        let settled = client.get_period(&period_id);
        assert_eq!(settled.state, PeriodState::Settled);
        assert_eq!(settled.allocated_total, 80);
        assert_eq!(
            client.get_allocation(&period_id, &participant_a),
            Some((30, evidence_a))
        );
    }

    #[test]
    #[should_panic]
    fn cannot_settle_underallocated_period() {
        let env = Env::default();
        let contract_id = env.register(EconomicSettlement, ());
        let client = EconomicSettlementClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let operator = Address::generate(&env);
        let period_id = BytesN::from_array(&env, &[1u8; 32]);
        let digest = BytesN::from_array(&env, &[2u8; 32]);
        env.mock_all_auths();
        client.initialize(&admin);
        client.set_operator(&operator, &true);
        client.propose_period(
            &operator,
            &period_id,
            &String::from_str(&env, "PI-TEST"),
            &100,
            &0,
            &1,
            &digest,
        );
        client.approve_period(&admin, &period_id);
        client.settle_period(&operator, &period_id, &digest);
    }
}
