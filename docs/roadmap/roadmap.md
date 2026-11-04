# `vyapaar-core` — TDD & Engineering Roadmap

## Milestone 1: Core Value Objects & Identifier Unification (P0)
- [x] Port `types.rs`, `instrument.rs`, `error.rs`, and `wire.rs` from `honba-core`.
- [x] Add `ids.rs` lifting `ClientOrderId`, `AlgoId`, `AccountId`, and `TransactionId` from `honba-kokoro` into `vyapaar-core`.
- [x] Enforce non-negative invariant validation on `Price` and `Quantity` deserialization.

## Milestone 2: Domain Events, Orders, Decisions & Ports (P0)
- [x] Port `market.rs`, `order.rs`, `decision.rs`, `events.rs`, `calendar.rs`, `sizing.rs`, `signal.rs`, `portfolio.rs`, `screener.rs`, and `ports.rs`.
- [x] Ensure `Exchange` includes `Nse`, `Bse`, `Nfo`, `Mcx`, `Cds` so downstream crates never redefine `Exchange`.

## Milestone 3: Golden Vectors, Schema Compatibility & Proptests (P0)
- [x] Port `schema/domain/domain_schema.json`, `schema/compatibility/`, and `schema/golden/`.
- [x] Port and pass `tests/core_tests.rs`, `tests/golden_test.rs`, `tests/proptest_invariants.rs`, and `tests/schema_compatibility_test.rs`.
