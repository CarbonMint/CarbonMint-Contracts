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
