#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# smoke-test-testnet.sh
#
# Automates a full testnet deployment smoke test for the CarbonMint contract.
#
# This script:
#   1. Generates a temporary identity and funds it via Friendbot.
#   2. Builds and deploys the contract to testnet.
#   3. Initializes the contract.
#   4. Invokes read-only entrypoints to verify basic functionality.
#   5. Runs verify-wasm-hash.sh to ensure the deployed bytecode matches.
#
# Usage:
#   ./scripts/smoke-test-testnet.sh
# ---------------------------------------------------------------------------
set -euo pipefail

# ---- constants --------------------------------------------------------
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT" || { echo "Error: could not cd to $PROJECT_ROOT" >&2; exit 2; }

IDENTITY="smoke-deployer-$$"
NETWORK="testnet"

cleanup() {
    echo "==> Cleaning up identity $IDENTITY"
    # The CLI doesn't have a direct 'remove' command for keys in this version,
    # but we can try to leave it or just accept it's a throwaway in the global keystore.
    # We will just print a message.
}
trap cleanup EXIT

# ---- pre-requisites ---------------------------------------------------
if ! command -v stellar &>/dev/null; then
    echo "Error: 'stellar' CLI not found." >&2
    exit 2
fi

# ---- 1. Generate identity and fund ------------------------------------
echo "==> Generating identity: $IDENTITY"
stellar keys generate --global "$IDENTITY" --network "$NETWORK"

echo "==> Funding via Friendbot..."
ADDRESS=$(stellar keys address "$IDENTITY")
curl -s "https://friendbot.stellar.org/?addr=$ADDRESS" > /dev/null

# ---- 2. Build and Deploy ----------------------------------------------
echo "==> Building contract..."
make build

echo "==> Deploying to testnet..."
DEPLOY_OUTPUT=$(stellar contract deploy --wasm target/wasm32-unknown-unknown/release/carbonmint_contract.wasm --source "$IDENTITY" --network "$NETWORK")
CONTRACT_ID=$(echo "$DEPLOY_OUTPUT" | tr -d '[:space:]')
if [[ -z "$CONTRACT_ID" ]]; then
    echo "Error: Failed to extract CONTRACT_ID from deploy output" >&2
    exit 1
fi
echo "    Contract ID: $CONTRACT_ID"

# ---- 3. Initialize ----------------------------------------------------
echo "==> Initializing contract..."
stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source "$IDENTITY" \
  --network "$NETWORK" \
  -- initialize --admin "$ADDRESS"

# ---- 4. Smoke test read-only entrypoints ------------------------------
echo "==> Invoking version()..."
VERSION=$(stellar contract invoke --id "$CONTRACT_ID" --source "$IDENTITY" --network "$NETWORK" -- version | tr -d '"')
echo "    Version: $VERSION"

echo "==> Invoking is_paused()..."
PAUSED=$(stellar contract invoke --id "$CONTRACT_ID" --source "$IDENTITY" --network "$NETWORK" -- is_paused)
echo "    Paused: $PAUSED"

echo "==> Invoking batch_count()..."
BATCH_COUNT=$(stellar contract invoke --id "$CONTRACT_ID" --source "$IDENTITY" --network "$NETWORK" -- batch_count)
echo "    Batch Count: $BATCH_COUNT"

# ---- 5. Verify WASM hash ----------------------------------------------
echo "==> Verifying deployed WASM hash..."
./scripts/verify-wasm-hash.sh "$CONTRACT_ID" "$NETWORK" --skip-build

echo ""
echo "  ✅ SMOKE TEST PASSED"
exit 0
