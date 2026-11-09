//! Return-series statistics shared by the statistical validation gates.
//!
//! Thin, exact-`Decimal` helpers built on the canonical [`crate::quant`]
//! transforms (single source of truth — no local re-derivation of variance or
//! Sharpe). Mirrors the conventions the validation gates expect:
//!
//! - geometric (compound) annualized return via `powd` (`e^(y·ln x)`, pure
//!   `Decimal`, no `f64`),
//! - annualized Sharpe ratio with fails-to-zero semantics for degenerate
//!   series (fewer than two observations or zero return dispersion),
//! - maximum peak-to-trough drawdown as a positive fraction.

use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;

use crate::quant::{QuantError, QuantSeries, max_drawdown, sharpe_ratio};
/// Geometric annualized return of a periodic-returns series:
/// `(∏(1 + r))^(periods_per_year / n) - 1`.
///
/// Returns `0` for an empty series or a non-positive cumulative equity factor.
/// Uses `checked_powd` (`e^(y·ln x)`), which the `maths` feature evaluates in
/// pure `Decimal`, so no float enters the computation.
pub fn annualized_return(returns: &[Decimal], periods_per_year: Decimal) -> Decimal {
    if returns.is_empty() {
        return Decimal::ZERO;
    }
    let mut equity_factor = Decimal::ONE;
    for &r in returns {
        equity_factor *= Decimal::ONE + r;
    }
    if equity_factor <= Decimal::ZERO {
        return Decimal::ZERO;
    }
    let n = Decimal::from(returns.len() as u64);
    let exponent = periods_per_year / n;
    equity_factor
        .checked_powd(exponent)
        .map(|annual| annual - Decimal::ONE)
        .unwrap_or(Decimal::ZERO)
}

/// Annualized Sharpe ratio of a returns series with fails-to-zero semantics:
/// fewer than two observations, a zero-dispersion (constant) series, or a
/// negative annualization factor all yield exactly `0` — mirroring the
/// "degenerate input ⇒ zero score" convention the validation gates need so a
/// sub-window can never win a ranking bootstrapped from an undefined Sharpe.
pub fn sharpe_or_zero(returns: &[Decimal], annualization_factor: Decimal) -> Decimal {
    match sharpe_ratio(
        &QuantSeries::new(returns.to_vec()),
        Decimal::ZERO,
        annualization_factor,
    ) {
        Ok(s) => s,
        Err(QuantError::InsufficientData { .. }) | Err(QuantError::DivisionByZero) => Decimal::ZERO,
        Err(QuantError::InvalidArgument(_)) | Err(QuantError::WindowLargerThanSeries { .. }) => {
            Decimal::ZERO
        }
    }
}

/// Maximum peak-to-trough drawdown of an equity curve reconstructed from the
/// returns series, as a positive fraction; `0` for any degenerate input.
///
/// The equity path is `∏(1 + r)`, starting at `1`, and [`max_drawdown`] is the
/// canonical kernel implementation.
pub fn max_drawdown_fraction(returns: &[Decimal]) -> Decimal {
    if returns.is_empty() {
        return Decimal::ZERO;
    }
    let mut equity: Vec<Decimal> = Vec::with_capacity(returns.len() + 1);
    let mut level = Decimal::ONE;
    equity.push(level);
    for &r in returns {
        level *= Decimal::ONE + r;
        equity.push(level);
    }
    max_drawdown(&QuantSeries::new(equity)).unwrap_or(Decimal::ZERO)
}
