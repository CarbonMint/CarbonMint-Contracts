# Math Module

The `carbonmint_contract::math` module provides reusable checked and
saturating arithmetic helpers for `i128` and `u64` types used throughout
the CarbonMint contract.

## Motivation

Before this module existed, every arithmetic operation in the contract
repeated the same `ok_or(Error::Overflow)` pattern inline.  The math module
extracts those wrappers into a single well‑tested location, making the
codebase easier to audit, maintain, and extend.

## Design

All functions are **panic‑free** by construction (they never call `unwrap`
or `expect` internally).  They delegate directly to the standard library's
built‑in `checked_*` / `saturating_*` methods and map the `None` case to
[`Error::Overflow`] where applicable.

### Function overview

| Function                  | Returns                  | Error case              |
|---------------------------|--------------------------|-------------------------|
| `checked_add(a, b)`       | `Result<i128, Error>`    | `a + b` overflows i128  |
| `checked_sub(a, b)`       | `Result<i128, Error>`    | `a - b` underflows i128 |
| `checked_mul(a, b)`       | `Result<i128, Error>`    | `a * b` overflows i128  |
| `checked_div(a, b)`       | `Result<i128, Error>`    | `b == 0` or overflow    |
| `saturating_add(a, b)`    | `i128`                   | clamps to `±MAX`        |
| `saturating_sub(a, b)`    | `i128`                   | clamps to `MIN`         |
| `checked_add_u64(a, b)`   | `Result<u64, Error>`     | `a + b` overflows u64   |
| `saturating_add_u64(a, b)` | `u64`                   | clamps to `u64::MAX`    |

### Where each is used

- **`checked_add` / `checked_sub`** — token balance transfers, retirement
  burns, cumulative minted total, circulating supply.  These are
  operations where overflow would represent a critical bug.
- **`checked_add_u64`** — monotonic batch and certificate id counters.
- **`saturating_*`** — available for future aggregate counters where
  clamping (rather than erroring) is the safer behaviour (see
  [ADR‑0026](../docs/adr/0026-prefer-saturating-math-for-aggregates.md)).

## Tests

The module carries its own `#[cfg(test)]` sub-module with unit tests that
cover:

- Normal (non‑overflow) results for every function
- Overflow / underflow / division‑by‑zero error paths
- Clamp behaviour for saturating variants
- Edge cases (`i128::MIN / -1`, `i128::MAX + 1`, `u64::MAX + 1`, etc.)
- **Maximum-value arithmetic paths**: identity operations, zero-products, and
  near-boundary success cases at `i128::MAX`, `i128::MIN`, and `u64::MAX`
  (e.g. `checked_add(i128::MAX, 0)`, `checked_mul(i128::MAX, 1)`,
  `checked_div(i128::MAX, i128::MAX)`, `saturating_sub(i128::MIN, i128::MAX)`,
  `saturating_add_u64(u64::MAX, u64::MAX)`)

The contract-level integration tests in `src/test.rs` also exercise these
paths end-to-end through mint, transfer, buy, and retire operations using
`i128::MAX` amounts, verifying that `Error::Overflow` is returned (not a
panic) when the ceiling is legitimately exceeded.

Run them with:

```bash
cargo test -p carbonmint-contract math::
```

The existing contract‑level tests in `src/test.rs` also exercise the math
module indirectly via mint, transfer, buy, and retire operations.

## Usage

```rust
use carbonmint_contract::math;

// Checked — fails with Error::Overflow on overflow.
let sum = math::checked_add(a, b)?;
let diff = math::checked_sub(a, b)?;

// Saturating — clamps instead of failing.
let total = math::saturating_add(a, b);

// u64 counters.
let next_id = math::checked_add_u64(counter, 1)?;
```

## Versioning

The public API of this module is covered by the contract's
[semantic versioning](../docs/versioning.md) guarantees.
