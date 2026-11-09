//! Unified overfitting validation block and statistical gate evaluation —
//! GAP-0002 / PLAN-0002 (Honba parity, Tier 1).
//!
//! Six independent gates defend against the classic "impressive backtest,
//! broken live deployment" failure modes:
//!
//! | Gate | Trace | Threshold (default) |
//! |------|-------|---------------------|
//! | Deflated Sharpe Ratio (multiple-testing) | [`dsr`] | DSR ≥ 95% |
//! | Probability of Backtest Overfitting (CSCV) | [`pbo`] | PBO < 50% |
//! | Walk-Forward Efficiency | [`walk_forward`] | WFE ≥ 50% and OOS Sharpe > 0 |
//! | Holdout retention (70/30 split) | [`holdout`] | retention ≥ 50%, holdout Sharpe > 0 |
//! | Block-bootstrap Monte Carlo | [`monte_carlo`] | DD₉₅ ≤ 30% and ruin probability ≤ 1% |
//! | Cost-stress sensitivity (1×/2×/3×) | [`cost_stress`] | 2× Sharpe > 0 |
//!
//! All arithmetic is exact `Decimal` (zero floats); the Monte Carlo
//! bootstrap is fully seeded, so identical inputs replay byte-for-byte.
//! These types are numeric-analysis outputs, not wire contracts, and
//! deliberately carry no `JsonSchema` derives (see `quant::QuantSeries` for
//! the precedent).
//!
//! [`dsr`]: crate::analytics::dsr

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub mod cost_stress;
pub mod distribution;
pub mod dsr;
pub mod holdout;
pub mod monte_carlo;
pub mod pbo;
pub mod stats;
pub mod walk_forward;

use crate::analytics::cost_stress::{CostStressConfig, CostStressResult, compute_cost_stress};
use crate::analytics::dsr::{DsrConfig, DsrResult, compute_dsr};
use crate::analytics::holdout::{HoldoutConfig, HoldoutResult, compute_holdout};
use crate::analytics::monte_carlo::{MonteCarloConfig, MonteCarloResult, run_block_bootstrap};
use crate::analytics::pbo::{PboConfig, PboResult, compute_pbo};
use crate::analytics::walk_forward::{WalkForwardConfig, WalkForwardResult, compute_walk_forward};

/// Comprehensive statistical validation block for a backtest or parameter
/// sweep: the six-gate audit card.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationBlock {
    /// Deflated Sharpe Ratio validation result (selection-bias penalty).
    pub dsr: Option<DsrResult>,
    /// Probability of Backtest Overfitting result (multi-trial sweeps only).
    pub pbo: Option<PboResult>,
    /// Walk-Forward Efficiency validation result.
    pub walk_forward: Option<WalkForwardResult>,
    /// One-shot holdout validation result.
    pub holdout: Option<HoldoutResult>,
    /// Block-bootstrap Monte Carlo drawdown and ruin result.
    pub monte_carlo: Option<MonteCarloResult>,
    /// Cost-stress friction sensitivity result.
    pub cost_stress: Option<CostStressResult>,
    /// Overall verdict: `true` only when every evaluated gate passed.
    pub passed: bool,
    /// Human-readable warnings and failure reasons (empty ⇒ passed).
    pub warnings: Vec<String>,
}

