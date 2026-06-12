#![no_std]

//! # CarbonMint
//!
//! A tokenized carbon-credit marketplace smart contract for the Stellar
//! Soroban platform. Carbon credits are tracked per batch in a semi-fungible
//! manner: balances are keyed by `(owner, batch_id)`.

mod error;
mod events;
mod storage;
mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, contractmeta, Address, Env, String};

pub use crate::error::Error;
pub use crate::types::{Batch, Retirement};

/// Monotonic on-chain version of the contract logic.
pub const VERSION: u32 = 1;

contractmeta!(key = "name", val = "CarbonMint");
contractmeta!(
    key = "desc",
    val = "Tokenized carbon-credit marketplace for Stellar Soroban"
);

#[contract]
pub struct CarbonMintContract;

#[contractimpl]
impl CarbonMintContract {
    /// Returns the contract version number.
    pub fn version(_env: Env) -> u32 {
        VERSION
    }

    /// Initializes the registry with an `admin` address.
    ///
    /// The admin governs minting authorization. Calling this more than once
    /// returns [`Error::AlreadyInitialized`].
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if storage::has_admin(&env) {
            return Err(Error::AlreadyInitialized);
        }
        storage::set_admin(&env, &admin);
        storage::extend_instance(&env);
        Ok(())
    }

    /// Returns the current registry admin address.
    ///
    /// Returns [`Error::NotInitialized`] if the contract has not been set up.
    pub fn get_admin(env: Env) -> Result<Address, Error> {
        storage::get_admin(&env).ok_or(Error::NotInitialized)
    }

    /// Pauses or unpauses minting. Only the registry admin may call this.
    ///
    /// While paused, [`mint_batch`](Self::mint_batch) rejects new mints with
    /// [`Error::Paused`]; existing batches can still be traded and retired.
    /// Requires authorization from the admin.
    pub fn set_paused(env: Env, paused: bool) -> Result<(), Error> {
        let admin = storage::get_admin(&env).ok_or(Error::NotInitialized)?;
        admin.require_auth();

        storage::set_paused(&env, paused);
        storage::extend_instance(&env);

        events::paused(&env, &admin, paused);
        Ok(())
    }

    /// Returns whether minting is currently paused.
    pub fn is_paused(env: Env) -> bool {
        storage::get_paused(&env)
    }

    /// Mints a new batch of carbon credits and returns its id.
    ///
    /// Requires authorization from `issuer`. The full `amount` is credited to
    /// the issuer's balance for the new batch. The batch is created listed at
    /// the supplied `price`. Returns [`Error::Paused`] while minting is paused.
    pub fn mint_batch(
        env: Env,
        issuer: Address,
        project_id: String,
        vintage: u32,
        amount: i128,
        price: i128,
    ) -> Result<u64, Error> {
        if !storage::has_admin(&env) {
            return Err(Error::NotInitialized);
        }
        if storage::get_paused(&env) {
            return Err(Error::Paused);
        }
        issuer.require_auth();

        if amount <= 0 || price < 0 {
            return Err(Error::InvalidAmount);
        }

        let id = storage::get_batch_counter(&env)
            .checked_add(1)
            .ok_or(Error::Overflow)?;

        let batch = Batch {
            id,
            issuer: issuer.clone(),
            project_id,
            vintage,
            supply: amount,
            price,
            listed: true,
        };
        let total_minted = storage::get_total_minted(&env)
            .checked_add(amount)
            .ok_or(Error::Overflow)?;

        storage::set_batch(&env, &batch);
        storage::set_balance(&env, &issuer, id, amount);
        storage::set_batch_counter(&env, id);
        storage::set_total_minted(&env, total_minted);
        storage::extend_instance(&env);

        events::minted(&env, &issuer, id, amount);
        Ok(id)
    }

    /// Returns the batch record for `batch_id`.
    ///
    /// Returns [`Error::BatchNotFound`] if no such batch exists.
    pub fn get_batch(env: Env, batch_id: u64) -> Result<Batch, Error> {
        storage::get_batch(&env, batch_id).ok_or(Error::BatchNotFound)
    }

    /// Returns the credit balance held by `owner` for `batch_id`.
    ///
    /// Holders with no credits return zero.
    pub fn balance_of(env: Env, owner: Address, batch_id: u64) -> i128 {
        storage::get_balance(&env, &owner, batch_id)
    }

    /// Lists `batch_id` for sale and/or updates its unit `price`.
    ///
    /// Only the batch issuer may call this. Requires authorization from the
    /// issuer recorded on the batch.
    pub fn list(env: Env, batch_id: u64, price: i128) -> Result<(), Error> {
        let mut batch = storage::get_batch(&env, batch_id).ok_or(Error::BatchNotFound)?;
        batch.issuer.require_auth();

        if price < 0 {
            return Err(Error::InvalidAmount);
        }

        batch.price = price;
        batch.listed = true;
        storage::set_batch(&env, &batch);

        events::listed(&env, &batch.issuer, batch_id, price);
        Ok(())
    }

    /// Removes `batch_id` from sale without changing its price.
    ///
    /// Only the batch issuer may call this. While delisted, [`buy`](Self::buy)
    /// rejects purchases with [`Error::NotListed`]. Requires authorization from
    /// the issuer recorded on the batch.
    pub fn unlist(env: Env, batch_id: u64) -> Result<(), Error> {
        let mut batch = storage::get_batch(&env, batch_id).ok_or(Error::BatchNotFound)?;
        batch.issuer.require_auth();

        batch.listed = false;
        storage::set_batch(&env, &batch);

        events::delisted(&env, &batch.issuer, batch_id);
        Ok(())
    }

    /// Buys `amount` credits of `batch_id` from the batch issuer/seller.
    ///
    /// Requires authorization from `buyer` and that the batch is currently
    /// listed (otherwise [`Error::NotListed`]). Payment is mocked: this
    /// transfers credits from the seller to the buyer and emits a `bought`
    /// event with the quoted price, but does not move an underlying payment
    /// asset.
    pub fn buy(env: Env, buyer: Address, batch_id: u64, amount: i128) -> Result<(), Error> {
        buyer.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let batch = storage::get_batch(&env, batch_id).ok_or(Error::BatchNotFound)?;
        if !batch.listed {
            return Err(Error::NotListed);
        }
        let seller = batch.issuer.clone();

        if storage::get_balance(&env, &seller, batch_id) < amount {
            return Err(Error::InsufficientBalance);
        }
        if !storage::move_balance(&env, &seller, &buyer, batch_id, amount) {
            return Err(Error::Overflow);
        }

        events::bought(&env, &buyer, &seller, batch_id, amount, batch.price);
        Ok(())
    }

    /// Transfers `amount` credits of `batch_id` from `from` to `to` directly,
    /// without going through the marketplace listing.
    ///
    /// Requires authorization from `from`. Returns [`Error::SameAccount`] if
    /// `from` and `to` are equal, [`Error::BatchNotFound`] for an unknown
    /// batch, and [`Error::InsufficientBalance`] if `from` does not hold enough
    /// credits.
    pub fn transfer(
        env: Env,
        from: Address,
        to: Address,
        batch_id: u64,
        amount: i128,
    ) -> Result<(), Error> {
        from.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        if from == to {
            return Err(Error::SameAccount);
        }

        if !storage::has_batch(&env, batch_id) {
            return Err(Error::BatchNotFound);
        }

        if storage::get_balance(&env, &from, batch_id) < amount {
            return Err(Error::InsufficientBalance);
        }
        if !storage::move_balance(&env, &from, &to, batch_id, amount) {
            return Err(Error::Overflow);
        }

        events::transferred(&env, &from, &to, batch_id, amount);
        Ok(())
    }

    /// Retires (permanently burns) `amount` credits of `batch_id` held by
    /// `holder`, recording a retirement certificate and returning its id.
    ///
    /// Requires authorization from `holder`. The holder's balance is reduced
    /// and the batch's running retired total is increased.
    pub fn retire(
        env: Env,
        holder: Address,
        batch_id: u64,
        amount: i128,
    ) -> Result<u64, Error> {
        holder.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        if !storage::has_batch(&env, batch_id) {
            return Err(Error::BatchNotFound);
        }

        let balance = storage::get_balance(&env, &holder, batch_id);
        if balance < amount {
            return Err(Error::InsufficientBalance);
        }

        let new_balance = balance.checked_sub(amount).ok_or(Error::Overflow)?;
        storage::set_balance(&env, &holder, batch_id, new_balance);

        let retired_total = storage::get_total_retired(&env, batch_id)
            .checked_add(amount)
            .ok_or(Error::Overflow)?;
        storage::set_total_retired(&env, batch_id, retired_total);

        let cert_id = storage::get_retirement_counter(&env)
            .checked_add(1)
            .ok_or(Error::Overflow)?;
        let cert = Retirement {
            id: cert_id,
            batch_id,
            holder: holder.clone(),
            amount,
        };
        storage::set_retirement(&env, &cert);
        storage::set_retirement_counter(&env, cert_id);
        storage::extend_instance(&env);

        events::retired(&env, &holder, batch_id, amount, cert_id);
        Ok(cert_id)
    }

    /// Returns the retirement certificate for `cert_id`.
    ///
    /// Returns [`Error::BatchNotFound`] if no such certificate exists.
    pub fn get_retirement(env: Env, cert_id: u64) -> Result<Retirement, Error> {
        storage::get_retirement(&env, cert_id).ok_or(Error::BatchNotFound)
    }

    /// Returns the total amount of credits retired for `batch_id`.
    pub fn total_retired(env: Env, batch_id: u64) -> i128 {
        storage::get_total_retired(&env, batch_id)
    }

    /// Returns the number of batches minted so far (also the highest batch id).
    pub fn batch_count(env: Env) -> u64 {
        storage::get_batch_counter(&env)
    }

    /// Returns the number of retirement certificates issued so far.
    pub fn retirement_count(env: Env) -> u64 {
        storage::get_retirement_counter(&env)
    }

    /// Returns the cumulative amount of credits minted across all batches.
    ///
    /// This total never decreases; retirements are tracked separately via
    /// [`total_retired`](Self::total_retired).
    pub fn total_minted(env: Env) -> i128 {
        storage::get_total_minted(&env)
    }

    /// Returns whether `batch_id` is currently listed for sale.
    ///
    /// Returns [`Error::BatchNotFound`] if no such batch exists.
    pub fn is_listed(env: Env, batch_id: u64) -> Result<bool, Error> {
        let batch = storage::get_batch(&env, batch_id).ok_or(Error::BatchNotFound)?;
        Ok(batch.listed)
    }

    /// Returns the still-circulating supply for `batch_id`, i.e. the original
    /// minted supply minus the amount that has been retired.
    ///
    /// Returns [`Error::BatchNotFound`] if no such batch exists.
    pub fn circulating_supply(env: Env, batch_id: u64) -> Result<i128, Error> {
        let batch = storage::get_batch(&env, batch_id).ok_or(Error::BatchNotFound)?;
        let retired = storage::get_total_retired(&env, batch_id);
        batch.supply.checked_sub(retired).ok_or(Error::Overflow)
    }
}
