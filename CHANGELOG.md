# Changelog

All notable changes to this project are documented here. The format is based
on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
