//! Storage helpers for the CarbonMint contract.
//!
//! Instance storage holds the admin address and the batch / retirement
//! counters. Persistent storage holds the per-batch records, per-`(owner,
//! batch)` balances, retirement certificates, and per-batch retired totals.
//! All persistent reads and writes bump the entry's time-to-live.

use soroban_sdk::{Address, Env};

use crate::math;
use crate::types::{Batch, DataKey, Retirement};

/// Number of ledgers (~5s each) after which persistent entries expire if not
/// bumped. Roughly 30 days.
const PERSISTENT_LIFETIME: u32 = 518_400;
/// Bump threshold: extend the TTL once it drops below this many ledgers.
const PERSISTENT_THRESHOLD: u32 = 103_680;

/// Number of ledgers after which the instance entry expires if not bumped.
const INSTANCE_LIFETIME: u32 = 518_400;
/// Bump threshold for the instance entry.
const INSTANCE_THRESHOLD: u32 = 103_680;

/// Extend the time-to-live of the instance storage entry.
pub fn extend_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_THRESHOLD, INSTANCE_LIFETIME);
}

/// Extend the time-to-live of a persistent storage entry.
pub fn extend_persistent(env: &Env, key: &DataKey) {
    env.storage()
        .persistent()
        .extend_ttl(key, PERSISTENT_THRESHOLD, PERSISTENT_LIFETIME);
}

/// Returns `true` if the contract has been initialized.
pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

/// Stores the registry admin address.
pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

/// Reads the registry admin address.
pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

/// Reads the current batch counter (number of batches minted so far).
pub fn get_batch_counter(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::BatchCounter)
        .unwrap_or(0)
}

/// Writes the batch counter.
pub fn set_batch_counter(env: &Env, value: u64) {
    env.storage().instance().set(&DataKey::BatchCounter, &value);
}

/// Reads the current retirement counter (number of certificates issued).
pub fn get_retirement_counter(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::RetirementCounter)
        .unwrap_or(0)
}

/// Writes the retirement counter.
pub fn set_retirement_counter(env: &Env, value: u64) {
    env.storage()
        .instance()
        .set(&DataKey::RetirementCounter, &value);
}

/// Reads whether minting is currently paused, defaulting to `false`.
pub fn get_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false)
}

/// Writes the paused flag.
pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&DataKey::Paused, &paused);
}

/// Reads the running total of credits minted across all batches.
pub fn get_total_minted(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalMinted)
        .unwrap_or(0i128)
}

/// Writes the running total of credits minted across all batches.
pub fn set_total_minted(env: &Env, value: i128) {
    env.storage().instance().set(&DataKey::TotalMinted, &value);
}

/// Returns `true` if a batch with the given id exists.
pub fn has_batch(env: &Env, id: u64) -> bool {
    env.storage().persistent().has(&DataKey::Batch(id))
}

/// Reads a batch record by id.
pub fn get_batch(env: &Env, id: u64) -> Option<Batch> {
    let key = DataKey::Batch(id);
    let batch = env.storage().persistent().get(&key);
    if batch.is_some() {
        extend_persistent(env, &key);
    }
    batch
}

/// Writes a batch record.
pub fn set_batch(env: &Env, batch: &Batch) {
    let key = DataKey::Batch(batch.id);
    env.storage().persistent().set(&key, batch);
    extend_persistent(env, &key);
}

/// Reads the balance of `owner` for `batch_id`, defaulting to zero.
pub fn get_balance(env: &Env, owner: &Address, batch_id: u64) -> i128 {
    let key = DataKey::Balance(owner.clone(), batch_id);
    let balance = env.storage().persistent().get(&key).unwrap_or(0i128);
    if balance != 0 {
        extend_persistent(env, &key);
    }
    balance
}

/// Writes the balance of `owner` for `batch_id`.
pub fn set_balance(env: &Env, owner: &Address, batch_id: u64, amount: i128) {
    let key = DataKey::Balance(owner.clone(), batch_id);
    env.storage().persistent().set(&key, &amount);
    extend_persistent(env, &key);
}

/// Moves `amount` credits of `batch_id` from `from` to `to`.
///
/// Returns `false` without writing anything if `from` holds fewer than
/// `amount` credits or if either side would overflow `i128`; otherwise both
/// balances are updated and `true` is returned. `amount` is assumed to be
/// positive (validated by callers).
pub fn move_balance(env: &Env, from: &Address, to: &Address, batch_id: u64, amount: i128) -> bool {
    let from_balance = get_balance(env, from, batch_id);
    if from_balance < amount {
        return false;
    }
    let to_balance = get_balance(env, to, batch_id);
    match (
        math::checked_sub(from_balance, amount),
        math::checked_add(to_balance, amount),
    ) {
        (Ok(new_from), Ok(new_to)) => {
            set_balance(env, from, batch_id, new_from);
            set_balance(env, to, batch_id, new_to);
            true
        }
        _ => false,
    }
}

/// Reads a retirement certificate by id.
pub fn get_retirement(env: &Env, id: u64) -> Option<Retirement> {
    let key = DataKey::Retirement(id);
    let cert = env.storage().persistent().get(&key);
    if cert.is_some() {
        extend_persistent(env, &key);
    }
    cert
}

/// Writes a retirement certificate.
pub fn set_retirement(env: &Env, cert: &Retirement) {
    let key = DataKey::Retirement(cert.id);
    env.storage().persistent().set(&key, cert);
    extend_persistent(env, &key);
}

/// Reads the running total of retired credits for `batch_id`.
pub fn get_total_retired(env: &Env, batch_id: u64) -> i128 {
    let key = DataKey::TotalRetired(batch_id);
    env.storage().persistent().get(&key).unwrap_or(0i128)
}

/// Writes the running total of retired credits for `batch_id`.
pub fn set_total_retired(env: &Env, batch_id: u64, amount: i128) {
    let key = DataKey::TotalRetired(batch_id);
    env.storage().persistent().set(&key, &amount);
    extend_persistent(env, &key);
}

/// Monotonic version of the current on-chain storage layout.
///
/// Bumped (and persisted via [set_storage_schema_version]) whenever the
/// [DataKey] layout changes. Defaults to [CURRENT_STORAGE_SCHEMA_VERSION]
/// for contracts initialized before the field existed.
pub const CURRENT_STORAGE_SCHEMA_VERSION: u32 = 1;

/// Reads the persisted storage-schema version, defaulting to the current
/// constant when the field has not been written yet.
pub fn get_storage_schema_version(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::StorageSchemaVersion)
        .unwrap_or(CURRENT_STORAGE_SCHEMA_VERSION)
}

/// Persists the storage-schema version and keeps its TTL fresh.
pub fn set_storage_schema_version(env: &Env, version: u32) {
    env.storage()
        .instance()
        .set(&DataKey::StorageSchemaVersion, &version);
    extend_instance(env);
}
