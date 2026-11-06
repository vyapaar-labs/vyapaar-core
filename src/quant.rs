//! Canonical quant transform primitives: returns, volatility, rolling stats.
//!
//! This is the single canonical home for the pure, zero-I/O numeric transforms
//! that downstream crates (`vyapaar-market-indicators`, `vyapaar-agent-analytics`,
//! `vyapaar-runtime-backtest`, and the future `vyapaar-ledger-allocation`
//! covariance/return inputs) have historically each re-derived locally:
//!
//! - period-over-period **simple** and **log** returns
//! - **realized** and RiskMetrics-style **EWMA** volatility
//! - fixed-window **rolling mean / stdev**
//! - the two most commonly re-derived summary statistics: **Sharpe ratio** and
//!   **max drawdown**
//! - **correlation** and **beta** against a benchmark
//!
//! Everything here is pure math over exact [`Decimal`] values: no network, no
//! filesystem, no clock queries, matching the zero-I/O domain-purity invariant
//! of the rest of `vyapaar-core`. Replaying the same series produces the same
//! result byte-for-byte.
//!
//! # Zero-floats note
//!
//! [`log_returns`] needs a natural logarithm, which `Decimal` does not ship
//! natively. This crate enables `rust_decimal`'s `maths` feature, whose
//! [`MathematicalOps::ln`] / [`MathematicalOps::sqrt`] are themselves computed
//! in pure `Decimal` (192-bit `DecWide` intermediates, no `f64`), so no float
//! enters this module even as an intermediate — including inside `sqrt` and
//! `ln`. There is no `f32`/`f64` anywhere in this file.

use core::ops::Deref;
use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;
use thiserror::Error;

use crate::types::{Money, Price};

/// Errors raised by the quant transform primitives.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum QuantError {
    /// Not enough observations for the requested computation.
    #[error("insufficient data: needed {needed} observations, got {got}")]
    InsufficientData {
        /// Observations the computation requires.
        needed: usize,
        /// Observations actually supplied.
        got: usize,
    },

    /// A division by zero was encountered: a zero prior price in a return,
    /// a zero-variance series where a standard-deviation denominator is
    /// required, or a zero peak in a drawdown.
    #[error("division by zero in quant computation")]
    DivisionByZero,

    /// A rolling window is larger than the series it is applied to.
    #[error("rolling window {window} is larger than series length {len}")]
    WindowLargerThanSeries {
        /// The requested window size.
        window: usize,
        /// The length of the series.
        len: usize,
    },

    /// An argument is outside its valid domain (zero window, out-of-range
    /// EWMA decay, negative annualization factor, mismatched series lengths).
    #[error("invalid quant argument: {0}")]
    InvalidArgument(&'static str),
}

/// An ordered series of exact decimal values: a price path, an equity curve,
/// or a returns series.
///
/// A thin newtype over `Vec<Decimal>` that gives the quant functions one
/// ergonomic input type and converts from the kernel's [`Price`]/[`Money`]
/// value objects so callers do not hand-unwrap them.
///
/// This is a numeric-transform input, not a wire contract, so it deliberately
/// carries no serde/schemars derives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantSeries(Vec<Decimal>);

impl QuantSeries {
    /// Wraps an already-ordered series of exact decimals.
    pub fn new(values: Vec<Decimal>) -> Self {
        Self(values)
    }

    /// Number of observations in the series.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// `true` when the series has no observations.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The observation at `index`, if in range.
    pub fn get(&self, index: usize) -> Option<&Decimal> {
        self.0.get(index)
    }

    /// The series as a slice.
    pub fn as_slice(&self) -> &[Decimal] {
        &self.0
    }

    /// Unwraps the underlying value vector.
    pub fn into_inner(self) -> Vec<Decimal> {
        self.0
    }
}

