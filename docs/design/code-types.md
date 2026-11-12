# `vyapaar-core` — Canonical Value Objects Reference

Documents `vyapaar-core/src/types.rs`: the kernel's immutable, self-validating
value objects. Every type here is the single source of truth for its domain
concept across all Vyapaar bounded contexts.

## `Exchange`

Canonical exchange enum — downstream bounded contexts must never redefine their
own exchange enums, only re-use this one.

| Variant | Wire Code | Market |
|---------|-----------|--------|
| `Exchange::Nse` | `"NSE"` | National Stock Exchange of India |
| `Exchange::Bse` | `"BSE"` | Bombay Stock Exchange |
| `Exchange::Nfo` | `"NFO"` | NSE Futures & Options (derivatives) |
| `Exchange::Mcx` | `"MCX"` | Multi Commodity Exchange of India |
| `Exchange::Cds` | `"CDS"` | Currency Derivatives Segment |

Serializes as `UPPERCASE` strings (`"NSE"`, `"BSE"`, …); `FromStr` /
`as_str()` are case-insensitive and reject unknown codes with
`CoreError::InvalidExchange`.

## `InstrumentId`

```rust
pub struct InstrumentId {
    pub exchange: Exchange,
    pub symbol: String,          // e.g. "RELIANCE", "TCS", "INFY" (upper-cased on construction)
    pub token: Option<String>,   // broker-assigned instrument token
    pub isin: Option<String>,    // ISIN
}
```

Ordered by `(exchange, symbol, token, isin)` so it can key a `BTreeMap` and
sort datasets deterministically. Constructors: `InstrumentId::new(exchange,
symbol)`, `.with_token(...)`, `.with_isin(...)`. Display form is
`"{exchange}:{symbol}"` (e.g. `NSE:RELIANCE`).

## `Price` — Exact Decimal Price

```rust
pub struct Price(pub Decimal);  // serde transparent, serializes as decimal string
```

**Invariant:** never negative, no matter how it is constructed.

- `Price::try_new(d)` — validated primary constructor; rejects negative with
  `CoreError::NegativePrice`.
- `Price::new(d)` — debug-assert convenience constructor for guaranteed-positive
  call sites.
- `Price::from_rupees(s)` — parse from a Rupees string (e.g. `"2450.75"`).
- `Price::ZERO` — zero constant.
- Accessors: `is_positive()`, `is_tick_aligned(tick_size)`, `value()`.
- **Serde:** a validating wire visitor rejects negative JSON payloads (LLM
  output, REST requests, replay vectors) at the boundary in both debug and
  release.

## `Quantity` — Exact Decimal Quantity

```rust
pub struct Quantity(pub Decimal);
```

**Invariant:** never negative (zero is legal for flat positions).

- `Quantity::try_new(d)` — validated primary constructor.
- `Quantity::new(d)` — debug-assert convenience.
- `Quantity::from_u64(n)` — from integer units.
- `Quantity::ZERO` — zero constant.
- Accessors: `is_positive()`, `is_lot_multiple(lot_size)`, `value()`.
- **Serde:** rejects negative wire payloads in both debug and release.

## `Money`

```rust
pub struct Money {
    pub amount: Decimal,
    pub currency: Currency,
}
```

- `Money::new(amount, currency)` (const), `Money::inr(amount)`,
  `Money::zero_inr()` (the default for `FillReport::fee` before a cost
  calculation runs).
- `Money::notional(price, qty)` — computes `price * qty` in INR.

## `PriceBand` — Scrip-wise Circuit Limits

```rust
pub struct PriceBand {
    pub lower: Price,
    pub upper: Price,
    pub reference_price: Price,
    pub band_pct: Option<Decimal>,   // e.g. 0.05 = 5%, 0.10 = 10%, 0.20 = 20%
}
```

- `PriceBand::new(lower, upper, reference_price)`.
- `PriceBand::contains(price)` — inclusive `[lower, upper]` check.

## `Currency` & `MinorUnit`

| Variant | ISO Code | Minor Exponent | Minor Unit |
|---------|----------|----------------|------------|
| `Currency::Inr` (default) | INR | 2 | paisa / paise |
| `Currency::Usd` | USD | 2 | cent / cents |
| `Currency::Eur` | EUR | 2 | cent / cents |
| `Currency::Gbp` | GBP | 2 | penny / pence |

- `Currency::ALL` — stable ordered slice.
- `Currency::code()`, `Currency::minor_exponent()`, `Currency::minor_unit()`.
- `Currency::from_code(code)` — case-insensitive ISO 4217 parse
  (`CoreError::InvalidCurrency` on unknown).
- `MinorUnit { singular, plural }` — the display names table, shared by the
  Python SDK through `vyapaar._vyapaar.currency_minor_units`.

## `CountryCode`

| Variant | Code |
|---------|------|
| `CountryCode::In` (default) | IN |
| `CountryCode::Us` | US |
| `CountryCode::Uk` | UK (also parses `GB`) |
| `CountryCode::Jp` | JP |

## Zero-Floats Policy

All financial fields are `rust_decimal::Decimal`. No `f32`/`f64` appears in any
financial struct. `Price` and `Quantity` serialize as exact decimal strings
(serde-with-str) to preserve precision across the wire, and their deserializers
validate the non-negative invariant at the boundary.

## Related

- `vyapaar-core/src/ids.rs` — `ClientOrderId`, `AlgoId`, `AccountId`, `TransactionId`.
- `vyapaar-core/src/error.rs` — the `CoreError` taxonomy used by all the above.
- `vyapaar-core/src/instrument.rs` — `Instrument` reference data that keys on `InstrumentId`.
- `vyapaar-core/src/market.rs` — `Tick`, `Bar`, `DepthLevel`.
- `vyapaar-core/src/order.rs` — `OrderRequest`, `FillReport`, `FeeBreakdown`.
