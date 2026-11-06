//! Golden-vector and edge-case tests for `vyapaar_core::quant`.
//!
//! Golden vectors are hand-computed closed-form values (or identities) so a
//! regression in any primitive fails loudly.

use rust_decimal::Decimal;
use vyapaar_core::{QuantError, QuantSeries, *};

/// Asserts `a` and `b` agree to `digits` decimal places.
fn assert_close_dp(a: Decimal, b: Decimal, digits: u32) {
    let ra = a.round_dp(digits);
    let rb = b.round_dp(digits);
    assert_eq!(ra, rb, "expected {b} (rounded to {digits} dp), got {a}");
}

fn d(s: &str) -> Decimal {
    s.parse().unwrap()
}

// ---------------------------------------------------------------------------
// simple_returns
// ---------------------------------------------------------------------------

#[test]
fn simple_returns_golden() {
    let series: QuantSeries = vec![d("100"), d("110"), d("99")].into();
    let rets = simple_returns(&series).unwrap();
    assert_eq!(
        rets.as_slice(),
        &[d("0.1"), d("-0.1")],
        "(110-100)/100 = 0.1; (99-110)/110 = -0.1"
    );
}

#[test]
fn simple_returns_requires_two_points() {
    let series: QuantSeries = vec![d("100")].into();
    assert_eq!(
        simple_returns(&series).unwrap_err(),
        QuantError::InsufficientData { needed: 2, got: 1 }
    );
    let empty: QuantSeries = vec![].into();
    assert_eq!(
        simple_returns(&empty).unwrap_err(),
        QuantError::InsufficientData { needed: 2, got: 0 }
    );
}

#[test]
fn simple_returns_rejects_zero_prior_price() {
    let series: QuantSeries = vec![d("0"), d("10")].into();
    assert_eq!(
        simple_returns(&series).unwrap_err(),
        QuantError::DivisionByZero
    );
}

// ---------------------------------------------------------------------------
// log_returns
// ---------------------------------------------------------------------------

#[test]
fn log_returns_golden() {
    // ln(110/100) = ln(1.1), ln(99/110) = ln(0.9); reference values to 20 dp.
    let series: QuantSeries = vec![d("100"), d("110"), d("99")].into();
    let rets = log_returns(&series).unwrap();
    assert_eq!(rets.len(), 2);
    assert_close_dp(rets[0], d("0.09531017980432486004"), 10);
    assert_close_dp(rets[1], d("-0.10536051565782630122"), 10);
}

#[test]
fn log_returns_rejects_non_positive_prices() {
    let series: QuantSeries = vec![d("100"), d("0")].into();
    assert_eq!(
        log_returns(&series).unwrap_err(),
        QuantError::DivisionByZero
    );
    let series: QuantSeries = vec![d("100"), d("-5")].into();
    assert_eq!(
        log_returns(&series).unwrap_err(),
        QuantError::DivisionByZero
    );
}

// ---------------------------------------------------------------------------
// realized_volatility
// ---------------------------------------------------------------------------

#[test]
fn realized_volatility_of_constant_returns_is_zero() {
    let returns: QuantSeries = vec![d("0.01"), d("0.01"), d("0.01")].into();
    assert_eq!(
        realized_volatility(&returns, Decimal::from(252)).unwrap(),
        Decimal::ZERO
    );
}

#[test]
fn realized_volatility_requires_two_returns() {
    let returns: QuantSeries = vec![d("0.01")].into();
    assert_eq!(
        realized_volatility(&returns, Decimal::from(252)).unwrap_err(),
        QuantError::InsufficientData { needed: 2, got: 1 }
    );
}

#[test]
fn realized_volatility_rejects_negative_annualization() {
    let returns: QuantSeries = vec![d("0.01"), d("-0.01")].into();
    assert_eq!(
        realized_volatility(&returns, d("-1")).unwrap_err(),
        QuantError::InvalidArgument("annualization_factor must be non-negative")
    );
}

// ---------------------------------------------------------------------------
// ewma_volatility
// ---------------------------------------------------------------------------

#[test]
fn ewma_volatility_constant_returns_is_their_value() {
    // var stays at 0.0004 under any lambda in (0, 1); sqrt(0.0004) = 0.02.
    let returns: QuantSeries = vec![d("0.02"), d("0.02"), d("0.02")].into();
    assert_eq!(ewma_volatility(&returns, d("0.94")).unwrap(), d("0.02"));
}

