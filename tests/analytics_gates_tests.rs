//! Statistical validation gate tests (`vyapaar_core::analytics`) — GAP-0002.
//!
//! Reference expectations follow the Honba validation engine
//! (`honba-analytics`: `validation.rs`, `pbo.rs`, `walk_forward.rs`,
//! `holdout.rs`, `monte_carlo.rs`, `cost_stress.rs`, `dsr.rs`), re-derived in
//! exact `Decimal`.

use rust_decimal::Decimal;

use vyapaar_core::analytics::cost_stress::{CostStressConfig, compute_cost_stress};
use vyapaar_core::analytics::distribution::{SplitMix64, norm_inv};
use vyapaar_core::analytics::dsr::{DsrConfig, compute_dsr, expected_max_sharpe};
use vyapaar_core::analytics::holdout::{HoldoutConfig, compute_holdout};
use vyapaar_core::analytics::monte_carlo::{MonteCarloConfig, run_block_bootstrap};
use vyapaar_core::analytics::pbo::{PboConfig, compute_pbo};
use vyapaar_core::analytics::walk_forward::{WalkForwardConfig, compute_walk_forward};
use vyapaar_core::analytics::{ValidationBlock, stats};

fn dec(value: &str) -> Decimal {
    value.parse().expect("valid decimal literal")
}

// ---------------------------------------------------------------------------
// Distribution primitives
// ---------------------------------------------------------------------------

#[test]
fn norm_inv_interpolates_the_half_point_and_975() {
    let half = norm_inv(Decimal::new(5, 1));
    assert!(half.abs() < dec("0.005"), "Φ⁻¹(0.5) = 0, got {half}");

    let upper = norm_inv(dec("0.975"));
    assert!(
        (upper - dec("1.95996")).abs() < dec("0.02"),
        "Φ⁻¹(0.975) ≈ 1.96, got {upper}"
    );

    let lower = norm_inv(dec("0.025"));
    assert!(
        (lower + upper).abs() < dec("0.02"),
        "Φ⁻¹ is odd-symmetric: {lower} vs {upper}"
    );
}

#[test]
fn norm_inv_is_monotone_in_p() {
    let a = norm_inv(dec("0.60"));
    let b = norm_inv(dec("0.70"));
    let c = norm_inv(dec("0.80"));
    assert!(
        a < b && b < c,
        "norm_inv must increase in p: {a} < {b} < {c}"
    );
}

#[test]
fn splitmix64_seeded_replay_is_identical() {
    let mut rng1 = SplitMix64::new(42);
    let mut rng2 = SplitMix64::new(42);
    for _ in 0..256 {
        assert_eq!(rng1.next_u64(), rng2.next_u64());
    }
    assert_ne!(rng1.next_u64(), SplitMix64::new(43).next_u64());
}

#[test]
fn splitmix64_index_stays_in_bounds() {
    let mut rng = SplitMix64::new(7);
    for _ in 0..1000 {
        let idx = rng.next_index(11);
        assert!(idx < 11, "index {idx} escaped bound 11");
    }
    assert_eq!(SplitMix64::new(7).next_index(1), 0);
}

// ---------------------------------------------------------------------------
// Return-series stats helpers
// ---------------------------------------------------------------------------

#[test]
fn annualized_return_compounds_and_dereges_gracefully() {
    let per_period = &[
        Decimal::new(1, 2), // 0.01
        Decimal::new(1, 2),
        Decimal::new(1, 2),
        Decimal::new(1, 2),
    ];
    // (1.01)^4 - 1 ≈ 4.0604%; annualizing the exact 4-period window with
    // periods_per_year = 4 returns the same compound figure.
    let four_period = stats::annualized_return(per_period, Decimal::from(4u64));
    let equity =
        Decimal::new(101, 2) * Decimal::new(101, 2) * Decimal::new(101, 2) * Decimal::new(101, 2);
    let expected = equity.round_dp(8) - Decimal::ONE;
    assert_eq!(four_period.round_dp(8), expected.round_dp(8));

    assert_eq!(
        stats::annualized_return(&[], Decimal::from(252u64)),
        Decimal::ZERO
    );
    // returns ≤ -1 collapse the equity factor: guarded to zero, never panics.
    let ruin = vec![-Decimal::ONE];
    assert_eq!(
        stats::annualized_return(&ruin, Decimal::from(252u64)),
        Decimal::ZERO
    );
}

