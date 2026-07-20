use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, Events},
    vec, Address, Env, IntoVal, String, Symbol,
};

use crate::types::TransferItem;
use crate::{CarbonMintContract, CarbonMintContractClient};
use crate::bench_support::setup_contract;
use crate::fuzz_harness::setup_contract as setup_fuzz_contract;

/// Registers the contract and returns its client together with the env.
fn setup<'a>() -> (Env, CarbonMintContractClient<'a>, Address) {
    let env = Env::default();
    let contract_id = env.register_contract(None, CarbonMintContract);
    let client = CarbonMintContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    (env, client, admin)
}

/// Convenience helper to build a project id string in tests.
fn project_id(env: &Env) -> String {
    String::from_str(env, "PROJ-001")
}

#[test]
fn test_benchmark_support_helper_sets_up_contract() {
    let (env, client, admin) = setup_contract();
    env.mock_all_auths();
    client.initialize(&admin);

    assert_eq!(client.get_admin(), admin);
}

#[test]
fn test_fuzz_harness_helper_sets_up_contract() {
    let (env, client, admin) = setup_fuzz_contract();
    env.mock_all_auths();
    client.initialize(&admin);

    assert_eq!(client.get_admin(), admin);
}

#[test]
fn test_initialize_sets_admin() {
    let (_env, client, admin) = setup();
    client.initialize(&admin);
    assert_eq!(client.get_admin(), admin);
}

#[test]
fn test_set_admin_rotates_admin() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let new_admin = Address::generate(&env);
    client.set_admin(&new_admin);

    assert_eq!(client.get_admin(), new_admin);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_set_admin_before_init_fails() {
    let (env, client, _admin) = setup();
    env.mock_all_auths();

    let new_admin = Address::generate(&env);
    client.set_admin(&new_admin);
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
fn test_retire_records_self_beneficiary() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);

    let cert_id = client.retire(&issuer, &id, &100);
    let cert = client.get_retirement(&cert_id);
    assert_eq!(cert.beneficiary, String::from_str(&env, "self"));
}

#[test]
fn test_retire_for_records_named_beneficiary() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);

    let beneficiary = String::from_str(&env, "ACME Airlines");
    let cert_id = client.retire_for(&issuer, &id, &250, &beneficiary);

    let cert = client.get_retirement(&cert_id);
    assert_eq!(cert.holder, issuer);
    assert_eq!(cert.amount, 250);
    assert_eq!(cert.beneficiary, beneficiary);
    assert_eq!(client.balance_of(&issuer, &id), 750);
    assert_eq!(client.total_retired(&id), 250);
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
fn test_listing_info_reflects_state() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let buyer = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);

    let listing = client.listing_info(&id);
    assert_eq!(listing.batch_id, id);
    assert_eq!(listing.seller, issuer);
    assert_eq!(listing.price, 5);
    assert!(listing.listed);
    assert_eq!(listing.available, 1_000);

    // Selling reduces the seller's available amount.
    client.buy(&buyer, &id, &300);
    let listing = client.listing_info(&id);
    assert_eq!(listing.available, 700);

    // Delisting flips the listed flag but keeps the price.
    client.unlist(&id);
    let listing = client.listing_info(&id);
    assert!(!listing.listed);
    assert_eq!(listing.price, 5);
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
#[should_panic(expected = "Error(Contract, #10)")]
fn test_transfer_to_self_fails() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);

    // Transferring to oneself is rejected and leaves the balance untouched.
    client.transfer(&issuer, &issuer, &id, &10);
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
fn test_total_minted_accumulates_across_batches() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    assert_eq!(client.total_minted(), 0);

    let issuer = Address::generate(&env);
    client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);
    client.mint_batch(&issuer, &project_id(&env), &2025, &500, &7);

    assert_eq!(client.total_minted(), 1_500);

    // Retiring credits does not reduce the cumulative minted total.
    client.retire(&issuer, &1, &200);
    assert_eq!(client.total_minted(), 1_500);
    assert_eq!(client.total_retired(&1), 200);
}

#[test]
fn test_set_paused_blocks_minting() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    assert!(!client.is_paused());
    client.set_paused(&true);
    assert!(client.is_paused());

    // Unpausing restores minting.
    client.set_paused(&false);
    assert!(!client.is_paused());

    let issuer = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &100, &1);
    assert_eq!(id, 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_mint_while_paused_fails() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    client.set_paused(&true);

    let issuer = Address::generate(&env);
    client.mint_batch(&issuer, &project_id(&env), &2024, &100, &1);
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

#[test]
fn test_storage_schema_version_exposed() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);
    assert_eq!(client.storage_schema_version(), 1);
}

#[test]
fn test_storage_schema_version_persisted_on_init() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);
    // storage_schema_version is written to instance storage during init.
    assert_eq!(client.storage_schema_version(), 1);
}

