//! Property-based invariants for `vyapaar_core::quant` primitives.

use proptest::prelude::*;
use rust_decimal::Decimal;
use vyapaar_core::{
    QuantSeries, beta, correlation, log_returns, max_drawdown, rolling_mean, simple_returns,
};

/// A positive, strictly-increasing or constant decimal series for returns /
/// equity curves, free of division-by-zero landmines.
fn positive_series() -> impl Strategy<Value = Vec<Decimal>> {
    proptest::collection::vec(1u64..100_000u64, 1..30)
        .prop_map(|v| v.into_iter().map(Decimal::from).collect::<Vec<_>>())
}

/// A strictly-increasing series is a valid equity curve for `max_drawdown`.
fn strictly_increasing_series() -> impl Strategy<Value = Vec<Decimal>> {
    proptest::collection::vec(1u64..1_000u64, 2..30)
        .prop_map(|mut v| {
            v.sort_unstable();
            v.dedup();
            v.into_iter().map(Decimal::from).collect::<Vec<_>>()
        })
        .prop_filter("need at least 2 distinct points", |v| v.len() >= 2)
}

/// A strategy producing two positive decimal series of equal length.
fn equal_length_pair() -> impl Strategy<Value = (Vec<Decimal>, Vec<Decimal>)> {
    (2usize..30)
        .prop_flat_map(|n| {
            (
                proptest::collection::vec(1u64..100_000u64, n),
                proptest::collection::vec(1u64..100_000u64, n),
            )
        })
        .prop_map(|(a, b)| {
            (
                a.into_iter().map(Decimal::from).collect::<Vec<_>>(),
                b.into_iter().map(Decimal::from).collect::<Vec<_>>(),
            )
        })
}

proptest! {
    // simple_returns of a constant series are all zero.
    #[test]
    fn prop_simple_returns_of_constant_series_are_zero(
        seed in 1u64..100_000u64,
        n in 2usize..50,
    ) {
        let series: QuantSeries = (0..n).map(|_| Decimal::from(seed)).collect();
        let rets = simple_returns(&series).unwrap();
        prop_assert_eq!(rets.len(), n - 1);
        for r in rets.as_slice() {
            prop_assert_eq!(*r, Decimal::ZERO);
        }
    }

    // log_returns of a constant positive series are all zero.
    #[test]
    fn prop_log_returns_of_constant_series_are_zero(
        seed in 1u64..100_000u64,
        n in 2usize..50,
    ) {
        let series: QuantSeries = (0..n).map(|_| Decimal::from(seed)).collect();
        let rets = log_returns(&series).unwrap();
        prop_assert_eq!(rets.len(), n - 1);
        for r in rets.as_slice() {
            prop_assert_eq!(*r, Decimal::ZERO);
        }
    }

    // max_drawdown of a strictly increasing series is zero.
    #[test]
    fn prop_max_drawdown_of_increasing_series_is_zero(values in strictly_increasing_series()) {
        let series = QuantSeries::new(values);
        let dd = max_drawdown(&series).unwrap();
        prop_assert_eq!(dd, Decimal::ZERO);
    }

    // correlation(a, a) is 1 to within sqrt's decimal precision (the square
    // root of a non-perfect-square is only ever approximate).
    #[test]
    fn prop_correlation_self_is_one(values in positive_series()) {
        prop_assume!(values.len() >= 2);
        let series = QuantSeries::new(values);
        let c = correlation(&series, &series).unwrap();
        prop_assert_eq!(c.round_dp(6), Decimal::ONE);
    }

    // correlation is symmetric.
    #[test]
    fn prop_correlation_is_symmetric((a, b) in equal_length_pair()) {
        let sa = QuantSeries::new(a);
        let sb = QuantSeries::new(b);
        prop_assert_eq!(correlation(&sa, &sb), correlation(&sb, &sa));
    }

    // beta(a, a) == 1 for a series with non-zero variance.
    #[test]
    fn prop_beta_self_is_one(values in positive_series()) {
        prop_assume!(values.len() >= 2);
        let series = QuantSeries::new(values);
        let b = beta(&series, &series);
        prop_assert_eq!(b, Ok(Decimal::ONE));
    }

    // rolling_mean preserves length: n - window + 1, and a window of the whole
    // series equals the plain mean.
    #[test]
    fn prop_rolling_mean_length(values in positive_series(), window in 1usize..40) {
        prop_assume!(!values.is_empty());
        let series = QuantSeries::new(values);
        if window > series.len() {
            prop_assert!(rolling_mean(&series, window).is_err());
        } else {
            let out = rolling_mean(&series, window).unwrap();
            prop_assert_eq!(out.len(), series.len() - window + 1);
            if window == series.len() {
                let total: Decimal = series.as_slice().iter().sum();
                let expected = total / Decimal::from(series.len() as u64);
                prop_assert_eq!(out.as_slice()[0], expected);
            }
        }
    }
}
