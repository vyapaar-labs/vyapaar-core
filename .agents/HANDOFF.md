# HANDOFF — vyapaar-core

**Status**: Tier 0 complete; plan 0001 + analytics gates merged. Docs rewritten
this change.

## Phase / Slice Status
- **Tier 0 (COMPLETE)**: Shared Kernel implemented (migration + plan 0001 +
  analytics gates). Modules: `types`, `ids`, `error`, `instrument`, `market`,
  `order`, `decision`, `events`, `wire`, `calendar`, `portfolio`, `screener`,
  `signal`, `ports`, `sizing`, `quant`, `analytics/`. `schema/` golden +
  compatibility vectors. `cargo test --profile prod` green.
- **Docs (COMPLETE, this change)**: `docs/roadmap/` dismantled. Completed
  milestones folded into `docs/design/` (architecture status + module
  references); only remaining item (call-site consolidation) stays as gap 0001
  tracked by plan 0002. Added `code-ids.md` and `code-wire-contracts.md` so
  every module in `src/` maps to a design doc. Corrected `code-events.md`
  (`OrderEventKind` = Acked/Filled/Rejected/Cancelled; `TraceContext` does
  derive serde + JsonSchema) and `code-decision.md` (`validate()` requires
  `algo_id` on `CancelOrder` too).

## Decisions & Discoveries
- Zero-floats: `rust_decimal` `maths` feature keeps `ln`/`sqrt` pure `Decimal`
  (no `f64` intermediates, even in `quant.rs`).
- Zero-I/O purity + zero inline tests in `src/` verified.
- Wire contracts: `SCHEMA_VERSION = 8`, `API_VERSION = "v1"`; generated schema
  files are never hand-edited (`make codegen`).
- Gap 0001 open: `vyapaar-market-indicators` / `vyapaar-agent-analytics` still
  re-derive return/vol/Sharpe/drawdown locally instead of delegating to
  `vyapaar_core::quant`.

## Immediate Next Step
- Execute plan `0002` (gap 0001): audit downstream local math, delegate to
  `vyapaar_core::quant`, add parity tests on shared golden vectors.

## Operating Invariants
- Canonical profile `prod`: `cargo build/check/test --profile prod`.
- Tests in `tests/` parallel to `src/`; no in-file `#[test]` blocks.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets --profile prod -- -D warnings`, `cargo test --workspace --profile prod` green.
- Docs reflect the actual source tree (`docs/design/` per module; `docs/gaps/`
  per stub; `docs/plans/` per roadmap item).
