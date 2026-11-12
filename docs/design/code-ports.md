# `vyapaar-core` — Hexagonal Port Traits Reference

Documents `vyapaar-core/src/ports.rs`: object-safe hexagonal port traits
(interfaces only, no implementations). `vyapaar-core` is zero-I/O — these traits
are abstract contracts implemented in downstream crates and injected as
`Arc<dyn Trait>`.

## Port Errors

```rust
pub enum PortError {
    ConnectionFailed(String),  // connection lost / could not establish
    Timeout,                   // operation timed out
    NotFound(String),          // resource does not exist
    Unsupported(String),       // operation not supported
    CalculationFailed(String), // sizing/calculation failure
}
pub type PortResult<T> = Result<T, PortError>;
```

## Connector State

```rust
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ConnectorState {
    Disconnected,
    Connecting,
    Connected,
    Degraded { reason: String },
}
```

## Request / Subscription Types

| Type | Fields |
|------|--------|
| `BarRequest` | `instrument`, `timeframe: Timeframe`, `from`, `to` (`DateTime<Utc>`) |
| `TickRequest` | `instrument`, `from`, `to` |
| `SubscriptionParams` | `instruments: Vec<InstrumentId>`, `depth: bool` |
| `DatedInstrument` | `instrument: Instrument`, `as_of: DateTime<Utc>` (dated reference data — corporate actions / expiry rollovers) |
| `MarketEvent` | `Tick(Tick)`, `Bar(Bar)`, `ConnectorStateChanged(ConnectorState)` (tagged `kind`) |

## Market Data Ports

| Trait | Method | Purpose |
|-------|--------|---------|
| `BarReader` | `read_bars(&BarRequest) -> PortResult<Vec<Bar>>` | Read historical bars |
| `TickReader` | `read_ticks(&TickRequest) -> PortResult<Vec<Tick>>` | Read historical ticks |
| `QuoteReader` | `latest_quote(&InstrumentId) -> PortResult<Tick>` | Latest quote tick |
| `DepthReader` | `depth(&InstrumentId) -> PortResult<Vec<DepthLevel>>` | L2/L3 depth book |
| `MarketDataConnector` | `state()`, `subscribe(&SubscriptionParams)`, `poll() -> PortResult<Vec<MarketEvent>>` | Live market-data connection (WebSocket, replay feed) |

## Instrument & Execution Ports

| Trait | Method | Purpose |
|-------|--------|---------|
| `InstrumentMaster` | `lookup(&InstrumentId) -> PortResult<DatedInstrument>`, `search(&str) -> PortResult<Vec<InstrumentId>>` | Dated instrument reference data |
| `ExecutionGateway` | `submit(&OrderRequest) -> PortResult<OrderAck>`, `cancel(&str) -> PortResult<()>`, `modify(&ModifyRequest) -> PortResult<OrderAck>` | Order execution |

## Position Sizing Port

```rust
pub trait PositionSizer: Send + Sync {
    fn calculate_size(&self, &PositionSizingRequest) -> Result<PositionSizingResult, PortError>;
}
```

Decouples signal generation from sizing algorithms. Concrete sizing adapters
live in downstream bounded contexts; the kernel owns only the grammar
(`SizingMethod`, `RiskBudget`) and pure math (`calculate_position_size`).

## Re-Exports

- `SessionWindow`, `TradingCalendar` re-exported from `crate::calendar` (see
  [code-wire-contracts](code-wire-contracts.md)).

## Zero-I/O Purity

No trait performs I/O. Implementations in downstream crates handle the actual
network/disk access. Every trait is object-safe for `Arc<dyn Trait>` injection.

## Related

- `vyapaar-core/src/sizing.rs` — `PositionSizingRequest`, `PositionSizingResult`,
  `SizingMethod`, `RiskBudget` (see [code-sizing-analytics](code-sizing-analytics.md)).
- `vyapaar-core/src/market.rs` — `Tick`, `Bar`, `DepthLevel`.
- `vyapaar-core/src/order.rs` — `OrderRequest`, `ModifyRequest`, `OrderAck`.
