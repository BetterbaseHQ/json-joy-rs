# Parity Audit (json-joy@18.0.0)

Last updated: 2026-09-18

This document tracks known, explicit parity gaps between:

- Upstream source of truth: `json-joy/packages`
- Local port: `crates/`

It is a review checkpoint artifact and should be updated as gaps are closed.

## Upstream review: v18.0.0 → v18.30.0 (2026-09-18)

Upstream advanced from v18.0.0 (2026-02) to v18.30.0 (2026-09-08,
1,233 commits). Reviewed changes by applicability:

### Ported in this pass

- **RGA `prevId()`/`nextId()`** (upstream commits `78e3028c1`,
  `96132112b`): added `Rga::prev_id`/`Rga::next_id` with `skip_deleted`
  support, returning `(id, chunk_arena_index)`. Tests mirror upstream
  `StrNode.spec.ts` iteration scenarios.
- **Array iteration over missing children** (upstream commit `a89fbfa05`
  + baseline `ArrNode.view()` semantics): `ArrNode::view` now skips
  elements whose child node is missing from the index instead of
  rendering `null`. This was a pre-existing port divergence from v18.0.0
  upstream behavior, closed by this change.

### Reviewed, not applicable to the Rust port

- **`ValNode` UNDEFINED singleton sharing** (`daa0f27ab`): JS
  object-identity issue; Rust `ValNode` stores a `Ts` ID and resolves
  through `NodeIndex` — no shared node singleton exists.
- **Prototype pollution fixes** (`87333a5b02`, `40bcaf3bef`,
  `fa6f33308b`): JS `__proto__` attack surface; Rust maps/objects are
  immune.
- **Schema OO generic / type inference** (`415f92d42`, `5fb1e4148`):
  TypeScript type-level only, no runtime behavior.
- **Structural hash file moves** (`9d1941893c`), **print utils move**
  (`12a83e6c67`), **iterator helpers move** (`2275c0a84c`): upstream
  file-layout reorganizations; local module mapping is documented in
  `AGENTS.md` and layout parity is explicitly not required.
- **json-pack NFSv4 removal** (−82k lines upstream): local port never
  included the NFS server family; nothing to remove.
- **Jest→Vitest test migrations, formatter/linter churn, UI-app
  packages** (click-json, click-type, peritext-ui, storybook,
  collaborative-*): out of port scope.

### Deferred (candidate future work)

- **Delta computation & codecs** (~15 commits, `json-crdt/delta/**`):
  `Model.delta()` for all node types, binary/compact/verbose delta
  codecs, columnar batch encoding (RLE, zigzag), `delta/sync.ts`
  protocol helpers. Potentially relevant to Betterbase sync (delta sync
  instead of full-blob). Large surface (~25 upstream files).
- **Patch batch codec family** (`json-crdt-patch/batch/**`): binary,
  compact, verbose, and columnar batch codecs with metadata support.
  Local `batch.rs` (87 lines) predates this family. Includes the "con"
  encoding fix (`1e055c9c65`) inside the columnar codec.
- **Delta correctness fixes** (`4b7674427b`, `bfc2c78cc9`): only affect
  the deferred delta feature.
- **Version vector primitives, clock `.has()`, clock vector marshaling**
  (`f26970fe94`, `6925f0a55`, `9b59c3cd53`).
- **json-type Reactive JSON RPC binary codec** (`2b8b7eb598`),
  undefined field lookup fix (`c16f9c44ca`).
- **json-pack null-prototype object encoding** (`c64e73cdca`): JS
  specific; revisit only if fixture parity diverges.
- **`json-hash` extraction** into standalone upstream package
  (`502e246b99`): cosmetic, watch during next sync.
- **Monorepo split beginning**: upstream created a placeholder
  `packages/json-crdt` package (empty re-export). Track before the next
  full sync; also note `sonic-forest` no longer exists in the upstream
  monorepo.

### Wire-format safety note

The `json-crdt-patch` binary codec (upstream commit `ba21ae64c4`)
changed method names only — the wire format is unchanged. Betterbase's
patch log format v1 remains compatible; no action taken.

## Current gate status

- `just test-gates`: pass (2026-09-18)
- `just test`: pass (2026-09-18)
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

- `crates/json-joy/src/json_crdt/codec/structural/binary.rs` (AUD-021 hardening): the structural, sidecar, and indexed decoders (element loops and clock tables) and the binary patch decoder (op count, InsObj/InsVec/InsArr/Del element loops, CBOR array/map reservations) reject declared counts that exceed the available input with `DecodeError::EndOfInput`, instead of materializing zero/null-filled elements from bytes past EOF the way upstream does. Also fixes a direct-index panic in `decode_vec_logical` and rejects truncated binary chunk spans. Permanent divergence: malformed/truncated models (malicious or corrupt import/decrypt output) must not force allocations unbounded by input size. Tested by `truncated_vec_count_errors_instead_of_allocating`, `truncated_logical_vec_does_not_panic`, `truncated_bin_chunk_span_errors`, `truncated_clock_table_count_errors_instead_of_allocating`, and `truncated_op_count_errors_instead_of_fabricating`; well-formed round-trips are unaffected (every element consumes ≥1 byte). Empty/short-but-complete patch inputs remain Ok (`short_inputs_are_tolerated`).

- `crates/json-joy/src/json_crdt/nodes/mod.rs` (`VecNode::view`): upstream pushes JS `undefined` for unset/missing vec elements; the Rust port renders `Value::Null`, matching `JSON.stringify` output of upstream views. Accepted divergence (serde_json has no undefined).
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
