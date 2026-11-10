# Event Reference

This document defines the lifecycle event schemas for the `carbonmint-contract` Soroban contract.

To ensure off-chain indexers and data pipelines can safely reconstruct circulating and retired supply without ambiguity or schema desynchronization, all core lifecycle events use versioned, normalized payloads with unambiguous units and explicit actors.

## Schema Versioning

- **Current Version**: `EVENT_SCHEMA_VERSION = 1`
- **Standard Unit**: `UNIT_TCO2E = "tCO2e"` (metric tonnes of CO2 equivalent)

All lifecycle event payloads include a `version: u32` header. Off-chain parsers reject payloads where `version` does not match supported versions or where units are ambiguous.

---

## 1. Mint Event (`minted`)

Emitted when an authorized issuer creates a new batch of carbon credits.

- **Topic**: `("minted", issuer: Address)`
- **Payload (`MintEvent`)**:
  | Field | Type | Description |
  |---|---|---|
  | `version` | `u32` | Schema version (`1`) |
  | `batch_id` | `u64` | Unique batch identifier |
  | `quantity` | `i128` | Number of carbon credits minted |
  | `unit` | `Symbol` | Standard unit (`"tCO2e"`) |
  | `issuer` | `Address` | Account that created the batch |
  | `project_id` | `String` | Originating carbon offset project identifier |
  | `vintage` | `u32` | Vintage year (e.g. `2024`) |
  | `outcome` | `Symbol` | Outcome tag (`"minted"`) |

---

## 2. Transfer Event (`transfer`)

Emitted when carbon credits are transferred directly between two accounts.

- **Topic**: `("transfer", from: Address, to: Address)`
- **Payload (`TransferEvent`)**:
  | Field | Type | Description |
  |---|---|---|
  | `version` | `u32` | Schema version (`1`) |
  | `batch_id` | `u64` | Batch identifier of the credits |
  | `quantity` | `i128` | Number of carbon credits transferred |
  | `unit` | `Symbol` | Standard unit (`"tCO2e"`) |
  | `from` | `Address` | Sender address |
  | `to` | `Address` | Recipient address |
  | `outcome` | `Symbol` | Outcome tag (`"transfer"`) |

---

## 3. Trade / Buy Event (`bought`)

Emitted when a buyer purchases listed credits on the marketplace.

- **Topic**: `("bought", buyer: Address, seller: Address)`
- **Payload (`TradeEvent`)**:
  | Field | Type | Description |
  |---|---|---|
  | `version` | `u32` | Schema version (`1`) |
  | `batch_id` | `u64` | Batch identifier |
  | `quantity` | `i128` | Number of carbon credits purchased |
  | `unit` | `Symbol` | Standard unit (`"tCO2e"`) |
  | `buyer` | `Address` | Buyer address |
  | `seller` | `Address` | Seller / issuer address |
  | `unit_price` | `i128` | Price per credit |
  | `total_price` | `i128` | Total settlement amount (`quantity * unit_price`) |
  | `outcome` | `Symbol` | Outcome tag (`"settled"`) |

---

## 4. Retirement Event (`retired`)

Emitted when credits are permanently burned and retired on behalf of a beneficiary.

- **Topic**: `("retired", holder: Address)`
- **Payload (`RetirementEvent`)**:
  | Field | Type | Description |
  |---|---|---|
  | `version` | `u32` | Schema version (`1`) |
  | `batch_id` | `u64` | Batch identifier |
  | `quantity` | `i128` | Number of carbon credits permanently burned |
  | `unit` | `Symbol` | Standard unit (`"tCO2e"`) |
  | `holder` | `Address` | Account that executed the retirement |
  | `certificate_id` | `u64` | Unique on-chain certificate identifier |
  | `beneficiary` | `String` | Named beneficiary |
  | `outcome` | `Symbol` | Outcome tag (`"burned"`) |

---

## Parsing and Validation Rules

Indexers and clients should use the parsing utilities (`parse_mint_event`, `parse_transfer_event`, `parse_trade_event`, `parse_retirement_event`):

1. **Version Check**: Payloads with unhandled `version` numbers must be rejected (`ParseError::UnsupportedVersion`).
2. **Quantity Bounds**: Quantities `<= 0` must be rejected (`ParseError::InvalidQuantity`).
3. **Unit Validation**: Only `"tCO2e"` is permitted as a valid unit (`ParseError::InvalidUnit`).
4. **Outcome Validation**: State transition outcome tags must match expected values (`ParseError::InvalidOutcome`).
