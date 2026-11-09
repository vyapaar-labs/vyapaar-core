//! Walk-Forward Efficiency (WFE) — rolling out-of-sample stability gate.
//!
//! Robert Pardo's walk-forward efficiency measures parameter stability by
//! splitting the historical series into an in-sample (optimization) segment
//! and an out-of-sample (validation) segment:
//!
//! ```text
//! WFE = annualized_return(OOS) / annualized_return(IS)
//! ```
//!
//! A strategy achieving 30% annualized in-sample but only 6% out-of-sample
//! yields WFE = 20% and fails the default 50% gate — the signature of a
//! parameter set that fit the optimization window rather than the market.

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

use crate::analytics::stats::{annualized_return, sharpe_or_zero};

/// Configuration for walk-forward analysis (all fractions exact).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalkForwardConfig {
    /// In-sample fraction of the total series (default `0.60` = 60%).
    pub in_sample_fraction: Decimal,
    /// Minimum acceptable walk-forward efficiency ratio (default `0.50`).
    pub min_wfe: Decimal,
    /// Annualization factor (default `252` trading sessions per year).
    pub annualization_factor: Decimal,
}

impl Default for WalkForwardConfig {
    fn default() -> Self {
        Self {
            in_sample_fraction: Decimal::new(6, 1),
            min_wfe: Decimal::new(5, 1),
            annualization_factor: Decimal::new(252, 0),
        }
    }
}

/// Result of walk-forward analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalkForwardResult {
    /// In-sample annualized (geometric) return.
    pub is_annual_return: Decimal,
    /// Out-of-sample annualized (geometric) return.
    pub oos_annual_return: Decimal,
    /// In-sample annualized Sharpe ratio.
    pub is_sharpe: Decimal,
    /// Out-of-sample annualized Sharpe ratio.
    pub oos_sharpe: Decimal,
    /// Walk-forward efficiency: `return_OOS / return_IS`.
    pub wfe: Decimal,
    /// Whether the strategy passed the stability gate.
    pub passed: bool,
}

/// Computes walk-forward efficiency by splitting the returns series into an
/// in-sample prefix and an out-of-sample suffix.
///
/// Series shorter than twenty observations fail outright. A vanishingly small
/// in-sample return with positive out-of-sample return scores WFE = 1 (nothing
/// to degrade); a small in-sample return with negative out-of-sample return
/// scores 0.
pub fn compute_walk_forward(returns: &[Decimal], config: WalkForwardConfig) -> WalkForwardResult {
    let n = returns.len();
    if n < 20 {
        return WalkForwardResult {
            is_annual_return: Decimal::ZERO,
            oos_annual_return: Decimal::ZERO,
            is_sharpe: Decimal::ZERO,
            oos_sharpe: Decimal::ZERO,
            wfe: Decimal::ZERO,
            passed: false,
        };
    }

    let clamped = config
        .in_sample_fraction
        .clamp(Decimal::new(2, 1), Decimal::new(8, 1));
    let split = (Decimal::from(n as u64) * clamped)
        .floor()
        .to_u64()
        .unwrap_or(0) as usize;
    let split = split.clamp(1, n - 1);
    let is_returns = &returns[..split];
    let oos_returns = &returns[split..];

    let is_ret = annualized_return(is_returns, config.annualization_factor);
    let oos_ret = annualized_return(oos_returns, config.annualization_factor);
    let is_sh = sharpe_or_zero(is_returns, config.annualization_factor);
    let oos_sh = sharpe_or_zero(oos_returns, config.annualization_factor);

    let tiny = Decimal::new(1, 6);
    let wfe = if is_ret.abs() > tiny {
        oos_ret / is_ret
    } else if oos_ret > Decimal::ZERO {
        Decimal::ONE
    } else {
        Decimal::ZERO
    };

    let passed = wfe >= config.min_wfe && oos_sh > Decimal::ZERO;

    WalkForwardResult {
        is_annual_return: is_ret,
        oos_annual_return: oos_ret,
        is_sharpe: is_sh,
        oos_sharpe: oos_sh,
        wfe,
        passed,
    }
}
