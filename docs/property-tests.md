# Property Tests

This note documents the **property-tests** of the carbonmint-contract contract.

carbonmint-contract is a Soroban smart contract on the Stellar network. This page is part of the
project's reference documentation and describes the property-tests in detail, covering the relevant
entrypoints, storage layout, and invariants where applicable.

A lightweight fuzzing scaffold now lives under `fuzz/` as a starting point for property-style exploration.
See the README, the sources under `src/`, and the harness under `fuzz/` for the authoritative implementation.
