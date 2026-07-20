# Testnet Deployment

This page documents the **testnet deployment** workflow for `carbonmint-contract`.

## Prerequisites

- [Stellar CLI](https://developers.stellar.org/docs/tools/cli) installed.
- Rust toolchain with the `wasm32-unknown-unknown` target (see
  [`rust-toolchain.toml`](../rust-toolchain.toml)).

## Fund the deployer account

Testnet accounts are funded for free using **Friendbot**, which provisions
10 000 testnet XLM (no real-world value):

```sh
# Generate a local identity (if you do not already have one)
stellar keys generate --global deployer --network testnet

# Fund it via Friendbot
curl "https://friendbot.stellar.org/?addr=$(stellar keys address deployer)"
```

The Stellar CLI also funds new identities automatically the first time you use
them on testnet, so for simple deployments you can skip the `curl` step.

> See [`deployment-funding.md`](deployment-funding.md) for the full breakdown
> of base reserves, resource fees, and storage rent required at deployment time.

## Build and deploy

```sh
# 1. Build the release WASM
make build

# 2. (optional) shrink the binary
make optimize

# 3. Deploy to testnet
## Automated Smoke Test

To quickly verify that the contract builds, deploys, initializes, and answers
basic read queries correctly on the Stellar testnet, run the automated smoke test script:

```sh
./scripts/smoke-test-testnet.sh
```

This script will automatically:
1. Generate a temporary deployer identity.
2. Fund it via Friendbot.
3. Build the WASM binary.
4. Deploy the contract to the testnet.
5. Initialize the contract.
6. Verify read-only entrypoints (`version`, `is_paused`, `batch_count`).
7. Verify the deployed WASM hash matches the local build.

## Manual Deployment

If you prefer to deploy manually step-by-step:

### 1. Fund the deployer account

```sh
stellar keys generate --global deployer --network testnet
curl -s "https://friendbot.stellar.org/?addr=$(stellar keys address deployer)"
```

### 2. Build and deploy

```sh
make build
make deploy NETWORK=testnet SOURCE=deployer
```

Note the **contract id** printed after the deploy step.

## Initialize the contract
### 3. Initialize the contract

```sh
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source deployer \
  --network testnet \
  -- initialize --admin <ADMIN_ADDRESS>
```

## Smoke-test key entrypoints

```sh
# Check the logic version
stellar contract invoke --id <CONTRACT_ID> --source deployer \
  --network testnet -- version

# Confirm the contract is not paused
stellar contract invoke --id <CONTRACT_ID> --source deployer \
  --network testnet -- is_paused

# Confirm the batch counter starts at zero
stellar contract invoke --id <CONTRACT_ID> --source deployer \
  --network testnet -- batch_count
```

## Verify the deployed WASM hash
### 4. Verify the deployed WASM hash

```sh
make verify-wasm-hash CONTRACT_ID=<CONTRACT_ID> NETWORK=testnet
```

A passing result confirms the on-chain bytecode matches the local build.
See [`deployment-guide.md`](deployment-guide.md) for details on what the
verification script does.
