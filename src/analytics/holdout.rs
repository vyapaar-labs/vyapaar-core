//! One-shot temporal holdout validation.
//!
//! Evaluates strategy returns on a pristine terminal holdout partition
//! (default 70/30 in-sample/holdout split) and measures Sharpe retention
//! out-of-sample.

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

use crate::analytics::stats::sharpe_or_zero;

/// Configuration for one-shot holdout evaluation (all fractions exact).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HoldoutConfig {
    /// Fraction of the return series reserved for the terminal holdout
    /// (default `0.30`, giving the 70/30 in-sample/holdout split).
    pub holdout_fraction: Decimal,
    /// Minimum allowed retention of the training Sharpe on the holdout
    /// (default `0.50` = 50%).
    pub min_retention_ratio: Decimal,
    /// Total distinct trials tested before this holdout look.
    pub trial_count: usize,
    /// Annualization factor (default `252` trading sessions per year).
    pub annualization_factor: Decimal,
}

impl Default for HoldoutConfig {
    fn default() -> Self {
        Self {
            holdout_fraction: Decimal::new(3, 1),
            min_retention_ratio: Decimal::new(5, 1),
            trial_count: 1,
            annualization_factor: Decimal::new(252, 0),
        }
    }
}

/// Result of one-shot holdout validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HoldoutResult {
    /// In-sample (training) Sharpe ratio.
    pub train_sharpe: Decimal,
    /// Out-of-sample (holdout) Sharpe ratio.
    pub holdout_sharpe: Decimal,
    /// Retention ratio `Sharpe_holdout / Sharpe_train`.
    pub retention_ratio: Decimal,
    /// Whether the watch passed the retention gate.
    pub passed: bool,
}

/// Evaluates one-shot holdout performance on a pristine terminal block.
///
/// Series shorter than ten observations or a zero-length holdout split yield
/// a failing result; a non-positive training Sharpe with positive holdout
/// Sharpe counts as full retention (the strategy only got better OOS).
pub fn compute_holdout(returns: &[Decimal], config: HoldoutConfig) -> HoldoutResult {
    let n = returns.len();
    if n < 10 {
        return HoldoutResult {
            train_sharpe: Decimal::ZERO,
            holdout_sharpe: Decimal::ZERO,
            retention_ratio: Decimal::ZERO,
            passed: false,
        };
    }

    let holdout_len = ((Decimal::from(n as u64)
        * config
            .holdout_fraction
            .clamp(Decimal::new(5, 2), Decimal::new(5, 1)))
    .floor())
    .to_u64()
    .unwrap_or(0) as usize;
    let holdout_len = holdout_len.min(n.saturating_sub(1)).max(1);
    let train_len = n - holdout_len;

    let train_sharpe = sharpe_or_zero(&returns[..train_len], config.annualization_factor);
    let holdout_sharpe = sharpe_or_zero(&returns[train_len..], config.annualization_factor);

    let retention = if train_sharpe > Decimal::ZERO {
        holdout_sharpe / train_sharpe
    } else if holdout_sharpe > Decimal::ZERO {
        Decimal::ONE
    } else {
        Decimal::ZERO
    };

    let passed = holdout_sharpe > Decimal::ZERO && retention >= config.min_retention_ratio;

    HoldoutResult {
        train_sharpe,
        holdout_sharpe,
        retention_ratio: retention,
        passed,
    }
}
