# Versioning

This note documents the **versioning** of the carbonmint-contract contract.

carbonmint-contract is a Soroban smart contract on the Stellar network. This page is part of the
project's reference documentation and describes the versioning in detail, covering the relevant
entrypoints, storage layout, and invariants where applicable.

See the README and the sources under src/ for the authoritative implementation.

## Storage schema version

In addition to the logic `version()`, the contract reports a separate
`storage_schema_version()` (persisted to instance storage on `initialize`). This
tracks the on-chain storage layout independently of logic changes so indexers can
detect schema migrations. See [Storage Model](storage-model.md) for the version table.