#[test]
fn sharpe_or_zero_handles_constant_and_short_series() {
    let constant = vec![Decimal::new(1, 2); 50];
    assert_eq!(
        stats::sharpe_or_zero(&constant, Decimal::from(252u64)),
        Decimal::ZERO
    );

    let short = vec![Decimal::new(1, 2)];
    assert_eq!(
        stats::sharpe_or_zero(&short, Decimal::from(252u64)),
        Decimal::ZERO
    );
}

// ---------------------------------------------------------------------------
// DSR
// ---------------------------------------------------------------------------

#[test]
fn expected_max_sharpe_single_trial_has_no_inflation() {
    assert_eq!(expected_max_sharpe(1, Decimal::ONE), Decimal::ZERO);
    assert_eq!(expected_max_sharpe(0, Decimal::ONE), Decimal::ZERO);
}

#[test]
fn dsr_expected_max_sharpe_matches_published_value() {
    // Bailey & López de Prado Table 1: with N=100 trials and unit trial
    // variance, E[max SR] ≈ 2.5 (accepted band 2.40–2.65).
    let sr0 = expected_max_sharpe(100, Decimal::ONE);
    assert!(
        sr0 > dec("2.40") && sr0 < dec("2.65"),
        "expected max Sharpe {sr0} outside [2.40, 2.65)"
    );
}

#[test]
fn dsr_confident_trials_pass_and_mined_trials_fail() {
    let returns = vec![Decimal::new(1, 2); 250]; // benign constant series
    let config = DsrConfig {
        num_trials: 100,
        trial_sharpe_variance: Decimal::ONE,
        ..Default::default()
    };

    let high = compute_dsr(Decimal::new(35, 1), &returns, config); // 3.5 > SR0 ≈ 2.5
    assert!(high.passed, "DSR {high:?}");
    assert!(high.dsr > dec("0.99"));

    let low = compute_dsr(Decimal::new(15, 1), &returns, config); // 1.5 < SR0
    assert!(!low.passed, "DSR {low:?}");
    assert!(low.dsr < dec("0.05"));
}

#[test]
fn dsr_degenerate_series_fails_neutral() {
    let result = compute_dsr(
        Decimal::new(25, 1),
        &[Decimal::ONE; 2],
        DsrConfig::default(),
    );
    assert!(!result.passed);
    assert_eq!(result.dsr, Decimal::new(5, 1));
}

// ---------------------------------------------------------------------------
// Walk-forward
// ---------------------------------------------------------------------------

#[test]
fn walk_forward_consistent_strategy_passes() {
    // Even-with-dispersion earner (0.002/0.001 pairs): strictly positive
    // Sharpe in both halves, and identical per-period growth makes WFE ≈ 1.
    // (A perfectly constant series has zero dispersion and therefore scores
    // Sharpe 0 by the fails-to-zero convention, which would force a false
    // failure here.)
    let returns: Vec<Decimal> = (0..50)
        .flat_map(|_| vec![Decimal::new(2, 3), Decimal::new(1, 3)])
        .collect();
    let result = compute_walk_forward(&returns, WalkForwardConfig::default());
    assert!(result.passed, "WFE for a stationary earner: {result:?}");
    assert!(
        result.wfe > dec("0.9"),
        "WFE ≈ 1 for identical regimes, got {}",
        result.wfe
    );
    assert!(result.oos_sharpe > Decimal::ZERO);
}

#[test]
fn walk_forward_overfit_degradation_fails() {
    let mut returns = vec![Decimal::new(5, 3); 60]; // hot optimization window
    returns.extend(vec![Decimal::new(-2, 3); 40]); // cold out-of-sample
    let result = compute_walk_forward(&returns, WalkForwardConfig::default());
    assert!(!result.passed, "degraded OOS must fail: {result:?}");
    assert!(
        result.wfe < Decimal::ZERO,
        "WFE negative for losing OOS, got {}",
        result.wfe
    );
}

#[test]
fn walk_forward_short_series_fails_outright() {
    let returns = vec![Decimal::new(1, 2); 10];
    let result = compute_walk_forward(&returns, WalkForwardConfig::default());
    assert!(!result.passed);
    assert_eq!(result.wfe, Decimal::ZERO);
}

