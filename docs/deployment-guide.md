# Deployment Guide

This note documents the **deployment-guide** of the carbonmint-contract contract.

carbonmint-contract is a Soroban smart contract on the Stellar network. This page is part of the
project's reference documentation and describes the deployment-guide in detail, covering the relevant
entrypoints, storage layout, and invariants where applicable.

See the README and the sources under src/ for the authoritative implementation.

## Funding

Deploying the contract requires XLM in the deployer account to cover:

1. **Base reserves** — each ledger entry created (WASM upload + contract
   instance) locks an additional 0.5 XLM in the account.
2. **Transaction resource fees** — charged per CPU instruction, ledger I/O,
   and storage rent at the time of each transaction.

**Recommended minimum: ≥ 5 XLM** in the deployer account before starting.

For the full breakdown of base reserves, per-call resource fees, storage rent,
and testnet Friendbot setup, see
[**`docs/deployment-funding.md`**](deployment-funding.md).

## Deploy

Build and deploy the contract using the Makefile:

```sh
# build the wasm binary
make build

# (optional) shrink the wasm before deploying
make optimize

# deploy to testnet using the `default` identity
make deploy NETWORK=testnet SOURCE=default
```

After deployment, note the returned **contract id** – you will need it for
hash verification and contract interactions.

## Verify deployed WASM hash

Once a contract is deployed, you can verify that the bytecode on the ledger
matches the local build by comparing SHA-256 hashes:

```sh
make verify-wasm-hash CONTRACT_ID=<CONTRACT_ID> NETWORK=testnet
```

Or use the script directly:

```sh
./scripts/verify-wasm-hash.sh <CONTRACT_ID> testnet
```

What the script does:

| Step | Action |
| --- | --- |
| 1 | Build the contract (or skip with `--skip-build`). |
| 2 | Compute the SHA-256 hash of the local `.wasm` file via `stellar contract hash`. |
| 3 | Fetch the on-chain WASM hash via `stellar contract info --id <ID>`. |
| 4 | Compare the two hashes and report pass / fail. |

A passing verification confirms the deployed contract was built from the
exact source in this repository at this revision. A failing verification
indicates a mismatch in source, build toolchain, or compiler version.
