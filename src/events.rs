//! Event publishers, schemas, and parsers for the CarbonMint contract.
//!
//! Lifecycle events for minting, transfers, trades, and retirements are
//! normalized and schema-versioned to ensure off-chain indexers can safely
//! reconstruct circulating and retired carbon credit supplies without ambiguity.

use soroban_sdk::{contracttype, symbol_short, Address, Env, String, Symbol};

/// Current version for carbon credit lifecycle event schemas.
pub const EVENT_SCHEMA_VERSION: u32 = 1;

/// Standard unit of measure for carbon credits (metric tonnes of CO2 equivalent).
pub const UNIT_TCO2E: Symbol = symbol_short!("tCO2e");

/// Versioned event payload emitted on carbon batch minting.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MintEvent {
    pub version: u32,
    pub batch_id: u64,
    pub quantity: i128,
    pub unit: Symbol,
    pub issuer: Address,
    pub project_id: String,
    pub vintage: u32,
    pub outcome: Symbol,
}

/// Versioned event payload emitted on carbon credit transfers.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferEvent {
    pub version: u32,
    pub batch_id: u64,
    pub quantity: i128,
    pub unit: Symbol,
    pub from: Address,
    pub to: Address,
    pub outcome: Symbol,
}

/// Versioned event payload emitted on marketplace credit purchase (trade).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeEvent {
    pub version: u32,
    pub batch_id: u64,
    pub quantity: i128,
    pub unit: Symbol,
    pub buyer: Address,
    pub seller: Address,
    pub unit_price: i128,
    pub total_price: i128,
    pub outcome: Symbol,
}

/// Versioned event payload emitted on carbon credit retirement (burn).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetirementEvent {
    pub version: u32,
    pub batch_id: u64,
    pub quantity: i128,
    pub unit: Symbol,
    pub holder: Address,
    pub certificate_id: u64,
    pub beneficiary: String,
    pub outcome: Symbol,
}

/// Errors returned by off-chain or on-chain event parsers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    UnsupportedVersion(u32),
    InvalidQuantity,
    InvalidPrice,
    InvalidUnit,
    InvalidOutcome,
}

/// Validates and parses a [`MintEvent`] payload.
pub fn parse_mint_event(event: &MintEvent) -> Result<MintEvent, ParseError> {
    if event.version != EVENT_SCHEMA_VERSION {
        return Err(ParseError::UnsupportedVersion(event.version));
    }
    if event.quantity <= 0 {
        return Err(ParseError::InvalidQuantity);
    }
    if event.unit != symbol_short!("tCO2e") {
        return Err(ParseError::InvalidUnit);
    }
    if event.outcome != symbol_short!("minted") {
        return Err(ParseError::InvalidOutcome);
    }
    Ok(event.clone())
}

/// Validates and parses a [`TransferEvent`] payload.
pub fn parse_transfer_event(event: &TransferEvent) -> Result<TransferEvent, ParseError> {
    if event.version != EVENT_SCHEMA_VERSION {
        return Err(ParseError::UnsupportedVersion(event.version));
    }
    if event.quantity <= 0 {
        return Err(ParseError::InvalidQuantity);
    }
    if event.unit != symbol_short!("tCO2e") {
        return Err(ParseError::InvalidUnit);
    }
    if event.outcome != symbol_short!("transfer") {
        return Err(ParseError::InvalidOutcome);
    }
    Ok(event.clone())
}

/// Validates and parses a [`TradeEvent`] payload.
pub fn parse_trade_event(event: &TradeEvent) -> Result<TradeEvent, ParseError> {
    if event.version != EVENT_SCHEMA_VERSION {
        return Err(ParseError::UnsupportedVersion(event.version));
    }
    if event.quantity <= 0 {
        return Err(ParseError::InvalidQuantity);
    }
    if event.unit_price < 0 || event.total_price < 0 {
        return Err(ParseError::InvalidPrice);
    }
    if event.unit != symbol_short!("tCO2e") {
        return Err(ParseError::InvalidUnit);
    }
    if event.outcome != symbol_short!("settled") {
        return Err(ParseError::InvalidOutcome);
    }
    Ok(event.clone())
}

/// Validates and parses a [`RetirementEvent`] payload.
pub fn parse_retirement_event(event: &RetirementEvent) -> Result<RetirementEvent, ParseError> {
    if event.version != EVENT_SCHEMA_VERSION {
        return Err(ParseError::UnsupportedVersion(event.version));
    }
    if event.quantity <= 0 {
        return Err(ParseError::InvalidQuantity);
    }
    if event.unit != symbol_short!("tCO2e") {
        return Err(ParseError::InvalidUnit);
    }
    if event.outcome != symbol_short!("burned") {
        return Err(ParseError::InvalidOutcome);
    }
    Ok(event.clone())
}

/// Publishes a normalized `minted` event when a new batch is created.
///
/// Topics: `("minted", issuer)`; data: [`MintEvent`].
pub fn minted(
    env: &Env,
    issuer: &Address,
    batch_id: u64,
    amount: i128,
    project_id: &String,
    vintage: u32,
) {
    let topics = (symbol_short!("minted"), issuer.clone());
    let payload = MintEvent {
        version: EVENT_SCHEMA_VERSION,
        batch_id,
        quantity: amount,
        unit: UNIT_TCO2E,
        issuer: issuer.clone(),
        project_id: project_id.clone(),
        vintage,
        outcome: symbol_short!("minted"),
    };
    env.events().publish(topics, payload);
}

/// Publishes a normalized `bought` event when credits change hands.
///
/// Topics: `("bought", buyer, seller)`; data: [`TradeEvent`].
pub fn bought(
    env: &Env,
    buyer: &Address,
    seller: &Address,
    batch_id: u64,
    amount: i128,
    price: i128,
) {
    let topics = (symbol_short!("bought"), buyer.clone(), seller.clone());
    let total_price = amount.saturating_mul(price);
    let payload = TradeEvent {
        version: EVENT_SCHEMA_VERSION,
        batch_id,
        quantity: amount,
        unit: UNIT_TCO2E,
        buyer: buyer.clone(),
        seller: seller.clone(),
        unit_price: price,
        total_price,
        outcome: symbol_short!("settled"),
    };
    env.events().publish(topics, payload);
}

/// Publishes a normalized `retired` event when credits are permanently burned.
///
/// Topics: `("retired", holder)`; data: [`RetirementEvent`].
pub fn retired(
    env: &Env,
    holder: &Address,
    batch_id: u64,
    amount: i128,
    certificate_id: u64,
    beneficiary: &String,
) {
    let topics = (symbol_short!("retired"), holder.clone());
    let payload = RetirementEvent {
        version: EVENT_SCHEMA_VERSION,
        batch_id,
        quantity: amount,
        unit: UNIT_TCO2E,
        holder: holder.clone(),
        certificate_id,
        beneficiary: beneficiary.clone(),
        outcome: symbol_short!("burned"),
    };
    env.events().publish(topics, payload);
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

/// Publishes a normalized `transfer` event when credits move between holders directly.
///
/// Topics: `("transfer", from, to)`; data: [`TransferEvent`].
pub fn transferred(env: &Env, from: &Address, to: &Address, batch_id: u64, amount: i128) {
    let topics = (symbol_short!("transfer"), from.clone(), to.clone());
    let payload = TransferEvent {
        version: EVENT_SCHEMA_VERSION,
        batch_id,
        quantity: amount,
        unit: UNIT_TCO2E,
        from: from.clone(),
        to: to.clone(),
        outcome: symbol_short!("transfer"),
    };
    env.events().publish(topics, payload);
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
