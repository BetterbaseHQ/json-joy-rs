# Parity Audit (json-joy@18.0.0)

Last updated: 2026-02-22

This document tracks known, explicit parity gaps between:

- Upstream source of truth: `json-joy/packages`
- Local port: `crates/`

It is a review checkpoint artifact and should be updated as gaps are closed.

## Current gate status

- `just test-gates`: pass (2026-02-22)
- `just test`: pass (2026-02-22)
- `cargo test -p json-joy --test upstream_port_diff_workflows --offline`: pass (2026-02-22)
- `cargo test -p json-joy --test upstream_port_model_api_workflow --offline`: pass (2026-02-22)
- `cargo test -p json-joy --test upstream_port_model_api_proxy_fanout_workflow --offline`: pass (2026-02-22)

## Package layout and source-family parity snapshot

`src` file counts (upstream package -> local crate mapping currently used):

| Upstream package | Local crate | Upstream `src` files | Local `src` files |
| --- | --- | ---: | ---: |
| `base64` | `base64` | 26 | 13 |
| `buffers` | `buffers` | 61 | 14 |
| `json-expression` | `json-expression` | 29 | 23 |
| `json-joy` | `json-joy` | 1044 | 107 |
| `json-pack` | `json-joy-json-pack` | 398 | 125 |
| `json-path` | `json-joy-json-path` | 24 | 8 |
| `json-pointer` | `json-joy-json-pointer` | 31 | 33 |
| `json-random` | `json-joy-json-random` | 18 | 10 |
| `json-type` | `json-joy-json-type` | 123 | 39 |
| `util` | `util` | 71 | 23 |

Notes:

- `json-pointer` local `src` count is +2 vs upstream because Rust requires crate/module scaffolding files (`lib.rs`, `codegen/mod.rs`) that have no direct TS counterparts.
- `json-path` includes explicit `codegen`, `util`, and `value` modules mapped from upstream package families. Key parser/evaluator semantics are aligned with upstream test families. Upstream-mapped integration matrices cover:
  - `upstream_port_json_path_matrix.rs` — canonical bookstore queries from `testJsonPathExec`.
  - `upstream_port_json_path_descendant_matrix.rs` — descendant-selector behavior and codegen/eval equivalence.
  - `upstream_port_json_path_demo_matrix.rs` — complex TypeScript-AST queries with path-shape assertions.
  - `upstream_port_json_path_exec_matrix.rs` — root-format errors, combined selectors, and codegen-vs-eval parity.
  - `upstream_port_json_path_functions_matrix.rs` — function extension scenarios (`length`, `count`, `match`, `search`, `value`).
  - `upstream_port_json_path_parser_matrix.rs` — parser-shape scenarios for unions, recursive+filter composition, and error handling.
  - `upstream_port_json_path_util_matrix.rs` — utility helper behavior (`json_path_to_string`, `json_path_equals`, `get_accessed_properties`).
  - `upstream_port_json_path_expression_inventory.rs` — broad set of known-valid and known-invalid parser cases.
- `json-pack` integration matrices cover: `ws`, `resp`, `rm`, `rpc`, `rpc_real_traces`, `xdr`, `xdr_schema_validator`, `avro_schema_validator`, `avro`, `cbor`, `ejson`, `msgpack`, `msgpack_util`, `msgpack_shallow_read`, `surface_types`, `bencode`, `ubjson`, `ssh`, `json_binary`, `bson`, `ion`, `ion_import`, `json`, `json_pack_util`, and `codecs` (all at `crates/json-joy-json-pack/tests/upstream_port_*_matrix.rs`).
- `json-type` codegen families (`capacity`, `json`, `discriminator`, `binary`) have upstream-mapped parity coverage at `crates/json-joy-json-type/tests/upstream_port_json_type_codegen_matrix.rs`.
- `json-crdt` log codec mirrors upstream component encoding flow (`LogEncoder`/`LogDecoder` with `ndjson`, `seq.cbor`, `sidecar`, `binary`, `compact`, `verbose`, `none` formats).
- Prefixed crate naming is intentional and documented in `AGENTS.md` package mapping.