impl ValidationBlock {
    /// Evaluates the five single-series overfitting gates on one backtest
    /// return series.
    ///
    /// `observed_sharpe` is the backtest's annualized Sharpe ratio (the same
    /// quantity [`dsr`] penalizes for `trial_count` trials); `trial_count`
    /// rounds up to at least one. PBO is not evaluable for a single series —
    /// use [`ValidationBlock::evaluate_sweep`] for that.
    pub fn evaluate_backtest(
        returns: &[Decimal],
        observed_sharpe: Decimal,
        trial_count: usize,
    ) -> Self {
        let mut warnings = Vec::new();

        // 1. DSR — selection bias / non-normality penalty.
        let dsr_cfg = DsrConfig {
            num_trials: trial_count.max(1),
            ..Default::default()
        };
        let dsr = Some(compute_dsr(observed_sharpe, returns, dsr_cfg));
        if let Some(d) = &dsr
            && !d.passed
        {
            warnings.push(format!(
                "DSR failed: observed Sharpe {} deflated to {:.1} (threshold: 95.0, SR0: {})",
                observed_sharpe.round_dp(2),
                (d.dsr * Decimal::new(100, 0)).round_dp(1),
                d.expected_max_sharpe.round_dp(2)
            ));
        }

        // 2. Walk-Forward Efficiency — rolling in/out-of-sample stability.
        let wf = Some(compute_walk_forward(returns, WalkForwardConfig::default()));
        if let Some(w) = &wf
            && !w.passed
        {
            warnings.push(format!(
                "Walk-Forward Efficiency failed: WFE {:.1} (min required: 50.0)",
                (w.wfe * Decimal::new(100, 0)).round_dp(1)
            ));
        }

        // 3. One-shot Holdout — pristine terminal 70/30 partition retention.
        let holdout = Some(compute_holdout(
            returns,
            HoldoutConfig {
                trial_count: trial_count.max(1),
                ..Default::default()
            },
        ));
        if let Some(h) = &holdout
            && !h.passed
        {
            warnings.push(format!(
                "Holdout validation failed: retention {:.1} (min: 50.0, holdout Sharpe: {})",
                (h.retention_ratio * Decimal::new(100, 0)).round_dp(1),
                h.holdout_sharpe.round_dp(2)
            ));
        }

        // 4. Block-bootstrap Monte Carlo — tail risk across resampled paths.
        let mc = Some(run_block_bootstrap(returns, MonteCarloConfig::default()));
        if let Some(m) = &mc
            && !m.passed
        {
            warnings.push(format!(
                "Monte Carlo bootstrap failed: 95pct worst DD {:.1} (threshold: 30.0, ruin prob: {:.1})",
                (m.drawdown_95 * Decimal::new(100, 0)).round_dp(1),
                (m.prob_ruin * Decimal::new(100, 0)).round_dp(1)
            ));
        }

        // 5. Cost-Stress — 2×/3× friction sensitivity ("Friction Fragile").
        let cost_stress = Some(compute_cost_stress(returns, CostStressConfig::default()));
        if let Some(c) = &cost_stress
            && !c.passed
        {
            warnings.push(format!(
                "Cost-stress failed: 2x friction Sharpe {} collapsed below 0 (Friction Fragile)",
                c.stress_2x_sharpe.round_dp(2)
            ));
        }

        Self {
            dsr,
            pbo: None, // PBO is evaluable only across multi-trial sweeps
            walk_forward: wf,
            holdout,
            monte_carlo: mc,
            cost_stress,
            passed: warnings.is_empty(),
            warnings,
        }
    }

    /// Evaluates the full six-gate audit across a multi-trial parameter
    /// sweep: the single-series gates on the winning trial plus PBO over all
    /// `M ≥ 2` trials' return series.
    pub fn evaluate_sweep(
        best_returns: &[Decimal],
        best_sharpe: Decimal,
        trial_returns: &[Vec<Decimal>],
    ) -> Self {
        let mut block = Self::evaluate_backtest(best_returns, best_sharpe, trial_returns.len());

        // 6. PBO across parameter-sweep trials.
        if trial_returns.len() >= 2 {
            let pbo = Some(compute_pbo(trial_returns, PboConfig::default()));
            if let Some(p) = &pbo
                && !p.passed
            {
                block.warnings.push(format!(
                    "PBO failed: Probability of Backtest Overfitting {:.1} exceeds 50.0",
                    (p.pbo * Decimal::new(100, 0)).round_dp(1)
                ));
                block.passed = false;
            }
            block.pbo = pbo;
        }

        block
    }

    /// `true` when every evaluated gate passed (no warnings recorded).
    pub fn is_pass(&self) -> bool {
        self.passed
    }

    /// `true` when the audit carries failure reasons.
    pub fn has_failures(&self) -> bool {
        !self.warnings.is_empty()
    }
}
