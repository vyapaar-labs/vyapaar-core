# `vyapaar-core` — DDD Architecture & Shared Kernel Specification

## 1. Bounded Context & Responsibility
`vyapaar-core` is the **Shared Kernel (Tier 0)** of the Vyapaar Labs ecosystem. Every Rust crate and Python FFI boundary depends on `vyapaar-core`, while `vyapaar-core` depends on **zero internal crates** and performs **zero I/O**.

## 2. Tactical DDD Building Blocks

### 2.1 Strongly-Typed Identifiers (`src/ids.rs` & `src/types.rs`)
* `InstrumentId`: Canonical `(Exchange, symbol, Option<token>, Option<isin>)` with deterministic `Ord` for `BTreeMap` keys (`NSE:RELIANCE`).
* `ClientOrderId(pub String)`: Client-assigned idempotent order identifier.
* `AlgoId(pub String)`: SEBI Retail Algo Circular mandatory identifier (`^[A-Z0-9_]{4,32}$`).
* `AccountId(pub String)`: Trading account identifier.
* `TransactionId(pub String)`: Double-entry ledger journal transaction identifier.

### 2.2 Immutable Self-Validating Value Objects (`src/types.rs`)
* `Price(Decimal)`: Exact 128-bit decimal per-unit price. Rejects negative numbers upon construction (`Price::try_new`, `Price::from_rupees`) and JSON deserialization.
* `Quantity(Decimal)`: Exact 128-bit decimal share/lot quantity. Rejects negative numbers and validates lot multiples (`is_lot_multiple`).
* `Money { amount: Decimal, currency: Currency }`: Exact monetary value paired with `Currency` (`Inr`, `Usd`, `Eur`, `Gbp`) and `MinorUnit` (`paisa`/`paise`).
* `PriceBand { lower: Price, upper: Price, reference_price: Price, band_pct: Option<Decimal> }`: Scrip-wise NSE/BSE daily circuit limits (2%, 5%, 10%, 20%).

### 2.3 Canonical Market, Order & Portfolio Contracts
* `market.rs`: `Tick` (`Ltp`, `Quote`, `Depth`), `Bar` (OHLCV + turnover in `Decimal`), `DepthLevel`.
* `order.rs`: `OrderSide` (`Buy`, `Sell`), `OrderType` (`Market`, `Limit`, `StopLoss`, `StopLimit`), `TimeInForce` (`Day`, `Ioc`, `Gtc`), `OrderStatus`, `OrderRequest`, `ModifyRequest`, `CancelRequest`, `OrderAck`, `FillReport`, `FeeBreakdown`.
* `decision.rs`: Closed AI `Decision` enum (`Buy`, `Sell`, `Hold`, `CancelAll`) validated with `#[serde(deny_unknown_fields)]`.
* `events.rs`: Authoritative `DomainEvent` (`OrderPlaced`, `OrderModified`, `OrderCancelled`, `OrderFilled`, `RiskBreached`, `PositionLiquidated`, `MarginCallIssued`, `SizingCalculated`).
* `ports.rs`: Object-safe hexagonal port traits (`BarReader`, `TickReader`, `QuoteReader`, `DepthReader`, `MarketDataConnector`, `InstrumentMaster`, `ExecutionGateway`, `PositionSizer`, `TradingCalendar`).
