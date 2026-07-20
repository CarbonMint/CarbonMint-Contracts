#![no_main]

use libfuzzer_sys::fuzz_target;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

use carbonmint_contract::{CarbonMintContract, CarbonMintContractClient};

fuzz_target!(|data: &[u8]| {
    let env = Env::default();
    let contract_id = env.register_contract(None, CarbonMintContract);
    let client = CarbonMintContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let buyer = Address::generate(&env);

    env.mock_all_auths();
    let _ = client.initialize(&admin);

    let project_id = String::from_str(&env, "FUZZ-001");

    let amount = if data.is_empty() {
        1i128
    } else {
        let byte = data[0] as i128;
        if byte == 0 { 1 } else { byte % 1000 + 1 }
    };
    let price = if data.len() > 1 {
        let byte = data[1] as i128;
        if byte == 0 { 1 } else { byte % 100 + 1 }
    } else {
        1
    };

    let _ = client.mint_batch(&issuer, &project_id, &2024u32, &amount, &price);
    let _ = client.buy(&buyer, &1u64, &((amount / 2).max(1)));
});
