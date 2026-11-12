# Plan 0002 — Call-Site Consolidation: Delegate Downstream Quant Math to `vyapaar_core::quant`

**Status**: ready for implementation

## 0. Objective

Close gap 0001: eliminate duplicated quant transform implementations in
`vyapaar-market-indicators` and `vyapaar-agent-analytics` by delegating to the
canonical `vyapaar_core::quant` primitives implemented in plan 0001.

## 1. Gaps Covered

| Gap | Title | Severity |
|-----|-------|----------|
| [0001](../gaps/0001-call-site-consolidation.md) | Phase 2 call-site consolidation: delegate return/vol math to `vyapaar_core::quant` | Medium |

## 2. Implementation Phases

### Phase A — Audit downstream local math

1. Grep `vyapaar-market-indicators/src/` for local implementations of:
   simple/log returns, rolling mean/stdev, EMA/SMA, volatility, Sharpe ratio,
   max drawdown, correlation, beta.
2. Grep `vyapaar-agent-analytics/src/` for the same set of transforms.
3. Document each local implementation and its `vyapaar_core::quant` equivalent.

### Phase B — Replace local math with `vyapaar_core::quant` calls

1. Add the `vyapaar-core` dependency to `Cargo.toml` of each downstream crate
   (if not already present).
2. Replace local function bodies with calls to `vyapaar_core::quant::*`.
3. Delete now-unused local helper functions.
4. Ensure the zero-floats invariant: no `f32`/`f64` in the delegated call path.

### Phase C — Parity tests

1. Add integration tests in `vyapaar-market-indicators/tests/` and
   `vyapaar-agent-analytics/tests/` asserting identical outputs between
   `vyapaar_core::quant` and downstream indicators on shared golden vectors.
2. Update `docs/design/code-quant.md` with the delegation matrix.
3. Verify `cargo test --workspace --profile prod` passes across all affected
   crates.

## 3. Exit Criteria

- No local re-derivation of `simple_returns`, `log_returns`, `rolling_mean`,
  `rolling_stdev`, `realized_volatility`, `ewma_volatility`, `sharpe_ratio`,
  `max_drawdown`, `correlation`, `beta` remains in downstream crates.
- `cargo test --workspace --profile prod` green across `vyapaar-core`,
  `vyapaar-market-indicators`, `vyapaar-agent-analytics`.
- `cargo fmt --check` and `cargo clippy --workspace --all-targets --profile prod -- -D warnings` clean.

## 4. Related

- `vyapaar-core/src/quant.rs` — canonical quant primitives (plan 0001).
- `vyapaar-core/docs/design/code-quant.md` — the reference for the canonical API.
- `vyapaar-market-indicators/src/` — downstream indicator implementations.
- `vyapaar-agent-analytics/src/` — downstream analytics implementations.
- `.agents/AGENTS.md` §4.1 — "Consolidation Over Duplication" invariant.
