# json-joy-rs

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL_3.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

Rust implementation of the [json-joy](https://github.com/streamich/json-joy) CRDT, patch, diff, and codec library. Provides a complete port of core json-joy functionality with native bridges for WASM and Python.

This project ports and adapts the upstream `json-joy` library by [streamich](https://github.com/streamich/json-joy), pinned against `json-joy@18.0.0` with 1,398 compatibility fixtures ensuring parity.

## Crates

| Crate | Description |
|-------|-------------|
| `json-joy` | Core CRDT document model, patch protocol, diff, extensions, JSON utilities |
| `json-joy-json-equal` | Deep equality comparison for JSON values |
| `json-joy-json-pack` | Binary serialization (CBOR, MessagePack, BSON, Avro, Ion, UBJSON, Bencode, and more) |
| `json-joy-json-type` | Type system and schema framework |
| `json-joy-json-path` | JSONPath (RFC 9535) evaluation |
| `json-joy-json-pointer` | JSON Pointer (RFC 6901) utilities |
| `json-expression` | High-performance JSON expression evaluator |
| `json-joy-json-random` | Random JSON value generator for testing |
| `sonic-forest` | Arena-based splay tree for dual-tree data structures |
| `json-joy-wasm` | WASM bridge via wasm-bindgen |
| `json-joy-ffi` | UniFFI bridge for Python and other languages |

## Quick Start

```bash
just check
```

`just check` runs formatting, strict clippy (`-D warnings` across all targets/features), compatibility gates, and full workspace tests.

If running cargo directly, use `mise` for pinned toolchains:

```bash
mise x -- cargo clippy --workspace --all-features --all-targets -- -D warnings
mise x -- cargo test --workspace
```

## Compatibility

Parity with upstream is verified through a fixture-driven harness and live differential testing.

```bash
just compat-fixtures    # Generate upstream compatibility fixtures
just parity-fixtures    # Run fixture parity tests
just parity-live        # Live TS<->WASM differential check
just parity             # Run both
```

See `tests/compat/PARITY_AUDIT.md` for the full parity tracking log.

## Repository Layout

```
crates/
  json-equal/               Deep equality comparison
  json-joy/                 Core library and parity target
  json-joy-json-pack/       Binary serialization formats
  json-joy-json-type/       Type system and schema
  json-joy-json-path/       JSONPath (RFC 9535)
  json-joy-json-pointer/    JSON Pointer (RFC 6901)
  json-expression/          Expression evaluator
  json-joy-json-random/     Random JSON generator
  sonic-forest/             Splay tree utilities
  json-joy-wasm/            WASM bridge
  json-joy-ffi/             UniFFI bridge (cdylib)
bindings/
  python/                   Python packaging and generated bindings
  wasm/                     WASM benchmark and interop harness
tests/
  compat/                   Fixture corpus, manifest, and xfail policy
```

## Scope

JS editor ecosystem adapter APIs (Slate/ProseMirror/Quill-specific helpers) are intentionally out of scope in this Rust/WASM port. For those integrations, use upstream JS [json-joy](https://github.com/streamich/json-joy).

`json-pack` NFS protocol families are not a current parity target.

## Related

- [betterbase-dev](https://github.com/BetterbaseHQ/betterbase-dev) -- Platform orchestration
- [@betterbase/sdk](https://github.com/BetterbaseHQ/betterbase) -- Client SDK (uses json-joy-rs for CRDT operations)
- [json-joy](https://github.com/streamich/json-joy) -- Upstream TypeScript library

## License

AGPL-3.0-only
