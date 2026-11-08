use super::*;
use soroban_sdk::{testutils::Address as _, Env};

fn id(env: &Env, byte: u8) -> BytesN<32> {
    BytesN::from_array(env, &[byte; 32])
}

fn proposal(
    _env: &Env,
    period_id: BytesN<32>,
    asset_id: BytesN<32>,
    amounts: [i128; 6],
    policy_version: u32,
    input_digest: BytesN<32>,
) -> PeriodProposal {
    PeriodProposal {
        period_id,
        asset_id,
        collected_revenue: amounts[0],
        operating_costs: amounts[1],
        tax_and_statutory: amounts[2],
        refunds_and_due_liabilities: amounts[3],
        reserve_replenishment: amounts[4],
        approved_commitments: amounts[5],
        policy_version,
        input_digest,
    }
}

fn setup_period(env: &Env, client: &EconomicSettlementClient, period_id: BytesN<32>) -> (Address, Address, i128) {
    let admin = Address::generate(env);
    let operator = Address::generate(env);
    client.try_initialize(&admin.clone()).unwrap().unwrap().unwrap();
    client.try_set_operator(&operator, &true).unwrap().unwrap().unwrap();
    let distributable = client.try_propose_period(&operator, &proposal(env, period_id, id(env, 2), [1_000, 200, 100, 100, 50, 50], 1, id(env, 3)))
    .unwrap();
    (admin, operator, distributable)
}

#[test]
fn settles_only_after_exact_conservation() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EconomicSettlement, ());
    let client = EconomicSettlementClient::new(&env, &contract_id);
    let period_id = id(&env, 1);
    let (admin, operator, distributable) = setup_period(&env, &client, period_id.clone());
    assert_eq!(distributable, 500);

    client.try_approve_period(&period_id.clone()).unwrap().unwrap().unwrap();
    assert_eq!(
        client.try_settle_period(&operator, &period_id, &id(&env, 4)),
        Err(Ok(Error::AllocationNotConserved))
    );

    let participant = Address::generate(&env);
    client.try_record_allocation(&operator, &period_id, &participant, &500, &id(&env, 5))
    .unwrap();
    client.try_settle_period(&operator, &period_id, &id(&env, 6))
    .unwrap();

    let period = client.get_period(&period_id).unwrap();
    assert_eq!(period.status, PeriodStatus::Settled);
    assert_eq!(period.allocated, period.distributable);
    assert_eq!(
        client.try_settle_period(&operator, &period_id, &id(&env, 7)),
        Err(Ok(Error::AlreadySettled))
    );
    let _ = admin;

#[test]
fn rejects_underfunded_period() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EconomicSettlement, ());
    let client = EconomicSettlementClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    client.try_initialize(&admin).unwrap().unwrap().unwrap();
    client.try_set_operator(&operator, &true).unwrap().unwrap().unwrap();

    let result = client.try_propose_period(&operator, &proposal(&env, id(&env, 8), id(&env, 9), [100, 70, 20, 20, 0, 0], 1, id(&env, 10)));
    assert_eq!(result, Err(Ok(Error::InvalidDeductions)));

#[test]
fn rejects_zero_allocation_and_duplicate_participant() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EconomicSettlement, ());
    let client = EconomicSettlementClient::new(&env, &contract_id);
    let period_id = id(&env, 11);
    let (_, operator, _) = setup_period(&env, &client, period_id.clone());
    client.try_approve_period(&period_id.clone()).unwrap().unwrap().unwrap();
    let participant = Address::generate(&env);

    assert_eq!(
        client.try_record_allocation(&operator, &period_id, &participant, &0, &id(&env, 12)),
        Err(Ok(Error::InvalidAllocationAmount))
    );
    client.try_record_allocation(&operator, &period_id, &participant, &400, &id(&env, 13)).unwrap();
    assert_eq!(
        client.try_record_allocation(&operator, &period_id, &participant, &1, &id(&env, 14)),
        Err(Ok(Error::DuplicateAllocation))
    );
    let period = client.get_period(&period_id).unwrap();
    assert_eq!(period.allocated, 400, "duplicate attempt must not mutate period totals");
    assert_eq!(
        client.get_allocation(&period_id, &participant).unwrap().0,
        400,
        "duplicate attempt must not overwrite the original allocation"
    );

#[test]
fn rejects_allocation_above_remaining_budget() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EconomicSettlement, ());
    let client = EconomicSettlementClient::new(&env, &contract_id);
    let period_id = id(&env, 15);
    let (_, operator, _) = setup_period(&env, &client, period_id.clone());
    client.try_approve_period(&period_id.clone()).unwrap().unwrap().unwrap();
    let participant_a = Address::generate(&env);
    let participant_b = Address::generate(&env);
    client.try_record_allocation(&operator, &period_id, &participant_a, &499, &id(&env, 16)).unwrap();
    assert_eq!(
        client.try_record_allocation(&operator, &period_id, &participant_b, &2, &id(&env, 17)),
        Err(Ok(Error::AllocationExceedsDistributable))
    );
    let period = client.get_period(&period_id).unwrap();
    assert_eq!(period.allocated, 499, "rejected allocation must not mutate period totals");
    assert!(client.get_allocation(&period_id, &participant_b).is_none());

#[test]
fn rejects_zero_policy_version() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EconomicSettlement, ());
    let client = EconomicSettlementClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    client.try_initialize(&admin).unwrap().unwrap().unwrap();
    client.try_set_operator(&operator, &true).unwrap().unwrap().unwrap();
    assert_eq!(
        client.try_propose_period(&operator, &proposal(&env, id(&env, 18), id(&env, 19), [100, 0, 0, 0, 0, 0], 0, id(&env, 20))),
        Err(Ok(Error::InvalidPolicyVersion))
    );

