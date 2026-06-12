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

use soroban_sdk::{contract, contractimpl, Address, Env};

use crate::error::Error;

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
}
