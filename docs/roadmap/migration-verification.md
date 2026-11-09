# `vyapaar-core` — Migration Verification Milestone

Verification commands for the `honba-core → vyapaar-core` migration:

```bash
cd /home/pmallapp/Devel/Finance/vyapaar-labs/vyapaar-core
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Migration Milestone Status

| Milestone | Status |
|-----------|--------|
| All source files ported (`types`, `instrument`, `market`, `order`, `decision`, `events`, `calendar`, `sizing`, `ports`, `portfolio`, `screener`, `signal`, `wire`) | Complete |
| `src/ids.rs` created (new, from `honba-kokoro`) | Complete |
| `schema/` golden vectors & compatibility snapshots ported | Complete |
| `tests/` ported (`core_tests`, `golden_test`, `proptest_invariants`, `schema_compatibility_test`) | Complete |
| Zero-I/O domain purity | Complete |
| Zero-floats policy | Complete |
| Zero inline tests in `src/` | Complete |
| `cargo fmt --check` | Clean |
| `cargo clippy --all-targets -- -D warnings` | Clean |
| `cargo test` | Passing |
