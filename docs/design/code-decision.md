# `vyapaar-core` — Closed Decision Grammar Reference

Documents `vyapaar-core/src/decision.rs`: the NoFX untrusted-AI closed decision
grammar. All LLM decisions must strictly deserialize into the closed `Decision`
enum (`#[serde(deny_unknown_fields)]`) and pass validation before reaching the
risk gate or execution service.

## `Decision` Enum

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Decision {
    PlaceOrder { symbol, side, qty, price_limit, algo_id, rationale },
    ModifyOrder { instrument, new_qty, new_price, algo_id, rationale },
    CancelOrder { instrument, algo_id, rationale },
    CancelAll { algo_id, reason },
    Hold { reason },
}
```

| Variant | Purpose | Required Fields |
|---------|---------|-----------------|
| `PlaceOrder` | Submit new order | `symbol`, `side`, `qty`, `algo_id` |
| `ModifyOrder` | Amend open order | `instrument`, `algo_id` |
| `CancelOrder` | Cancel single order | `instrument` |
| `CancelAll` | Emergency cancel-all | (none required) |
| `Hold` | No action | (none required) |

`price_limit` is optional — when `None`, a market order is requested. `algo_id` is
mandatory for SEBI Retail Algo Circular audit trail.

## `Decision::validate()`

Structural sanity rules enforced on every decision:

| Check | Applies To | Error |
|-------|-----------|-------|
| Symbol non-empty | `PlaceOrder` | `Validation("Symbol cannot be empty")` |
| Algo ID non-empty | `PlaceOrder`, `ModifyOrder` | `Validation("algo_id cannot be empty")` |
| Algo ID matches format | All variants with algo_id | `Validation(...)` |
| Quantity strictly positive | `PlaceOrder` | `Validation("Quantity must be strictly positive")` |
| Price limit positive | `PlaceOrder` (when set) | `Validation("Price limit must be positive")` |

## SEBI Algo ID Format

```
ALGO_ID_FORMAT = "^[A-Za-z0-9_-]{4,32}$"
```

4 to 32 characters of letters, digits, `_`, or `-`. Broker golden vectors use
dashed IDs (e.g. `momentum-alpha-v1`).

## `Decision::from_json_str(s)`

Parses a JSON string into a validated `Decision`: serde deserialization
(enforcing `deny_unknown_fields`) followed by `validate()`. Any unauthorized
field injected by an LLM yields a `CoreError::Validation`.

## Zero-Floats Note

All prices and quantities are `rust_decimal::Decimal`. No floating-point
arithmetic in the decision surface.

## Related

- `vyapaar-agent-core/src/decision.rs` — `DecisionParser` (stricter parser with
  price-band grounding, collar checks, and `RiskEngine` pre-trade gate).
- `vyapaar-core/src/types.rs` — `Price`, `Quantity`.
- `vyapaar-core/src/order.rs` — `OrderSide`.
