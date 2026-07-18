#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# verify-wasm-hash.sh
#
# Verify that the WASM hash of a deployed Soroban contract matches the hash
# of the locally built WASM binary.
#
# This script:
#   1. Builds the contract (unless --skip-build is passed).
#   2. Computes the SHA-256 hash of the local .wasm file via `stellar
#      contract hash`.
#   3. Fetches the WASM hash of the deployed contract from the ledger via
#      `stellar contract info`.
#   4. Compares the two hashes and prints a clear pass / fail message.
#
# Usage:
#   ./scripts/verify-wasm-hash.sh <CONTRACT_ID> [NETWORK]
#
# Arguments:
#   CONTRACT_ID  (required) The deployed contract id (e.g., C...).
#   NETWORK      (optional) Stellar network name (default: testnet).
#
# Options:
#   --skip-build  Skip the initial 'make build' step. Useful when iterating
#                 on hash verification without recompiling.
#
# Exit codes:
#   0  - Hash matches (verification passed).
#   1  - Hash does not match (verification failed).
#   2  - A prerequisite or invocation error occurred.
# ---------------------------------------------------------------------------
set -euo pipefail

# ---- constants --------------------------------------------------------
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT" || { echo "Error: could not cd to $PROJECT_ROOT" >&2; exit 2; }

CONTRACT_NAME="carbonmint_contract"
WASM="target/wasm32-unknown-unknown/release/${CONTRACT_NAME}.wasm"

# ---- argument parsing -------------------------------------------------
SKIP_BUILD=false
POSITIONAL=()

while [[ $# -gt 0 ]]; do
    case "$1" in
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        -*)
            echo "Error: unknown option $1" >&2
            echo "Usage: $0 <CONTRACT_ID> [NETWORK]" >&2
            exit 2
            ;;
        *)
            POSITIONAL+=("$1")
            shift
            ;;
    esac
done

CONTRACT_ID="${POSITIONAL[0]:-}"
NETWORK="${POSITIONAL[1]:-testnet}"

if [[ -z "$CONTRACT_ID" ]]; then
    echo "Error: CONTRACT_ID is required." >&2
    echo "Usage: $0 <CONTRACT_ID> [NETWORK]" >&2
    exit 2
fi

# ---- pre-requisites ---------------------------------------------------
if ! command -v stellar &>/dev/null; then
    echo "Error: 'stellar' CLI not found. Install it from:" >&2
    echo "  https://developers.stellar.org/docs/tools/cli" >&2
    exit 2
fi

# ---- 1. Build the contract -------------------------------------------
if [[ "$SKIP_BUILD" == false ]]; then
    echo "==> Building contract…"
    make build
else
    echo "==> Skipping build (--skip-build). Using existing WASM at:"
    echo "    $WASM"
fi

if [[ ! -f "$WASM" ]]; then
    echo "Error: WASM file not found at $WASM" >&2
    echo "       Run 'make build' first or remove --skip-build." >&2
    exit 2
fi

# ---- 2. Compute local hash -------------------------------------------
echo ""
echo "==> Computing local WASM hash…"
LOCAL_HASH=$(stellar contract hash --wasm "$WASM")
echo "    Local  hash:  $LOCAL_HASH"

# ---- 3. Fetch deployed hash -------------------------------------------
echo ""
echo "==> Fetching deployed WASM hash for contract $CONTRACT_ID…"
echo "    Network:      $NETWORK"
echo ""

# The `stellar contract info --id` output includes the WASM hash.
# We extract the 64-character hex SHA-256 hash regardless of label.
INFO_OUTPUT=$(stellar contract info --id "$CONTRACT_ID" --network "$NETWORK")
DEPLOYED_HASH=$(echo "$INFO_OUTPUT" | grep -oE '[0-9a-f]{64}' | head -1)

if [[ -z "$DEPLOYED_HASH" ]]; then
    echo "Error: could not retrieve WASM hash from the ledger." >&2
    echo "       Check that CONTRACT_ID=$CONTRACT_ID is correct and" >&2
    echo "       that you have access to the $NETWORK network." >&2
    echo "" >&2
    echo "Full output from 'stellar contract info':" >&2
    echo "$INFO_OUTPUT" >&2
    exit 2
fi

echo "    Deployed hash: $DEPLOYED_HASH"

# ---- 4. Compare ------------------------------------------------------
echo ""
echo "=========================================="
if [[ "$LOCAL_HASH" == "$DEPLOYED_HASH" ]]; then
    echo "  ✅ VERIFICATION PASSED"
    echo "  The local WASM matches the deployed contract."
    exit 0
else
    echo "  ❌ VERIFICATION FAILED"
    echo "  The local WASM does NOT match the deployed contract."
    echo ""
    echo "  Possible causes:"
    echo "    • The local source differs from the deployed version."
    echo "    • The build is not reproducible (e.g., timestamps,"
    echo "      compiler version differences)."
    echo "    • The contract id points to a different contract."
    exit 1
fi
