# `vyapaar-core` — Domain Events & Wire Contracts Reference

Documents `vyapaar-core/src/events.rs`, `vyapaar-core/src/wire.rs`, and related
cross-subsystem contracts.

## `DomainEvent` (`events.rs`)

Cross-subsystem domain events that propagate state changes between bounded
contexts. Immutable, carry a `TraceContext` for distributed observability.

### `Reason` / `ReasonGroup` / `ReasonPolarity`

| Type | Values | Purpose |
|------|--------|---------|
| `ReasonGroup` | `Risk`, `Signal`, `Execution`, `Compliance` | Originating subsystem |
| `ReasonPolarity` | `Positive`, `Negative`, `Neutral` | Favors or disfavors the action |
| `Reason` | `group`, `polarity`, `code`, `message` | Structured audit reason (e.g. code `"otr_breached"`) |

### `ExecutionEvent`

Wraps `OrderEvent` (post-submission order lifecycle) with pre-trade risk-gate
outcome. A risk refusal never produces an `OrderEvent` — it is refused before
reaching that scope.

## `OrderEvent` / `OrderEventKind` (`order.rs`)

| Variant | Meaning |
|---------|---------|
| `OrderEventKind::Placed` | Order accepted by broker |
| `OrderEventKind::Modified` | Order amended |
| `OrderEventKind::Cancelled` | Order cancelled |
| `OrderEventKind::Filled` | Order (partially) filled |
| `OrderEventKind::Rejected` | Order rejected |
| `OrderEventKind::Expired` | Order expired |

## `FillReport` (`order.rs`)

Execution confirmation with explicit timestamp, executed price, quantity, exchange
order ID, and transaction costs (`fee` / `fee_breakdown` as `FeeBreakdown` — v7
schema addition).

## `TraceContext` (`wire.rs`)

W3C distributed trace context carrier — zero-dependency, no serde/schemars
derives by design (it is an observability carrier, not a wire contract).

## Schema Versioning

`SCHEMA_VERSION: u32 = 8` (in `lib.rs`):
- v6: instrument master data, portfolio snapshots, signals, screener, domain events, `ports`
- v7: `FillReport::fee`/`fee_breakdown` (`FeeBreakdown`)
- v8: dynamic position sizing contracts, enriched `Signal`, `DomainEvent` aggregate envelope, `TraceContext`

`API_VERSION: &str = "v1"` — REST endpoint surface version.

## Zero-I/O Purity

All event types are pure data — no network, filesystem, or clock queries.
Timestamps are explicit `DateTime<Utc>` parameters, never `SystemTime::now()`.

## Related

- `vyapaar-agent-core/src/decision.rs` — `DecisionParser` validates `vyapaar_core::Decision` before risk gate.
- `vyapaar-ledger` — consumes `FillReport`/`DomainEvent` for double-entry FIFO ledger.