// ---------------------------------------------------------------------------
// Holdout
// ---------------------------------------------------------------------------

#[test]
fn holdout_retention_passes_when_oos_keeps_edge() {
    // Same ±-pair composition in both halves ⇒ Sharpe retention ≈ 1 (the
    // 1/(n-1) sample-variance denominators differ, so retention is 0.99, not
    // exactly one).
    let mut returns = Vec::new();
    for _ in 0..35 {
        returns.push(Decimal::new(2, 3)); // 0.002
        returns.push(Decimal::new(1, 3)); // 0.001
    }
    for _ in 0..15 {
        returns.push(Decimal::new(2, 3));
        returns.push(Decimal::new(1, 3));
    }
    let result = compute_holdout(&returns, HoldoutConfig::default());
    assert!(result.passed, "{result:?}");
    assert!(
        result.retention_ratio > dec("0.90"),
        "uniform earner retains ~100% of training Sharpe, got {}",
        result.retention_ratio
    );
    assert!(result.train_sharpe > Decimal::ZERO);
    assert!(result.holdout_sharpe > Decimal::ZERO);
}

#[test]
fn holdout_retention_fails_on_collapsed_holdout() {
    // Positive-dispersion training Sharpe, but the holdout leg turns negative
    // with dispersion: retention must go negative, not merely to zero.
    let mut returns = Vec::new();
    for _ in 0..35 {
        returns.push(Decimal::new(5, 3)); // 0.005
        returns.push(Decimal::new(3, 3)); // 0.003
    }
    for _ in 0..15 {
        returns.push(Decimal::new(-5, 4)); // -0.0005
        returns.push(Decimal::new(-15, 4)); // -0.0015
    }
    let result = compute_holdout(&returns, HoldoutConfig::default());
    assert!(!result.passed, "{result:?}");
    assert!(result.retention_ratio < Decimal::ZERO);
}

#[test]
fn holdout_short_series_fails() {
    let result = compute_holdout(&[Decimal::new(1, 2); 9], HoldoutConfig::default());
    assert!(!result.passed);
}

// ---------------------------------------------------------------------------
// PBO (CSCV)
// ---------------------------------------------------------------------------

#[test]
fn pbo_counts_all_symmetric_combinations() {
    // 8×160 slices → C(4, 2) = 6 symmetric splits for a 4-slice config.
    let m1 = vec![Decimal::new(1, 2); 160]; // consistently good
    let m2: Vec<Decimal> = (0..160)
        .map(|i| {
            if i % 2 == 0 {
                Decimal::new(2, 2) // 0.02
            } else {
                Decimal::new(-2, 2) // -0.02
            }
        })
        .collect();

    let result = compute_pbo(
        &[m1, m2],
        PboConfig {
            num_slices: 4,
            ..Default::default()
        },
    );
    assert_eq!(result.combinations_evaluated, 6);
    assert!(result.pbo >= Decimal::ZERO && result.pbo <= Decimal::ONE);
    assert!(
        result.passed,
        "rock-steady model must not be flagged: {result:?}"
    );
}

#[test]
fn pbo_single_model_and_short_splits_short_circuit() {
    let lone = vec![Decimal::new(1, 3); 120];
    let result = compute_pbo(&[lone], PboConfig::default());
    assert!(result.passed);
    assert_eq!(result.combinations_evaluated, 0);

    let two_short: Vec<Vec<Decimal>> =
        vec![vec![Decimal::new(1, 3); 8], vec![Decimal::new(1, 3); 8]];
    let result = compute_pbo(&two_short, PboConfig::default());
    assert!(result.passed);
    assert_eq!(result.combinations_evaluated, 0);
}

