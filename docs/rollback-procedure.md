# Rollback Procedure

This runbook describes how operators return integrations to a previously
deployed CarbonMint contract after a faulty release. Use it together with the
[deployment guide](deployment-guide.md), [upgrade strategy](upgrade-strategy.md),
and [mainnet checklist](mainnet-checklist.md).

## Scope and limitations

The current contract does not provide an in-place WASM upgrade entrypoint.
Each `stellar contract deploy` creates a new contract ID with independent
storage, and `initialize` creates empty state for that deployment. Consequently,
rollback means changing every integration back to a retained, known-good
contract ID. It does not delete the faulty deployment, copy or merge state, or
reverse transactions that already reached the ledger.

CarbonMint also has no global emergency stop. `set_paused(true)` blocks only
new minting; listing, buying, transferring, and retiring remain available.
Stopping those operations requires disabling write paths in the frontend,
backend, relayers, and scheduled jobs controlled by the project.

If there is no retained deployment, or the retained deployment's state is no
longer usable, perform a forward recovery instead of this cutback procedure.

## Preconditions

Before every production release, record the following in the release record:

- the network and the current and replacement contract IDs;
- the release tag or commit, logic version, storage schema version, and WASM
  hash for both deployments;
- the initialization transaction, deployment ledger, admin address, and
  expected counters for the replacement;
- a read-only snapshot of the retained deployment's admin, versions, counters,
  representative batches, balances, and retirement certificates;
- the configuration owners for the frontend, backend, SDK, relayers, and
  indexers, plus the person authorized to approve a cutback;
- the ledger at which traffic and event ingestion will switch; and
- the location of the approved WASM artifact and release test evidence.

Confirm that the retained contract and its required storage entries have not
expired before starting a release. State archival or missing state is a
rollback blocker; see [TTL and rent](ttl-and-rent.md).

Never place secret keys or seed phrases in a release record, command history,
issue, or incident channel. Commands below use a configured Stellar identity.

## Contain the incident

1. Name an incident lead and record the UTC time and latest observed ledger.
2. Stop new writes from project-controlled APIs, user interfaces, relayers,
   queues, and scheduled jobs. Keep reads available when they are trustworthy.
3. Stop indexer consumers at a recorded ledger boundary. Preserve their cursor
   and raw events; do not discard or rewrite indexed history.
4. If the defect affects minting, the admin may invoke `set_paused(true)` on the
   affected contract. Record the transaction hash. Do not treat this as a full
   contract pause.
5. Leave both deployments addressable while evidence is collected. A deployed
   CarbonMint instance cannot be deactivated by this contract.

## Decide whether cutback is safe

Identify every successful business write to the replacement contract after
its initialization and before containment. Compare transaction results and
events against the pre-release snapshot and indexer cursor.

| Observed state | Action |
| --- | --- |
| No business writes reached the replacement | Proceed with the cutback. |
| Writes reached only the retained contract | Proceed, then deduplicate queued requests before reopening traffic. |
| Any mint, list, unlist, buy, transfer, or retire call reached the replacement | Escalate for reconciliation before reopening writes. A routing cutback alone is not a state rollback. |
| A retirement reached the replacement | Do not replay or reverse it automatically. Retirement has no reversal entrypoint; obtain explicit maintainer and business approval for the recovery plan. |
| The retained contract or required storage is archived, inconsistent, or unavailable | Stop. Restore/repair state or deploy a forward fix; do not route traffic to it. |

Use transaction hashes and ledger sequence numbers, not wall-clock timestamps,
to define the reconciliation interval. Do not automatically replay failed or
ambiguous requests: the contract has no request-level idempotency key.

## Verify the retained deployment

Set these placeholders to values from the approved release record. The
`--send=no` flag guarantees that the health checks are simulated without
submitting a transaction.

