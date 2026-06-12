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

use soroban_sdk::{contract, contractimpl, Address, Env, String};

use crate::error::Error;
use crate::types::Batch;

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
}