#[test]
fn rejects_negative_revenue_and_negative_deductions() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EconomicSettlement, ());
    let client = EconomicSettlementClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    client.try_initialize(&admin).unwrap().unwrap().unwrap();
    client.try_set_operator(&operator, &true).unwrap().unwrap().unwrap();

    assert_eq!(
        client.try_propose_period(&operator, &proposal(&env, id(&env, 21), id(&env, 22), [-1, 0, 0, 0, 0, 0], 1, id(&env, 23))),
        Err(Ok(Error::InvalidDeductions))
    );
    assert_eq!(
        client.try_propose_period(&operator, &proposal(&env, id(&env, 24), id(&env, 25), [100, -1, 0, 0, 0, 0], 1, id(&env, 26))),
        Err(Ok(Error::InvalidDeductions))
    );

#[test]
fn detects_deduction_sum_overflow() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EconomicSettlement, ());
    let client = EconomicSettlementClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    client.try_initialize(&admin).unwrap().unwrap().unwrap();
    client.try_set_operator(&operator, &true).unwrap().unwrap().unwrap();

    assert_eq!(
        client.try_propose_period(&operator, &proposal(&env, id(&env, 27), id(&env, 28), [i128::MAX, i128::MAX, 1, 0, 0, 0], 1, id(&env, 29))),
        Err(Ok(Error::ArithmeticOverflow))
    );

#[test]
fn rejects_duplicate_period_id() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EconomicSettlement, ());
    let client = EconomicSettlementClient::new(&env, &contract_id);
    let period_id = id(&env, 31);
    let (_, operator, _) = setup_period(&env, &client, period_id.clone());

    assert_eq!(
        client.try_propose_period(&operator, &proposal(&env, period_id, id(&env, 32), [1_000, 0, 0, 0, 0, 0], 1, id(&env, 33))),
        Err(Ok(Error::PeriodAlreadyExists))
    );

#[test]
fn revoked_operator_cannot_record_or_settle() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EconomicSettlement, ());
    let client = EconomicSettlementClient::new(&env, &contract_id);
    let period_id = id(&env, 34);
    let (_, operator, _) = setup_period(&env, &client, period_id.clone());
    client.try_approve_period(&period_id.clone()).unwrap().unwrap().unwrap();
    client.try_set_operator(&operator, &false).unwrap().unwrap().unwrap();

    assert_eq!(
        client.try_record_allocation(&operator, &period_id, &Address::generate(&env), &500, &id(&env, 35)),
        Err(Ok(Error::Unauthorized))
    );
    assert_eq!(
        client.try_settle_period(&operator, &period_id, &id(&env, 36)),
        Err(Ok(Error::Unauthorized))
    );

#[test]
fn zero_distributable_period_can_settle_without_allocations() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EconomicSettlement, ());
    let client = EconomicSettlementClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    let period_id = id(&env, 37);
    client.try_initialize(&admin).unwrap().unwrap().unwrap();
    client.try_set_operator(&operator, &true).unwrap().unwrap().unwrap();

    assert_eq!(
        client.try_propose_period(&operator, &proposal(&env, period_id.clone(), id(&env, 38), [0, 0, 0, 0, 0, 0], 1, id(&env, 39)))
        .unwrap(),
        0
    );
    client.try_approve_period(&period_id.clone()).unwrap().unwrap().unwrap();
    client.try_settle_period(&operator, &period_id, &id(&env, 40))
        .unwrap();

    let period = client.get_period(&period_id).unwrap();
    assert_eq!(period.status, PeriodStatus::Settled);
    assert_eq!(period.allocated, 0);
    assert_eq!(period.distributable, 0);

#[test]
fn allocation_sum_overflow_is_rejected_without_state_mutation() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EconomicSettlement, ());
    let client = EconomicSettlementClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    let period_id = id(&env, 41);
    client.try_initialize(&admin).unwrap().unwrap().unwrap();
    client.try_set_operator(&operator, &true).unwrap().unwrap().unwrap();
    client.try_propose_period(&operator, &proposal(&env, period_id.clone(), id(&env, 42), [i128::MAX, 0, 0, 0, 0, 0], 1, id(&env, 43)))
    .unwrap();
    client.try_approve_period(&period_id.clone()).unwrap().unwrap().unwrap();

    client.try_record_allocation(&operator, &period_id, &Address::generate(&env), &i128::MAX - 1, &id(&env, 44))
    .unwrap();
    assert_eq!(
        client.try_record_allocation(&operator, &period_id, &Address::generate(&env), &2, &id(&env, 45)),
        Err(Ok(Error::ArithmeticOverflow))
    );
    let period = client.get_period(&period_id).unwrap();
    assert_eq!(period.allocated, i128::MAX - 1);

#[test]
fn settled_period_rejects_late_allocations() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EconomicSettlement, ());
    let client = EconomicSettlementClient::new(&env, &contract_id);
    let period_id = id(&env, 46);
    let (_, operator, distributable) = setup_period(&env, &client, period_id.clone());
    client.try_approve_period(&period_id.clone()).unwrap().unwrap().unwrap();
    let participant = Address::generate(&env);
    client.try_record_allocation(&operator, &period_id, &participant, &distributable, &id(&env, 47))
    .unwrap();
    client.try_settle_period(&operator, &period_id, &id(&env, 48))
        .unwrap();

    assert_eq!(
        client.try_record_allocation(&operator, &period_id, &Address::generate(&env), &1, &id(&env, 49)),
        Err(Ok(Error::InvalidState))
    );
