# CarbonMint

A tokenized **carbon-credit marketplace** smart contract for the
[Stellar](https://stellar.org) Soroban platform, written in Rust with
`soroban-sdk` 21.x.

CarbonMint lets issuers register batches of carbon credits, list them for sale,
sell them to buyers, and lets holders permanently **retire** credits in exchange
for an on-chain retirement certificate.

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
| `mint_batch(issuer, project_id, vintage, amount, price) -> u64` | Registers a batch and credits the issuer; returns the new batch id. Requires issuer auth. |
| `list(batch_id, price)` | Lists / reprices a batch. Requires issuer auth. |
| `buy(buyer, batch_id, amount)` | Transfers credits from the seller to the buyer (mock payment). Requires buyer auth. |
| `retire(holder, batch_id, amount) -> u64` | Burns credits and issues a retirement certificate; returns the certificate id. Requires holder auth. |

### Read-only

| Function | Description |
| --- | --- |
| `get_admin() -> Address` | The registry admin. |
| `balance_of(owner, batch_id) -> i128` | Credit balance of an owner for a batch. |
| `get_batch(batch_id) -> Batch` | The batch record. |
| `get_retirement(cert_id) -> Retirement` | A retirement certificate. |
| `total_retired(batch_id) -> i128` | Total credits retired for a batch. |
| `batch_count() -> u64` | Number of batches minted. |
| `retirement_count() -> u64` | Number of certificates issued. |

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

## Build

The contract targets `wasm32-unknown-unknown` and is built as a `cdylib`.

```sh
make build      # cargo build --target wasm32-unknown-unknown --release
make test       # run the unit test suite
make fmt        # format the source tree
make clippy     # lint with warnings denied
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
