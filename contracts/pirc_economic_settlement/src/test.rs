use super::*;
use soroban_sdk::{testutils::Address as _, Env};

fn id(env: &Env, byte: u8) -> BytesN<32> {
    BytesN::from_array(env, &[byte; 32])
}

fn setup_period(env: &Env, period_id: BytesN<32>) -> (Address, Address, i128) {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let operator = Address::generate(env);
    EconomicSettlement::initialize(env.clone(), admin.clone()).unwrap();
    EconomicSettlement::set_operator(env.clone(), operator.clone(), true).unwrap();
    let distributable = EconomicSettlement::propose_period(
        env.clone(),
        operator.clone(),
        period_id,
        id(env, 2),
        1_000,
        200,
        100,
        100,
        50,
        50,
        1,
        id(env, 3),
    )
    .unwrap();
    (admin, operator, distributable)
}

#[test]
fn settles_only_after_exact_conservation() {
    let env = Env::default();
    let period_id = id(&env, 1);
    let (admin, operator, distributable) = setup_period(&env, period_id.clone());
    assert_eq!(distributable, 500);

    EconomicSettlement::approve_period(env.clone(), period_id.clone()).unwrap();
    assert_eq!(
        EconomicSettlement::settle_period(
            env.clone(), operator.clone(), period_id.clone(), id(&env, 4)
        ),
        Err(Error::AllocationNotConserved)
    );

    let participant = Address::generate(&env);
    EconomicSettlement::record_allocation(
        env.clone(), operator.clone(), period_id.clone(), participant.clone(), 500, id(&env, 5)
    )
    .unwrap();
    EconomicSettlement::settle_period(
        env.clone(), operator.clone(), period_id.clone(), id(&env, 6)
    )
    .unwrap();

    let period = EconomicSettlement::get_period(env.clone(), period_id.clone()).unwrap();
    assert_eq!(period.status, PeriodStatus::Settled);
    assert_eq!(period.allocated, period.distributable);
    assert_eq!(
        EconomicSettlement::settle_period(env.clone(), operator, period_id, id(&env, 7)),
        Err(Error::AlreadySettled)
    );
    let _ = admin;
}

#[test]
fn rejects_underfunded_period() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    EconomicSettlement::initialize(env.clone(), admin).unwrap();
    EconomicSettlement::set_operator(env.clone(), operator.clone(), true).unwrap();

    let result = EconomicSettlement::propose_period(
        env.clone(), operator, id(&env, 8), id(&env, 9),
        100, 70, 20, 20, 0, 0, 1, id(&env, 10),
    );
    assert_eq!(result, Err(Error::InvalidDeductions));
}

#[test]
fn rejects_zero_allocation_and_duplicate_participant() {
    let env = Env::default();
    let period_id = id(&env, 11);
    let (_, operator, _) = setup_period(&env, period_id.clone());
    EconomicSettlement::approve_period(env.clone(), period_id.clone()).unwrap();
    let participant = Address::generate(&env);

    assert_eq!(
        EconomicSettlement::record_allocation(
            env.clone(), operator.clone(), period_id.clone(), participant.clone(), 0, id(&env, 12)
        ),
        Err(Error::InvalidAllocationAmount)
    );
    EconomicSettlement::record_allocation(
        env.clone(), operator.clone(), period_id.clone(), participant.clone(), 400, id(&env, 13)
    ).unwrap();
    assert_eq!(
        EconomicSettlement::record_allocation(
            env.clone(), operator.clone(), period_id.clone(), participant, 1, id(&env, 14)
        ),
        Err(Error::DuplicateAllocation)
    );
}

#[test]
fn rejects_allocation_above_remaining_budget() {
    let env = Env::default();
    let period_id = id(&env, 15);
    let (_, operator, _) = setup_period(&env, period_id.clone());
    EconomicSettlement::approve_period(env.clone(), period_id.clone()).unwrap();
    let participant_a = Address::generate(&env);
    let participant_b = Address::generate(&env);
    EconomicSettlement::record_allocation(
        env.clone(), operator.clone(), period_id.clone(), participant_a, 499, id(&env, 16)
    ).unwrap();
    assert_eq!(
        EconomicSettlement::record_allocation(
            env.clone(), operator, period_id, participant_b, 2, id(&env, 17)
        ),
        Err(Error::AllocationExceedsDistributable)
    );
}

#[test]
fn rejects_zero_policy_version() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    EconomicSettlement::initialize(env.clone(), admin).unwrap();
    EconomicSettlement::set_operator(env.clone(), operator.clone(), true).unwrap();
    assert_eq!(
        EconomicSettlement::propose_period(
            env.clone(), operator, id(&env, 18), id(&env, 19),
            100, 0, 0, 0, 0, 0, 0, id(&env, 20)
        ),
        Err(Error::InvalidPolicyVersion)
    );
}
