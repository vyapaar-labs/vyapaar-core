# `vyapaar-core` Agent Guidelines & Architecture Invariants

**Subsystem**: Kernel (Shared Kernel)
**Language**: Rust
**Primary Responsibility**: Canonical domain value objects, wire contracts, and single source of truth schemas.

---

## 1. Role & Architectural Hints
* `vyapaar-core` is the foundational kernel for the entire Vyapaar ecosystem. All other Rust crates (`vyapaar-ledger`, `vyapaar-risk`, `vyapaar-market`, `vyapaar-exec`, `vyapaar-runtime`, `vyapaar-agent`, `vyapaar-ffi`, `vyapaar-ui`) depend on `vyapaar-core`.
* **Zero-I/O Domain Purity**: `vyapaar-core` must NEVER perform I/O (no network, no disk, no system clock queries).
* **Single Source of Truth for Wire Contracts**:
  - All public wire types derive `JsonSchema` via `schemars`.
  - OpenAPI 3.1 specifications, JSON schemas, and MCP tool schemas are deterministically generated from these types.
  - Never edit generated schema files by hand; use `make codegen`.
* **Idea Inheritance**:
  - **NautilusTrader**: Strict value objects with explicit units and domain validations.
  - **NoFX**: Closed typed decision grammar (`Decision` enum with `#[serde(deny_unknown_fields)]`).

---

## 2. Coding Guidelines (DDD & TDD)
* **Ubiquitous Language**: `InstrumentId`, `Price`, `Quantity`, `Money`, `Currency`, `OrderSide`, `OrderType`, `TimeInForce`, `Decision`, `ClientOrderId`, `AlgoId`, `AccountId`, `TransactionId`.
* **Zero Floats Policy**: Floating-point numbers (`f32`/`f64`) are strictly forbidden. Always use `rust_decimal::Decimal`.
* **TDD & Parallel Test Directory Rule (MANDATORY)**:
  - All tests must live in `tests/` parallel to `src/`, or in a `src/tests/` module directory.
  - **STRICTLY FORBIDDEN**: In-file `#[test]` or `mod tests` inside production `src/` files.
  - Red-Green-Refactor cycle on every PR.
  - Use `proptest` for invariant validation and price/quantity quantization.

---

## 3. Retargetability & Interface-First
* Compiles cleanly to `wasm32-unknown-unknown` and native `x86_64` / `aarch64`.
* No operating system specific dependencies.

---

## 4. Development & CI Workflow
* **Linting & Formatting**:
  ```bash
  cargo fmt --check
  cargo clippy --all-targets -- -D warnings
  ```
* **Testing**:
  ```bash
  cargo test --all
  ```
