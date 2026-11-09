//! Block-bootstrap Monte Carlo simulation for equity drawdown and ruin
//! estimation.
//!
//! A historical backtest observes exactly one path of returns. Monte Carlo
//! re-simulates alternative orderings by drawing contiguous *blocks* of
//! historical returns (stationary moving-block bootstrap), preserving serial
//! autocorrelation and volatility clustering without parametric
//! assumptions, and reports the tail risk the single historical path
//! understates:
//!
//! 1. 95th-percentile maximum drawdown — the drawdown experienced by the
//!    worst 5% of resampled paths (gate: < 30%),
//! 2. probability of ruin — the probability that a path's equity falls to
//!    or below 50% of starting capital (gate: < 1%, per GAP-0002).

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::analytics::distribution::SplitMix64;

/// Configuration for block-bootstrap Monte Carlo simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonteCarloConfig {
    /// Number of resampled paths to generate (default `1000`, the floor of
    /// the 1,000–10,000 range GAP-0002 asks for).
    pub num_simulations: usize,
    /// Block length in bars. `0` selects `max(2, ⌊T^(1/3)⌋)` automatically.
    pub block_size: usize,
    /// Deterministic seed so identical inputs replay byte-for-byte.
    pub seed: u64,
    /// Maximum acceptable 95th-percentile worst-case drawdown (default `0.30`).
    pub max_acceptable_drawdown_95: Decimal,
    /// Drawdown fraction at or above which a path is "ruined" (default `0.50`).
    pub ruin_drawdown_threshold: Decimal,
    /// Maximum acceptable probability of ruin (default `0.01` = 1%).
    pub max_prob_ruin: Decimal,
}

impl Default for MonteCarloConfig {
    fn default() -> Self {
        Self {
            num_simulations: 1000,
            block_size: 0,
            seed: 42,
            max_acceptable_drawdown_95: Decimal::new(3, 1),
            ruin_drawdown_threshold: Decimal::new(5, 1),
            max_prob_ruin: Decimal::new(1, 2),
        }
    }
}

/// Results from block-bootstrap Monte Carlo analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonteCarloResult {
    /// Number of simulated paths evaluated.
    pub simulations_run: usize,
    /// Block size used for resampling.
    pub block_size: usize,
    /// Median maximum peak-to-trough drawdown across all simulated paths.
    pub median_max_drawdown: Decimal,
    /// 95th-percentile worst-case maximum drawdown (drawdown VaR 95%).
    pub drawdown_95: Decimal,
    /// 99th-percentile worst-case maximum drawdown (drawdown VaR 99%).
    pub drawdown_99: Decimal,
    /// Probability of a simulated path ending below its starting equity.
    pub prob_loss: Decimal,
    /// Probability of a path hitting or breaching the ruin drawdown threshold.
    pub prob_ruin: Decimal,
    /// Whether the strategy passed the tail-risk gate.
    pub passed: bool,
}

/// Runs a deterministic stationary moving-block bootstrap over the returns
/// series.
///
/// Fewer than five observations yield a failing result (nothing stable to
/// bootstrap). Fully seeded: identical `(returns, config)` pairs produce
/// identical outcomes.
pub fn run_block_bootstrap(returns: &[Decimal], config: MonteCarloConfig) -> MonteCarloResult {
    let t = returns.len();
    if t < 5 {
        return MonteCarloResult {
            simulations_run: 0,
            block_size: 0,
            median_max_drawdown: Decimal::ZERO,
            drawdown_95: Decimal::ZERO,
            drawdown_99: Decimal::ZERO,
            prob_loss: Decimal::ZERO,
            prob_ruin: Decimal::ZERO,
            passed: false,
        };
    }

    let requested_block = if config.block_size > 0 {
        config.block_size.min(t)
    } else {
        cube_root_floor(t)
    };
    let b = requested_block.max(2).min(t);

    let num_sims = config.num_simulations.max(10);
    let one = Decimal::ONE;
    let mut rng = SplitMix64::new(config.seed);

    let mut max_drawdowns: Vec<Decimal> = Vec::with_capacity(num_sims);
    let mut loss_count = 0usize;
    let mut ruin_count = 0usize;

    for _ in 0..num_sims {
        // Construct a synthetic path of length t by stitching contiguous
        // blocks drawn at random start indices.
        let mut equity = one;
        let mut peak = one;
        let mut max_dd = Decimal::ZERO;
        let mut bars_generated = 0usize;

        while bars_generated < t {
            let start = rng.next_index(t);
            let take = b.min(t - bars_generated);
            for j in 0..take {
                let idx = (start + j) % t;
                let r = returns[idx];
                equity *= one + r;
                if equity > peak {
                    peak = equity;
                }
                let dd = (peak - equity) / peak;
                if dd > max_dd {
                    max_dd = dd;
                }
            }
            bars_generated += take;
        }

        max_drawdowns.push(max_dd);
        if equity < one {
            loss_count += 1;
        }
        if max_dd >= config.ruin_drawdown_threshold {
            ruin_count += 1;
        }
    }

    max_drawdowns.sort();

    let sims = num_sims as u64;
    let last = num_sims - 1;
    let median_dd = max_drawdowns[((sims / 2) as usize).min(last)];
    let dd_95 = max_drawdowns[((sims * 95 / 100) as usize).min(last)];
    let dd_99 = max_drawdowns[((sims * 99 / 100) as usize).min(last)];

    let prob_loss = Decimal::from(loss_count as u64) / Decimal::from(sims);
    let prob_ruin = Decimal::from(ruin_count as u64) / Decimal::from(sims);

    let passed = dd_95 <= config.max_acceptable_drawdown_95 && prob_ruin <= config.max_prob_ruin;

    MonteCarloResult {
        simulations_run: num_sims,
        block_size: b,
        median_max_drawdown: median_dd,
        drawdown_95: dd_95,
        drawdown_99: dd_99,
        prob_loss,
        prob_ruin,
        passed,
    }
}

/// ⌊t^(1/3)⌋ via exact integer trialing (no floats): the largest integer `c`
/// with `c³ ≤ t`. The loop is bounded by `cbrt(t)`, tiny for realistic
/// series lengths.
fn cube_root_floor(t: usize) -> usize {
    let mut c = 0usize;
    while (c + 1) * (c + 1) * (c + 1) <= t {
        c += 1;
    }
    c
}
