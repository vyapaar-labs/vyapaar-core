# Gap 0001 — Phase 2 Call-Site Consolidation: `vyapaar_core::quant` Delegation

## What's Missing

Plan 0001 implemented `src/quant.rs` with canonical quant transform primitives
(`QuantSeries`, `simple_returns`, `log_returns`, `realized_volatility`,
`ewma_volatility`, `rolling_mean`, `rolling_stdev`, `sharpe_ratio`,
`max_drawdown`, `correlation`, `beta`, `historical_var`, `historical_cvar`,
`tracking_error`, `QuantError`). These are exported from `lib.rs` with golden-vector
tests (`tests/quant_tests.rs`, 33 tests) and property-based invariants
(`tests/quant_proptest.rs`).

However, **downstream crates still re-derive these transforms locally** instead of
delegating to `vyapaar_core::quant`:

- `vyapaar-market-indicators` computes SMA/EMA/RSI/etc. with its own local math
- `vyapaar-agent-analytics` computes Sharpe/drawdown/volatility with its own local math

This duplication is the "Phase 2 call-site consolidation" tracked as future work in
the plan 0001 HANDOFF entry. Until consolidated, a bug fix or precision change in
one crate does not propagate to the others, violating the workspace
"Consolidation Over Duplication" invariant (`.agents/AGENTS.md` §4.1).

## Why It Matters

- **Consolidation over duplication**: The same mathematical transforms exist in
  multiple bounded contexts, creating drift risk.
- **Zero-floats consistency**: Downstream local math may inadvertently introduce
  `f64` intermediates; `vyapaar_core::quant` guarantees pure `Decimal` (via
  `rust_decimal`'s `maths` feature).
- **Deterministic replay**: Only one canonical implementation ensures byte-for-byte
  identical results across backtesting and live paths.

## How to Close

1. Audit `vyapaar-market-indicators/src/*.rs` and `vyapaar-agent-analytics/src/*.rs`
   for local implementations of returns, volatility, rolling stats, Sharpe, and
   drawdown.
2. Replace local math with calls to `vyapaar_core::quant` functions (add
   `vyapaar-core` dependency where missing).
3. Add integration tests asserting parity between `vyapaar_core::quant` outputs and
   downstream indicator outputs on shared golden vectors.
4. Update `docs/design/code-quant.md` with the delegation matrix.

See `docs/plans/0002-call-site-consolidation.md` for the tracking plan.
