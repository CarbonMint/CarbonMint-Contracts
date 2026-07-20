# Resource Limits

This note documents the **resource-limits** of the carbonmint-contract contract.

carbonmint-contract is a Soroban smart contract on the Stellar network. This page is part of the
project's reference documentation and describes the resource-limits in detail, covering the relevant
entrypoints, storage layout, and invariants where applicable.

## Recipient Count in Batch Operations

The `batch_transfer` entrypoint is bounded to at most **50 recipients**
(`MAX_RECIPIENTS = 50`). This is a compile-time constant enforced at runtime:
any call with zero recipients or more than 50 recipients returns
`Error::TooManyRecipients` (error code 11).

### Rationale

Each recipient in a batch transfer incurs:

- One persistent-storage **read** of the recipient's current balance.
- One persistent-storage **write** of the new balance.

Both operations consume ledger entry reads / writes and CPU instructions.
Capping the recipient count gives validators a predictable worst-case resource
footprint and prevents a single transaction from monopolizing the ledger.

### Instruction Budget

With `MAX_RECIPIENTS = 50`, the worst-case instruction cost fits comfortably
within the Soroban transaction budget on both testnet and mainnet. The
per-recipient loop performs checked arithmetic (no panics) and respects the
TTL-extension pattern used throughout the contract.

### Storage Considerations

Each unique `(recipient, batch_id)` pair creates a new persistent storage
entry. Under Soroban's state-expiration model, these entries are subject to
rent. Batch transfers do not explicitly extend TTL beyond the per-entry
extend performed by `set_balance` (triggering `extend_persistent` for 30
days).

## Per-entrypoint Costs

| Entrypoint | Storage Reads | Storage Writes | Notes |
| --- | --- | --- | --- |
| `batch_transfer` | 2 + N | N + 1 | N recipients, bounded to 50 |

See the README and the sources under src/ for the authoritative implementation.
