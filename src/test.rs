#![cfg(test)]

use soroban_sdk::{
    testutils::Address as _,
    Address, Env, String,
};

use crate::{CarbonMintContract, CarbonMintContractClient};

/// Registers the contract and returns its client together with the env.
fn setup<'a>() -> (Env, CarbonMintContractClient<'a>, Address) {
    let env = Env::default();
    let contract_id = env.register(CarbonMintContract, ());
    let client = CarbonMintContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    (env, client, admin)
}

/// Convenience helper to build a project id string in tests.
fn project_id(env: &Env) -> String {
    String::from_str(env, "PROJ-001")
}

#[test]
fn test_initialize_sets_admin() {
    let (_env, client, admin) = setup();
    client.initialize(&admin);
    assert_eq!(client.get_admin(), admin);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_initialize_twice_fails() {
    let (_env, client, admin) = setup();
    client.initialize(&admin);
    client.initialize(&admin);
}

#[test]
fn test_mint_batch_credits_issuer() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);
    assert_eq!(id, 1);

    let batch = client.get_batch(&id);
    assert_eq!(batch.issuer, issuer);
    assert_eq!(batch.vintage, 2024);
    assert_eq!(batch.supply, 1_000);
    assert_eq!(batch.price, 5);
    assert!(batch.listed);

    assert_eq!(client.balance_of(&issuer, &id), 1_000);
    assert_eq!(client.batch_count(), 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_mint_batch_zero_amount_fails() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    client.mint_batch(&issuer, &project_id(&env), &2024, &0, &5);
}

#[test]
fn test_mint_batch_increments_ids() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let first = client.mint_batch(&issuer, &project_id(&env), &2024, &100, &1);
    let second = client.mint_batch(&issuer, &project_id(&env), &2025, &200, &2);
    assert_eq!(first, 1);
    assert_eq!(second, 2);
    assert_eq!(client.batch_count(), 2);
}

#[test]
fn test_buy_transfers_balances() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let buyer = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);

    client.buy(&buyer, &id, &300);

    assert_eq!(client.balance_of(&issuer, &id), 700);
    assert_eq!(client.balance_of(&buyer, &id), 300);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_buy_insufficient_balance_fails() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let buyer = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &100, &5);

    // Seller only has 100 credits; buying 101 must fail.
    client.buy(&buyer, &id, &101);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_buy_unknown_batch_fails() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let buyer = Address::generate(&env);
    client.buy(&buyer, &999, &1);
}

#[test]
fn test_retire_burns_and_issues_certificate() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);

    let cert_id = client.retire(&issuer, &id, &400);
    assert_eq!(cert_id, 1);

    // Burned credits leave the holder's balance.
    assert_eq!(client.balance_of(&issuer, &id), 600);

    let cert = client.get_retirement(&cert_id);
    assert_eq!(cert.id, 1);
    assert_eq!(cert.batch_id, id);
    assert_eq!(cert.holder, issuer);
    assert_eq!(cert.amount, 400);

    assert_eq!(client.retirement_count(), 1);
}

#[test]
fn test_total_retired_accumulates() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);

    assert_eq!(client.total_retired(&id), 0);

    client.retire(&issuer, &id, &100);
    client.retire(&issuer, &id, &250);

    assert_eq!(client.total_retired(&id), 350);
    assert_eq!(client.balance_of(&issuer, &id), 650);
    assert_eq!(client.retirement_count(), 2);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_retire_insufficient_balance_fails() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &100, &5);

    client.retire(&issuer, &id, &101);
}
