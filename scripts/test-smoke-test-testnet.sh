#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# test-smoke-test-testnet.sh
#
# Standalone unit tests for the smoke-test-testnet.sh script.
#
# Tests the script execution flow using mocked binaries for stellar, curl,
# and make.
#
# Usage:
#   ./scripts/test-smoke-test-testnet.sh
# ---------------------------------------------------------------------------
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$HERE/.." && pwd)"
TEST_TMPDIR="$(mktemp -d "/tmp/carbonmint-smoke-test-XXXXXX")"
EXIT_CODE=0
SCRIPT="$HERE/smoke-test-testnet.sh"

cleanup() {
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

make_mock_binaries() {
    local mock_dir="$TEST_TMPDIR/bin"
    mkdir -p "$mock_dir"

    # mock stellar
    cat > "$mock_dir/stellar" <<'MOCKSCRIPT'
#!/usr/bin/env bash
case "$*" in
    *"keys generate"*)
        echo "generated"
        ;;
    *"keys address"*)
        echo "G1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ"
        ;;
    *"contract deploy"*)
        echo "CABCDEF1234567890"
        ;;
    *"contract invoke"*"-- initialize"*)
        echo "null"
        ;;
    *"contract invoke"*"-- version"*)
        echo '"2"'
        ;;
    *"contract invoke"*"-- is_paused"*)
        echo "false"
        ;;
    *"contract invoke"*"-- batch_count"*)
        echo "0"
        ;;
    *)
        echo "mock-stellar: unexpected args: $*" >&2
        exit 1
        ;;
esac
MOCKSCRIPT
    chmod +x "$mock_dir/stellar"

    # mock curl
    cat > "$mock_dir/curl" <<'MOCKSCRIPT'
#!/usr/bin/env bash
echo '{"hash":"123"}'
MOCKSCRIPT
    chmod +x "$mock_dir/curl"

    # mock make
    cat > "$mock_dir/make" <<'MOCKSCRIPT'
#!/usr/bin/env bash
echo "make build mocked"
MOCKSCRIPT
    chmod +x "$mock_dir/make"

    # mock verify-wasm-hash.sh wrapper script (it's called via relative path)
    mkdir -p "$TEST_TMPDIR/mock_cwd/scripts"
    cat > "$TEST_TMPDIR/mock_cwd/scripts/verify-wasm-hash.sh" <<'MOCKSCRIPT'
#!/usr/bin/env bash
echo "verify mocked"
MOCKSCRIPT
    chmod +x "$TEST_TMPDIR/mock_cwd/scripts/verify-wasm-hash.sh"

    echo "$mock_dir"
}

# ---- unit tests ---------------------------------------------------------

test_e2e_smoke_script() {
    local mock_dir
    mock_dir=$(make_mock_binaries)
    
    # We will copy the script into the mock_cwd to trick it into using our mock verify script
    cp "$SCRIPT" "$TEST_TMPDIR/mock_cwd/scripts/"
    
    local rc=0
    # Run the script with the mocked path
    (
        cd "$TEST_TMPDIR/mock_cwd"
        PATH="$mock_dir:$PATH" ./scripts/smoke-test-testnet.sh >/dev/null 2>&1
    ) || rc=$?

    if [[ "$rc" -ne 0 ]]; then
        echo "    Expected exit code 0 when all commands succeed, got $rc"; return 1
    fi
    return 0
}

test_stellar_cli_not_found() {
    local save_path="$PATH"
    local check_path="/tmp"
    PATH="$check_path"
    
    local rc=0
    # In empty path, stellar shouldn't be found
    PATH="" "$SCRIPT" 2>/dev/null || rc=$?
    
    PATH="$save_path"

    if [[ "$rc" -ne 2 ]]; then
        echo "    Expected exit code 2 when stellar CLI is missing, got $rc"; return 1
    fi
    return 0
}

# ---- run tests --------------------------------------------------------

echo "=========================================="
echo "  test-smoke-test-testnet.sh - test suite"
echo "=========================================="
echo ""

run_test "E2E mock smoke test -> exit 0"            test_e2e_smoke_script
run_test "Missing stellar CLI -> exit 2"            test_stellar_cli_not_found

# ---- summary -----------------------------------------------------------
echo "=========================================="
if [[ "$EXIT_CODE" -eq 0 ]]; then
    echo "  ALL TESTS PASSED"
else
    echo "  SOME TESTS FAILED"
fi
echo "=========================================="
exit "$EXIT_CODE"