#[test]
fn test_set_admin_emits_adminset_event() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);
    let new_admin = Address::generate(&env);
    client.set_admin(&new_admin);
    let events = env.events().all();
    assert!(!events.is_empty());
    assert_eq!(client.get_admin(), new_admin);
}

#[test]
fn test_set_paused_emits_paused_event() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);
    client.set_paused(&true);
    let events = env.events().all();
    assert!(!events.is_empty());
    assert!(client.is_paused());
}

#[test]
fn test_list_emits_listed_event() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);
    let issuer = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);
    client.list(&id, &9);
    let events = env.events().all();
    assert!(!events.is_empty());
}

#[test]
fn test_unlist_emits_delisted_event() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);
    let issuer = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);
    client.unlist(&id);
    let events = env.events().all();
    assert!(!events.is_empty());
}

#[test]
fn test_retire_emits_retired_event() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);
    let issuer = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);
    client.retire(&issuer, &id, &100);
    let events = env.events().all();
    assert!(!events.is_empty());
}

#[test]
fn test_transfer_emits_transfer_event() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);
    let issuer = Address::generate(&env);
    let recipient = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);
    client.transfer(&issuer, &recipient, &id, &250);
    let events = env.events().all();
    assert!(!events.is_empty());
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_mint_batch_negative_price_fails() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);
    let issuer = Address::generate(&env);
    client.mint_batch(&issuer, &project_id(&env), &2024, &100, &-1);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_list_negative_price_fails() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);
    let issuer = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &100, &5);
    client.list(&id, &-1);
}

// ---------------------------------------------------------------------------
// batch_transfer tests
// ---------------------------------------------------------------------------

#[test]
fn test_batch_transfer_to_multiple_recipients() {
// Maximum-value arithmetic paths (contract-level)
// ---------------------------------------------------------------------------
// These tests drive the contract through the largest legal i128 amounts to
// verify that the checked-arithmetic helpers in `math` do not overflow at the
// boundaries, and that the contract correctly propagates `Error::Overflow`
// when the boundary is actually exceeded.

/// Minting a single batch whose supply equals `i128::MAX` must succeed: every
/// intermediate `checked_add` and `checked_mul` call must resolve to `Ok`.
#[test]
fn test_mint_batch_max_i128_supply_succeeds() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);

    let recipients = vec![
        &env,
        TransferItem {
            to: alice.clone(),
            amount: 300,
        },
        TransferItem {
            to: bob.clone(),
            amount: 200,
        },
    ];
    client.batch_transfer(&issuer, &id, &recipients);

    assert_eq!(client.balance_of(&issuer, &id), 500);
    assert_eq!(client.balance_of(&alice, &id), 300);
    assert_eq!(client.balance_of(&bob, &id), 200);

    // Batch transfer does not affect circulating supply.
    assert_eq!(client.circulating_supply(&id), 1_000);
}

#[test]
fn test_batch_transfer_single_recipient() {
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &i128::MAX, &1);
    assert_eq!(id, 1);

    let batch = client.get_batch(&id);
    assert_eq!(batch.supply, i128::MAX);
    assert_eq!(client.balance_of(&issuer, &id), i128::MAX);
    assert_eq!(client.total_minted(), i128::MAX);
}

/// Minting a second batch after the total_minted counter has already reached
/// `i128::MAX` must return `Error::Overflow` (contract error code 7), because
/// `checked_add(i128::MAX, any_positive)` would overflow.
#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_second_mint_overflows_total_minted() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    // Saturate total_minted at i128::MAX.
    client.mint_batch(&issuer, &project_id(&env), &2024, &i128::MAX, &1);
    // Any further positive mint must overflow the cumulative counter.
    client.mint_batch(&issuer, &project_id(&env), &2025, &1, &1);
}

/// A buyer purchasing the entire issuer supply in one call (full-supply buy)
/// exercises `move_balance` through `checked_sub(i128::MAX, i128::MAX)` and
/// `checked_add(0, i128::MAX)` — both of which must succeed.
#[test]
fn test_buy_entire_max_supply_succeeds() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let buyer = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &i128::MAX, &1);

    client.buy(&buyer, &id, &i128::MAX);

    assert_eq!(client.balance_of(&issuer, &id), 0);
    assert_eq!(client.balance_of(&buyer, &id), i128::MAX);
    // Circulating supply is unchanged by trading.
    assert_eq!(client.circulating_supply(&id), i128::MAX);
}

/// Transferring the full `i128::MAX` balance in a single call exercises the
/// same `move_balance` arithmetic path as the buy test, but through the
/// `transfer` entrypoint.
#[test]
fn test_transfer_entire_max_supply_succeeds() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let recipient = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);

    let recipients = vec![
        &env,
        TransferItem {
            to: recipient.clone(),
            amount: 400,
        },
    ];
    client.batch_transfer(&issuer, &id, &recipients);

    assert_eq!(client.balance_of(&issuer, &id), 600);
    assert_eq!(client.balance_of(&recipient, &id), 400);
}

