//! Probability of Backtest Overfitting (PBO) via Combinatorially-Symmetric
//! Cross-Validation (CSCV) — Bailey, Borwein, López de Prado & Zhu (2017).
//!
//! Degrees the *relative ranking* of the in-sample-best model across every
//! symmetric in-sample/out-of-sample split of the historical timeline: the
//! timeline is cut into `S` slices, all C(S, S/2) combinations of in-sample
//! slices are enumerated, and the share of combinations whose best in-sample
//! model ranks in the bottom half out-of-sample is the PBO. A strategy whose
//! tuned parameters mostly win their in-sample playouts by luck ranks below
//! the median OOS more often than chance, and PBO climbs toward 100%.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::analytics::stats::sharpe_or_zero;

/// Configuration for PBO CSCV evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PboConfig {
    /// Number of sub-period slices `S` (even; default `8`).
    pub num_slices: usize,
    /// Maximum acceptable PBO (default `0.50` = 50%).
    pub max_acceptable_pbo: Decimal,
}

impl Default for PboConfig {
    fn default() -> Self {
        Self {
            num_slices: 8,
            max_acceptable_pbo: Decimal::new(5, 1),
        }
    }
}

/// Result of Probability of Backtest Overfitting analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PboResult {
    /// Probability of backtest overfitting, `PBO ∈ [0, 1]`.
    pub pbo: Decimal,
    /// Number of symmetric cross-validation combinations evaluated.
    pub combinations_evaluated: usize,
    /// Median relative rank of the best in-sample model out-of-sample
    /// (`1.0` = always best, `0.0` = always worst).
    pub median_oos_percentile: Decimal,
    /// Whether PBO stayed below the acceptable threshold.
    pub passed: bool,
}

/// Computes PBO over a set of per-model returns series
/// (`M` models × `T` aligned periods).
///
/// Fewer than two models, or slices too short to split, short-circuit to a
/// passing result (no evidence is not evidence of overfitting). A
/// zero-dispersion sub-window contributes a Sharpe of exactly `0`, consistent
/// with [`crate::analytics::stats::sharpe_or_zero`].
pub fn compute_pbo(trial_returns: &[Vec<Decimal>], config: PboConfig) -> PboResult {
    let num_models = trial_returns.len();
    if num_models < 2 {
        return PboResult {
            pbo: Decimal::ZERO,
            combinations_evaluated: 0,
            median_oos_percentile: Decimal::ONE,
            passed: true,
        };
    }

    let min_len = trial_returns.iter().map(Vec::len).min().unwrap_or(0);
    let mut s = config.num_slices.max(4);
    if s % 2 != 0 {
        s += 1; // keep S even so every slice count is symmetric
    }
    if min_len < s * 2 {
        return PboResult {
            pbo: Decimal::ZERO,
            combinations_evaluated: 0,
            median_oos_percentile: Decimal::ONE,
            passed: true,
        };
    }

    let slice_len = min_len / s;
    let half_s = s / 2;
    let combinations = combinations(s, half_s);
    let total_combs = combinations.len();

    let mut overfit_count = 0usize;
    let mut oos_percentiles: Vec<Decimal> = Vec::with_capacity(total_combs);
    let model_scale = Decimal::from((num_models - 1) as u64);

    for is_indices in &combinations {
        // Evaluate IS/OOS Sharpe for every model under this split.
        let mut is_sharpes: Vec<(usize, Decimal)> = Vec::with_capacity(num_models);
        let mut oos_sharpes: Vec<(usize, Decimal)> = Vec::with_capacity(num_models);

        for (m_idx, returns) in trial_returns.iter().enumerate() {
            let mut is_sub: Vec<Decimal> = Vec::with_capacity(slice_len * half_s);
            let mut oos_sub: Vec<Decimal> = Vec::with_capacity(slice_len * half_s);
            for slice_idx in 0..s {
                let start = slice_idx * slice_len;
                let slice_data = &returns[start..start + slice_len];
                if is_indices.contains(&slice_idx) {
                    is_sub.extend_from_slice(slice_data);
                } else {
                    oos_sub.extend_from_slice(slice_data);
                }
            }

            let is_sh = sharpe_or_zero(&is_sub, Decimal::new(252, 0));
            let oos_sh = sharpe_or_zero(&oos_sub, Decimal::new(252, 0));
            is_sharpes.push((m_idx, is_sh));
            oos_sharpes.push((m_idx, oos_sh));
        }

        // Best model in-sample (stable sort keeps ties deterministic).
        is_sharpes.sort_by_key(|b| std::cmp::Reverse(b.1));
        let best_model_is = is_sharpes[0].0;

        // Rank of that model out-of-sample (stable sort, same tie discipline).
        oos_sharpes.sort_by_key(|b| std::cmp::Reverse(b.1));
        let oos_rank = oos_sharpes
            .iter()
            .position(|(m, _)| *m == best_model_is)
            .unwrap_or(num_models - 1);

        // Percentile: 1.0 best, 0.0 worst.
        let oos_pct = Decimal::ONE - Decimal::from(oos_rank as u64) / model_scale;
        if oos_pct < Decimal::new(5, 1) {
            // Below the median OOS rank: the in-sample champion underperformed
            // the median out-of-sample — evidence of overfit selection.
            overfit_count += 1;
        }
        oos_percentiles.push(oos_pct);
    }

    let pbo = if total_combs > 0 {
        Decimal::from(overfit_count as u64) / Decimal::from(total_combs as u64)
    } else {
        Decimal::ZERO
    };

    oos_percentiles.sort();
    let median_pct = if !oos_percentiles.is_empty() {
        oos_percentiles[oos_percentiles.len() / 2]
    } else {
        Decimal::ONE
    };

    PboResult {
        pbo,
        combinations_evaluated: total_combs,
        median_oos_percentile: median_pct,
        passed: pbo <= config.max_acceptable_pbo,
    }
}

/// Every k-combination of `0..n`, deterministically ordered (colex, via
/// backtracking); unconditional-free and allocation-light over `Vec`s.
fn combinations(n: usize, k: usize) -> Vec<Vec<usize>> {
    let mut results = Vec::new();
    let mut current: Vec<usize> = Vec::with_capacity(k);

    fn backtrack(
        start: usize,
        n: usize,
        k: usize,
        current: &mut Vec<usize>,
        results: &mut Vec<Vec<usize>>,
    ) {
        if current.len() == k {
            results.push(current.clone());
            return;
        }
        if n - start < k - current.len() {
            return; // not enough remaining picks
        }
        for i in start..n {
            current.push(i);
            backtrack(i + 1, n, k, current, results);
            current.pop();
        }
    }

    backtrack(0, n, k, &mut current, &mut results);
    results
}
