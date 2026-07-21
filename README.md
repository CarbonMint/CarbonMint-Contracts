# CarbonMint

A tokenized **carbon-credit marketplace** smart contract for the
[Stellar](https://stellar.org) Soroban platform, written in Rust with
`soroban-sdk` 21.x.

CarbonMint lets issuers register batches of carbon credits, list them for sale,
sell them to buyers, and lets holders permanently **retire** credits in exchange
for an on-chain retirement certificate.

For contributor guidance on contract changes, tests, and review expectations,
see [docs/contributing-guide.md](docs/contributing-guide.md).

## Model

Credits are **semi-fungible**: every credit within the same batch is
interchangeable, but credits from different batches are tracked separately.
Balances are keyed by `(owner, batch_id)`.

- **Batch** — a registered lot of credits with an issuer, project id, vintage,
  original supply, listed price, and a listing flag.
- **Retirement** — a certificate recording the permanent burning of a quantity
  of credits by a holder.

## Contract API

### State-changing

| Function | Description |
| --- | --- |
| `initialize(admin)` | Sets the registry admin. Callable once. |
| `set_admin(new_admin)` | Rotates the registry admin. Requires current-admin auth. |
| `set_paused(paused)` | Pauses or unpauses minting. Requires admin auth. |
| `mint_batch(issuer, project_id, vintage, amount, price) -> u64` | Registers a batch and credits the issuer; returns the new batch id. Requires issuer auth; rejected while paused. |
| `list(batch_id, price)` | Lists / reprices a batch. Requires issuer auth. |
| `unlist(batch_id)` | Removes a batch from sale (price preserved). Requires issuer auth. |
| `buy(buyer, batch_id, amount)` | Transfers credits from the seller to the buyer (mock payment). Requires buyer auth and a listed batch. |
| `transfer(from, to, batch_id, amount)` | Transfers credits directly between holders, bypassing the listing. Requires `from` auth; `from` must differ from `to`. |
| `retire(holder, batch_id, amount) -> u64` | Burns credits and issues a retirement certificate; returns the certificate id. Requires holder auth. |
| `retire_for(holder, batch_id, amount, beneficiary) -> u64` | Like `retire`, but records a named beneficiary on the certificate. Requires holder auth. |

### Read-only

| Function | Description |
| --- | --- |
| `get_admin() -> Address` | The registry admin. |
| `balance_of(owner, batch_id) -> i128` | Credit balance of an owner for a batch. |
| `get_batch(batch_id) -> Batch` | The batch record. |
| `get_retirement(cert_id) -> Retirement` | A retirement certificate. |
| `is_listed(batch_id) -> bool` | Whether a batch is currently listed for sale. |
| `listing_info(batch_id) -> Listing` | Compact sale view: seller, price, listed flag and available amount. |
| `is_paused() -> bool` | Whether minting is currently paused. |
| `total_retired(batch_id) -> i128` | Total credits retired for a batch. |
| `total_minted() -> i128` | Cumulative credits minted across all batches. |
| `circulating_supply(batch_id) -> i128` | Minted supply minus retired credits. |
| `batch_count() -> u64` | Number of batches minted. |
| `retirement_count() -> u64` | Number of certificates issued. |
| `version() -> u32` | The contract logic version. |

## Errors

| Code | Variant | Meaning |
| --- | --- | --- |
| 1 | `AlreadyInitialized` | `initialize` called more than once. |
| 2 | `NotInitialized` | Contract used before `initialize`. |
| 3 | `BatchNotFound` | Unknown batch or certificate id. |
| 4 | `InvalidAmount` | Amount is zero / negative or price is negative. |
| 5 | `InsufficientBalance` | Holder lacks enough credits. |
| 6 | `Unauthorized` | Caller not permitted. |
| 7 | `Overflow` | Arithmetic overflow. |
| 8 | `NotListed` | Batch is not currently listed for sale. |
| 9 | `Paused` | Minting is paused by the admin. |
| 10 | `SameAccount` | Transfer source and destination are identical. |

## Events

Off-chain indexers can reconstruct registry state from these events:

| Event | Topics | Data |
| --- | --- | --- |
| `minted` | `issuer` | `(batch_id, amount)` |
| `listed` | `issuer` | `(batch_id, price)` |
| `delisted` | `issuer` | `batch_id` |
| `bought` | `buyer, seller` | `(batch_id, amount, price)` |
| `transfer` | `from, to` | `(batch_id, amount)` |
| `retired` | `holder` | `(batch_id, amount, certificate_id)` |
| `paused` | `admin` | `paused` |
| `adminset` | `old_admin` | `new_admin` |

## Build

The contract targets `wasm32-unknown-unknown` and is built as a `cdylib`.

```sh
make build      # cargo build --target wasm32-unknown-unknown --release
make check      # type-check all targets without producing a wasm
make test       # run the unit test suite
make fmt        # format the source tree
make clippy     # lint with warnings denied
make doc        # build the rustdoc API documentation
make wasm-size  # build and print the compiled wasm size
make verify-wasm-hash CONTRACT_ID=<ID>  # verify deployed wasm hash
```

The compiled WASM is written to
`target/wasm32-unknown-unknown/release/carbonmint_contract.wasm`.

## Deploy

Deployment uses the [Stellar CLI](https://developers.stellar.org/docs/tools/cli).

```sh
# optional: shrink the wasm before deploying
make optimize

# deploy to testnet using the `default` identity
make deploy NETWORK=testnet SOURCE=default
```

## Verify deployed WASM hash

After deploying, you can verify that the WASM binary on the ledger matches
the one you built locally. This confirms the deployed code is exactly what
is in this repository.

```sh
# build, hash your local wasm, fetch the deployed hash, and compare
make verify-wasm-hash CONTRACT_ID=<CONTRACT_ID> NETWORK=testnet

# skip the build step if the wasm hasn't changed
./scripts/verify-wasm-hash.sh <CONTRACT_ID> testnet --skip-build
```

The script:
1. Builds the contract (unless `--skip-build` is passed).
2. Computes the SHA-256 hash of the local `.wasm` file.
3. Retrieves the hash of the deployed contract from the ledger.
4. Prints **Verification passed** if they match or **Verification failed**
   if they differ.

## Example flow

```sh
# 1. initialize the registry
stellar contract invoke ... -- initialize --admin <ADMIN>

# 2. mint a batch (returns the batch id)
stellar contract invoke ... -- mint_batch \
  --issuer <ISSUER> --project_id "PROJ-001" --vintage 2024 \
  --amount 1000 --price 5

# 3. a buyer purchases credits
stellar contract invoke ... -- buy --buyer <BUYER> --batch_id 1 --amount 100

# 4. the buyer retires credits and receives a certificate id
stellar contract invoke ... -- retire --holder <BUYER> --batch_id 1 --amount 100
```

## License

Licensed under the [MIT License](LICENSE).
