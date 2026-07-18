#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# test-verify-wasm-hash.sh
#
# Standalone unit tests for the verify-wasm-hash.sh script.
#
# Tests the core comparison logic, hash extraction patterns, argument
# parsing, and an end-to-end invocation with a mocked stellar CLI.
#
# Usage:
#   ./scripts/test-verify-wasm-hash.sh
# ---------------------------------------------------------------------------
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$HERE/.." && pwd)"
TEST_TMPDIR="$(mktemp -d "/tmp/carbonmint-test-XXXXXX")"
EXIT_CODE=0
SCRIPT="$HERE/verify-wasm-hash.sh"

WASM_REL="target/wasm32-unknown-unknown/release/carbonmint_contract.wasm"
WASM_ABS="$PROJECT_ROOT/$WASM_REL"
WASM_BACKUP="${WASM_ABS}.bak"
FAKE_WASM_CREATED=0

cleanup() {
    if [[ -f "$WASM_BACKUP" ]]; then
        mv "$WASM_BACKUP" "$WASM_ABS"
    elif [[ "$FAKE_WASM_CREATED" -eq 1 && -f "$WASM_ABS" ]]; then
        rm -f "$WASM_ABS"
    fi
    rm -rf "$TEST_TMPDIR"
}
trap cleanup EXIT

# ---- helpers -----------------------------------------------------------

run_test() {
    local name="$1"
    shift
    echo ":: Test: $name"
    if "$@"; then
        echo "   PASS"
    else
        echo "   FAIL"
        EXIT_CODE=1
    fi
    echo ""
}

install_fake_wasm() {
    mkdir -p "$(dirname "$WASM_ABS")"
    if [[ -f "$WASM_ABS" ]]; then
        mv "$WASM_ABS" "$WASM_BACKUP"
    fi
    echo "not-a-real-wasm-binary-but-thats-ok" > "$WASM_ABS"
    FAKE_WASM_CREATED=1
}

make_mock_stellar() {
    local local_hash="$1"
    local deployed_hash="$2"
    local contract_info_output="${3:-}"

    local mock_dir="$TEST_TMPDIR/bin"
    mkdir -p "$mock_dir"

    cat > "$mock_dir/stellar" <<MOCKSCRIPT
#!/usr/bin/env bash
case "\$*" in
    *"contract hash"*)
        echo "$local_hash"
        ;;
    *"contract info"*)
        if [[ -n "$contract_info_output" ]]; then
            echo -e "$contract_info_output"
        else
            echo "Contract ID: C..."
            echo "  wasm hash: $deployed_hash"
        fi
        ;;
    *)
        echo "mock-stellar: unexpected args: \$*" >&2
        exit 1
        ;;
esac
MOCKSCRIPT
    chmod +x "$mock_dir/stellar"
    echo "$mock_dir"
}

# ---- unit tests ---------------------------------------------------------

test_hashes_match() {
    local hash="abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
    if [[ "$hash" != "$hash" ]]; then
        echo "    Expected equal hashes to match"; return 1
    fi
    return 0
}

test_hashes_differ() {
    local a="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    local b="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    if [[ "$a" == "$b" ]]; then
        echo "    Expected different hashes to differ"; return 1
    fi
    return 0
}

test_hash_extraction() {
    local output="Contract ID: CABCD...
  wasm hash: cafe1234cafe1234cafe1234cafe1234cafe1234cafe1234cafe1234cafe1234"
    local extracted
    extracted=$(echo "$output" | grep -oE '[0-9a-f]{64}' | head -1)
    local expected="cafe1234cafe1234cafe1234cafe1234cafe1234cafe1234cafe1234cafe1234"
    if [[ "$extracted" != "$expected" ]]; then
        echo "    Hash extraction failed: got '$extracted'"; return 1
    fi
    return 0
}

test_hash_extraction_with_surrounding_text() {
    local output="WASM Hash: 1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef (some note)"
    local extracted
    extracted=$(echo "$output" | grep -oE '[0-9a-f]{64}' | head -1)
    local expected="1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
    if [[ "$extracted" != "$expected" ]]; then
        echo "    Hash extraction failed: got '$extracted'"; return 1
    fi
    return 0
}

test_hash_extraction_no_hash() {
    local output="Contract ID: CABCD...\n  No hash information found"
    local extracted
    extracted=$(echo -e "$output" | grep -oE '[0-9a-f]{64}' | head -1)
    if [[ -n "$extracted" ]]; then
        echo "    Expected empty extraction but got '$extracted'"; return 1
    fi
    return 0
}

