# Upgrade Strategy

CarbonMint currently uses versioned deployments rather than in-place contract
upgrades. The contract does not expose an entrypoint that calls
`update_current_contract_wasm`, so deploying a new WASM creates a new contract
ID and independent storage. Existing state is not copied automatically.

For each release, operators must record the old and new contract IDs, WASM
hashes, logic versions, storage schema versions, initialization transactions,
and the ledger boundary used by integrations and indexers. Storage changes must
include a reviewed migration and reconciliation plan before production cutover.

If a release fails, follow the [rollback procedure](rollback-procedure.md).
That runbook explains when integrations can safely return to the retained
deployment and when post-cutover writes require forward recovery instead.

An in-place upgrade mechanism must not be assumed until an authenticated
upgrade entrypoint, migration rules, events, and tests are implemented and
reviewed in this repository.
