//! Deterministic distribution primitives for the statistical validation gates.
//!
//! Everything here is pure math: a seeded 64-bit pseudo-random generator used
//! by the block-bootstrap Monte Carlo (its outputs are integer *indices*, never
//! financial quantities) and the inverse standard-normal CDF needed by the
//! Deflated Sharpe Ratio.
//!
//! # Zero-floats note
//!
//! [`norm_inv`] is computed by bisection against
//! `rust_decimal::MathematicalOps::norm_cdf`, which is itself evaluated in
//! pure `Decimal` (192-bit intermediates inside `erf`). There is no
//! `f32`/`f64` anywhere in this file — statistical precision is exchange-grade
//! exact and every run replays byte-for-byte.

use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;

/// Deterministic 64-bit SplitMix pseudo-random number generator.
///
/// statistical properties follow Steele & Lea (2014) / splitmix64 as used in
/// Java 17+ `SplittableRandom`; chosen because it is tiny, thread-free,
/// allocation-free, and — for gate audits — the reproducibility contract
/// matters far more than cryptographic strength. A `seed` of `0` maps to a
/// non-zero default state so the very first draw is never degenerate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// Creates a generator initialized with `seed`. Seed `0` maps to a fixed
    /// non-zero state (`0xda942042e4dd58b5`).
    pub fn new(seed: u64) -> Self {
        let state = if seed == 0 { 0xda942042e4dd58b5 } else { seed };
        Self { state }
    }

    /// Returns the next pseudorandom `u64`.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Returns a uniformly distributed index in `0..upper` (zero for
    /// degenerate bounds).
    pub fn next_index(&mut self, upper: usize) -> usize {
        if upper <= 1 {
            0
        } else {
            (self.next_u64() % (upper as u64)) as usize
        }
    }
}

/// Inverse standard normal CDF Φ⁻¹(p), computed by bisection against the
/// exact-`Decimal` [`MathematicalOps::norm_cdf`].
///
/// `p` outside `(0, 1)` is clamped into the interior of that domain. Accuracy
/// is bounded by the underlying CDF approximation's error; far beyond what any
/// decision threshold in [`crate::analytics`] needs. Deterministic.
pub fn norm_inv(p: Decimal) -> Decimal {
    let lo_bound = Decimal::new(1, 28);
    let p = p.clamp(lo_bound, Decimal::ONE - lo_bound);
    let mut lo = Decimal::new(-12, 0);
    let mut hi = Decimal::new(12, 0);
    // Bisection: 72 halvings drive the interval to ≲1e-19 wide, past the 28
    // significant decimal digits Decimal can represent.
    for _ in 0..72 {
        let mid = (lo + hi) / Decimal::TWO;
        if mid.norm_cdf() < p {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) / Decimal::TWO
}
