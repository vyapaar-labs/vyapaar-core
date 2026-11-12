# `vyapaar-core` — Wire Contracts Reference (Instrument, Market, Order, Portfolio, Screener, Signal, Calendar)

Documents the canonical wire-contract modules not covered by dedicated docs:
`instrument.rs`, `market.rs`, `order.rs`, `portfolio.rs`, `screener.rs`,
`signal.rs`, and `calendar.rs`. Every type derives `JsonSchema` and
`Serialize`/`Deserialize` — these are the single source of truth for the JSON
wire format.

## Instrument Reference Data (`instrument.rs`)

Reference/master data for a tradable instrument — distinct from
`types::InstrumentId`, which is only the lookup key.

- `InstrumentProfile` — type alias for `Instrument`.
- `InstrumentKind` — `Equity`, `Future`, `Option`, `Index` (snake_case).
- `OptionType` — `Call`, `Put`.
- `PositionSide` — `Long`, `Short` (UPPERCASE, with `.opposite()`); distinct
  from `order::OrderSide` (the side of a single order/fill vs. a net position).
- `Instrument` — full profile: `id: InstrumentId`, `kind`, `currency`,
  `country`, `lot_size: Quantity`, `tick_size: Decimal`, optional `expiry`,
  `strike`, `option_type`, `underlying`. Constructors include `Instrument::equity(...)`.
- `TradeId` / `VenueOrderId` — string identifiers for a trade and a
  broker-assigned venue order id (re-exported from `lib.rs`).

## Market Data (`market.rs`)

### `DepthLevel`

```rust
pub struct DepthLevel {
    pub price: Price,
    pub quantity: Quantity,
    pub orders_count: Option<u32>,
}
```

### `Tick`

Tagged enum normalized across Indian broker feed shapes
(`#[serde(tag = "type", content = "data", rename_all = "snake_case")]`):

| Variant | Payload |
|---------|---------|
| `Ltp` | `instrument`, `price`, `volume`, `exchange_ts`, `receive_ts` |
| `Quote` | `instrument`, `bid`/`ask` + quantities, optional `last_price`, `exchange_ts`, `receive_ts`, optional `band: PriceBand` |
| `Depth` | `instrument`, `bids`/`asks: Vec<DepthLevel>`, optional `last_price`, `exchange_ts`, `receive_ts` |

Convenience accessors: `instrument()`, `receive_ts()`, `last_price()` (best
available price: explicit `last_price`, else bid, else best bid level).

### `Bar`

Standard OHLCV bar over `InstrumentId` with `open`/`high`/`low`/`close` as
`Price` (plus turnover fields), exact `Decimal` throughout.

## Order Contracts (`order.rs`)

### Enums

| Enum | Variants | Serde |
|------|----------|-------|
| `OrderSide` | `Buy`, `Sell` (`.opposite()`) | UPPERCASE |
| `OrderType` | `Market`, `Limit(Price)`, `StopLoss(Price)`, `StopLimit { trigger, limit }` | tagged |
| `TimeInForce` | `Day` (default), `Ioc`, `Gtc` | UPPERCASE |
| `OrderStatus` | `PendingNew`, `New`, `PartiallyFilled`, `Filled`, `Cancelled`, `Rejected`, `Expired`; `is_terminal()`, `is_active()` | snake_case |
| `RejectReason` | `ExchangeRejected(String)`, `Expired`, `InsufficientMargin` | tagged |

`OrderType` helpers: `limit_price()` and `trigger_price()` extract the relevant
price component.

### Request / Ack / Report types

| Type | Purpose |
|------|---------|
| `OrderRequest` | Place-order request: `client_order_id: Uuid`, `instrument`, `side`, `order_type`, `quantity`, `tif`, mandatory `algo_id` (SEBI Retail Algo Circular), optional `tag`, `client_id`, `correlation_id`, `trace_context`. Constructors: `OrderRequest::new(...)` (random v4 UUID) and `OrderRequest::with_id(...)` (deterministic for replay). Builder: `with_client_id`, `with_correlation_id`, `with_trace_context`. |
| `ModifyRequest` | Amend an open order: `new_qty` / `new_price` (None leaves unchanged), `algo_id`, `client_id`. |
| `CancelRequest` | Cancel an order: `instrument`, `client_order_id: Uuid`, optional `algo_id`, `client_id`. |
| `OrderAck` | Broker ack: `order_id`, `client_order_id`, optional `exchange_order_id`, `status`, `timestamp`, optional `correlation_id` / `client_id` / `trace_context`. |
| `FillReport` | Execution confirmation: `fill_id`, `order_id`, `client_order_id`, `instrument`, `side`, `price`, `quantity`, `algo_id`, `timestamp`, `account_id`, `fee: Money` (defaults `zero_inr`), optional `fee_breakdown`, `trace_context`. Builders: `with_fee`, `with_fee_breakdown` (sets `fee` to the breakdown total), `with_account_id`. |
| `FeeBreakdown` | Itemized Indian equity/F&O transaction costs: `brokerage`, `stt`, `exchange_charges`, `gst`, `stamp_duty`, `sebi_charges` (all `Money`); `total()` sums them. The statutory calculation itself lives in `vyapaar-market-finance::india::costs` — the kernel owns only the wire shape. |

