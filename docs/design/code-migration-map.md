# `vyapaar-core` — Implementation Record (honba-core → vyapaar-core)

> **Tier**: Tier 0 (Execute First — Zero Internal Dependencies)
> **Status**: Migration complete; plan 0001 (quant primitives) and the analytics
> gates merged.

This document records how the `vyapaar-core` Shared Kernel came to be and the
TDD/DDD corrections applied during the migration and subsequent plan work. The
workspace's `honba-core` source tree was the origin of most modules; the kernel
was re-implemented against the Tier 0 contract with the strict invariants below.

## Module Map & Provenance

| Module | Origin / Action |
|--------|-----------------|
| `lib.rs` | Re-exports every module; owns `SCHEMA_VERSION` and `API_VERSION`. |
| `types.rs` | Ported; added `Exchange::{Nfo, Cds}` and `Price::try_new` / `Quantity::try_new` rejecting negatives. |
| `ids.rs` | **NEW** — `ClientOrderId`, `AlgoId`, `AccountId`, `TransactionId` lifted from `honba-kokoro` so all crates share canonical IDs. |
| `error.rs` | Ported `CoreError` verbatim. |
| `instrument.rs` | Ported `Instrument`, `InstrumentKind`, `OptionType`, `PositionSide`, `TradeId`, `VenueOrderId`; added `InstrumentProfile`, `DatasetId`. |
| `market.rs` | Ported `Tick`, `DepthLevel`, `Bar`. |
| `order.rs` | Ported order enums and request/ack/fill types; v7 added `FillReport::fee`/`fee_breakdown` (`FeeBreakdown`). |
| `decision.rs` | Ported `Decision` with `#[serde(deny_unknown_fields)]` + `validate()`. |
| `events.rs` | Ported `DomainEvent`, `ExecutionEvent`, `Reason`, `TradingState`. |
| `calendar.rs` | Ported `SessionWindow`, `TradingCalendar`. |
| `sizing.rs` | Ported `SizingMethod`, `RiskBudget`, `PositionSizingRequest/Result`, `calculate_position_size`. |
| `ports.rs` | Ported all hexagonal port traits (`BarReader`, `TickReader`, `QuoteReader`, `DepthReader`, `MarketDataConnector`, `InstrumentMaster`, `ExecutionGateway`, `PositionSizer`). |
| `portfolio.rs` | Ported `PortfolioSnapshot` wire types. |
| `screener.rs` | Ported `ScreenerScanRequest`, `ScreenerRow`, `FilterOp`, `ScreenerFilterGroup`. |
| `signal.rs` | Ported `Signal`, `SignalDirection`, `SignalHorizon`, `Timeframe`. |
| `wire.rs` | Ported W3C `TraceContext`. |
| `quant.rs` | **NEW** (plan 0001) — canonical quant transform primitives. |
| `analytics/` | **NEW** — pure-`Decimal` statistical validation gates (`ValidationBlock` + 6 gates). |
| `schema/**` | Golden JSON vectors + compatibility snapshots (`domain`, `compatibility`, `golden`). |
| `tests/*.rs` | `core_tests.rs`, `golden_test.rs`, `proptest_invariants.rs`, `schema_compatibility_test.rs`, `quant_tests.rs`, `quant_proptest.rs`, `analytics_gates_tests.rs`. |

## Corrections & Decisions

- **Identifier unification**: aggregate IDs moved into the kernel (split-brain
  ubiquitous-language fix) so every crate shares one `ClientOrderId`/`AlgoId`/
  `AccountId`/`TransactionId`.
- **Exchange vocabulary**: `Nfo` and `Cds` added so downstream crates never
  redefine `Exchange`.
- **Wire-boundary validation**: `Price`/`Quantity` deserialize through a
  validating visitor — negative wire payloads are rejected in both debug and
  release builds.
- **TraceContext**: W3C `traceparent` serialization/parsing with zero external
  dependencies.
- **Zero-floats**: `rust_decimal` with the `maths` feature; `ln`/`sqrt` stay in
  pure `Decimal` (no `f64` intermediates).
- **Schema versioning**: `SCHEMA_VERSION = 8`; changes to any wire type bump it.

## Strict Invariants Checklist

- [x] **Zero-I/O**: no `tokio`, `std::fs`, `std::net`, or `Utc::now()` in `src/`.
- [x] **Zero-floats**: zero `f32`/`f64` fields in any financial struct.
- [x] **Zero inline tests**: no `#[cfg(test)]` / `#[test]` inside `src/`.
- [x] **WASM retargetable**: compiles to `wasm32-unknown-unknown`.

## Verification Commands

```bash
# Run from the repository root
make prod      # cargo build --workspace --profile prod
make test      # cargo test --workspace --profile prod
make check     # cargo check --workspace --profile prod
make clippy    # cargo clippy --workspace --all-targets --profile prod -- -D warnings
make fmt       # cargo fmt
```

## Remaining Work

See the [gaps](../gaps/README.md) (0001, tracked by
[plan 0002](../plans/0002-call-site-consolidation.md)) and
[plans](../plans/README.md) indexes.