impl Deref for QuantSeries {
    type Target = [Decimal];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<Decimal>> for QuantSeries {
    fn from(values: Vec<Decimal>) -> Self {
        Self(values)
    }
}

impl From<&[Price]> for QuantSeries {
    fn from(prices: &[Price]) -> Self {
        Self(prices.iter().map(|p| p.value()).collect())
    }
}

impl From<&[Money]> for QuantSeries {
    fn from(money: &[Money]) -> Self {
        Self(money.iter().map(|m| m.amount).collect())
    }
}

impl FromIterator<Decimal> for QuantSeries {
    fn from_iter<T: IntoIterator<Item = Decimal>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Arithmetic mean of a non-empty slice.
fn mean(values: &[Decimal]) -> Decimal {
    values.iter().copied().sum::<Decimal>() / Decimal::from(values.len() as u64)
}

/// Sample variance (`n - 1` denominator).
fn sample_variance(values: &[Decimal]) -> Result<Decimal, QuantError> {
    let n = values.len();
    if n < 2 {
        return Err(QuantError::InsufficientData { needed: 2, got: n });
    }
    let m = mean(values);
    let sum_sq: Decimal = values.iter().map(|v| (*v - m) * (*v - m)).sum();
    Ok(sum_sq / Decimal::from((n - 1) as u64))
}

/// Sample standard deviation (square root of [`sample_variance`]).
fn sample_stdev(values: &[Decimal]) -> Result<Decimal, QuantError> {
    let var = sample_variance(values)?;
    // Variance of a real series is non-negative; `sqrt` of a negative value
    // would return `None`.
    var.sqrt().ok_or(QuantError::DivisionByZero)
}

// ---------------------------------------------------------------------------
// Returns
// ---------------------------------------------------------------------------

/// Period-over-period simple returns: `(p[t] - p[t-1]) / p[t-1]`.
///
/// The output has `n - 1` entries for an `n`-entry series. A zero prior price
/// is a division-by-zero error.
pub fn simple_returns(series: &QuantSeries) -> Result<QuantSeries, QuantError> {
    let n = series.len();
    if n < 2 {
        return Err(QuantError::InsufficientData { needed: 2, got: n });
    }
    let mut out = Vec::with_capacity(n - 1);
    for window in series.as_slice().windows(2) {
        let prev = window[0];
        if prev.is_zero() {
            return Err(QuantError::DivisionByZero);
        }
        out.push((window[1] - window[0]) / prev);
    }
    Ok(QuantSeries::new(out))
}

/// Natural-log returns: `ln(p[t] / p[t-1])`.
///
/// The output has `n - 1` entries for an `n`-entry series. Both the prior and
/// the current price must be strictly positive; `Decimal::ln` is provided by
/// `rust_decimal`'s `maths` feature and is computed entirely in `Decimal`
/// (192-bit intermediates), so no floating point enters this function.
pub fn log_returns(series: &QuantSeries) -> Result<QuantSeries, QuantError> {
    let n = series.len();
    if n < 2 {
        return Err(QuantError::InsufficientData { needed: 2, got: n });
    }
    let mut out = Vec::with_capacity(n - 1);
    for window in series.as_slice().windows(2) {
        if window[0] <= Decimal::ZERO || window[1] <= Decimal::ZERO {
            return Err(QuantError::DivisionByZero);
        }
        out.push((window[1] / window[0]).ln());
    }
    Ok(QuantSeries::new(out))
}

// ---------------------------------------------------------------------------
// Volatility
// ---------------------------------------------------------------------------

/// Sample standard deviation of a returns series, scaled by the square root of
/// the annualization factor (e.g. `252` for daily returns → annualized).
///
/// Requires at least two returns. `annualization_factor` must be non-negative.
pub fn realized_volatility(
    returns: &QuantSeries,
    annualization_factor: Decimal,
) -> Result<Decimal, QuantError> {
    if annualization_factor < Decimal::ZERO {
        return Err(QuantError::InvalidArgument(
            "annualization_factor must be non-negative",
        ));
    }
    let std = sample_stdev(returns.as_slice())?;
    let scale = annualization_factor
        .sqrt()
        .ok_or(QuantError::DivisionByZero)?;
    Ok(std * scale)
}

/// RiskMetrics-style exponentially-weighted moving variance/volatility.
///
/// Variance is updated recursively over the returns series:
///
/// ```text
/// var[0] = r[0]^2
/// var[i] = lambda * var[i-1] + (1 - lambda) * r[i]^2
/// ```
///
/// and the result is `sqrt(var[n-1])`. `lambda` is the decay factor and must
/// lie in the open interval `(0, 1)` (0.94 is the classic RiskMetrics value).
pub fn ewma_volatility(returns: &QuantSeries, lambda: Decimal) -> Result<Decimal, QuantError> {
    let n = returns.len();
    if n < 1 {
        return Err(QuantError::InsufficientData { needed: 1, got: n });
    }
    if !(lambda > Decimal::ZERO && lambda < Decimal::ONE) {
        return Err(QuantError::InvalidArgument(
            "lambda must lie in the open interval (0, 1)",
        ));
    }
    let mut var = returns.as_slice()[0] * returns.as_slice()[0];
    for &r in &returns.as_slice()[1..] {
        var = lambda * var + (Decimal::ONE - lambda) * r * r;
    }
    var.sqrt().ok_or(QuantError::DivisionByZero)
}

// ---------------------------------------------------------------------------
// Rolling statistics
// ---------------------------------------------------------------------------

/// Fixed-window simple moving average.
///
/// The output has `n - window + 1` entries for an `n`-entry series.
pub fn rolling_mean(series: &QuantSeries, window: usize) -> Result<QuantSeries, QuantError> {
    if window == 0 {
        return Err(QuantError::InvalidArgument(
            "rolling window must be positive",
        ));
    }
    let n = series.len();
    if window > n {
        return Err(QuantError::WindowLargerThanSeries { window, len: n });
    }
    let divisor = Decimal::from(window as u64);
    let out = series
        .as_slice()
        .windows(window)
        .map(|w| w.iter().copied().sum::<Decimal>() / divisor)
        .collect();
    Ok(QuantSeries::new(out))
}

/// Fixed-window sample standard deviation (`n - 1` denominator, matching
/// [`realized_volatility`]).
///
/// The output has `n - window + 1` entries. The window must be at least 2 (a
/// single-observation sample standard deviation is undefined).
pub fn rolling_stdev(series: &QuantSeries, window: usize) -> Result<QuantSeries, QuantError> {
    if window < 2 {
        return Err(QuantError::InvalidArgument(
            "rolling stdev window must be at least 2",
        ));
    }
    let n = series.len();
    if window > n {
        return Err(QuantError::WindowLargerThanSeries { window, len: n });
    }
    let mut out = Vec::with_capacity(n - window + 1);
    for w in series.as_slice().windows(window) {
        out.push(sample_stdev(w)?);
    }
    Ok(QuantSeries::new(out))
}

// ---------------------------------------------------------------------------
// Summary statistics
// ---------------------------------------------------------------------------

/// Annualized Sharpe ratio: `(mean(returns) - risk_free_rate) / stdev(returns)`
/// scaled by the square root of the annualization factor.
///
/// Requires at least two returns and a strictly positive standard deviation
/// (a constant returns series has no well-defined Sharpe ratio and raises
/// [`QuantError::DivisionByZero`]).
pub fn sharpe_ratio(
    returns: &QuantSeries,
    risk_free_rate: Decimal,
    annualization_factor: Decimal,
) -> Result<Decimal, QuantError> {
    if annualization_factor < Decimal::ZERO {
        return Err(QuantError::InvalidArgument(
            "annualization_factor must be non-negative",
        ));
    }
    let n = returns.len();
    if n < 2 {
        return Err(QuantError::InsufficientData { needed: 2, got: n });
    }
    let std = sample_stdev(returns.as_slice())?;
    if std.is_zero() {
        return Err(QuantError::DivisionByZero);
    }
    let excess = mean(returns.as_slice()) - risk_free_rate;
    let scale = annualization_factor
        .sqrt()
        .ok_or(QuantError::DivisionByZero)?;
    Ok((excess / std) * scale)
}

/// Maximum drawdown of an equity/price series, as a positive fraction
/// (e.g. `0.25` for a 25% drawdown). Returns zero for a series that never
/// declines from its running peak.
///
/// `max_drawdown = max_t (peak[t] - value[t]) / peak[t]` over the running
/// peak. Requires at least one observation and a non-zero peak at every step.
pub fn max_drawdown(equity_curve: &QuantSeries) -> Result<Decimal, QuantError> {
    let n = equity_curve.len();
    if n == 0 {
        return Err(QuantError::InsufficientData { needed: 1, got: n });
    }
    let mut peak = equity_curve.as_slice()[0];
    let mut max_dd = Decimal::ZERO;
    for &v in equity_curve.as_slice() {
        if v > peak {
            peak = v;
        }
        if peak.is_zero() {
            return Err(QuantError::DivisionByZero);
        }
        let dd = (peak - v) / peak;
        if dd > max_dd {
            max_dd = dd;
        }
    }
    Ok(max_dd)
}

/// Pearson correlation between two equal-length series.
///
/// `corr(a, b) = cov(a, b) / (stdev(a) * stdev(b))`, which cancels the
/// `n - 1` denominators into `cov / sqrt(ss_a * ss_b)`. Requires equal
/// lengths of at least two and a non-zero variance in both series.
pub fn correlation(a: &QuantSeries, b: &QuantSeries) -> Result<Decimal, QuantError> {
    if a.len() != b.len() {
        return Err(QuantError::InvalidArgument("series lengths must match"));
    }
    let n = a.len();
    if n < 2 {
        return Err(QuantError::InsufficientData { needed: 2, got: n });
    }
    let ma = mean(a.as_slice());
    let mb = mean(b.as_slice());
    let mut cov = Decimal::ZERO;
    let mut ss_a = Decimal::ZERO;
    let mut ss_b = Decimal::ZERO;
    for i in 0..n {
        let da = a.as_slice()[i] - ma;
        let db = b.as_slice()[i] - mb;
        cov += da * db;
        ss_a += da * da;
        ss_b += db * db;
    }
    if ss_a.is_zero() || ss_b.is_zero() {
        return Err(QuantError::DivisionByZero);
    }
    let denom = (ss_a * ss_b).sqrt().ok_or(QuantError::DivisionByZero)?;
    Ok(cov / denom)
}

/// Beta of an asset's returns against a benchmark's returns:
/// `cov(asset, benchmark) / var(benchmark)` (both sample `n - 1` denominators
/// cancel). Requires equal lengths of at least two and non-zero benchmark
/// variance.
pub fn beta(
    asset_returns: &QuantSeries,
    benchmark_returns: &QuantSeries,
) -> Result<Decimal, QuantError> {
    if asset_returns.len() != benchmark_returns.len() {
        return Err(QuantError::InvalidArgument("series lengths must match"));
    }
    let n = asset_returns.len();
    if n < 2 {
        return Err(QuantError::InsufficientData { needed: 2, got: n });
    }
    let ma = mean(asset_returns.as_slice());
    let mb = mean(benchmark_returns.as_slice());
    let mut cov = Decimal::ZERO;
    let mut ss_b = Decimal::ZERO;
    for i in 0..n {
        let da = asset_returns.as_slice()[i] - ma;
        let db = benchmark_returns.as_slice()[i] - mb;
        cov += da * db;
        ss_b += db * db;
    }
    if ss_b.is_zero() {
        return Err(QuantError::DivisionByZero);
    }
    Ok(cov / ss_b)
}