## Explicit non-parity choices currently in tree

### Harness-level accepted failures (`tests/compat/xfail.toml`)

Current xfail scenarios:

- none

No active compat xfails remain.

### In-code stubs and intentional behavior notes

- `crates/json-joy/src/json_crdt/draft.rs`: redo methods are explicit stubs.
- `crates/json-joy-json-pack/src/ejson/encoder.rs`: Decimal128 encoder keeps upstream "return 0" stub behavior.
- `crates/json-joy-json-pack/src/ejson/decoder.rs`: Decimal128 decoder returns zero 16-byte stub (matching upstream stub behavior).
- `crates/json-joy-json-pointer/src/findByPointer/v1.rs`..`v5.rs`: variants are mirrored for path/layout parity, but delegate to `v6` implementation.
- `crates/json-joy-json-pointer/src/codegen/find.rs` and `crates/json-joy-json-pointer/src/codegen/findRef.rs`: upstream emits specialized JS code; Rust uses closure wrappers over runtime traversal.
- `crates/json-joy-json-path/src/codegen.rs`: upstream generates specialized JS code; Rust uses pre-parsed AST closures over `JsonPathEval`.
- `crates/sonic-forest/src/util/mod.rs`: key-based helpers (`find`, `insert`, `find_or_next_lower`) take a `key_of` closure instead of direct node-field access to fit arena-indexed Rust nodes.
- `crates/sonic-forest/src/llrb-tree/LlrbTree.rs`: `get_or_next_lower`, `for_each`, `iterator0`, and `iterator` intentionally panic with "Method not implemented." to match upstream stubs; `clear()` intentionally mirrors upstream and only clears `root`.
- `crates/sonic-forest/src/radix/radix.rs`: string-key prefix math uses Unicode scalar (`char`) boundaries to stay Rust-safe; upstream JS indexes UTF-16 code units.
- `crates/sonic-forest/src/radix/radix.rs` and `crates/sonic-forest/src/radix/binaryRadix.rs`: debug print paths intentionally emit a generic `[value]` marker instead of full JS-style runtime value stringification.
- `crates/sonic-forest/src/TreeNode.rs`: stores `v` as `Option<V>` so `Tree.delete()` can return owned values from an arena-backed structure without removing nodes from the vector.

## sonic-forest parity status

Upstream reference:

- `sonic-forest/src`

Current local status:

- upstream source files: 81
- local source files: 60

Top-level families:

- upstream: `SortedMap`, `Tree.ts`, `TreeNode.ts`, `avl`, `data-types`, `llrb-tree`, `print`, `radix`, `red-black`, `splay`, `trie`, `types.ts`, `types2.ts`, `util`, `util2.ts`
- local: `lib.rs`, `Tree.rs`, `TreeNode.rs`, `avl`, `data-types`, `llrb-tree`, `print`, `radix`, `red-black`, `splay`, `trie`, `types.rs`, `util` (split to `first/next/swap/print/mod`), `util2.rs`

Upstream test families are covered by Rust parity matrices:

- `upstream_port_sorted_map_matrix.rs`
- `upstream_port_tree_matrix.rs`
- `upstream_port_util_matrix.rs`
- `upstream_port_avl_matrix.rs`
- `upstream_port_llrb_tree_matrix.rs`
- `upstream_port_radix_matrix.rs`
- `upstream_port_radix_slice_matrix.rs`
- `upstream_port_red_black_map_matrix.rs`
- `upstream_port_red_black_util_matrix.rs`

Remaining differences are mostly Rust file/module decomposition and intentional upstream-stub parity (`Method not implemented`) surfaces in `SortedMap` and `LlrbTree`.

## Recommended next review slices

1. `json-path`: continue porting additional upstream parser/evaluator corner cases (especially high-complexity nested filter/function combinations) into matrix tests to widen behavioral coverage.
