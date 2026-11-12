# `vyapaar-core` — Domain Events & Trace Context Reference

Documents `vyapaar-core/src/events.rs` and `vyapaar-core/src/wire.rs`: the
authoritative cross-subsystem domain events and the W3C trace context carrier.

## `DomainEvent` (`events.rs`)

Authoritative platform state-transition events designed for distributed audit
logging, event-sourced ledger reconstruction, and asynchronous
cross-bounded-context bus delivery. Immutable; tagged
(`#[serde(tag = "type", rename_all = "snake_case")]`); every variant carries an
explicit `timestamp: DateTime<Utc>` and an optional `trace_context`.

| Variant | Purpose |
|---------|---------|
| `OrderPlaced` | Client order dispatched: `order_id`, `client_order_id: Uuid`, `instrument`, `algo_id`, `side`, `quantity`, optional `price`. |
| `OrderModified` | In-flight amendment: original vs revised `quantity` and `price`. |
| `OrderCancelled` | Withdrawal: optional `algo_id`, `source` (e.g. `user`, `rms`, `market_close`), `reason`. |
| `OrderFilled` | Fill execution: `fill: Box<FillReport>`. |
| `RiskBreached` | Pre-trade gate denial: `rule_id`, `attempted_value`, `threshold_limit`, `message`. |
| `PositionLiquidated` | Emergency square-off / MIS auto-liquidation. |
| `MarginCallIssued` | Intraday margin shortfall: `required_maintenance_equity`, `current_equity`, `shortfall` (all `Money`). |
| `SizingCalculated` | Sizing determination: `request: Box<PositionSizingRequest>`, `result: PositionSizingResult`. |

Accessors: `DomainEvent::timestamp()` and `DomainEvent::trace_context()`.

## `ExecutionEvent` (`events.rs`)

Wraps `OrderEvent` (post-submission order lifecycle) with the pre-trade risk-gate
outcome:

| Variant | Meaning |
|---------|---------|
| `Submitted { order_id, instrument }` | Order request submitted for risk evaluation. |
| `RiskRefused { order_id, reasons: Vec<Reason> }` | Risk gate refused before the order reached the exchange. |
| `Order(OrderEvent)` | Post-submission order lifecycle event. |

A risk refusal never produces an `OrderEvent` — it is refused before reaching
that scope.

## `Reason` / `ReasonGroup` / `ReasonPolarity` / `TradingState` (`events.rs`)

- `ReasonGroup` — `Risk`, `Signal`, `Execution`, `Compliance` (originating subsystem).
- `ReasonPolarity` — `Positive`, `Negative`, `Neutral`.
- `Reason` — `group`, `polarity`, `code` (machine-stable, e.g. `"otr_breached"`), `message`.
- `TradingState` — `Active`, `Halted { reason }`, `Paused { resume_at }`.

## `TraceContext` (`wire.rs`)

Zero-dependency W3C Trace Context Level 1 carrier (`traceparent` header):
`trace_id: [u8; 16]`, `span_id: [u8; 8]`, `trace_flags: u8`. Derives
`Serialize`/`Deserialize` and `JsonSchema` (it rides on wire payloads such as
`DomainEvent`), but adds **no external dependencies**.

- `TraceContext::new(...)` / `new_root(...)` (sampled) / `child_span(...)`.
- `is_sampled()` — bit 0 of `trace_flags` (`SAMPLED = 0x01`).
- `to_traceparent()` — formats `00-{trace_id_hex}-{span_id_hex}-{trace_flags_hex}`.
- `from_traceparent(header)` — strict W3C parser (version `00`, non-zero
  trace/span ids); invalid headers yield `CoreError::Validation`.

## Schema Versioning

`SCHEMA_VERSION: u32 = 8` (in `lib.rs`):

- v6: instrument master data, portfolio snapshots, signals, screener, domain
  events, `ports`.
- v7: `FillReport::fee` / `fee_breakdown` (`FeeBreakdown`).
- v8: dynamic position sizing contracts, enriched `Signal`, `DomainEvent`
  aggregate envelope, `TraceContext`.

`API_VERSION: &str = "v1"` — REST endpoint surface version.

## Zero-I/O Purity

All event types are pure data — no network, filesystem, or clock queries.
Timestamps are explicit `DateTime<Utc>` parameters, never `SystemTime::now()`.

## Related

- `vyapaar-core/src/order.rs` — `OrderEvent`/`OrderEventKind` and `FillReport`
  (see [code-wire-contracts](code-wire-contracts.md)).
- `vyapaar-core/src/sizing.rs` — `PositionSizingRequest`/`PositionSizingResult`.
- `vyapaar-agent-core/src/decision.rs` — `DecisionParser` validates
  `vyapaar_core::Decision` before the risk gate.
- `vyapaar-ledger` — consumes `FillReport`/`DomainEvent` for the double-entry
  FIFO ledger.
