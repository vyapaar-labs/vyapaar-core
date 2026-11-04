# व्यापार・核心 vyapaar-core

> *Shared Kernel: Canonical Decimal value objects, InstrumentId, Tick, Order, Decision, and OpenAPI schemas.*

Foundational zero-float domain types, invariants, and serialization contracts shared across all Vyapaar subsystems.

## Architecture

- Canonical value objects: `Price`, `Quantity`, `Money`, `Currency`, `InstrumentId`, `Side`, `OrderType`, `TimeInForce`
- Event & order types: `Tick`, `Order`, `Fill`, `Decision`, `CancelAll`
- Precision & safety: Strict `rust_decimal::Decimal`, zero IEEE-754 floats
- Schema compatibility tests & golden serialization fixtures

## Build & Test

```bash
make prod
make check
test
```

License: AGPL-3.0.
