# Known Limitations

This note documents the **known-limitations** of the carbonmint-contract contract.

carbonmint-contract is a Soroban smart contract on the Stellar network. This page is part of the
project's reference documentation and describes the known-limitations in detail, covering the relevant
entrypoints, storage layout, and invariants where applicable.

## Batch-transfer Recipient Bound

The `batch_transfer` entrypoint accepts at most **50 recipients** per invocation
(`MAX_RECIPIENTS = 50`). This limit protects validators from excessive
per-transaction storage writes and keeps instruction budgets predictable.
Callers who need to distribute to more than 50 addresses should split the work
across multiple transactions.

## Single-batch Per Transfer

All recipients in a `batch_transfer` call receive credits from the **same**
`batch_id`. Mixed-batch distributions require separate invocations.

## No Batch Mint, Buy or Retire

Currently `batch_transfer` is the only batch operation. Minting, marketplace
buying, and retirement of credits must be performed one batch / one recipient
at a time.

## Mock Payment Asset

The `buy` entrypoint performs a mock transfer of credits: it emits a `bought`
event with the quoted price but does **not** move an underlying payment asset.
Production deployments must integrate a real payment mechanism.

## Upgradability

The contract itself is not upgradable via a proxy pattern. New versions must
be deployed to a fresh contract address and state must be migrated manually.

## No On-chain Identity Proofing

Issuers are identified solely by their Stellar account address. There is no
on-chain verification that a given address represents a legitimate carbon-
credit project beyond the registry admin's off-chain vetting during minting.

See the README and the sources under src/ for the authoritative implementation.