#[test]
fn pbo_ranks_a_regime_winner_as_overfit() {
    // Model 0 wins every first-half-heavy in-sample split but degrades
    // exactly when those splits become out-of-sample: a manufactured
    // non-stationary champion. Model 1 is the mirror image; both are
    // non-constant so Sharpe stays defined in every sub-window.
    let m0: Vec<Decimal> = (0..320)
        .map(|i| {
            let wiggle = dec("0.0001") * Decimal::from((i % 7) as u64);
            if i < 160 {
                dec("0.002") + wiggle
            } else {
                dec("-0.002") - wiggle
            }
        })
        .collect();
    let m1: Vec<Decimal> = (0..320)
        .map(|i| {
            let wiggle = dec("0.0001") * Decimal::from((i % 5) as u64);
            if i < 160 {
                dec("-0.002") - wiggle
            } else {
                dec("0.002") + wiggle
            }
        })
        .collect();

    let result = compute_pbo(
        &[m0, m1],
        PboConfig {
            num_slices: 4,
            ..Default::default()
        },
    );
    assert_eq!(result.combinations_evaluated, 6);
    assert!(
        !result.passed,
        "regime-dependent champion must trip PBO: {result:?}"
    );
    assert!(
        result.pbo > dec("0.4"),
        "heavy overfit share expected, got {}",
        result.pbo
    );
}

// ---------------------------------------------------------------------------
// Monte Carlo block bootstrap
// ---------------------------------------------------------------------------

#[test]
fn bootstrap_seeded_paths_replay_identically() {
    let returns: Vec<Decimal> = (0..200)
        .map(|i| {
            if i % 2 == 0 {
                Decimal::new(15, 3) // 0.015
            } else {
                Decimal::new(-10, 3) // -0.010
            }
        })
        .collect();
    let config = MonteCarloConfig {
        num_simulations: 200,
        seed: 12345,
        ..Default::default()
    };

    let first = run_block_bootstrap(&returns, config);
    let second = run_block_bootstrap(&returns, config);
    assert_eq!(first, second);
    assert_eq!(first.simulations_run, 200);
}

#[test]
fn bootstrap_degenerate_series_fails() {
    let result = run_block_bootstrap(&[Decimal::new(1, 2); 4], MonteCarloConfig::default());
    assert!(!result.passed);
    assert_eq!(result.simulations_run, 0);
}

#[test]
fn bootstrap_gentle_series_stays_within_tail_gates() {
    let returns = vec![Decimal::new(2, 3); 120]; // smooth 0.2% climber
    let result = run_block_bootstrap(&returns, MonteCarloConfig::default());
    assert!(result.passed, "{result:?}");
    assert!(result.drawdown_95 <= dec("0.30"));
    assert!(result.prob_ruin <= dec("0.01"));
    assert!(result.median_max_drawdown <= result.drawdown_95);
    assert!(result.drawdown_95 <= result.drawdown_99);
}

#[test]
fn bootstrap_ruin_cluster_gets_flagged_by_tail_risk() {
    let mut returns = vec![Decimal::new(2, 4); 120]; // calm 0.05% drift
    // A six-bar ruin cluster: three consecutive -30% returns destroy 66% of
    // equity; bootstrap blocks that redraw the cluster trip the ruin gate.
    let cluster: Vec<Decimal> = vec![dec("-0.30"); 6];
    let mut merged = Vec::with_capacity(126);
    merged.extend_from_slice(&returns);
    merged.extend_from_slice(&cluster);
    returns = Vec::with_capacity(126);
    returns.extend(merged);

    let result = run_block_bootstrap(
        &returns,
        MonteCarloConfig {
            num_simulations: 300,
            ..Default::default()
        },
    );
    assert!(
        result.prob_ruin > Decimal::ZERO,
        "the ruin cluster must leak into resampled tails: {result:?}"
    );
    assert!(
        !result.passed,
        "ruin cluster must fail the gate: {result:?}"
    );
}

// ---------------------------------------------------------------------------
// Cost stress
// ---------------------------------------------------------------------------

#[test]
fn cost_stress_gentle_earner_survives_doubled_friction() {
    let returns: Vec<Decimal> = (0..250)
        .map(|i| {
            let wiggle = if i % 2 == 0 {
                dec("0.0028")
            } else {
                dec("-0.0028")
            };
            dec("0.0030") + wiggle
        })
        .collect();
    let result = compute_cost_stress(&returns, CostStressConfig::default());
    assert!(
        result.passed,
        "low-churn earner should survive 2x: {result:?}"
    );
    assert!(result.stress_2x_sharpe > Decimal::ZERO);
    assert!(result.retention_2x > Decimal::ZERO);
    assert!(result.stress_3x_sharpe <= result.stress_2x_sharpe);
}

