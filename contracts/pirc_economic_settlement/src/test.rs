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
            env.clone(), operator.clone(), period_id.clone(), participant.clone(), 1, id(&env, 14)
        ),
        Err(Error::DuplicateAllocation)
    );
    let period = EconomicSettlement::get_period(env.clone(), period_id.clone()).unwrap();
    assert_eq!(period.allocated, 400, "duplicate attempt must not mutate period totals");
    assert_eq!(
        EconomicSettlement::get_allocation(env, period_id, participant).unwrap().0,
        400,
        "duplicate attempt must not overwrite the original allocation"
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
            env.clone(), operator, period_id.clone(), participant_b.clone(), 2, id(&env, 17)
        ),
        Err(Error::AllocationExceedsDistributable)
    );
    let period = EconomicSettlement::get_period(env.clone(), period_id.clone()).unwrap();
    assert_eq!(period.allocated, 499, "rejected allocation must not mutate period totals");
    assert!(EconomicSettlement::get_allocation(env, period_id, participant_b).is_none());
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


#[test]
fn rejects_negative_revenue_and_negative_deductions() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    EconomicSettlement::initialize(env.clone(), admin).unwrap();
    EconomicSettlement::set_operator(env.clone(), operator.clone(), true).unwrap();

    assert_eq!(
        EconomicSettlement::propose_period(
            env.clone(), operator.clone(), id(&env, 21), id(&env, 22),
            -1, 0, 0, 0, 0, 0, 1, id(&env, 23)
        ),
        Err(Error::InvalidDeductions)
    );
    assert_eq!(
        EconomicSettlement::propose_period(
            env.clone(), operator, id(&env, 24), id(&env, 25),
            100, -1, 0, 0, 0, 0, 1, id(&env, 26)
        ),
        Err(Error::InvalidDeductions)
    );
}

#[test]
fn detects_deduction_sum_overflow() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    EconomicSettlement::initialize(env.clone(), admin).unwrap();
    EconomicSettlement::set_operator(env.clone(), operator.clone(), true).unwrap();

    assert_eq!(
        EconomicSettlement::propose_period(
            env.clone(), operator, id(&env, 27), id(&env, 28),
            i128::MAX, i128::MAX, 1, 0, 0, 0, 1, id(&env, 29)
        ),
        Err(Error::ArithmeticOverflow)
    );
}


#[test]
fn rejects_duplicate_period_id() {
    let env = Env::default();
    let period_id = id(&env, 31);
    let (_, operator, _) = setup_period(&env, period_id.clone());

    assert_eq!(
        EconomicSettlement::propose_period(
            env.clone(),
            operator,
            period_id,
            id(&env, 32),
            1_000,
            0,
            0,
            0,
            0,
            0,
            1,
            id(&env, 33),
        ),
        Err(Error::PeriodAlreadyExists)
    );
}

#[test]
fn revoked_operator_cannot_record_or_settle() {
    let env = Env::default();
    let period_id = id(&env, 34);
    let (_, operator, _) = setup_period(&env, period_id.clone());
    EconomicSettlement::approve_period(env.clone(), period_id.clone()).unwrap();
    EconomicSettlement::set_operator(env.clone(), operator.clone(), false).unwrap();

    assert_eq!(
        EconomicSettlement::record_allocation(
            env.clone(),
            operator.clone(),
            period_id.clone(),
            Address::generate(&env),
            500,
            id(&env, 35),
        ),
        Err(Error::Unauthorized)
    );
    assert_eq!(
        EconomicSettlement::settle_period(env.clone(), operator, period_id, id(&env, 36)),
        Err(Error::Unauthorized)
    );
}

#[test]
fn zero_distributable_period_can_settle_without_allocations() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    let period_id = id(&env, 37);
    EconomicSettlement::initialize(env.clone(), admin).unwrap();
    EconomicSettlement::set_operator(env.clone(), operator.clone(), true).unwrap();

    assert_eq!(
        EconomicSettlement::propose_period(
            env.clone(),
            operator.clone(),
            period_id.clone(),
            id(&env, 38),
            0,
            0,
            0,
            0,
            0,
            0,
            1,
            id(&env, 39),
        )
        .unwrap(),
        0
    );
    EconomicSettlement::approve_period(env.clone(), period_id.clone()).unwrap();
    EconomicSettlement::settle_period(env.clone(), operator, period_id.clone(), id(&env, 40))
        .unwrap();

    let period = EconomicSettlement::get_period(env, period_id).unwrap();
    assert_eq!(period.status, PeriodStatus::Settled);
    assert_eq!(period.allocated, 0);
    assert_eq!(period.distributable, 0);
}

#[test]
fn allocation_sum_overflow_is_rejected_without_state_mutation() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    let period_id = id(&env, 41);
    EconomicSettlement::initialize(env.clone(), admin).unwrap();
    EconomicSettlement::set_operator(env.clone(), operator.clone(), true).unwrap();
    EconomicSettlement::propose_period(
        env.clone(),
        operator.clone(),
        period_id.clone(),
        id(&env, 42),
        i128::MAX,
        0,
        0,
        0,
        0,
        0,
        1,
        id(&env, 43),
    )
    .unwrap();
    EconomicSettlement::approve_period(env.clone(), period_id.clone()).unwrap();

    EconomicSettlement::record_allocation(
        env.clone(),
        operator.clone(),
        period_id.clone(),
        Address::generate(&env),
        i128::MAX - 1,
        id(&env, 44),
    )
    .unwrap();
    assert_eq!(
        EconomicSettlement::record_allocation(
            env.clone(),
            operator,
            period_id.clone(),
            Address::generate(&env),
            2,
            id(&env, 45),
        ),
        Err(Error::ArithmeticOverflow)
    );
    let period = EconomicSettlement::get_period(env, period_id).unwrap();
    assert_eq!(period.allocated, i128::MAX - 1);
}

#[test]
fn settled_period_rejects_late_allocations() {
    let env = Env::default();
    let period_id = id(&env, 46);
    let (_, operator, distributable) = setup_period(&env, period_id.clone());
    EconomicSettlement::approve_period(env.clone(), period_id.clone());
    let participant = Address::generate(&env);
    EconomicSettlement::record_allocation(
        env.clone(),
        operator.clone(),
        period_id.clone(),
        participant,
        distributable,
        id(&env, 47),
    )
    .unwrap();
    EconomicSettlement::settle_period(env.clone(), operator.clone(), period_id.clone(), id(&env, 48))
        .unwrap();

    assert_eq!(
        EconomicSettlement::record_allocation(
            env.clone(),
            operator,
            period_id,
            Address::generate(&env),
            1,
            id(&env, 49),
        ),
        Err(Error::InvalidState)
    );
}
