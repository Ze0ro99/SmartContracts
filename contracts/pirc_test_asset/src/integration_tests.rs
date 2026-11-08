use super::*;
use soroban_sdk::{testutils::Address as _, Env};

#[test]
fn mint_transfer_allowance_and_burn_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PiRcTestAsset, ());
    let client = PiRcTestAssetClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let spender = Address::generate(&env);

    client.initialize(
        &admin,
        &String::from_str(&env, "PiRC Test Asset"),
        &String::from_str(&env, "PIRCT"),
        &7,
        &1_000,
    );
    client.mint(&admin, &alice, &600);
    assert_eq!(client.balance(&alice), 600);
    assert_eq!(client.total_supply(), 600);

    client.transfer(&alice, &bob, &100);
    assert_eq!(client.balance(&alice), 500);
    assert_eq!(client.balance(&bob), 100);

    let expiration = env.ledger().sequence() + 1_000;
    client.approve(&alice, &spender, &250, &expiration);
    client.transfer_from(&spender, &alice, &bob, &200);
    assert_eq!(client.balance(&alice), 300);
    assert_eq!(client.balance(&bob), 300);
    assert_eq!(client.allowance(&alice, &spender), 50);

    client.burn(&bob, &50);
    assert_eq!(client.balance(&bob), 250);
    assert_eq!(client.total_supply(), 550);
}

#[test]
fn rejects_non_admin_mint_and_supply_cap_overflow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PiRcTestAsset, ());
    let client = PiRcTestAssetClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let other = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.initialize(
        &admin,
        &String::from_str(&env, "PiRC Test Asset"),
        &String::from_str(&env, "PIRCT"),
        &7,
        &100,
    );

    assert_eq!(
        client.try_mint(&other, &recipient, &1),
        Err(Ok(Error::Unauthorized)),
    );
    client.mint(&admin, &recipient, &100);
    assert_eq!(
        client.try_mint(&admin, &recipient, &1),
        Err(Ok(Error::SupplyCapExceeded)),
    );
}

#[test]
fn rejects_invalid_decimals_and_zero_transfers() {
    let env = Env::default();
    env.mock_all_auths();

    let invalid_id = env.register(PiRcTestAsset, ());
    let invalid_client = PiRcTestAssetClient::new(&env, &invalid_id);
    let admin = Address::generate(&env);
    assert_eq!(
        invalid_client.try_initialize(
            &admin,
            &String::from_str(&env, "Invalid"),
            &String::from_str(&env, "BAD"),
            &19,
            &100,
        ),
        Err(Ok(Error::InvalidDecimals)),
    );

    let valid_id = env.register(PiRcTestAsset, ());
    let valid_client = PiRcTestAssetClient::new(&env, &valid_id);
    valid_client.initialize(
        &admin,
        &String::from_str(&env, "Valid"),
        &String::from_str(&env, "VAL"),
        &7,
        &100,
    );
    let alice = Address::generate(&env);
    assert_eq!(
        valid_client.try_transfer(&alice, &admin, &0),
        Err(Ok(Error::InvalidAmount)),
    );
}
