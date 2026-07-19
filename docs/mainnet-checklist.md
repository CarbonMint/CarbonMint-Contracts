# Mainnet Checklist

This note documents the **mainnet-checklist** of the carbonmint-contract contract.

carbonmint-contract is a Soroban smart contract on the Stellar network. This page is part of the
project's reference documentation and describes the mainnet-checklist in detail, covering the relevant
entrypoints, storage layout, and invariants where applicable.

See the README and the sources under src/ for the authoritative implementation.

## Pre-deployment

- [ ] Run `make test` – all tests pass.
- [ ] Run `make clippy` – no warnings.
- [ ] Run `make fmt-check` – formatting is clean.
- [ ] Confirm the `Cargo.toml` version accurately reflects the release.
- [ ] Tag the release commit in git.

## Deployment

- [ ] Build the release WASM: `make build`.
- [ ] (Optional) Optimize the WASM: `make optimize`.
- [ ] Deploy to mainnet: `make deploy NETWORK=mainnet SOURCE=<identity>`.
- [ ] Record the returned contract id.

## Post-deployment verification

- [ ] **Verify the deployed WASM hash**:
      `make verify-wasm-hash CONTRACT_ID=<ID> NETWORK=mainnet`
- [ ] Confirm the script reports **Verification passed**.
- [ ] Initialize the contract: invoke `initialize` with the admin address.
- [ ] Smoke-test a few entrypoints (`version`, `is_paused`, `batch_count`).
- [ ] Publish the contract id and verification proof in the release notes.