```sh
ROLLBACK_NETWORK="mainnet"
ROLLBACK_CONTRACT_ID="C..."
ROLLBACK_READER="operator"
RECORDED_WASM_HASH="64-character-hex-hash"

DEPLOYED_WASM_HASH="$(stellar contract info hash \
  --id "$ROLLBACK_CONTRACT_ID" \
  --network "$ROLLBACK_NETWORK")"
test "$DEPLOYED_WASM_HASH" = "$RECORDED_WASM_HASH"

stellar contract invoke --id "$ROLLBACK_CONTRACT_ID" \
  --source "$ROLLBACK_READER" --network "$ROLLBACK_NETWORK" \
  --send=no -- version
stellar contract invoke --id "$ROLLBACK_CONTRACT_ID" \
  --source "$ROLLBACK_READER" --network "$ROLLBACK_NETWORK" \
  --send=no -- storage_schema_version
stellar contract invoke --id "$ROLLBACK_CONTRACT_ID" \
  --source "$ROLLBACK_READER" --network "$ROLLBACK_NETWORK" \
  --send=no -- get_admin
stellar contract invoke --id "$ROLLBACK_CONTRACT_ID" \
  --source "$ROLLBACK_READER" --network "$ROLLBACK_NETWORK" \
  --send=no -- batch_count
stellar contract invoke --id "$ROLLBACK_CONTRACT_ID" \
  --source "$ROLLBACK_READER" --network "$ROLLBACK_NETWORK" \
  --send=no -- retirement_count
stellar contract invoke --id "$ROLLBACK_CONTRACT_ID" \
  --source "$ROLLBACK_READER" --network "$ROLLBACK_NETWORK" \
  --send=no -- total_minted
```

Compare every result with the release record. Also query representative known
batches, balances, and retirement certificates used by application smoke
tests. Stop if the hash, admin, versions, counters, or sampled state differ.

## Perform the cutback

After the incident lead approves the decision:

1. Update the canonical contract ID in backend and relayer configuration.
2. Update frontend and SDK configuration. Purge configuration caches so clients
   cannot continue submitting to the replacement contract.
3. Point indexers and event filters at the retained contract ID. Resume from the
   recorded pre-cutover cursor, then account separately for any events emitted
   by the replacement contract.
4. Confirm all project-controlled services report the retained ID. Search for
   the replacement ID in deployed configuration before reopening traffic.
5. Enable read traffic and repeat the read-only checks above through the same
   RPC and application paths users rely on.
6. Enable write traffic gradually and monitor errors, counters, balances, and
   events. If minting was paused on the retained deployment, unpause it only
   after validation and explicit incident-lead approval.

Do not redeploy the old WASM and call the result a rollback: that creates
another empty contract ID. Route to the recorded retained deployment unless a
separate, reviewed migration plan explicitly requires a new deployment.

## Reconcile writes

When the replacement accepted writes, maintain two immutable ledgers of facts:
transactions on the retained contract and transactions on the replacement.
Export the affected transaction hashes, ledger numbers, event payloads, batch
IDs, accounts, amounts, and retirement certificate IDs. Then:

1. classify each request as retained-only, replacement-only, duplicated,
   failed, or ambiguous;
2. derive expected balances, batch listings, minted totals, and retired totals
   for both contract IDs;
3. have maintainers review a forward-recovery or compensating-action plan;
4. obtain business approval for any action that changes credit ownership or
   retirement reporting; and
5. execute approved actions one at a time with transaction evidence.

Never edit indexer data to make the two histories appear continuous, and never
reuse a retirement certificate ID from another contract. Contract ID plus
certificate ID identifies the on-chain record.

## Validate and close

Keep write traffic restricted until all of the following are true:

- the application, SDK, backend, relayers, and indexers use the approved ID;
- the deployed WASM hash, logic version, storage schema, and admin match the
  release record;
- representative reads and one explicitly approved low-risk write succeed;
- indexer cursors are advancing without gaps or duplicate processing;
- all post-cutover writes are reconciled or have an approved owner and plan;
- dashboards and alerts identify the active contract ID; and
- stakeholders have received the incident impact and recovery status.

Attach configuration change references, approvals, transaction hashes, ledger
boundaries, command output, reconciliation results, and follow-up actions to the
incident record. Keep the faulty deployment ID on a denylist in project
configuration to prevent accidental reuse.
