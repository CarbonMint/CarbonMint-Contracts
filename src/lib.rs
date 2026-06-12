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

use soroban_sdk::{contract, contractimpl, Address, Env, String};

pub use crate::error::Error;
pub use crate::types::{Batch, Retirement};

#[contract]
pub struct CarbonMintContract;

#[contractimpl]
impl CarbonMintContract {
    /// Returns the contract version string.
    pub fn version(_env: Env) -> u32 {
        1
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

    /// Mints a new batch of carbon credits and returns its id.
    ///
    /// Requires authorization from `issuer`. The full `amount` is credited to
    /// the issuer's balance for the new batch. The batch is created listed at
    /// the supplied `price`.
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
        storage::set_batch(&env, &batch);
        storage::set_balance(&env, &issuer, id, amount);
        storage::set_batch_counter(&env, id);
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

    /// Buys `amount` credits of `batch_id` from the batch issuer/seller.
    ///
    /// Requires authorization from `buyer`. Payment is mocked: this transfers
    /// credits from the seller to the buyer and emits a `bought` event with the
    /// quoted price, but does not move an underlying payment asset.
    pub fn buy(env: Env, buyer: Address, batch_id: u64, amount: i128) -> Result<(), Error> {
        buyer.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let batch = storage::get_batch(&env, batch_id).ok_or(Error::BatchNotFound)?;
        let seller = batch.issuer.clone();

        let seller_balance = storage::get_balance(&env, &seller, batch_id);
        if seller_balance < amount {
            return Err(Error::InsufficientBalance);
        }

        let new_seller = seller_balance
            .checked_sub(amount)
            .ok_or(Error::Overflow)?;
        let buyer_balance = storage::get_balance(&env, &buyer, batch_id);
        let new_buyer = buyer_balance
            .checked_add(amount)
            .ok_or(Error::Overflow)?;

        storage::set_balance(&env, &seller, batch_id, new_seller);
        storage::set_balance(&env, &buyer, batch_id, new_buyer);

        events::bought(&env, &buyer, &seller, batch_id, amount, batch.price);
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
