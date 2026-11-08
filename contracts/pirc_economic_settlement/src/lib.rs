#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    PeriodAlreadyExists = 4,
    PeriodNotFound = 5,
    InvalidDeductions = 6,
    InvalidState = 7,
    DuplicateAllocation = 8,
    AllocationExceedsDistributable = 9,
    AllocationNotConserved = 10,
    AlreadySettled = 11,
    ArithmeticOverflow = 12,
    InvalidAllocationAmount = 13,
    InvalidPolicyVersion = 14,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PeriodStatus {
    Proposed,
    Approved,
    Settled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Period {
    pub asset_id: BytesN<32>,
    pub collected_revenue: i128,
    pub deductions: i128,
    pub distributable: i128,
    pub allocated: i128,
    pub policy_version: u32,
    pub input_digest: BytesN<32>,
    pub settlement_digest: Option<BytesN<32>>,
    pub status: PeriodStatus,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Operator(Address),
    Period(BytesN<32>),
    Allocation(BytesN<32>, Address),
}

/// Grouped period inputs keep the Soroban entry point within the host ABI parameter limit.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PeriodProposal {
    pub period_id: BytesN<32>,
    pub asset_id: BytesN<32>,
    pub collected_revenue: i128,
    pub operating_costs: i128,
    pub tax_and_statutory: i128,
    pub refunds_and_due_liabilities: i128,
    pub reserve_replenishment: i128,
    pub approved_commitments: i128,
    pub policy_version: u32,
    pub input_digest: BytesN<32>,
}

#[contract]
pub struct EconomicSettlement;

#[contractimpl]
impl EconomicSettlement {
    /// One-time initialization. The admin must authorize the call.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    /// Enable or disable an operator. Only the admin may change this allowlist.
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
        if enabled {
            env.storage()
                .persistent()
                .set(&DataKey::Operator(operator), &true);
        } else {
            env.storage()
                .persistent()
                .remove(&DataKey::Operator(operator));
        }
        Ok(())
    }

    /// Record a period using collected revenue and aggregate deductions.
    /// All amounts are integer atomic units of exactly one declared asset.
    pub fn propose_period(
        env: Env,
        operator: Address,
        proposal: PeriodProposal,
    ) -> Result<i128, Error> {
        Self::require_operator(&env, &operator)?;
        let PeriodProposal {
            period_id,
            asset_id,
            collected_revenue,
            operating_costs,
            tax_and_statutory,
            refunds_and_due_liabilities,
            reserve_replenishment,
            approved_commitments,
            policy_version,
            input_digest,
        } = proposal;
        if env.storage().persistent().has(&DataKey::Period(period_id.clone())) {
            return Err(Error::PeriodAlreadyExists);
        }
        if policy_version == 0 {
            return Err(Error::InvalidPolicyVersion);
        }

        let values = [
            collected_revenue,
            operating_costs,
            tax_and_statutory,
            refunds_and_due_liabilities,
            reserve_replenishment,
            approved_commitments,
        ];
        if values.iter().any(|value| *value < 0) {
            return Err(Error::InvalidDeductions);
        }

        let deductions = operating_costs
            .checked_add(tax_and_statutory)
            .and_then(|v| v.checked_add(refunds_and_due_liabilities))
            .and_then(|v| v.checked_add(reserve_replenishment))
            .and_then(|v| v.checked_add(approved_commitments))
            .ok_or(Error::ArithmeticOverflow)?;

        if deductions > collected_revenue {
            return Err(Error::InvalidDeductions);
        }
        let distributable = collected_revenue
            .checked_sub(deductions)
            .ok_or(Error::ArithmeticOverflow)?;

        let period = Period {
            asset_id,
            collected_revenue,
            deductions,
            distributable,
            allocated: 0,
            policy_version,
            input_digest,
            settlement_digest: None,
            status: PeriodStatus::Proposed,
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

    /// Admin approval freezes the accounting inputs for this period.
    pub fn approve_period(env: Env, period_id: BytesN<32>) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();

        let key = DataKey::Period(period_id.clone());
        let mut period: Period = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::PeriodNotFound)?;
        if period.status != PeriodStatus::Proposed {
            return Err(Error::InvalidState);
        }
        period.status = PeriodStatus::Approved;
        env.storage().persistent().set(&key, &period);
        env.events()
            .publish((symbol_short!("period"), symbol_short!("approved")), period_id);
        Ok(())
    }

    /// Record one allocation per participant. Allocation values cannot exceed
    /// the approved distributable amount; participant arrays are never stored.
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
            return Err(Error::InvalidAllocationAmount);
        }

        let period_key = DataKey::Period(period_id.clone());
        let mut period: Period = env
            .storage()
            .persistent()
            .get(&period_key)
            .ok_or(Error::PeriodNotFound)?;
        if period.status != PeriodStatus::Approved {
            return Err(Error::InvalidState);
        }

        let allocation_key = DataKey::Allocation(period_id.clone(), participant.clone());
        if env.storage().persistent().has(&allocation_key) {
            return Err(Error::DuplicateAllocation);
        }
        let next_total = period
            .allocated
            .checked_add(amount)
            .ok_or(Error::ArithmeticOverflow)?;
        if next_total > period.distributable {
            return Err(Error::AllocationExceedsDistributable);
        }

        period.allocated = next_total;
        env.storage().persistent().set(&period_key, &period);
        env.storage()
            .persistent()
            .set(&allocation_key, &(amount, evidence_hash));
        env.events().publish(
            (symbol_short!("alloc"), symbol_short!("recorded")),
            (period_id, participant, amount),
        );
        Ok(next_total)
    }

    /// Finalize once, and only when allocations conserve the full distributable.
    pub fn settle_period(
        env: Env,
        operator: Address,
        period_id: BytesN<32>,
        settlement_digest: BytesN<32>,
    ) -> Result<(), Error> {
        Self::require_operator(&env, &operator)?;
        let key = DataKey::Period(period_id.clone());
        let mut period: Period = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::PeriodNotFound)?;
        if period.status == PeriodStatus::Settled {
            return Err(Error::AlreadySettled);
        }
        if period.status != PeriodStatus::Approved {
            return Err(Error::InvalidState);
        }
        if period.allocated != period.distributable {
            return Err(Error::AllocationNotConserved);
        }

        period.status = PeriodStatus::Settled;
        period.settlement_digest = Some(settlement_digest.clone());
        env.storage().persistent().set(&key, &period);
        env.events().publish(
            (symbol_short!("period"), symbol_short!("settled")),
            (period_id, settlement_digest, period.distributable),
        );
        Ok(())
    }

    pub fn get_period(env: Env, period_id: BytesN<32>) -> Result<Period, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Period(period_id))
            .ok_or(Error::PeriodNotFound)
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
        if !env
            .storage()
            .persistent()
            .has(&DataKey::Operator(operator.clone()))
        {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test;
