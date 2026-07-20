//! Event publishers for the CarbonMint contract.
//!
//! Each marketplace action emits a structured event so off-chain indexers can
//! reconstruct registry state: `minted`, `listed`, `delisted`, `bought`,
//! `transfer`, `retired`, `paused`, `adminset`, and `batch_transfer`.

use soroban_sdk::{symbol_short, Address, Env};

/// Publishes a `minted` event when a new batch is created.
///
/// Topics: `("minted", issuer)`; data: `(batch_id, amount)`.
pub fn minted(env: &Env, issuer: &Address, batch_id: u64, amount: i128) {
    let topics = (symbol_short!("minted"), issuer.clone());
    env.events().publish(topics, (batch_id, amount));
}

/// Publishes a `bought` event when credits change hands.
///
/// Topics: `("bought", buyer, seller)`; data: `(batch_id, amount, price)`.
pub fn bought(
    env: &Env,
    buyer: &Address,
    seller: &Address,
    batch_id: u64,
    amount: i128,
    price: i128,
) {
    let topics = (symbol_short!("bought"), buyer.clone(), seller.clone());
    env.events().publish(topics, (batch_id, amount, price));
}

/// Publishes a `retired` event when credits are permanently burned.
///
/// Topics: `("retired", holder)`; data: `(batch_id, amount, certificate_id)`.
pub fn retired(env: &Env, holder: &Address, batch_id: u64, amount: i128, certificate_id: u64) {
    let topics = (symbol_short!("retired"), holder.clone());
    env.events()
        .publish(topics, (batch_id, amount, certificate_id));
}

/// Publishes a `listed` event when a batch is listed or repriced.
///
/// Topics: `("listed", issuer)`; data: `(batch_id, price)`.
pub fn listed(env: &Env, issuer: &Address, batch_id: u64, price: i128) {
    let topics = (symbol_short!("listed"), issuer.clone());
    env.events().publish(topics, (batch_id, price));
}

/// Publishes a `delisted` event when a batch is removed from sale.
///
/// Topics: `("delisted", issuer)`; data: `batch_id`.
pub fn delisted(env: &Env, issuer: &Address, batch_id: u64) {
    let topics = (symbol_short!("delisted"), issuer.clone());
    env.events().publish(topics, batch_id);
}

/// Publishes a `transfer` event when credits move between holders directly.
///
/// Topics: `("transfer", from, to)`; data: `(batch_id, amount)`.
pub fn transferred(env: &Env, from: &Address, to: &Address, batch_id: u64, amount: i128) {
    let topics = (symbol_short!("transfer"), from.clone(), to.clone());
    env.events().publish(topics, (batch_id, amount));
}

/// Publishes a `paused` event when the admin toggles the pause flag.
///
/// Topics: `("paused", admin)`; data: `paused` (the new flag value).
pub fn paused(env: &Env, admin: &Address, paused: bool) {
    let topics = (symbol_short!("paused"), admin.clone());
    env.events().publish(topics, paused);
}

/// Publishes a `batch_transfer` event when credits are transferred to
/// multiple recipients in a single invocation.
///
/// Topics: `("batch_transfer", from)`; data: `(batch_id, recipient_count,
/// total_amount)`.
pub fn batch_transferred(
    env: &Env,
    from: &Address,
    batch_id: u64,
    recipient_count: u32,
    total_amount: i128,
) {
    let topics = (symbol_short!("batch_xfr"), from.clone());
    env.events()
        .publish(topics, (batch_id, recipient_count, total_amount));
}

/// Publishes an `adminset` event when the registry admin is rotated.
///
/// Topics: `("adminset", old_admin)`; data: `new_admin`.
pub fn admin_set(env: &Env, old_admin: &Address, new_admin: &Address) {
    let topics = (symbol_short!("adminset"), old_admin.clone());
    env.events().publish(topics, new_admin.clone());
}
