//! Minimal fuzz-harness support helpers for the contract.
//!
//! The module is intentionally lightweight and mirrors the in-process Soroban
//! test harness so the fuzz target can reuse the same contract setup code.

#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::{CarbonMintContract, CarbonMintContractClient};

pub(crate) fn setup_contract<'a>() -> (Env, CarbonMintContractClient<'a>, Address) {
    let env = Env::default();
    let contract_id = env.register_contract(None, CarbonMintContract);
    let client = CarbonMintContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    (env, client, admin)
}
