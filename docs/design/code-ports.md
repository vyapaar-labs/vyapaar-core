# `vyapaar-core` — Hexagonal Port Traits Reference

Documents `vyapaar-core/src/ports.rs`: object-safe hexagonal port traits (interfaces
only, no implementations). `vyapaar-core` is zero-I/O — these traits are abstract
contracts implemented in downstream crates and injected as `Arc<dyn Trait>`.

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
pub enum ConnectorState {
    Disconnected,
    Connecting,
    Connected,
    Degraded { reason: String },
}
```

## Market Data Ports

| Trait | Method | Purpose |
|-------|--------|---------|
| `BarReader` | `read_bars(&BarRequest) -> PortResult<Vec<Bar>>` | Read historical bars |
| `TickReader` | `read_ticks(&TickRequest) -> PortResult<Vec<Tick>>` | Read historical ticks |
| `QuoteReader` | `latest_quote(&InstrumentId) -> PortResult<Tick>` | Latest quote tick |
| `DepthReader` | `depth(&InstrumentId) -> PortResult<Vec<DepthLevel>>` | L2/L3 depth book |
| `MarketDataConnector` | `state()`, `subscribe()`, `poll()` | Live market-data connection |

## Instrument & Execution Ports

| Trait | Method | Purpose |
|-------|--------|---------|
| `InstrumentMaster` | `lookup(&InstrumentId)`, `search(&str)` | Dated instrument reference data |
| `ExecutionGateway` | `submit(&OrderRequest)`, `cancel(&str)`, `modify(&ModifyRequest)` | Order execution |

## Position Sizing Port

```rust
pub trait PositionSizer: Send + Sync {
    fn calculate_size(&self, &PositionSizingRequest) -> Result<PositionSizingResult, PortError>;
}
```

Decouples signal generation from sizing algorithms. Concrete sizing adapters live
in downstream bounded contexts; the kernel owns only the grammar
(`SizingMethod`, `RiskBudget`) and pure math.

## Request/Response Types

| Type | Fields |
|------|--------|
| `BarRequest` | `instrument`, `timeframe`, `from`, `to` |
| `TickRequest` | `instrument`, `from`, `to` |
| `SubscriptionParams` | `instruments`, `depth` |
| `DatedInstrument` | `instrument`, `as_of` |
| `MarketEvent` | `Tick(Tick)`, `Bar(Bar)`, `ConnectorStateChanged(ConnectorState)` |

## Re-Exports

- `SessionWindow`, `TradingCalendar` from `crate::calendar`.

## Zero-I/O Purity

No trait performs I/O. Implementations in downstream crates (`vyapaar-market-feed`,
`vyapaar-exec-zerodha`, etc.) handle the actual network/disk access.

## Related

- `vyapaar-core/src/sizing.rs` — `PositionSizingRequest`, `PositionSizingResult`, `SizingMethod`, `RiskBudget`.
- `vyapaar-core/src/market.rs` — `Tick`, `Bar`, `DepthLevel`.
- `vyapaar-core/src/order.rs` — `OrderRequest`, `ModifyRequest`, `OrderAck`.
