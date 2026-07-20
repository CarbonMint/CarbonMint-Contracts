# Mainnet Checklist

This note documents the **mainnet-checklist** of the carbonmint-contract contract.

carbonmint-contract is a Soroban smart contract on the Stellar network. This page is part of the
project's reference documentation and describes the mainnet-checklist in detail, covering the relevant
entrypoints, storage layout, and invariants where applicable.

See the README and the sources under src/ for the authoritative implementation.

## Funding

See [`deployment-funding.md`](deployment-funding.md) for the complete
breakdown. Before proceeding, confirm:

- [ ] Deployer account exists on mainnet and holds **≥ 5 XLM** available
      balance (covers base reserves for the WASM upload + contract instance
      entries, plus a buffer for transaction resource fees).
- [ ] Simulate the `initialize` call with
      `stellar contract simulate` and verify the reported resource fee is
      within the available balance.
- [ ] A funded operator account separate from the admin key is available for
      day-to-day contract invocations.

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
