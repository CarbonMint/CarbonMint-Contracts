#![no_std]

//! # CarbonMint
//!
//! A tokenized carbon-credit marketplace smart contract for the Stellar
//! Soroban platform. Carbon credits are tracked per batch in a semi-fungible
//! manner: balances are keyed by `(owner, batch_id)`.

use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct CarbonMintContract;

#[contractimpl]
impl CarbonMintContract {
    /// Returns the contract version string.
    pub fn version(_env: Env) -> u32 {
        1
    }
}
