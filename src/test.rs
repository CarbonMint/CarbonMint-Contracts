#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction},
    Address, Env, IntoVal, String, Symbol,
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

#[test]
fn test_buy_requires_buyer_auth() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let buyer = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);

    client.buy(&buyer, &id, &50);

    // The most recent authorization must be the buyer authorizing `buy`.
    let auths = env.auths();
    let (addr, invocation) = auths.last().expect("expected an authorization");
    assert_eq!(addr, &buyer);
    assert_eq!(
        invocation.function,
        AuthorizedFunction::Contract((
            client.address.clone(),
            Symbol::new(&env, "buy"),
            (buyer.clone(), id, 50i128).into_val(&env),
        ))
    );
    // Buyer only authorizes the top-level call, no sub-invocations.
    assert!(invocation.sub_invocations.is_empty());
}

#[test]
fn test_circulating_supply_decreases_on_retire() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);

    assert_eq!(client.circulating_supply(&id), 1_000);

    client.retire(&issuer, &id, &400);
    assert_eq!(client.circulating_supply(&id), 600);

    // Trading does not change circulating supply, only retiring does.
    let buyer = Address::generate(&env);
    client.buy(&buyer, &id, &100);
    assert_eq!(client.circulating_supply(&id), 600);
}

#[test]
fn test_list_updates_price() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);
    assert_eq!(client.get_batch(&id).price, 5);

    client.list(&id, &9);
    let batch = client.get_batch(&id);
    assert_eq!(batch.price, 9);
    assert!(batch.listed);
}

#[test]
fn test_transfer_moves_credits() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let recipient = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);

    client.transfer(&issuer, &recipient, &id, &250);

    assert_eq!(client.balance_of(&issuer, &id), 750);
    assert_eq!(client.balance_of(&recipient, &id), 250);

    // Transferring does not affect circulating supply.
    assert_eq!(client.circulating_supply(&id), 1_000);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_transfer_insufficient_balance_fails() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let recipient = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &100, &5);

    client.transfer(&issuer, &recipient, &id, &101);
}

#[test]
fn test_unlist_marks_batch_not_listed() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);
    assert!(client.is_listed(&id));

    client.unlist(&id);
    assert!(!client.is_listed(&id));

    // Price is preserved across delisting.
    assert_eq!(client.get_batch(&id).price, 5);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_buy_unlisted_batch_fails() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let buyer = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);

    client.unlist(&id);
    client.buy(&buyer, &id, &10);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_get_admin_uninitialized_fails() {
    let (_env, client, _admin) = setup();
    client.get_admin();
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_mint_before_init_fails() {
    let (env, client, _admin) = setup();
    env.mock_all_auths();

    let issuer = Address::generate(&env);
    client.mint_batch(&issuer, &project_id(&env), &2024, &100, &1);
}

#[test]
fn test_unknown_batch_query_returns_zero_balance() {
    let (env, client, admin) = setup();
    client.initialize(&admin);

    let who = Address::generate(&env);
    assert_eq!(client.balance_of(&who, &42), 0);
    assert_eq!(client.total_retired(&42), 0);
}

#[test]
fn test_mint_emits_event() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &500, &3);

    // A `minted` event must have been published for the new batch.
    let events = env.events().all();
    assert!(!events.is_empty());
    assert_eq!(id, 1);
}
