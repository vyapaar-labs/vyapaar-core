# Plans Index — `vyapaar-core`

Per workspace rule `.agents/rules/gap-tracking.md`:
- Gaps are individual features (`docs/gaps/XXXX-*.md`).
- Plans are collections of gaps (`docs/plans/XXXX-*.md`).
- When implemented, plan documents are cleaned up from `docs/plans/` and recorded in this index.

## Active Plans

## Implemented Plans

| # | Title | Target Scope | Implemented In |
|---|-------|--------------|----------------|
| 0001 | Canonical Quant Transform Primitives (Returns, Volatility, Rolling Stats) | `src/quant.rs` | `QuantSeries`, `simple_returns`, `log_returns`, `realized_volatility`, `ewma_volatility`, `rolling_mean`, `rolling_stdev`, `sharpe_ratio`, `max_drawdown`, `correlation`, `beta`, `historical_var`, `historical_cvar`, `tracking_error`, and `QuantError` in `src/quant.rs`; exported from `lib.rs`; golden-vector tests in `tests/quant_tests.rs` (33) and property-based invariants in `tests/quant_proptest.rs`; commits `9ce4880` + `43a11f0`. Phase 2 call-site consolidation (delegating `vyapaar-market-indicators` / `vyapaar-agent-analytics` to these primitives) is tracked as future work and not part of this plan. |
