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

## What the suite covers

The unit suite (`src/test.rs`) exercises:

- initialization, admin rotation and pause control,
- batch minting, listing, buying, transferring and retiring,
- error paths (uninitialized, overflow, insufficient balance, not-listed, same-account),
- event emission for `minted`, `listed`, `delisted`, `bought`, `transferred`, `retired`, `paused` and `adminset`.

New behaviour should land with a corresponding test so coverage does not regress.