### `OrderEvent` / `OrderEventKind`

Tag-only and full lifecycle stream types (snake_case):

| `OrderEventKind` | `OrderEvent` |
|------------------|--------------|
| `Acked` | `Acked(OrderAck)` |
| `Filled` | `Filled(Box<FillReport>)` |
| `Rejected` | `Rejected { order_id, reason: RejectReason }` |
| `Cancelled` | `Cancelled { order_id }` |

`OrderEvent::kind()` returns the tag-only kind for filtering/logging.

## Portfolio Snapshots (`portfolio.rs`)

Wire contracts crossing the Heart (`vyapaar-ledger`) → House (`vyapaar-runtime`)
→ Face (`vyapaar-ui`) boundary:

| Type | Fields |
|------|--------|
| `PortfolioMode` | `Live`, `Paper`, `Backtest` |
| `PortfolioSource` | `Broker`, `Simulated` |
| `PortfolioPosition` | `instrument`, `side: PositionSide`, `quantity`, `avg_price`, `unrealized_pnl`, `realized_pnl` |
| `PortfolioHolding` | `instrument`, `quantity`, `avg_price`, `current_value` |
| `PortfolioGreeks` | `delta`, `gamma`, `theta`, `vega` (all `Decimal`) |
| `PortfolioTotals` | Account-level cash and margin totals |
| `PortfolioAccount` | Account-level aggregate |
| `PortfolioSnapshot` | The full snapshot envelope |

## Screener (`screener.rs`)

- `MetricPeriod` — type alias reusing `signal::Timeframe` (a metric's period and
  a bar's interval are the same concept).
- `FilterOp` — `Eq`, `Ne`, `Gt`, `Gte`, `Lt`, `Lte`, `Between`.
- `MetricKeySpec { metric, period }`, `MetricRef(MetricKeySpec)`, `MetricDefinition`.
- `ScreenerFilterPredicate`, `ScreenerFilterGroup`, `SortSpec`.
- `ScreenerScanRequest`, `ScreenerScanResponse`, `ScreenerRow`.

## Signals (`signal.rs`)

- `Timeframe` — `M1`, `M5`, `M15`, `M30`, `H1`, `D1`, `W1`.
- `SignalDirection` — `Bullish`, `Bearish`, `Neutral`.
- `SignalHorizon` — `Intraday`, `Swing`, `Positional`.
- `Signal` — `instrument`, `direction`, `horizon`, `timeframe`, `strength` (in
  `[0, 1]`), `generated_by`, `timestamp` (plus optional execution/stop levels).
- `SignalScanRequest`, `SignalScanResponse`.

## Trading Calendar (`calendar.rs`)

- `SessionWindow { open, close }` — `NaiveTime` session bounds with
  `SessionWindow::new(...)` and `contains(time)`.
- `TradingCalendar` — object-safe (`Arc<dyn TradingCalendar>`) abstract port:
  `is_trading_day`, `is_holiday`, `session_for_date`, `holidays`,
  `timezone_offset`, with default methods `is_market_open(utc_time)` and
  `next_trading_day(date)`. Zero-I/O: calendar state is in-memory and
  deterministic; concrete holiday tables live downstream.

## Zero-Floats & Zero-I/O

All prices, quantities, and amounts are `rust_decimal::Decimal`; all timestamps
are explicit `DateTime<Utc>` fields. None of these modules performs I/O.

## Related

- `vyapaar-core/src/events.rs` / `wire.rs` — see [code-events](code-events.md)
  (`DomainEvent`, `ExecutionEvent`, `TraceContext`).
- `vyapaar-core/src/ports.rs` — the hexagonal ports that consume these
  contracts — see [code-ports](code-ports.md).