test_skip_build_parsing() {
    local skip=false
    local args=("--skip-build")
    for arg in "${args[@]}"; do
        case "$arg" in
            --skip-build) skip=true ;;
        esac
    done
    if [[ "$skip" != true ]]; then
        echo "    --skip-build should set skip=true"; return 1
    fi
    return 0
}

test_contract_id_parsing() {
    local positional=("CABCDEF12345" "testnet")
    local contract_id="${positional[0]:-}"
    local network="${positional[1]:-testnet}"
    if [[ "$contract_id" != "CABCDEF12345" ]]; then
        echo "    Expected contract_id CABCDEF12345, got '$contract_id'"; return 1
    fi
    if [[ "$network" != "testnet" ]]; then
        echo "    Expected network testnet, got '$network'"; return 1
    fi
    return 0
}

test_default_values() {
    local positional=()
    local contract_id="${positional[0]:-}"
    local network="${positional[1]:-testnet}"
    if [[ -n "$contract_id" ]]; then
        echo "    Expected empty contract_id, got '$contract_id'"; return 1
    fi
    if [[ "$network" != "testnet" ]]; then
        echo "    Expected default network 'testnet', got '$network'"; return 1
    fi
    return 0
}

# Verify that `command -v` correctly detects a missing binary
# (mirrors the CLI-not-found check in verify-wasm-hash.sh)
test_cli_not_found_check() {
    # Find any directory that does NOT contain 'stellar' in PATH
    local save_path="$PATH"
    local check_path="/tmp"
    PATH="$check_path"
    if command -v stellar &>/dev/null; then
        # stellar was found in /tmp, fallback to empty PATH test via subshell
        PATH=""
        if command -v stellar &>/dev/null; then
            echo "    stellar should not be findable in empty PATH"; return 1
        fi
    fi
    PATH="$save_path"
    return 0
}

# ---- end-to-end mock tests --------------------------------------------

test_e2e_hashes_match() {
    install_fake_wasm
    local hash="deadbeefcafebabedeadbeefcafebabedeadbeefcafebabedeadbeefcafebabe"
    local mock_dir
    mock_dir=$(make_mock_stellar "$hash" "$hash")
    local rc=0
    PATH="$mock_dir:$PATH" \
    "$SCRIPT" "CABCD..." "testnet" --skip-build 2>/dev/null \
        && rc=0 || rc=$?
    if [[ "$rc" -ne 0 ]]; then
        echo "    Expected exit code 0 when hashes match, got $rc"; return 1
    fi
    return 0
}

test_e2e_hashes_differ() {
    install_fake_wasm
    local mock_dir
    mock_dir=$(make_mock_stellar \
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
    local rc=0
    PATH="$mock_dir:$PATH" \
    "$SCRIPT" "CABCD..." "testnet" --skip-build 2>/dev/null \
        && rc=0 || rc=$?
    if [[ "$rc" -ne 1 ]]; then
        echo "    Expected exit code 1 when hashes differ, got $rc"; return 1
    fi
    return 0
}

test_e2e_missing_contract_id() {
    local mock_dir
    mock_dir=$(make_mock_stellar "dummy" "dummy")
    local rc=0
    PATH="$mock_dir:$PATH" "$SCRIPT" "" "testnet" --skip-build 2>/dev/null \
        && rc=0 || rc=$?
    if [[ "$rc" -ne 2 ]]; then
        echo "    Expected exit code 2 when CONTRACT_ID missing, got $rc"; return 1
    fi
    return 0
}

# ---- run tests --------------------------------------------------------

echo "=========================================="
echo "  verify-wasm-hash.sh - test suite"
echo "=========================================="
echo ""

run_test "Hashes match (logic)"                    test_hashes_match
run_test "Hashes differ (logic)"                   test_hashes_differ
run_test "Hash extraction from info output"        test_hash_extraction
run_test "Hash extraction with surrounding text"   test_hash_extraction_with_surrounding_text
run_test "Hash extraction when no hash present"    test_hash_extraction_no_hash
run_test "--skip-build flag parsing"               test_skip_build_parsing
run_test "CONTRACT_ID parsing"                     test_contract_id_parsing
run_test "Default values"                          test_default_values
run_test "CLI not found check logic"               test_cli_not_found_check
echo "--- end-to-end mock tests ---"
echo ""
run_test "E2E: hashes match -> exit 0"             test_e2e_hashes_match
run_test "E2E: hashes differ -> exit 1"            test_e2e_hashes_differ
run_test "E2E: missing CONTRACT_ID -> exit 2"      test_e2e_missing_contract_id

# ---- summary -----------------------------------------------------------
echo "=========================================="
if [[ "$EXIT_CODE" -eq 0 ]]; then
    echo "  ALL TESTS PASSED"
else
    echo "  SOME TESTS FAILED"
fi
echo "=========================================="
exit "$EXIT_CODE"