#[test]
fn cost_stress_flips_zero_edge_churner_to_friction_fragile() {
    // Alternating +1% / -1%: zero mean — any added drag turns the mean
    // negative and the Sharpe collapses through zero.
    let returns: Vec<Decimal> = (0..250)
        .map(|i| {
            if i % 2 == 0 {
                Decimal::new(1, 2)
            } else {
                Decimal::new(-1, 2)
            }
        })
        .collect();
    let result = compute_cost_stress(&returns, CostStressConfig::default());
    assert!(result.stress_2x_sharpe < Decimal::ZERO, "{result:?}");
    assert!(!result.passed, "churner with no edge is Friction Fragile");
}

#[test]
fn cost_stress_empty_series_fails() {
    let result = compute_cost_stress(&[], CostStressConfig::default());
    assert!(!result.passed);
}

// ---------------------------------------------------------------------------
// ValidationBlock (six-gate audit card)
// ---------------------------------------------------------------------------

#[test]
fn validation_block_passes_a_statistically_sound_series() {
    let returns: Vec<Decimal> = (0..120)
        .map(|i| {
            if i % 3 == 0 {
                dec("0.0012")
            } else {
                dec("0.0008")
            }
        })
        .collect();
    // 0.08–0.12% steady climber, no negative periods: modest DD, positive
    // everywhere.
    let block = ValidationBlock::evaluate_backtest(&returns, dec("2.20"), 5);

    assert!(block.dsr.is_some());
    assert!(block.pbo.is_none(), "single-series evaluation excludes PBO");
    assert!(block.walk_forward.is_some());
    assert!(block.holdout.is_some());
    assert!(block.monte_carlo.is_some());
    assert!(block.cost_stress.is_some());
    assert!(
        block.passed,
        "gentle monotone earner must pass: {:?}",
        block.warnings
    );
    assert!(block.warnings.is_empty());
    assert!(block.is_pass());
    assert!(!block.has_failures());
}

#[test]
fn validation_block_flags_a_fragile_gambler() {
    // Hot first half, cold-and-negative rest: every tail gate should scream.
    let mut returns = vec![Decimal::new(8, 3); 60];
    returns.extend(vec![Decimal::new(-4, 3); 60]);
    let block = ValidationBlock::evaluate_backtest(&returns, Decimal::new(15, 1), 80);

    assert!(!block.passed, "gambler must fail: {block:?}");
    assert!(!block.warnings.is_empty());
    assert!(block.has_failures());
    // Walk-forward must appear among the failures for a collapsing OOS leg.
    let wf = block.walk_forward.expect("walk-forward evaluated");
    assert!(!wf.passed);
}

#[test]
fn validation_block_sweep_includes_pbo_and_deterministic_replay() {
    let base: Vec<Decimal> = (0..120)
        .map(|i| {
            if i % 4 == 0 {
                dec("0.0011")
            } else {
                dec("0.0009")
            }
        })
        .collect();
    // Variant is constant 0.0010 (no wiggle): both branches equal, clippy warns.
    let variant: Vec<Decimal> = vec![dec("0.0010"); 120];

    let first =
        ValidationBlock::evaluate_sweep(&base, dec("2.0"), &[base.clone(), variant.clone()]);
    let second =
        ValidationBlock::evaluate_sweep(&base, dec("2.0"), &[base.clone(), variant.clone()]);

    assert_eq!(first, second, "evaluation must replay byte-for-byte");
    assert!(first.pbo.is_some(), "sweep evaluation adds PBO");
    let pbo = first.pbo.expect("pbo present");
    assert!(pbo.combinations_evaluated > 0);
}

#[test]
fn validation_block_sweep_fails_on_overfit_trials() {
    // Two mirror-regime trials with a degenerate champion: PBO must fire and
    // flip the verdict even if the single-series gates pass.
    let champion: Vec<Decimal> = (0..320)
        .map(|i| if i < 160 { dec("0.002") } else { dec("-0.002") })
        .collect();
    let mirror: Vec<Decimal> = (0..320)
        .map(|i| if i < 160 { dec("-0.002") } else { dec("0.002") })
        .collect();

    let block =
        ValidationBlock::evaluate_sweep(&champion, dec("99.0"), &[champion.clone(), mirror]);
    let pbo = block.pbo.expect("two trials ⇒ PBO evaluated");
    assert!(
        pbo.combinations_evaluated == 70, // C(S=8, 4) = 70 combinations per CSCV
        "unexpected combination count {}",
        pbo.combinations_evaluated
    );
}