#[test]
fn ewma_volatility_decays_towards_latest_shocks() {
    // var[0] = 0.04; var[1] = 0.5*0.04 + 0.5*0 = 0.02; sqrt(0.02) to 10 dp.
    let returns: QuantSeries = vec![d("0.2"), d("0")].into();
    assert_close_dp(
        ewma_volatility(&returns, d("0.5")).unwrap(),
        d("0.1414213562373095"),
        10,
    );
}

#[test]
fn ewma_volatility_rejects_out_of_range_lambda() {
    let returns: QuantSeries = vec![d("0.01"), d("0.02")].into();
    assert_eq!(
        ewma_volatility(&returns, d("1")).unwrap_err(),
        QuantError::InvalidArgument("lambda must lie in the open interval (0, 1)")
    );
    assert_eq!(
        ewma_volatility(&returns, d("0")).unwrap_err(),
        QuantError::InvalidArgument("lambda must lie in the open interval (0, 1)")
    );
    assert_eq!(
        ewma_volatility(&returns, d("-0.1")).unwrap_err(),
        QuantError::InvalidArgument("lambda must lie in the open interval (0, 1)")
    );
}

// ---------------------------------------------------------------------------
// rolling_mean / rolling_stdev
// ---------------------------------------------------------------------------

#[test]
fn rolling_mean_golden() {
    let series: QuantSeries = vec![d("1"), d("2"), d("3"), d("4")].into();
    assert_eq!(
        rolling_mean(&series, 3).unwrap().as_slice(),
        &[d("2"), d("3")]
    );
}

#[test]
fn rolling_stdev_golden() {
    // Window of 2 over [0, 2]: sample stdev = sqrt(2).
    let series: QuantSeries = vec![d("0"), d("2")].into();
    let out = rolling_stdev(&series, 2).unwrap();
    assert_eq!(out.len(), 1);
    assert_close_dp(out[0], d("1.41421356237309504880"), 10);
}

#[test]
fn rolling_errors() {
    let series: QuantSeries = vec![d("1"), d("2"), d("3")].into();
    assert_eq!(
        rolling_mean(&series, 0).unwrap_err(),
        QuantError::InvalidArgument("rolling window must be positive")
    );
    assert_eq!(
        rolling_mean(&series, 4).unwrap_err(),
        QuantError::WindowLargerThanSeries { window: 4, len: 3 }
    );
    assert_eq!(
        rolling_stdev(&series, 1).unwrap_err(),
        QuantError::InvalidArgument("rolling stdev window must be at least 2")
    );
    assert_eq!(
        rolling_stdev(&series, 4).unwrap_err(),
        QuantError::WindowLargerThanSeries { window: 4, len: 3 }
    );
}

// ---------------------------------------------------------------------------
// sharpe_ratio
// ---------------------------------------------------------------------------

#[test]
fn sharpe_ratio_zero_excess_is_zero() {
    let returns: QuantSeries = vec![d("0"), d("0.04")].into();
    // mean = 0.02, risk_free = 0.02 => excess = 0 => ratio 0 regardless of scale.
    assert_eq!(sharpe_ratio(&returns, d("0.02"), d("4")).unwrap(), d("0"));
}

#[test]
fn sharpe_ratio_constant_returns_is_division_by_zero() {
    let returns: QuantSeries = vec![d("0.01"), d("0.01")].into();
    assert_eq!(
        sharpe_ratio(&returns, d("0"), d("252")).unwrap_err(),
        QuantError::DivisionByZero
    );
}

#[test]
fn sharpe_ratio_requires_two_returns() {
    let returns: QuantSeries = vec![d("0.01")].into();
    assert_eq!(
        sharpe_ratio(&returns, d("0"), d("252")).unwrap_err(),
        QuantError::InsufficientData { needed: 2, got: 1 }
    );
}

// ---------------------------------------------------------------------------
// max_drawdown
// ---------------------------------------------------------------------------

