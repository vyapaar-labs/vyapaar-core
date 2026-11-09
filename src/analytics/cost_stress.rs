//! Cost-stress sensitivity under elevated execution frictions (1×, 2×, 3×).
//!
//! Indian friction includes STT (delivery sell side), GST on exchange and
//! SEBI charges, exchange turnover fees, SEBI turnover, stamp duty and
//! bid-ask slippage. Strategies that churn capital often look profitable on
//! paper yet collapse once friction doubles. The gate steps through
//! `1×, 2×, 3×` statutory-cost regimes and labels a strategy
//! **"Friction Fragile"** when the annualized Sharpe at `2×` collapses
//! to `≤ 0`: its paper edge was a subsidy from understated costs, not alpha.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::analytics::stats::sharpe_or_zero;

/// Configuration for cost-stress testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CostStressConfig {
    /// One-step transaction-cost drag per period, as a fraction of capital
    /// (default `0.0002` = 2 bps). Callers may pass the exact-per-bar drag
    /// derived from `vyapaar-market-finance::india::costs` instead.
    pub cost_per_period: Decimal,
    /// Annualization factor (default `252`).
    pub annualization_factor: Decimal,
}

impl Default for CostStressConfig {
    fn default() -> Self {
        Self {
            cost_per_period: Decimal::new(2, 4),
            annualization_factor: Decimal::new(252, 0),
        }
    }
}

/// Result of cost-stress testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CostStressResult {
    /// Baseline (1× cost) annualized Sharpe ratio.
    pub baseline_sharpe: Decimal,
    /// Stressed (2× cost) annualized Sharpe ratio.
    pub stress_2x_sharpe: Decimal,
    /// Severe-stress (3× cost) annualized Sharpe ratio.
    pub stress_3x_sharpe: Decimal,
    /// Retention `Sharpe_2x / Sharpe_1x` (0 when the baseline is not positive).
    pub retention_2x: Decimal,
    /// Whether alpha stays positive under 2× cost stress.
    pub passed: bool,
}

/// Computes cost-stress sensitivity by subtracting one additional
/// `cost_per_period` drag per incremental stress leg.
///
/// An empty series yields a failing result.
pub fn compute_cost_stress(returns: &[Decimal], config: CostStressConfig) -> CostStressResult {
    if returns.is_empty() {
        return CostStressResult {
            baseline_sharpe: Decimal::ZERO,
            stress_2x_sharpe: Decimal::ZERO,
            stress_3x_sharpe: Decimal::ZERO,
            retention_2x: Decimal::ZERO,
            passed: false,
        };
    }

    let sh_1x = sharpe_or_zero(returns, config.annualization_factor);

    let returns_2x: Vec<Decimal> = returns
        .iter()
        .map(|&r| r - config.cost_per_period)
        .collect();
    let sh_2x = sharpe_or_zero(&returns_2x, config.annualization_factor);

    let returns_3x: Vec<Decimal> = returns
        .iter()
        .map(|&r| r - Decimal::new(2, 0) * config.cost_per_period)
        .collect();
    let sh_3x = sharpe_or_zero(&returns_3x, config.annualization_factor);

    let retention = if sh_1x > Decimal::ZERO {
        sh_2x / sh_1x
    } else {
        Decimal::ZERO
    };

    let passed = sh_2x > Decimal::ZERO;

    CostStressResult {
        baseline_sharpe: sh_1x,
        stress_2x_sharpe: sh_2x,
        stress_3x_sharpe: sh_3x,
        retention_2x: retention,
        passed,
    }
}
