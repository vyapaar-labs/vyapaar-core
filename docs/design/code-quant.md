# `vyapaar-core` — Canonical Quant Transform Primitives Reference

Documents `vyapaar-core/src/quant.rs`: the single canonical home for pure,
zero-I/O numeric transforms over exact `Decimal` values. Downstream crates
(`vyapaar-market-indicators`, `vyapaar-agent-analytics`,
`vyapaar-runtime-backtest`, and the future `vyapaar-ledger-allocation`) should
delegate to these rather than re-deriving locally — tracked by
[gap 0001](../gaps/0001-call-site-consolidation.md) /
[plan 0002](../plans/0002-call-site-consolidation.md).

## `QuantSeries`

```rust
pub struct QuantSeries(Vec<Decimal>);
```

An ordered series of exact decimal values (price path, equity curve, or returns
series). Thin newtype over `Vec<Decimal>` with conversions from `&[Price]`,
`&[Money]`, `Vec<Decimal>`, and `&[Decimal]`.

## Transform Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `simple_returns` | `(&QuantSeries) -> Result<QuantSeries>` | `(p[t] - p[t-1]) / p[t-1]`, n-1 outputs |
| `log_returns` | `(&QuantSeries) -> Result<QuantSeries>` | `ln(p[t] / p[t-1])`, n-1 outputs |
| `realized_volatility` | `(&QuantSeries, Decimal) -> Result<Decimal>` | Sample stdev × sqrt(annualization) |
| `ewma_volatility` | `(&QuantSeries, Decimal) -> Result<Decimal>` | RiskMetrics EWMA, lambda in (0,1) |
| `rolling_mean` | `(&QuantSeries, usize) -> Result<QuantSeries>` | Fixed-window SMA |
| `rolling_stdev` | `(&QuantSeries, usize) -> Result<QuantSeries>` | Fixed-window sample stdev |
| `sharpe_ratio` | `(&QuantSeries, Decimal, Decimal) -> Result<Decimal>` | Annualized Sharpe |
| `max_drawdown` | `(&QuantSeries) -> Result<Decimal>` | Max peak-to-trough decline |
| `correlation` | `(&QuantSeries, &QuantSeries) -> Result<Decimal>` | Pearson correlation |
| `beta` | `(&QuantSeries, &QuantSeries) -> Result<Decimal>` | Asset beta vs benchmark |
| `historical_var` | `(&QuantSeries, Decimal) -> Result<Decimal>` | Empirical VaR (nearest-rank) |
| `historical_cvar` | `(&QuantSeries, Decimal) -> Result<Decimal>` | Empirical CVaR / Expected Shortfall |
| `tracking_error` | `(&QuantSeries, &QuantSeries, Decimal) -> Result<Decimal>` | Vol of return differences |

All transforms are exported from `lib.rs`.

## `QuantError`

| Variant | Trigger |
|---------|---------|
| `InsufficientData { needed, got }` | Not enough observations |
| `DivisionByZero` | Zero prior price, zero variance, zero peak |
| `WindowLargerThanSeries { window, len }` | Rolling window > series length |
| `InvalidArgument(&'static str)` | Bad argument (zero window, bad lambda, out-of-range annualization, mismatched lengths) |

## Zero-Floats Note

All math is pure `Decimal`. `log_returns` uses `rust_decimal`'s `maths` feature
(`MathematicalOps::ln` / `MathematicalOps::sqrt`), computed in pure `Decimal`
(192-bit `DecWide` intermediates, no `f64`). There is **no `f32`/`f64`
anywhere in this file**, including as intermediates.

## Determinism

Replaying the same series produces byte-for-byte identical results. The module
is zero-I/O: no network, filesystem, or clock queries.

## Tests

- `vyapaar-core/tests/quant_tests.rs` — 33 golden-vector tests.
- `vyapaar-core/tests/quant_proptest.rs` — property-based invariants.

## Related

- Gap 0001 / Plan 0002 — call-site consolidation to delegate downstream math here.
- `vyapaar-core/src/analytics/` — statistical gates that build on similar math
  (see [code-sizing-analytics](code-sizing-analytics.md)).
