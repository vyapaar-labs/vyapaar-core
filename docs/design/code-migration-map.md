# `vyapaar-core` — Historical Migration Map (honba-core → vyapaar-core)

> **Tier**: Tier 0 (Execute First — Zero Internal Dependencies)
> **Source**: `/home/pmallapp/Devel/Finance/honba-workspace/honba-core`
> **Target**: `/home/pmallapp/Devel/Finance/vyapaar-labs/vyapaar-core`
> **Status**: Migration complete.

## File-by-File Migration Map

| Source Path (`honba-core/`) | Target Path (`vyapaar-core/`) | Action & DDD Refactoring Required |
| :--- | :--- | :--- |
| `Cargo.toml` | `Cargo.toml` | Keep `vyapaar-core` name; copy `[profile.*]` sections. |
| `Makefile` | `Makefile` | Port and replace `honba-core` with `vyapaar-core`. |
| `src/lib.rs` | `src/lib.rs` | Re-export all modules (`calendar`, `decision`, `error`, `events`, `ids`, `instrument`, `market`, `order`, `portfolio`, `ports`, `screener`, `signal`, `sizing`, `types`, `wire`). |
| *(from `honba-kokoro/src/repository.rs`)* | `src/ids.rs` **[NEW]** | Extract `ClientOrderId`, `AlgoId`, `AccountId`, `TransactionId` with `JsonSchema` derives into `vyapaar-core` so all crates share canonical IDs. |
| `src/types.rs` | `src/types.rs` | Add `Nfo` and `Cds` variants to `Exchange` (`Nse`, `Bse`, `Nfo`, `Mcx`, `Cds`). Add `Price::try_new` and `Quantity::try_new` rejecting negative `Decimal` values. |
| `src/error.rs` | `src/error.rs` | Port `CoreError` verbatim. |
| `src/instrument.rs` | `src/instrument.rs` | Port `Instrument`, `InstrumentKind`, `OptionType`, `PositionSide`, `TradeId`, `VenueOrderId`. |
| `src/market.rs` | `src/market.rs` | Port `Tick`, `DepthLevel`, `Bar`. |
| `src/order.rs` | `src/order.rs` | Port `OrderSide`, `OrderType`, `TimeInForce`, `OrderStatus`, `OrderRequest`, `ModifyRequest`, `CancelRequest`, `OrderAck`, `FillReport`, `FeeBreakdown`, `OrderEvent`. |
| `src/decision.rs` | `src/decision.rs` | Port `Decision` enum with `#[serde(deny_unknown_fields)]`. |
| `src/events.rs` | `src/events.rs` | Port `DomainEvent`, `ExecutionEvent`, `Reason`, `TradingState`. |
| `src/calendar.rs` | `src/calendar.rs` | Port `SessionWindow`, `TradingCalendar`. |
| `src/sizing.rs` | `src/sizing.rs` | Port `SizingMethod`, `RiskBudget`, `PositionSizingRequest`, `PositionSizingResult`, `calculate_position_size`. |
| `src/ports.rs` | `src/ports.rs` | Port all hexagonal port traits (`BarReader`, `TickReader`, `QuoteReader`, `DepthReader`, `MarketDataConnector`, `InstrumentMaster`, `ExecutionGateway`, `PositionSizer`). |
| `src/portfolio.rs` | `src/portfolio.rs` | Port `PortfolioSnapshot` wire types. |
| `src/screener.rs` | `src/screener.rs` | Port `ScreenerScanRequest`, `ScreenerRow`, `FilterOp`, `ScreenerFilterGroup`. |
| `src/signal.rs` | `src/signal.rs` | Port `Signal`, `SignalDirection`, `SignalHorizon`, `Timeframe`. |
| `src/wire.rs` | `src/wire.rs` | Port W3C `TraceContext`. |
| `schema/**` | `schema/**` | Copy golden JSON vectors and schema compatibility snapshots; update any `"honba"` strings if present. |
| `tests/*.rs` | `tests/*.rs` | Port `core_tests.rs`, `golden_test.rs`, `proptest_invariants.rs`, `schema_compatibility_test.rs`. Delete placeholder `tests/unit_tests.rs`. Replace `honba_core` with `vyapaar_core`. |

## Strict Invariants Checklist

- [x] **Zero-I/O**: No `tokio`, `std::fs`, `std::net`, or `Utc::now()` in `src/` (except optional helper constructors if explicitly documented; prefer explicit `timestamp: DateTime<Utc>`).
- [x] **Zero-Floats**: Zero `f32` or `f64` fields in any financial struct.
- [x] **Zero Inline Tests**: `grep -rn "#\[cfg(test)\]" src/` and `grep -rn "#\[test\]" src/` must return 0 matches.

## Verification Commands

```bash
cd /home/pmallapp/Devel/Finance/vyapaar-labs/vyapaar-core
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```
