# Architecture

This repository is structured as a Rust-core + bindings monorepo, following the same high-level model as Glean.

1. One authoritative Rust core crate.
2. One dedicated FFI crate exposing a stable cross-language API.
3. Generated language bindings committed/published from `bindings/python/src/json_joy_rs/generated`.
4. One local pinned bindgen tool in the workspace for reproducible generation.

## Crates

- `crates/json-joy-core`
  - Business logic and data structures.
  - No language-specific concerns.

- `crates/json-joy-ffi`
  - UniFFI UDL + exported API surface.
  - Produces `cdylib` for Python consumers.

- `tools/embedded-uniffi-bindgen`
  - Runs UniFFI bindgen from a workspace-controlled version.

## Binding generation flow

1. Build Rust FFI library: `cargo build -p json-joy-ffi`
2. Generate Python bindings: `bin/generate-bindings.sh python`
3. Package artifacts in `bindings/python`.

## Versioning guidance

- `json-joy-core` can evolve rapidly.
- `json-joy-ffi` should provide a stable semver API contract for non-Rust consumers.
- The Python package should track `json-joy-ffi` releases.
