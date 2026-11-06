# 0001 — Canonical Quant Transform Primitives (Returns, Volatility, Rolling Stats)

> **Status**: Proposed — not yet implemented.
> **Target Crates**: vyapaar-core
> **Depends on reading**:
> - vyapaar-agent/docs/plans/0001-mcp-gateway-discovery-and-quantitative-engines.md

---

## 0. Executive Summary & Objective

Fincept's "Quant" toolset (series algebra, returns, realized/EWMA volatility, Sharpe, drawdown, correlation/beta) has no single canonical home in Vyapaar today — pieces are scattered across `vyapaar-market-indicators` (technical indicators), `vyapaar-agent-analytics` (trade/equity stats), and `vyapaar-runtime-backtest` (basket backtests), each with its own ad-hoc return/volatility math. This closes that gap narrowly: lift only the truly primitive, zero-I/O numeric transforms into `vyapaar-core` itself, next to the existing value objects, since `vyapaar-core` already has zero dependencies on other crates and these are pure math over its own `Price`/`Quantity`/`Decimal` types. This is deliberately not the full Fincept Quant toolset — basket backtests and epidemic (SIR/SEIR) models stay exactly where they are.

Note explicitly: `vyapaar-agent/docs/plans/0001-mcp-gateway-discovery-and-quantitative-engines.md` already places the `VyapaarResultEnvelope` (`vyapaar/result/v1`) in `vyapaar-agent-core::result` — that decision is not revisited here. This doc covers only the numeric primitives, not how results get wrapped for the MCP gateway.

---

## 1. Context: Why `vyapaar-core`, Why Only These Primitives

`vyapaar-core` is the shared kernel (`src/types.rs`, `src/market.rs`, `src/portfolio.rs`, `src/decision.rs`, `src/signal.rs`, `src/screener.rs`) with the strict zero-float / `Decimal` discipline enforced workspace-wide. Return and volatility math is some of the most duplicated logic in quant codebases precisely because every downstream crate needs it and none of them should own it:

- `vyapaar-market-indicators` needs returns/volatility as inputs to several of its 100+ streaming indicators, and currently re-derives them locally.
- `vyapaar-agent-analytics` needs the same math for trade/equity-curve statistics (Sharpe, drawdown) and also re-derives it.
- `vyapaar-runtime-backtest` needs it for performance reporting on basket runs.

None of these crates should be the canonical owner, because none of them is upstream of the others — but `vyapaar-core` is upstream of all three. Putting the primitives there means every consumer gets identical, decimal-exact arithmetic with no risk of drift between a backtest's Sharpe ratio and a live dashboard's Sharpe ratio computed from a slightly different formula.

Explicitly out of scope for this doc (stay in their current homes): basket-level backtest orchestration (`vyapaar-runtime-backtest`), any indicator requiring streaming/stateful update semantics beyond a rolling window (`vyapaar-market-indicators` keeps its own indicator trait machinery), epidemic/SIR-SEIR-style models, and anything requiring I/O or async.

---

## 2. Component Specifications

New module: `vyapaar-core/src/quant.rs`, exported from `lib.rs` alongside the existing `types`, `market`, `portfolio`, `decision`, `signal`, `screener` modules.

Proposed surface (free functions operating on `&[Decimal]` / `&[Price]` slices, plus a thin `QuantSeries` newtype wrapper for ergonomic chaining):

- `QuantSeries(Vec<Decimal>)` — a newtype wrapper around an ordered `Decimal` series (e.g. a price or equity-curve series), with `From<&[Price]>` / `From<&[Money]>` conversions so callers don't hand-unwrap `vyapaar-core` value objects.
- `simple_returns(series: &QuantSeries) -> QuantSeries` — period-over-period simple returns `(p[t] - p[t-1]) / p[t-1]`.
- `log_returns(series: &QuantSeries) -> QuantSeries` — log returns; since `Decimal` has no native `ln`, this phase documents the precision approach (e.g. a `Decimal`-friendly series expansion or an explicitly-flagged `f64` boundary conversion solely for the log step, isolated and justified as an exception to the zero-floats policy — final precision approach is an implementation-time decision, not fixed by this plan).
- `realized_volatility(returns: &QuantSeries, annualization_factor: Decimal) -> Decimal` — sample stdev of a returns series scaled by `annualization_factor.sqrt()`.
- `ewma_volatility(returns: &QuantSeries, lambda: Decimal) -> Decimal` — exponentially-weighted moving variance/volatility (RiskMetrics-style decay).
- `rolling_mean(series: &QuantSeries, window: usize) -> QuantSeries` and `rolling_stdev(series: &QuantSeries, window: usize) -> QuantSeries` — fixed-window rolling statistics, the building blocks both indicators and analytics need.
- `sharpe_ratio(returns: &QuantSeries, risk_free_rate: Decimal, annualization_factor: Decimal) -> Decimal` and `max_drawdown(equity_curve: &QuantSeries) -> Decimal` — the two most commonly re-derived summary statistics.
- `correlation(a: &QuantSeries, b: &QuantSeries) -> Decimal` and `beta(asset_returns: &QuantSeries, benchmark_returns: &QuantSeries) -> Decimal`.

All functions are pure, zero-I/O, deterministic, and take/return owned or borrowed `Decimal`-backed types only — no network, no filesystem, no clock queries, matching the zero-I/O domain purity invariant already enforced in `vyapaar-core`.

Error handling: a `QuantError` enum (`InsufficientData { needed: usize, got: usize }`, `DivisionByZero`, `WindowLargerThanSeries`) returned via `Result`, consistent with existing `vyapaar-core` error patterns (see `decision.rs`/`signal.rs` for the existing style to match).

---

## 3. Implementation Phases

### Phase 1: Primitives in `vyapaar-core`
- Add `src/quant.rs` with the `QuantSeries` newtype and the function list above.
- Property-test the mathematical identities that must hold regardless of input (e.g. `simple_returns` of a constant series is all-zero; `max_drawdown` of a monotonically increasing series is zero; `correlation(a, a) == 1`).
- Golden-vector tests cross-checking a handful of hand-computed Sharpe/drawdown/volatility values.
- No changes to any other crate in this phase.

### Phase 2: Consolidation of existing call sites (flagged as future work, not executed here)
- `vyapaar-market-indicators` re-exports/delegates its return- and volatility-dependent indicators to `vyapaar_core::quant` instead of maintaining parallel math.
- `vyapaar-agent-analytics` delegates its Sharpe/drawdown/correlation trade-and-equity-curve statistics to the same primitives.
- This phase is explicitly a separate, future PR — this doc only establishes the canonical home; it does not perform the call-site migration.
