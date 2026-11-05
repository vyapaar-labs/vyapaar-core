//! # व्यापार・核 vyapaar-core
//!
//! Shared-kernel canonical types for the Vyapaar Indian equities trading platform.
//!
//! The kernel is deliberately slim: it owns only the ubiquitous language —
//! value objects, aggregate IDs, wire contracts, the closed [`Decision`]
//! grammar, and [`DomainEvent`]s. Bounded contexts own everything else.
//!
//! Provides:
//! - Canonical strongly-typed aggregate IDs ([`ClientOrderId`], [`AlgoId`], [`AccountId`], [`TransactionId`])
//! - Canonical [`Exchange`] and [`InstrumentId`], covering NSE, BSE, NFO, MCX, and CDS
//! - Exact decimal financial math ([`Price`], [`Quantity`], [`Money`])
//! - Canonical market data [`Tick`], [`DepthLevel`], [`Bar`]
//! - Canonical [`OrderRequest`], [`OrderAck`], [`FillReport`]
//! - Safe, closed, strictly-validated AI [`Decision`]
//! - Scrip-wise [`PriceBand`] limits for Indian market microstructure
//! - Canonical dynamic position sizing grammar ([`SizingMethod`], [`RiskBudget`], [`PositionSizingRequest`], [`PositionSizingResult`])
//! - Tactical DDD first-class aggregate events ([`DomainEvent`])
//! - Zero-dependency W3C distributed trace context carrier ([`TraceContext`])
//! - Abstract exchange trading calendar port ([`TradingCalendar`])

pub mod calendar;
pub mod decision;
pub mod error;
pub mod events;
pub mod ids;
pub mod instrument;
pub mod market;
pub mod order;
pub mod portfolio;
pub mod ports;
pub mod screener;
pub mod signal;
pub mod sizing;
pub mod types;
pub mod wire;

pub use calendar::{SessionWindow, TradingCalendar};
pub use decision::Decision;
pub use error::CoreError;
pub use events::{DomainEvent, ExecutionEvent, Reason, ReasonGroup, ReasonPolarity, TradingState};
pub use ids::{AccountId, AlgoId, ClientOrderId, TransactionId};
pub use instrument::{
    DatasetId, Instrument, InstrumentKind, InstrumentProfile, OptionType, PositionSide, TradeId,
    VenueOrderId,
};
pub use market::{Bar, DepthLevel, Tick};
pub use order::{
    CancelRequest, FeeBreakdown, FillReport, ModifyRequest, OrderAck, OrderEvent, OrderEventKind,
    OrderRequest, OrderSide, OrderStatus, OrderType, RejectReason, TimeInForce,
};
pub use portfolio::{
    PortfolioAccount, PortfolioGreeks, PortfolioHolding, PortfolioMode, PortfolioPosition,
    PortfolioSnapshot, PortfolioSource, PortfolioTotals,
};
pub use ports::{
    BarReader, BarRequest, ConnectorState, DatedInstrument, DepthReader, ExecutionGateway,
    InstrumentMaster, MarketDataConnector, MarketEvent, PortError, PortResult, PositionSizer,
    QuoteReader, SubscriptionParams, TickReader, TickRequest,
};
pub use screener::{
    FilterOp, MetricDefinition, MetricKeySpec, MetricPeriod, MetricRef, ScreenerFilterGroup,
    ScreenerFilterPredicate, ScreenerRow, ScreenerScanRequest, ScreenerScanResponse, SortSpec,
};
pub use signal::{
    Signal, SignalDirection, SignalHorizon, SignalScanRequest, SignalScanResponse, Timeframe,
};
pub use sizing::{
    PositionSizingRequest, PositionSizingResult, RiskBudget, SizingMethod, calculate_position_size,
};
pub use types::{
    CountryCode, Currency, Exchange, InstrumentId, MinorUnit, Money, Price, PriceBand, Quantity,
};
pub use wire::TraceContext;

/// Version of the JSON wire contract, owned here because this crate owns the
/// wire types.
///
/// The generated JSON Schema, the OpenAPI document, the MCP tool schemas and
/// the Python stubs all carry it; a change to any wire type bumps it.
///
/// v6: added instrument master data, portfolio snapshots, signals, the
/// screener, cross-subsystem domain events, and the `ports` module —
/// the real home for symbols that used to live in the now-empty
/// `vyapaar-ledger-{entities,messages,ports}` marker crates.
///
/// v7: added `FillReport::fee`/`fee_breakdown` (`FeeBreakdown`) — the
/// contract for per-fill transaction costs; the statutory calculation
/// itself lives in `vyapaar-market-finance::india::costs`.
///
/// v8: added dynamic position sizing contracts (`SizingMethod`, `RiskBudget`,
/// `PositionSizingRequest`, `PositionSizingResult`, `PositionSizer`), enriched
/// `Signal` with execution/stop levels, first-class tactical `DomainEvent`
/// aggregate envelope, and W3C `TraceContext` distributed observability carrier.
///
/// Migration note (Tier 0, `honba-core` → `vyapaar-core`, still wire-compatible
/// with v8): added the canonical aggregate IDs module (`ids`), `Exchange::{Nfo,
/// Cds}` variants, and hardened `Price`/`Quantity` deserialization to reject
/// negative values at the serde boundary. All v8 payloads deserialize
/// unchanged; the additions are strictly additive.
pub const SCHEMA_VERSION: u32 = 8;

/// Semantic version of the REST endpoint surface.
pub const API_VERSION: &str = "v1";

// Re-export foundational external crates for version consistency
pub use chrono;
pub use rust_decimal;
pub use uuid;