#[test]
fn max_drawdown_golden() {
    // Peaks: 100, 100, 100, 100, 120; drawdowns 0, 0.2, 0.1, 0.3, 0.
    let equity: QuantSeries = vec![d("100"), d("80"), d("90"), d("70"), d("120")].into();
    assert_eq!(max_drawdown(&equity).unwrap(), d("0.3"));
}

#[test]
fn max_drawdown_monotonic_is_zero() {
    let equity: QuantSeries = vec![d("100"), d("110"), d("120")].into();
    assert_eq!(max_drawdown(&equity).unwrap(), d("0"));
}

#[test]
fn max_drawdown_requires_data_and_non_zero_peak() {
    let empty: QuantSeries = vec![].into();
    assert_eq!(
        max_drawdown(&empty).unwrap_err(),
        QuantError::InsufficientData { needed: 1, got: 0 }
    );
    let zero_peak: QuantSeries = vec![d("0"), d("-10")].into();
    assert_eq!(
        max_drawdown(&zero_peak).unwrap_err(),
        QuantError::DivisionByZero
    );
}

// ---------------------------------------------------------------------------
// correlation / beta
// ---------------------------------------------------------------------------

#[test]
fn correlation_self_is_one() {
    let a: QuantSeries = vec![d("1"), d("2"), d("3"), d("4")].into();
    assert_eq!(correlation(&a, &a).unwrap(), Decimal::ONE);
}

#[test]
fn correlation_anti_correlated_is_minus_one() {
    let a: QuantSeries = vec![d("1"), d("2"), d("3"), d("4")].into();
    let b: QuantSeries = vec![d("4"), d("3"), d("2"), d("1")].into();
    assert_eq!(correlation(&a, &b).unwrap(), d("-1"));
}

#[test]
fn beta_perfect_multiple_is_the_multiple() {
    let asset: QuantSeries = vec![d("2"), d("4"), d("6"), d("8")].into();
    let benchmark: QuantSeries = vec![d("1"), d("2"), d("3"), d("4")].into();
    assert_eq!(beta(&asset, &benchmark).unwrap(), d("2"));
}

#[test]
fn correlation_and_beta_reject_mismatched_lengths() {
    let a: QuantSeries = vec![d("1"), d("2")].into();
    let b: QuantSeries = vec![d("1")].into();
    assert_eq!(
        correlation(&a, &b).unwrap_err(),
        QuantError::InvalidArgument("series lengths must match")
    );
    assert_eq!(
        beta(&a, &b).unwrap_err(),
        QuantError::InvalidArgument("series lengths must match")
    );
}

#[test]
fn correlation_of_constant_series_is_division_by_zero() {
    let constant: QuantSeries = vec![d("1"), d("1"), d("1")].into();
    let other: QuantSeries = vec![d("1"), d("2"), d("3")].into();
    assert_eq!(
        correlation(&constant, &other).unwrap_err(),
        QuantError::DivisionByZero
    );
}

#[test]
fn beta_constant_benchmark_is_division_by_zero() {
    let asset: QuantSeries = vec![d("1"), d("2"), d("3")].into();
    let benchmark: QuantSeries = vec![d("1"), d("1"), d("1")].into();
    assert_eq!(
        beta(&asset, &benchmark).unwrap_err(),
        QuantError::DivisionByZero
    );
}

// ---------------------------------------------------------------------------
// QuantSeries conversions
// ---------------------------------------------------------------------------

#[test]
fn quant_series_from_price_and_money_slices() {
    let prices = [
        Price::from_rupees("100").unwrap(),
        Price::from_rupees("110").unwrap(),
    ];
    let series = QuantSeries::from(&prices[..]);
    assert_eq!(series.as_slice(), &[d("100"), d("110")]);

    let money = [Money::inr(d("1000")), Money::inr(d("2000"))];
    let series = QuantSeries::from(&money[..]);
    assert_eq!(series.as_slice(), &[d("1000"), d("2000")]);
}

#[test]
fn quant_series_derefs_and_collects() {
    let series: QuantSeries = [d("1"), d("2"), d("3")].into_iter().collect();
    assert_eq!(series.len(), 3);
    assert!(!series.is_empty());
    assert_eq!(series.get(1), Some(&d("2")));
    assert_eq!(series[2], d("3")); // Deref to slice
    assert_eq!(series.into_inner(), vec![d("1"), d("2"), d("3")]);
}
