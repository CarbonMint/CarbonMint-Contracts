# Deployment Funding

This page documents the XLM funding required to deploy and operate the
`carbonmint-contract` on the Stellar Soroban platform.

## Background

Every operation on Stellar consumes XLM in two independent ways:

| Cost type | Description |
| --- | --- |
| **Base reserve** | XLM locked in the deployer account to maintain ledger entries. |
| **Transaction fees** | XLM burned to pay for execution: an *inclusion fee* plus a *resource fee* (CPU instructions, ledger I/O, storage rent). |

Fees are paid in **stroops** (1 XLM = 10 000 000 stroops).  The minimum
inclusion fee per operation is **100 stroops** (0.000 010 XLM) at normal
network traffic, rising only when the network enters surge-pricing mode.

## Deployer account minimum balance

Before the first transaction can be submitted, the deployer account must
exist on the ledger and hold its minimum balance:

```
minimum_balance = (2 + number_of_subentries) × base_reserve
```

The current base reserve is **0.5 XLM**.  A freshly funded account with no
subentries therefore requires **1 XLM** to exist.  Uploading the WASM and
deploying the contract instance each add a ledger entry (subentry), so after
deployment the account's locked reserve increases by **1 XLM** (2 × 0.5 XLM).

**Practical minimum for a fresh deployer account: ≥ 5 XLM**

| Item | Approximate cost |
| --- | --- |
| Account existence (2 base reserves) | 1.0 XLM locked |
| WASM upload ledger entry | 0.5 XLM locked |
| Contract instance ledger entry | 0.5 XLM locked |
| Upload transaction fee (resource + inclusion) | ~0.01 XLM burned |
| Deploy transaction fee | ~0.01 XLM burned |
| `initialize` transaction fee | ~0.001 XLM burned |
| Buffer for surge pricing / future calls | ~2 XLM |
| **Total recommended** | **≥ 5 XLM** |

> **Note** Locked reserves are not burned; they are returned if the ledger
> entry is removed.  Resource fees are burned permanently.

## Transaction fee breakdown (per call)

Soroban charges a *resource fee* on top of the standard inclusion fee:

```
tx.fee = inclusion_fee + resource_fee
```

The resource fee is determined at simulation time and covers:

- **CPU instructions** — proportional to contract execution complexity.
- **Ledger entry accesses** — one charge per storage key read or written.
- **Ledger I/O bytes** — proportional to data read from and written to
  persistent / instance storage.
- **Transaction size** — bytes of the serialised transaction envelope.
- **Storage rent** — upfront payment for extending a ledger entry's TTL.

Use `simulateTransaction` (via the Stellar RPC) to get exact resource
requirements for any call before submitting it:

```sh
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <IDENTITY> \
  --network testnet \
  -- version
```

The CLI runs simulation automatically and sets the resource footprint for you.

## Storage rent and TTL

CarbonMint uses two storage tiers (see [`storage-model.md`](storage-model.md)
for layout details):

| Tier | TTL (ledgers) | Approx. duration | Threshold |
| --- | --- | --- | --- |
| Instance | 518 400 | ~30 days | 103 680 |
| Persistent | 518 400 | ~30 days | 103 680 |

TTLs are bumped automatically on every read and write.  For a low-traffic
deployment where a particular batch record has not been touched in a while,
you may need to manually extend the TTL with:

```sh
stellar contract extend \
  --id <CONTRACT_ID> \
  --key <STORAGE_KEY> \
  --ledgers-to-extend 518400 \
  --source <IDENTITY> \
  --network mainnet
```

The resource fee charged for a TTL extension depends on the current dynamic
write-fee rate set by validators; use simulation to get the exact amount.

## Testnet setup (Friendbot)

On the testnet, use Friendbot to fund the deployer account for free:

```sh
# Fund via the CLI helper
stellar keys generate --global deployer --network testnet

# The CLI automatically funds new identities on testnet; or use Friendbot
# directly:
curl "https://friendbot.stellar.org/?addr=$(stellar keys address deployer)"
```

Friendbot provisions **10 000 XLM** (testnet tokens with no real-world value),
which is more than enough for repeated test deployments.

## Mainnet checklist additions

Before deploying to mainnet, confirm:

- [ ] Deployer account exists and holds **≥ 5 XLM** available balance.
- [ ] Additional XLM buffer is available for ongoing transaction fees and TTL
      extensions as the registry grows.
- [ ] You have verified the resource fee for `initialize` using
      `stellar contract simulate`.
- [ ] You have a funded operator account separate from the admin key for
      day-to-day contract invocations.

## References

- [Stellar Fees, Resource Limits, and Metering](https://developers.stellar.org/docs/learn/fundamentals/fees-resource-limits-metering)
- [Soroban State Archival](https://developers.stellar.org/docs/learn/fundamentals/contract-development/storage/state-archival)
- [Stellar Lab — Resource Limits & Fees](https://lab.stellar.org/network-limits)
- [`storage-model.md`](storage-model.md) — CarbonMint storage layout and TTL constants
- [`deployment-guide.md`](deployment-guide.md) — build and deploy workflow
- [`mainnet-checklist.md`](mainnet-checklist.md) — full pre-deployment checklist
