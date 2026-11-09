//! Deflated Sharpe Ratio (DSR) — Bailey & López de Prado (2014).
//!
//! Adjusts the observed Sharpe ratio for:
//! 1. Non-normality of returns (negative skewness and fat-tailed kurtosis),
//! 2. Selection bias / multiple testing across `N` independent strategy
//!    trials, inflating the expected maximum Sharpe under the null.
//!
//! # Zero-floats note
//!
//! Every step (moments, standard error, z-statistic, CDF) is evaluated in
//! exact `Decimal`; the normal CDF is `rust_decimal`'s pure-`Decimal`
//! `norm_cdf` and the inverse CDF is [`crate::analytics::distribution::norm_inv`],
//! bisected against that same CDF. No `f32`/`f64` appears anywhere, so the
//! gate result replays byte-for-byte.

use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;
use serde::{Deserialize, Serialize};

use crate::analytics::distribution::norm_inv;

/// Configuration for the Deflated Sharpe Ratio calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DsrConfig {
    /// Number of independent (tested) strategy trials `N ≥ 1`.
    pub num_trials: usize,
    /// Variance of the Sharpe ratios across all tested trials.
    pub trial_sharpe_variance: Decimal,
    /// Significance the deflated confidence must reach (default: `0.95`).
    pub threshold: Decimal,
}

impl Default for DsrConfig {
    fn default() -> Self {
        Self {
            num_trials: 1,
            trial_sharpe_variance: Decimal::ONE,
            threshold: Decimal::new(95, 2),
        }
    }
}

/// Result of Deflated Sharpe Ratio validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DsrResult {
    /// The unadjusted observed Sharpe ratio.
    pub observed_sharpe: Decimal,
    /// Expected maximum Sharpe under the null hypothesis (`SR₀`).
    pub expected_max_sharpe: Decimal,
    /// Standard error of the observed Sharpe (skew/kurtosis adjusted).
    pub standard_error: Decimal,
    /// Test statistic `z = (ŜR - SR₀) / SE`.
    pub z_stat: Decimal,
    /// Deflated Sharpe Ratio confidence `Φ(z) ∈ [0, 1]`.
    pub dsr: Decimal,
    /// Whether the confidence reached the configured threshold.
    pub passed: bool,
}

/// Expected maximum Sharpe ratio under the null across `N` trials:
///
/// ```text
/// E[max] ≈ √V · ((1 - γ)·Φ⁻¹(1 - 1/N) + γ·Φ⁻¹(1 - 1/(N·e)))
/// ```
///
/// A single trial (`N <= 1`) has no multiple-testing inflation: returns `0`.
pub fn expected_max_sharpe(num_trials: usize, trial_sharpe_variance: Decimal) -> Decimal {
    if num_trials <= 1 {
        return Decimal::ZERO;
    }
    let n = Decimal::from(num_trials as u64);
    let var_sqrt = trial_sharpe_variance.sqrt().unwrap_or(Decimal::ZERO);
    let e = Decimal::ONE.exp();
    // Euler–Mascheroni constant γ (16-decimal representation, matching the
    // precision the reference implementation uses).
    let euler_gamma = Decimal::new(5772156649015329, 16);

    let p1 = (Decimal::ONE - Decimal::ONE / n).clamp(Decimal::new(1, 12), Decimal::ONE);
    let p2 = (Decimal::ONE - Decimal::ONE / (n * e)).clamp(Decimal::new(1, 12), Decimal::ONE);

    let z1 = norm_inv(p1);
    let z2 = norm_inv(p2);

    var_sqrt * ((Decimal::ONE - euler_gamma) * z1 + euler_gamma * z2)
}

/// Sample skewness γ₃: third population moment over the cube of the sample
/// standard deviation; `0` for fewer than three observations or a
/// zero-dispersion series.
fn skewness(returns: &[Decimal], mean: Decimal, stdev: Decimal) -> Decimal {
    let n = returns.len();
    if n < 3 || stdev.is_zero() {
        return Decimal::ZERO;
    }
    let mut m3 = Decimal::ZERO;
    for &r in returns {
        let d = r - mean;
        m3 += d * d * d;
    }
    m3 / Decimal::from(n as u64) / (stdev * stdev * stdev)
}

