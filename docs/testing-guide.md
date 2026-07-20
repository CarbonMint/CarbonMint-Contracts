# Testing Guide

This note documents the **testing** approach for `carbonmint-contract`.

`carbonmint-contract` is a Soroban smart contract on the Stellar network. This page
describes how to run the unit tests and how coverage is reported.

## Running the tests

Tests use the Soroban SDK `testutils` harness and run against an in-process `Env`:

```sh
cargo test
```

To build the contract WASM:

```sh
cargo build --target wasm32-unknown-unknown --release
```

## Coverage reporting

Coverage is reported with [`cargo-tarpaulin`](https://github.com/xd009642/tarpaulin),
configured via `tarpaulin.toml` at the workspace root. The config:

- emits `Html`, `Lcov` and `Json` reports under `target/coverage/`,
- excludes generated/SDK shims and build artifacts,
- fails the run if line coverage drops below the configured threshold (baseline `80%`).

Run it with:

```sh
make coverage
# or directly:
cargo tarpaulin --config tarpaulin.toml --workspace --timeout 120
```

The `Html` report (`target/coverage/tarpaulin-report.html`) is the human-readable
view; `Lcov` (`target/coverage/lcov.info`) and `Json` feed CI dashboards.

## Fuzzing scaffold

A basic `cargo-fuzz` scaffold is available under `fuzz/` for exploratory input-driven testing. The target is intentionally small and uses the same in-process Soroban environment as the unit tests so it can be expanded without introducing a separate harness.

To bootstrap it in a Rust-enabled environment, run:

```sh
cargo install cargo-fuzz
cargo fuzz init
cargo fuzz run fuzz_target_1
```

The scaffold currently exercises the happy path for initialization, minting and buying with byte-driven payloads.
## Debug-assertion helpers

The crate ships compile-time-gated debug-assertion macros that provide richer
diagnostic output than the standard Rust macros when a test assertion fails.
They live in `src/debug_assertions.rs` and are available **only** in test
builds (they are stripped from the release WASM).

| Macro         | Purpose                                                |
|---------------|--------------------------------------------------------|
| `assert_ok!`  | Assert a `Result` is `Ok`, printing the error on panic |
| `assert_err!` | Assert a `Result` is a specific `Err` variant          |

### `assert_ok!`

```rust
use carbonmint_contract::assert_ok;
use carbonmint_contract::math;

let val = assert_ok!(math::checked_add(10, 20));
assert_eq!(val, 30);
```

On failure the macro includes the file, line, column, original expression
and the debug representation of the actual error.

### `assert_err!`

```rust
use carbonmint_contract::{assert_err, Error};

let res: Result<i128, Error> = Err(Error::Overflow);
assert_err!(res, Error::Overflow);
```

The expected error is specified as a pattern, so any `enum` variant works.
If the result is `Ok` or a *different* error, the macro panics with a
message showing both the expected pattern and the actual value.

These helpers are used throughout the math module tests (`src/math.rs`) to
make overflow and underflow assertions more readable and to improve failure
diagnostics when a checked arithmetic test breaks.

## What the suite covers

The unit suite (`src/test.rs`) exercises:

- initialization, admin rotation and pause control,
- batch minting, listing, buying, transferring and retiring,
- error paths (uninitialized, overflow, insufficient balance, not-listed, same-account),
- event emission for `minted`, `listed`, `delisted`, `bought`, `transferred`, `retired`, `paused` and `adminset`.

New behaviour should land with a corresponding test so coverage does not regress.
