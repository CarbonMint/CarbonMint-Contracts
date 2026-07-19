# Storage Model

This note documents the **storage model** of the `carbonmint-contract` contract.

`carbonmint-contract` is a Soroban smart contract on the Stellar network. This page
describes the on-chain storage layout in detail and is the authoritative reference
for the storage schema version reported by `storage_schema_version()`.

See the README and the sources under `src/` for the implementation.

## Storage schema versioning

The contract exposes two independent version numbers:

| Function | Meaning |
| --- | --- |
| `version()` | Version of the **contract logic** (bumped on behaviour changes). |
| `storage_schema_version()` | Version of the **storage layout** (bumped when the set of storage keys or their encoding changes). |

The storage-layout version is persisted to instance storage on `initialize` and is
intended for off-chain indexers: when the `DataKey` layout changes, bump both
`STORAGE_SCHEMA_VERSION` (in `src/lib.rs`) and `CURRENT_STORAGE_SCHEMA_VERSION`
(in `src/storage.rs`) in lock-step so migrations are detectable.

| `storage_schema_version` | Layout |
| --- | --- |
| `1` | `DataKey` = `Admin`, `BatchCounter`, `RetirementCounter`, `Paused`, `TotalMinted`, `Batch(u64)`, `Balance(Address, u64)`, `Retirement(u64)`, `TotalRetired(u64)`, `StorageSchemaVersion`. |

## Instance storage

Instance storage holds singleton values with a long TTL (extended on every write):

- `Admin` — the registry admin `Address`.
- `BatchCounter` — next batch id (`u64`).
- `RetirementCounter` — next retirement certificate id (`u64`).
- `Paused` — minting pause flag (`bool`, defaults to `false`).
- `TotalMinted` — cumulative credits minted across all batches (`i128`).
- `StorageSchemaVersion` — persisted storage-layout version (`u32`).

## Persistent storage

Persistent entries hold per-entity data and are TTL-extended on every read/write:

- `Batch(u64)` — a [`Batch`](src/types.rs) record keyed by batch id.
- `Balance(Address, u64)` — credit balance of an owner for a batch (`i128`).
- `Retirement(u64)` — a [`Retirement`](src/types.rs) certificate keyed by cert id.
- `TotalRetired(u64)` — running total of retired credits per batch (`i128`).

## Invariants

- All persistent reads/writes bump the entry TTL (≈30 days, threshold ≈6 days).
- `Balance` and `TotalRetired` default to `0` for unknown keys.
- Credit math uses checked arithmetic; overflow returns `Error::Overflow`.