/// Sample kurtosis γ₄ (raw, normal ≈ 3): fourth population moment over the
/// square of the sample variance; `3` for fewer than four observations or a
/// zero-dispersion series.
fn kurtosis(returns: &[Decimal], mean: Decimal, variance: Decimal) -> Decimal {
    let n = returns.len();
    if n < 4 {
        return Decimal::new(3, 0);
    }
    if variance.is_zero() {
        return Decimal::new(3, 0);
    }
    let mut m4 = Decimal::ZERO;
    for &r in returns {
        let d = r - mean;
        m4 += d * d * d * d;
    }
    m4 / Decimal::from(n as u64) / (variance * variance)
}

/// Computes the Deflated Sharpe Ratio from an observed annualized Sharpe and
/// the per-period returns series behind it.
///
/// Degenerate inputs (fewer than three returns, non-finite Sharpe) yield a
/// failing result with a neutral `dsr = 0.5`, like the reference engine.
pub fn compute_dsr(observed_sharpe: Decimal, returns: &[Decimal], config: DsrConfig) -> DsrResult {
    let n = returns.len();
    if n < 3 {
        return DsrResult {
            observed_sharpe,
            expected_max_sharpe: Decimal::ZERO,
            standard_error: Decimal::ONE,
            z_stat: Decimal::ZERO,
            dsr: Decimal::new(5, 1),
            passed: false,
        };
    }

    // Sample moments (n-1 variance, matching the kernel's quant module).
    let mut sum = Decimal::ZERO;
    for &r in returns {
        sum += r;
    }
    let mean = sum / Decimal::from(n as u64);
    let mut var_acc = Decimal::ZERO;
    for &r in returns {
        let d = r - mean;
        var_acc += d * d;
    }
    let variance = var_acc / Decimal::from((n - 1) as u64);
    let stdev = variance.sqrt().unwrap_or(Decimal::ZERO);

    let skew = skewness(returns, mean, stdev);
    let kurt = kurtosis(returns, mean, variance);

    // Bailey & López de Prado (2014) standard error:
    // σ² = (1 - γ₃·SR + (γ₄ - 1)/4 · SR²) / (T - 1)
    let sr_sq = observed_sharpe * observed_sharpe;
    let var_sr = (Decimal::ONE - skew * observed_sharpe
        + (kurt - Decimal::ONE) / Decimal::new(4, 0) * sr_sq)
        / Decimal::from((n - 1) as u64);
    let se = if var_sr > Decimal::ZERO {
        var_sr.sqrt().unwrap_or(Decimal::ONE)
    } else {
        Decimal::new(1, 12).sqrt().unwrap_or(Decimal::ONE)
    };

    let sr0 = expected_max_sharpe(config.num_trials, config.trial_sharpe_variance);
    let z = (observed_sharpe - sr0) / se;
    // `rust_decimal`'s polynomial-approximation `erf` evaluates `sum^16`
    // inside its error-function expansion, which *panics* ("Pow overflowed")
    // for |z| ≳ 53 — reachable whenever the observed Sharpe towers over SR₀
    // while a short sample keeps the standard error tiny. The CDF saturates in
    // the far tails long before that (Φ(7) = 1 − 1.28e-12), so clamping the
    // statistic into ±7 is verdict-neutral at the 0.95 threshold: |z| ≥ 7 is
    // already a certain pass/fail. The raw (unsaturated) z-stat is still
    // reported below.
    let dsr = z.clamp(Decimal::new(-7, 0), Decimal::new(7, 0)).norm_cdf();

    DsrResult {
        observed_sharpe,
        expected_max_sharpe: sr0,
        standard_error: se,
        z_stat: z,
        dsr,
        passed: dsr >= config.threshold,
    }
}
