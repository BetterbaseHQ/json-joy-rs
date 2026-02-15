# AGENTS.md

## Project workflow: compatibility-first TDD

This repository follows a strict compatibility-first, test-driven process for porting `json-joy`.

### Core rule

For each milestone/section:

1. Expand the oracle fixture surface first.
2. Write or update failing tests against those fixtures.
3. Implement only enough code to pass.
4. Stabilize and freeze that section before moving to the next.

Do not start implementation for a section until its fixture/test surface is in place.

## Oracle and compatibility source

- Upstream compatibility target is pinned to `json-joy@17.67.0` unless explicitly changed.
- Node oracle lives in `tools/oracle-node`.
- Fixtures live in `tests/compat/fixtures`.

## Required execution flow per section

1. Generate/update fixtures (`make compat-fixtures` or equivalent section-specific generator).
2. Ensure tests fail for unimplemented behavior.
3. Implement section code in Rust.
4. Run full tests (`make test`) before commit.

## Scope discipline

- Work section-by-section (M1, M2, M3...).
- Keep changes narrowly scoped to the active section.
- Avoid cross-section implementation unless required by failing tests in the active section.

## Quality gates

A section is considered complete only when:

- Fixture schema/integrity tests pass.
- Section compatibility tests pass.
- No regressions in existing tests.

## Documentation discipline

When workflow changes, update this file and relevant plan docs (`PORT_PLAN.md`) in the same change.
