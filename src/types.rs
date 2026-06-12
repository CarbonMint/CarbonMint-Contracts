use soroban_sdk::{contracttype, Address, String};

/// Keys used to address values in contract storage.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// The registry admin address (instance storage).
    Admin,
    /// Counter for the next batch id (instance storage).
    BatchCounter,
    /// Counter for the next retirement certificate id (instance storage).
    RetirementCounter,
    /// Whether minting is currently paused by the admin (instance storage).
    Paused,
    /// Running total of credits minted across all batches (instance storage).
    TotalMinted,
    /// A batch record keyed by batch id (persistent storage).
    Batch(u64),
    /// A balance keyed by (owner, batch id) (persistent storage).
    Balance(Address, u64),
    /// A retirement certificate keyed by certificate id (persistent storage).
    Retirement(u64),
    /// Running total of retired credits per batch (persistent storage).
    TotalRetired(u64),
}

/// A registered batch of carbon credits.
///
/// A batch is semi-fungible: every credit within the same batch is
/// interchangeable, while credits from different batches are tracked
/// separately.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Batch {
    /// Unique identifier assigned at mint time.
    pub id: u64,
    /// The account that issued / minted the batch.
    pub issuer: Address,
    /// Human-readable project identifier the credits originate from.
    pub project_id: String,
    /// Vintage year of the credits (e.g. 2024).
    pub vintage: u32,
    /// Total amount of credits originally minted for this batch.
    pub supply: i128,
    /// Current listed unit price (in the mock payment asset).
    pub price: i128,
    /// Whether the batch is currently listed for sale.
    pub listed: bool,
}

/// A compact view of a batch's marketplace listing state.
///
/// Returned by read-only queries so clients can render a batch's sale status
/// without fetching the full [`Batch`] record.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Listing {
    /// The batch the listing refers to.
    pub batch_id: u64,
    /// The account offering the credits for sale.
    pub seller: Address,
    /// The current unit price.
    pub price: i128,
    /// Whether the batch is currently listed for sale.
    pub listed: bool,
    /// The amount of credits the seller still holds and can sell.
    pub available: i128,
}

/// A retirement certificate recording the permanent burning of credits.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Retirement {
    /// Unique certificate identifier.
    pub id: u64,
    /// The batch the retired credits belong to.
    pub batch_id: u64,
    /// The holder that retired the credits.
    pub holder: Address,
    /// The amount of credits retired.
    pub amount: i128,
}
