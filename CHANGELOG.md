# Changelog

All notable changes to this project are documented here. The format is based
on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `batch_transfer` entrypoint: transfers credits to up to 50 recipients in a
  single atomic invocation, bounded by `MAX_RECIPIENTS` (issue #77).
- `TransferItem` struct and `TooManyRecipients` error (code 11).
- `batch_xfr` event emitted on successful batch transfers.
- Documentation for batch-transfer resource limits and known limitations.
- Storage schema versioning: `storage_schema_version()` view and a persisted
  `StorageSchemaVersion` instance key, written on `initialize`, so indexers can
  detect storage-layout migrations (issue #48).
- Coverage reporting: `tarpaulin.toml` config and a `make coverage` target that
  emits Html/Lcov/Json reports and enforces a coverage threshold (issue #55).
- Expanded unit coverage for event emission (minted/listed/delisted/transferred/
  retired/paused/adminset) and negative-amount/negative-price rejection paths.
- Committed `Cargo.lock` pinning `ed25519-dalek 2.2.0` so the SDK 21.7.x test
  harness resolves deterministically (the unpinned resolution pulled the
  incompatible `ed25519-dalek 3.0.0`).

## [0.2.0]

### Added

- Admin pause control: `set_paused`, `is_paused`, and a `Paused` error that
  blocks `mint_batch` while paused.
- Admin rotation via `set_admin`, emitting an `adminset` event.
- `retire_for` to retire credits on behalf of a named beneficiary, recorded on
  the retirement certificate.
- `listing_info` view returning a compact `Listing` (seller, price, listed
  flag, available amount).
- `total_minted` view tracking cumulative credits minted across all batches.
- `SameAccount` error rejecting self-transfers.
- `paused` and `adminset` events.

### Changed

- Retirement certificates now carry a `beneficiary` field (defaults to the
  `self` sentinel).
- Shared `retire` logic refactored into a single internal helper.

## [0.1.0]

### Added

- Initial CarbonMint marketplace: batch minting, listing, buying, direct
  transfers, and retirement with on-chain certificates.
