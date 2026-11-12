# `vyapaar-core` — Position Sizing & Statistical Validation Reference

Documents `vyapaar-core/src/sizing.rs` (dynamic position sizing grammar) and
`vyapaar-core/src/analytics/` (the statistical overfitting & tail-risk
validation gates).

## Position Sizing (`sizing.rs`)

### `SizingMethod`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SizingMethod {
    FixedFractional { risk_pct: Decimal },           // risk % of equity per trade
    VolatilityTarget { target_annual_vol: Decimal, lookback_bars: usize },  // inverse-vol sizing
    KellyCriterion { fraction: Decimal, win_rate: Decimal, payoff_ratio: Decimal },  // fractional Kelly
    EqualRiskContribution,                           // equal risk across active signal candidates
    FixedNotional { notional: Money },               // fixed cash, lot-rounded quantity
}
```

### Supporting types & entry point

- `RiskBudget` — risk parameters and constraints.
- `PositionSizingRequest` — inputs for a sizing calculation.
- `PositionSizingResult` — output quantity, notional, and risk metrics.
- `calculate_position_size(request) -> PositionSizingResult` — pure
  deterministic function (no I/O, no wall clock).

All arithmetic is exact `rust_decimal::Decimal`, zero floats, zero I/O.

## Analytics Validation Gates (`analytics/`)

Six independent gates defend against the "impressive backtest, broken live
deployment" failure mode. All arithmetic is exact `Decimal`; the block-bootstrap
Monte Carlo is fully seeded for byte-for-byte replay. These types are
numeric-analysis outputs, **not** wire contracts, and deliberately carry no
`JsonSchema` derives.

| Module | Gate | Default Threshold |
|--------|------|-------------------|
| `dsr` | Deflated Sharpe Ratio (multiple-testing / selection bias) | DSR ≥ 95% |
| `pbo` | Probability of Backtest Overfitting (CSCV) | PBO < 50% |
| `walk_forward` | Walk-Forward Efficiency | WFE ≥ 50%, OOS Sharpe > 0 |
| `holdout` | Holdout retention (70/30 split) | retention ≥ 50%, holdout Sharpe > 0 |
| `monte_carlo` | Block-bootstrap Monte Carlo | DD₉₅ ≤ 30%, ruin ≤ 1% |
| `cost_stress` | Cost-stress sensitivity (1×/2×/3×) | 2× Sharpe > 0 |

Sub-modules: `cost_stress`, `distribution`, `dsr`, `holdout`, `monte_carlo`,
`pbo`, `stats`, `walk_forward`.

### `ValidationBlock`

Comprehensive six-gate audit card:

- `evaluate_backtest(returns: &[Decimal], observed_sharpe, trial_count)` —
  single-series gates (DSR, WFE, holdout, Monte Carlo, cost-stress; `pbo: None`).
- `evaluate_sweep(best_returns, best_sharpe, trial_returns: &[Vec<Decimal>])` —
  full six-gate audit including PBO across `M ≥ 2` trials.
- `passed: bool` — true only when every evaluated gate passed.
- `warnings: Vec<String>` — human-readable failure reasons (empty ⇒ passed).
- `is_pass()`, `has_failures()` helpers.

## Zero-Floats Policy

All analytics and sizing math uses `rust_decimal::Decimal`. No `f32`/`f64` in
any financial calculation path.

## Tests

- `vyapaar-core/tests/analytics_gates_tests.rs` — gate behavior against golden
  vectors.

## Related

- `vyapaar-agent-analytics` — downstream analytics crate; should delegate to
  `vyapaar_core::quant` per gap 0001 / plan 0002.
- `vyapaar-core/src/quant.rs` — canonical quant primitives used by sizing.
- `vyapaar-core/src/ports.rs` — `PositionSizer` port trait
  (see [code-ports](code-ports.md)).
