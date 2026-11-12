# `vyapaar-core` — Closed Decision Grammar Reference

Documents `vyapaar-core/src/decision.rs`: the NoFX untrusted-AI closed decision
grammar. Language models are untrusted for direct execution — all LLM decisions
must strictly deserialize into the closed `Decision` enum
(`#[serde(deny_unknown_fields)]`) and pass validation before reaching the risk
gate or execution service.

## `Decision` Enum

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Decision {
    PlaceOrder  { symbol, side, qty, price_limit, algo_id, rationale },
    ModifyOrder { instrument, new_qty, new_price, algo_id, rationale },
    CancelOrder { instrument, algo_id, rationale },
    CancelAll   { algo_id, reason },
    Hold        { reason },
}
```

| Variant | Purpose | Required / Mandatory Fields |
|---------|---------|------------------------------|
| `PlaceOrder` | Submit new order | `symbol`, `side`, `qty`, `algo_id` (mandatory) |
| `ModifyOrder` | Amend open order | `instrument`, `algo_id` (mandatory) |
| `CancelOrder` | Cancel single order | `instrument`, `algo_id` (non-empty required) |
| `CancelAll` | Emergency cancel-all | `algo_id` / `reason` optional |
| `Hold` | No action | `reason` optional |

`price_limit` is optional — when `None`, a market order is requested. `algo_id`
is mandatory on order-carrying variants for the SEBI Retail Algo Circular audit
trail.

## `Decision::validate()`

Structural sanity rules enforced on every decision (`CoreError::Validation`):

| Check | Applies To |
|-------|-----------|
| Symbol non-empty (`"Symbol cannot be empty"`) | `PlaceOrder` |
| Algo ID non-empty (`"algo_id cannot be empty"`) | `PlaceOrder`, `ModifyOrder`, `CancelOrder` |
| Algo ID matches format | every variant carrying a non-optional `algo_id` |
| Quantity strictly positive | `PlaceOrder` |
| Price limit positive | `PlaceOrder` (when `price_limit` set) |

`CancelAll` and `Hold` only validate `algo_id` when one is supplied.

## SEBI Algo ID Format

```
ALGO_ID_FORMAT = "^[A-Za-z0-9_-]{4,32}$"
```

4 to 32 characters of ASCII letters, digits, `_`, or `-`. The check is
implemented as a length + character-class scan (`validate_algo_id`); brokers
and golden vectors use dashed ids (e.g. `momentum-alpha-v1`).

## `Decision::from_json_str(s)`

Parses a JSON string into a validated `Decision`: serde deserialization
(enforcing `deny_unknown_fields`) followed by `validate()`. Any unauthorized
field injected by an LLM yields `CoreError::Validation`.

## Zero-Floats Note

All prices and quantities are `rust_decimal::Decimal` (`Price`, `Quantity`);
`side` is `order::OrderSide`. No floating-point arithmetic in the decision
surface.

## Related

- `vyapaar-agent-core/src/decision.rs` — `DecisionParser` (stricter parser with
  price-band grounding, collar checks, and the `RiskEngine` pre-trade gate).
- `vyapaar-core/src/types.rs` — `Price`, `Quantity` (see [code-types](code-types.md)).
- `vyapaar-core/src/order.rs` — `OrderSide` (see [code-wire-contracts](code-wire-contracts.md)).
