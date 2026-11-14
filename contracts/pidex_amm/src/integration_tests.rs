use super::*;
use soroban_sdk::{testutils::Address as _, token::StellarAssetClient, Address, Env};

#[test]
fn initializes_deposits_swaps_and_withdraws_with_checked_reserves() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let trader = Address::generate(&env);
    let token_a = env.register_stellar_asset_contract_v2(admin.clone());
    let token_b = env.register_stellar_asset_contract_v2(admin.clone());
    let token_a_id = token_a.address();
    let token_b_id = token_b.address();
    let token_a_admin = StellarAssetClient::new(&env, &token_a_id);
    let token_b_admin = StellarAssetClient::new(&env, &token_b_id);
    token_a_admin.mint(&trader, &20_000);
    token_b_admin.mint(&trader, &40_000);

    let contract_id = env.register(PidexAmm, ());
    let client = PidexAmmClient::new(&env, &contract_id);
    client.initialize(&admin, &token_a_id, &token_b_id);

    let shares = client.deposit(&trader, &10_000, &20_000);
    assert!(shares > 0);
    let (reserve_a, reserve_b, total_shares) = client.get_reserves();
    assert_eq!(reserve_a, 10_000);
    assert_eq!(reserve_b, 20_000);
    assert_eq!(total_shares, shares);
    assert_eq!(client.shares_of(&trader), shares);

    let amount_out = client.swap(&trader, &true, &1_000, &1_800);
    assert!(amount_out >= 1_800);
    let (reserve_a_after, reserve_b_after, _) = client.get_reserves();
    assert_eq!(reserve_a_after, 11_000);
    assert!(reserve_b_after < reserve_b);
    assert!(reserve_a_after * reserve_b_after >= reserve_a * reserve_b);

    let (withdraw_a, withdraw_b) = client.withdraw(&trader, &shares);
    assert!(withdraw_a > 0);
    assert!(withdraw_b > 0);
    let (remaining_a, remaining_b, remaining_shares) = client.get_reserves();
    assert_eq!(remaining_a, 0);
    assert_eq!(remaining_b, 0);
    assert_eq!(remaining_shares, 0);
}

#[test]
fn rejects_slippage_and_blocks_operations_while_paused() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let trader = Address::generate(&env);
    let token_a = env.register_stellar_asset_contract_v2(admin.clone());
    let token_b = env.register_stellar_asset_contract_v2(admin.clone());
    let token_a_id = token_a.address();
    let token_b_id = token_b.address();
    StellarAssetClient::new(&env, &token_a_id).mint(&trader, &20_000);
    StellarAssetClient::new(&env, &token_b_id).mint(&trader, &40_000);

    let contract_id = env.register(PidexAmm, ());
    let client = PidexAmmClient::new(&env, &contract_id);
    client.initialize(&admin, &token_a_id, &token_b_id);
    client.deposit(&trader, &10_000, &20_000);

    assert_eq!(
        client.try_swap(&trader, &true, &1_000, &19_000),
        Err(Ok(Error::SlippageExceeded)),
    );

    client.pause(&true);
    assert_eq!(
        client.try_swap(&trader, &true, &100, &1),
        Err(Ok(Error::Paused)),
    );
    assert_eq!(
        client.try_deposit(&trader, &100, &200),
        Err(Ok(Error::Paused)),
    );
    assert_eq!(client.try_withdraw(&trader, &1), Err(Ok(Error::Paused)),);
}

#[test]
fn read_only_quote_matches_swap_and_does_not_mutate_reserves() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let trader = Address::generate(&env);
    let token_a = env.register_stellar_asset_contract_v2(admin.clone());
    let token_b = env.register_stellar_asset_contract_v2(admin.clone());
    let token_a_id = token_a.address();
    let token_b_id = token_b.address();
    StellarAssetClient::new(&env, &token_a_id).mint(&trader, &20_000);
    StellarAssetClient::new(&env, &token_b_id).mint(&trader, &40_000);

    let contract_id = env.register(PidexAmm, ());
    let client = PidexAmmClient::new(&env, &contract_id);
    client.initialize(&admin, &token_a_id, &token_b_id);
    client.deposit(&trader, &10_000, &20_000);

    let before = client.get_reserves();
    let quote = client.quote(&true, &1_000);
    assert_eq!(quote, 1_813);
    assert_eq!(client.get_reserves(), before);

    let actual = client.swap(&trader, &true, &1_000, &quote);
    assert_eq!(actual, quote);
}

#[test]
fn pool_info_exposes_pair_reserves_pause_and_fee_policy() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_a = env.register_stellar_asset_contract_v2(admin.clone());
    let token_b = env.register_stellar_asset_contract_v2(admin.clone());
    let token_a_id = token_a.address();
    let token_b_id = token_b.address();

    let contract_id = env.register(PidexAmm, ());
    let client = PidexAmmClient::new(&env, &contract_id);
    client.initialize(&admin, &token_a_id, &token_b_id);
    let (a, b, reserve_a, reserve_b, shares, paused, fee_bps) = client.get_pool_info();

    assert_eq!(a, token_a_id);
    assert_eq!(b, token_b_id);
    assert_eq!((reserve_a, reserve_b, shares), (0, 0, 0));
    assert!(!paused);
    assert_eq!(fee_bps, 30);
}

#[test]
fn read_only_quote_rejects_zero_amount_and_paused_pool() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let trader = Address::generate(&env);
    let token_a = env.register_stellar_asset_contract_v2(admin.clone());
    let token_b = env.register_stellar_asset_contract_v2(admin.clone());
    let token_a_id = token_a.address();
    let token_b_id = token_b.address();
    StellarAssetClient::new(&env, &token_a_id).mint(&trader, &20_000);
    StellarAssetClient::new(&env, &token_b_id).mint(&trader, &40_000);

    let contract_id = env.register(PidexAmm, ());
    let client = PidexAmmClient::new(&env, &contract_id);
    client.initialize(&admin, &token_a_id, &token_b_id);
    client.deposit(&trader, &10_000, &20_000);

    assert_eq!(client.try_quote(&true, &0), Err(Ok(Error::ZeroAmount)));
    client.pause(&true);
    assert_eq!(client.try_quote(&true, &1_000), Err(Ok(Error::Paused)));
}
