# Test Suite Guide

This repository uses a layered test strategy for runtime-core parity with
`json-joy@17.67.0`.

## Layers

1. Fixture contracts and fixture-driven parity:
- `crates/json-joy-core/tests/compat_fixtures.rs`
- `*_from_fixtures.rs` suites

2. Upstream-mapped behavior matrices:
- `crates/json-joy-core/tests/upstream_port_*.rs`

3. Seeded differential parity (Rust vs local Node oracle):
- `crates/json-joy-core/tests/differential_*.rs`

4. Property/state invariants:
- `crates/json-joy-core/tests/property_*.rs`

5. Meta coverage inventory:
- `crates/json-joy-core/tests/suite_coverage_inventory.rs`

## Recommended Commands

1. Regenerate fixtures:
- `make compat-fixtures`

2. Run fixture suites:
- `make test-core-fixtures`

3. Run upstream parity suites:
- `make test-core-upstream`

4. Run differential suites:
- `make test-core-differential`

5. Run property suites:
- `make test-core-property`

6. Run full workspace:
- `make test`

## Coverage Expectations

1. Fixture floors are enforced in:
- `crates/json-joy-core/tests/compat_fixtures.rs`

2. Runtime-core families tracked in:
- `CORE_PARITY_MATRIX.md`

3. Upstream implementation references and quirk rationale:
- `UPSTREAM_IMPLEMENTATION_REVIEW.md`
