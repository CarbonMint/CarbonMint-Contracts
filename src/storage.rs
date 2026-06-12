use soroban_sdk::{Address, Env};

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
    env.storage()
        .instance()
        .set(&DataKey::BatchCounter, &value);
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
