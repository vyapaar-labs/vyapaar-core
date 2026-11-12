# `vyapaar-core` — DDD Architecture & Shared Kernel Specification

## 1. Bounded Context & Responsibility

`vyapaar-core` is the **Shared Kernel (Tier 0)** of the Vyapaar Labs
ecosystem. Every Rust crate and the Python FFI boundary (`vyapaar-ffi`) depend
on `vyapaar-core`, while `vyapaar-core` depends on **zero internal crates** and
performs **zero I/O** (no network, no filesystem, no clock queries — timestamps
are explicit `DateTime<Utc>` parameters).

The kernel owns the ubiquitous language:

- Strongly-typed aggregate IDs — `ClientOrderId`, `AlgoId`, `AccountId`, `TransactionId`.
- Exact-decimal financial value objects — `Price`, `Quantity`, `Money`, `Currency`, `MinorUnit`.
- Canonical instrument, market, and order contracts — `InstrumentId`, `Tick`, `Bar`, `OrderRequest`, `OrderAck`, `FillReport`.
- The closed AI `Decision` grammar and authoritative `DomainEvent`s.
- Hexagonal port traits, the trading calendar, dynamic position sizing grammar, and the statistical validation gates.
- Single source of truth for wire contracts: every wire type derives `JsonSchema`; OpenAPI/JSON/MCP schemas are generated from them (`make codegen`).

## 2. Strict Invariants

| Invariant | Enforcement |
|-----------|-------------|
| **Zero-I/O domain purity** | No `tokio`, `std::fs`, `std::net`, or `SystemTime`/`Utc::now()` in `src/`; all transitions take explicit `DateTime<Utc>`. |
| **Zero-floats policy** | No `f32`/`f64` anywhere in `src/`; all financial math is `rust_decimal::Decimal` (with the `maths` feature — `ln`/`sqrt` computed in pure `Decimal`). |
| **Zero inline tests** | No `#[cfg(test)]` / `#[test]` inside `src/`; all tests live in `tests/` parallel to `src/` (parallel test directory rule). |
| **Wire-contract generation** | All wire types derive `JsonSchema` via `schemars`; generated schema files are never hand-edited. |
| **Retargetability** | Compiles to `wasm32-unknown-unknown` and native `x86_64` / `aarch64`; no OS-specific dependencies. |

## 3. Module Map

```
vyapaar-core/src/
├── lib.rs            # Re-exports + SCHEMA_VERSION (v8) + API_VERSION (v1)
├── error.rs          # CoreError taxonomy
├── ids.rs            # ClientOrderId, AlgoId, AccountId, TransactionId
├── types.rs          # Exchange, InstrumentId, Price, Quantity, Money, PriceBand, Currency, CountryCode, MinorUnit
├── instrument.rs     # Instrument / InstrumentProfile, InstrumentKind, OptionType, PositionSide, TradeId, VenueOrderId, DatasetId
├── market.rs         # Tick (Ltp | Quote | Depth), DepthLevel, Bar
├── order.rs          # OrderSide, OrderType, TimeInForce, OrderStatus, OrderRequest, ModifyRequest, CancelRequest, OrderAck, FillReport, FeeBreakdown, RejectReason, OrderEvent, OrderEventKind
├── decision.rs       # Closed Decision enum + validate()/from_json_str()
├── events.rs         # DomainEvent, ExecutionEvent, TradingState, Reason, ReasonGroup, ReasonPolarity
├── wire.rs           # TraceContext (W3C traceparent)
├── calendar.rs       # SessionWindow + TradingCalendar port
├── portfolio.rs      # PortfolioSnapshot, PortfolioPosition, PortfolioHolding, PortfolioGreeks, PortfolioTotals, PortfolioAccount
├── screener.rs       # ScreenerScanRequest/Response, FilterOp, MetricKeySpec, ScreenerRow, SortSpec
├── signal.rs         # Signal, SignalDirection, SignalHorizon, Timeframe, SignalScanRequest/Response
├── ports.rs          # Hexagonal port traits (BarReader, TickReader, ..., PositionSizer) + PortError/PortResult
├── sizing.rs         # SizingMethod, RiskBudget, PositionSizingRequest/Result, calculate_position_size
├── quant.rs          # QuantSeries + canonical quant transforms (plan 0001)
└── analytics/        # ValidationBlock + 6 statistical gates (DSR, PBO, WFE, holdout, Monte Carlo, cost stress)
```

