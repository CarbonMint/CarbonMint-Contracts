# Contributing Guide

This guide covers how to propose and land contract changes for CarbonMint.
It is intended for contributors who touch Rust contract code, storage layout,
events, auth rules, tests, or deployment-related artifacts.

## Scope

Contract changes may include:

- entrypoint behavior and authorization rules,
- storage layout or schema version changes,
- error handling and event emission,
- tests and documentation updates,
- deployment or wasm verification workflow changes.

## Before you start

- Read the main project documentation in the repository, especially the
  README, the architecture notes, the testing guide, and the upgrade strategy.
- Review the implementation in the contract sources under src/ before editing
  behavior that affects state transitions or public API surface.
- Keep changes aligned with the repository's Soroban/Soroban SDK 21.x setup and
  the existing contract conventions around checked arithmetic, auth, and errors.

## Recommended workflow

1. Create a dedicated branch for the change.
2. Identify the smallest set of files needed for the change.
3. Implement the contract change and keep behavior changes explicit.
4. Add or update automated tests in src/test.rs whenever contract behavior changes.
5. Update relevant documentation if the public API, errors, events, or storage model changes.
6. Run the standard validation commands before opening a pull request.

## Validation commands

Run the following from the repository root:

```sh
cargo test
cargo check --all-targets
cargo build --target wasm32-unknown-unknown --release
make fmt
```

For a full local quality pass, the project Makefile also provides:

```sh
make check
make test
make clippy
make doc
```

## Contract-specific checklist

When making contract changes, review the following before submitting:

- State-changing entrypoints should continue to require the appropriate auth.
- Invariants around balances, totals, and retirement accounting should remain intact.
- New or changed errors should be documented and tested.
- Event topics and payloads should remain consistent with existing indexer assumptions.
- Storage layout changes should be treated as a migration-sensitive change and reviewed with the upgrade strategy.
- Public API changes should be reflected in the README and contract documentation.

## Pull request expectations

A good contract change should include:

- a focused implementation with a clear rationale,
- automated test coverage for the changed behavior,
- documentation updates when the contract interface or semantics change,
- a summary of any deployment or upgrade implications.

See the README and the sources under src/ for the authoritative implementation.