#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn test_batch_transfer_zero_recipients_fails() {
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &i128::MAX, &1);

    client.transfer(&issuer, &recipient, &id, &i128::MAX);

    assert_eq!(client.balance_of(&issuer, &id), 0);
    assert_eq!(client.balance_of(&recipient, &id), i128::MAX);
    // Transfer does not affect circulating supply.
    assert_eq!(client.circulating_supply(&id), i128::MAX);
}

/// Retiring the entire `i128::MAX` supply in a single call exercises
/// `checked_sub(i128::MAX, i128::MAX)` (new balance → 0),
/// `checked_add(0, i128::MAX)` (total_retired counter), and
/// `checked_sub(i128::MAX, i128::MAX)` again inside `circulating_supply`.
#[test]
fn test_retire_entire_max_supply_succeeds() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);

    let recipients = vec![&env];
    client.batch_transfer(&issuer, &id, &recipients);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_batch_transfer_zero_amount_fails() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);

    let recipients = vec![
        &env,
        TransferItem {
            to: alice.clone(),
            amount: 0,
        },
    ];
    client.batch_transfer(&issuer, &id, &recipients);
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_batch_transfer_to_self_fails() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);

    let recipients = vec![
        &env,
        TransferItem {
            to: issuer.clone(),
            amount: 100,
        },
        TransferItem {
            to: alice.clone(),
            amount: 200,
        },
    ];
    client.batch_transfer(&issuer, &id, &recipients);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_batch_transfer_unknown_batch_fails() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);

    let recipients = vec![
        &env,
        TransferItem {
            to: alice.clone(),
            amount: 50,
        },
    ];
    client.batch_transfer(&issuer, &999, &recipients);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_batch_transfer_insufficient_aggregate_balance_fails() {
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &i128::MAX, &1);

    let cert_id = client.retire(&issuer, &id, &i128::MAX);
    assert_eq!(cert_id, 1);

    assert_eq!(client.balance_of(&issuer, &id), 0);
    assert_eq!(client.total_retired(&id), i128::MAX);
    assert_eq!(client.circulating_supply(&id), 0);
}

/// After retiring the full supply the issuer's balance is 0; any further
/// retire attempt must fail with `Error::InsufficientBalance` (#5), not panic
/// with an overflow.
#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_retire_after_full_retirement_fails_with_insufficient_balance() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &100, &5);

    // Combined total (150) exceeds the issuer's balance (100).
    let recipients = vec![
        &env,
        TransferItem {
            to: alice.clone(),
            amount: 80,
        },
        TransferItem {
            to: bob.clone(),
            amount: 70,
        },
    ];
    client.batch_transfer(&issuer, &id, &recipients);
}

#[test]
fn test_batch_transfer_emits_event() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);

    let recipients = vec![
        &env,
        TransferItem {
            to: alice.clone(),
            amount: 100,
        },
    ];
    client.batch_transfer(&issuer, &id, &recipients);

    let events = env.events().all();
    assert!(!events.is_empty());
}

#[test]
fn test_batch_transfer_requires_from_auth() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &1_000, &5);

    let recipients = vec![
        &env,
        TransferItem {
            to: alice.clone(),
            amount: 50,
        },
    ];
    let recipients_for_auth = vec![
        &env,
        TransferItem {
            to: alice.clone(),
            amount: 50,
        },
    ];
    client.batch_transfer(&issuer, &id, &recipients);

    let auths = env.auths();
    let (addr, invocation) = auths.last().expect("expected an authorization");
    assert_eq!(addr, &issuer);
    assert_eq!(
        invocation.function,
        AuthorizedFunction::Contract((
            client.address.clone(),
            Symbol::new(&env, "batch_transfer"),
            (issuer.clone(), id, recipients_for_auth).into_val(&env),
        ))
    );
    assert!(invocation.sub_invocations.is_empty());
}

#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn test_batch_transfer_exceeds_max_recipients_fails() {
    let (env, client, admin) = setup();
    env.mock_all_auths();
    client.initialize(&admin);

    let issuer = Address::generate(&env);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &10_000, &5);

    // Build 51 recipients (MAX_RECIPIENTS is 50).
    let mut recipients = vec![&env];
    for _ in 0..51 {
        let to = Address::generate(&env);
        recipients.push_back(TransferItem {
            to,
            amount: 1,
        });
    }
    client.batch_transfer(&issuer, &id, &recipients);
    let id = client.mint_batch(&issuer, &project_id(&env), &2024, &i128::MAX, &1);
    client.retire(&issuer, &id, &i128::MAX);
    // Balance is now 0; even 1 more should trigger InsufficientBalance.
    client.retire(&issuer, &id, &1);
}
