# TTL and Rent

This page documents the time-to-live (TTL) and storage-rent model used by
`carbonmint-contract`.

## Background

Soroban contracts pay **rent** for the ledger space they consume.  Every
ledger entry (batch records, balances, retirement certificates, instance
counters) carries a TTL measured in ledgers (~5 s each).  When the TTL reaches
zero the entry is *archived*; it must be restored before it can be read or
written again.

To avoid archival, the contract bumps TTLs proactively on every read and
write.

## TTL constants

The contract defines four constants in [`src/storage.rs`](../src/storage.rs):

| Constant | Value (ledgers) | Approximate duration |
| --- | --- | --- |
| `PERSISTENT_LIFETIME` | 518 400 | ~30 days |
| `PERSISTENT_THRESHOLD` | 103 680 | ~6 days |
| `INSTANCE_LIFETIME` | 518 400 | ~30 days |
| `INSTANCE_THRESHOLD` | 103 680 | ~6 days |

**Lifetime** is the TTL the entry is extended *to* on each bump.
**Threshold** is the minimum remaining TTL that triggers a bump — if the
remaining TTL is still above the threshold, no extension is performed (saving
rent).

## When TTLs are bumped

| Storage tier | Bumped on |
| --- | --- |
| Instance | Every state-changing entrypoint (`initialize`, `mint_batch`, `set_admin`, `set_paused`, `list`, `unlist`, `retire`, `retire_for`) |
| Persistent (`Batch`, `Balance`, `TotalRetired`, `Retirement`) | Every read *and* every write of the entry |

## Rent cost

The resource fee for a TTL extension depends on:

- The number of ledgers being added to the TTL.
- The size of the ledger entry (in bytes).
- The current dynamic write-fee rate set by network validators.

Use `stellar contract simulate` or the `simulateTransaction` RPC endpoint to
get the exact rent fee for a given extension before submitting it.

## Manual TTL extension

If an entry's TTL has dropped below the threshold between calls (e.g. a batch
that has not been traded in weeks), you can extend it manually:

```sh
stellar contract extend \
  --id <CONTRACT_ID> \
  --key <STORAGE_KEY> \
  --ledgers-to-extend 518400 \
  --source <IDENTITY> \
  --network mainnet
```

## References

- [Soroban State Archival](https://developers.stellar.org/docs/learn/fundamentals/contract-development/storage/state-archival)
- [`storage-model.md`](storage-model.md) — full storage layout and invariants
- [`deployment-funding.md`](deployment-funding.md) — XLM cost breakdown
