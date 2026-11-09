# `vyapaar-core` — Position Sizing & Analytics Reference

Documents `vyapaar-core/src/sizing.rs` and `vyapaar-core/src/analytics/`.

## Position Sizing (`sizing.rs`)

### `SizingMethod`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SizingMethod {
    FixedFractional { risk_pct: Decimal },           // risk % of equity per trade
    VolatilityTarget { target_annual_vol: Decimal, lookback_bars: usize },  // inverse vol sizing
    KellyCriterion { fraction: Decimal, win_rate: Decimal, payoff_ratio: Decimal },  // fractional Kelly
    EqualRiskContribution,  // equal risk across active signal candidates
    FixedNotional { notional: Money },  // fixed cash, lot-rounded quantity
}
```

### `RiskBudget`, `PositionSizingRequest`, `PositionSizingResult`

- `RiskBudget` — risk parameters and constraints.
- `PositionSizingRequest` — inputs for a sizing calculation.
- `PositionSizingResult` — output quantity, notional, and risk metrics.
- `calculate_position_size(request) -> PositionSizingResult` — pure deterministic function.

All arithmetic is exact `rust_decimal::Decimal`, zero floats, zero I/O.

## Analytics Validation Gates (`analytics/`)

Six independent gates defending against the "impressive backtest, broken live
deployment" failure mode. All arithmetic is exact `Decimal`; Monte Carlo is fully
seeded for byte-for-byte replay.

| Module | Gate | Default Threshold |
|--------|------|-------------------|
| `dsr` | Deflated Sharpe Ratio (multiple-testing / selection bias) | DSR ≥ 95% |
| `pbo` | Probability of Backtest Overfitting (CSCV) | PBO < 50% |
| `walk_forward` | Walk-Forward Efficiency | WFE ≥ 50%, OOS Sharpe > 0 |
| `holdout` | Holdout retention (70/30 split) | retention ≥ 50%, holdout Sharpe > 0 |
| `monte_carlo` | Block-bootstrap Monte Carlo | DD₉₅ ≤ 30%, ruin ≤ 1% |
| `cost_stress` | Cost-stress sensitivity (1×/2×/3×) | 2× Sharpe > 0 |

### `ValidationBlock`

Comprehensive six-gate audit card:
- `evaluate_backtest(returns, observed_sharpe, trial_count)` — single-series gates (DSR, WFE, holdout, Monte Carlo, cost-stress; PBO = None).
- `evaluate_sweep(best_returns, best_sharpe, trial_returns)` — full six-gate audit including PBO across `M ≥ 2` trials.
- `passed: bool` — true only when every evaluated gate passed.
- `warnings: Vec<String>` — human-readable failure reasons (empty ⇒ passed).

### Sub-modules

`cost_stress`, `distribution`, `dsr`, `holdout`, `monte_carlo`, `pbo`, `stats`,
`walk_forward`.

## Zero-Floats Policy

All analytics and sizing math uses `rust_decimal::Decimal`. No `f32`/`f64` in
any financial calculation path.

## Related

- `vyapaar-agent-analytics` — downstream analytics crate (should delegate to
  `vyapaar_core::quant` per gap 0001 / plan 0002).
- `vyapaar-core/src/quant.rs` — canonical quant primitives used by sizing.