Reference docs:

| Module | Doc |
|--------|-----|
| `types.rs` | [code-types](code-types.md) |
| `ids.rs`, `error.rs` | [code-ids](code-ids.md) |
| `instrument.rs`, `market.rs`, `order.rs`, `portfolio.rs`, `screener.rs`, `signal.rs`, `calendar.rs` | [code-wire-contracts](code-wire-contracts.md) |
| `decision.rs` | [code-decision](code-decision.md) |
| `events.rs`, `wire.rs` | [code-events](code-events.md) |
| `ports.rs` | [code-ports](code-ports.md) |
| `quant.rs` | [code-quant](code-quant.md) |
| `sizing.rs`, `analytics/` | [code-sizing-analytics](code-sizing-analytics.md) |
| Tier 0 implementation record | [code-migration-map](code-migration-map.md) |

## 4. Schema & Versioning

- `SCHEMA_VERSION: u32 = 8` — the JSON wire contract version owned by this
  crate (see `lib.rs` for the v6/v7/v8 changelog).
- `API_VERSION: &str = "v1"` — REST endpoint surface version.
- `schema/domain/domain_schema.json` — the generated canonical JSON Schema.
- `schema/compatibility/v{N}_domain_schema.json` — compatibility snapshots.
- `schema/golden/*.json` — golden serialization vectors (`tick_ltp`,
  `tick_quote`, `price_band`, `decision_*`).

## 5. Implementation Status

### Implemented & Verified

| Area | Module(s) | Tests |
|------|-----------|-------|
| Value objects + identifiers | `types.rs`, `ids.rs`, `error.rs` | `tests/core_tests.rs`, `tests/proptest_invariants.rs` |
| Market / order / instrument wire contracts | `market.rs`, `order.rs`, `instrument.rs` | `tests/core_tests.rs`, `tests/golden_test.rs` |
| Closed `Decision` grammar | `decision.rs` | `tests/core_tests.rs`, `tests/golden_test.rs` |
| Domain events + trace context | `events.rs`, `wire.rs` | `tests/core_tests.rs` |
| Ports + calendar | `ports.rs`, `calendar.rs` | `tests/core_tests.rs` |
| Sizing grammar | `sizing.rs` | `tests/core_tests.rs` |
| Quant transforms (plan 0001) | `quant.rs` | `tests/quant_tests.rs` (33), `tests/quant_proptest.rs` |
| Statistical validation gates | `analytics/` | `tests/analytics_gates_tests.rs` |
| Screener / signal / portfolio wire types | `screener.rs`, `signal.rs`, `portfolio.rs` | `tests/core_tests.rs` |
| Schema compatibility & golden vectors | `schema/` | `tests/schema_compatibility_test.rs`, `tests/golden_test.rs` |

### Verification Gate

```bash
make prod      # cargo build --workspace --profile prod
make test      # cargo test --workspace --profile prod
make check     # cargo check --workspace --profile prod
make clippy    # cargo clippy --workspace --all-targets --profile prod -- -D warnings
make fmt       # cargo fmt
```

### What Remains

- **Call-site consolidation** — downstream crates (`vyapaar-market-indicators`,
  `vyapaar-agent-analytics`) still re-derive returns/vol/Sharpe/drawdown locally
  instead of delegating to `vyapaar_core::quant`; tracked by
  [gap 0001](../gaps/0001-call-site-consolidation.md) /
  [plan 0002](../plans/0002-call-site-consolidation.md).

See the [gaps](../gaps/README.md) and [plans](../plans/README.md) indexes.
