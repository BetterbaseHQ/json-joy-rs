# json-joy-rs MVP scope (derived from less-db-js usage)

This scope is based on actual usage in:
- `/Users/nchapman/Code/lessisbetter/less-platform/less-db-js`

and source behavior in:
- `/Users/nchapman/Code/json-joy`

## 1. Required public API (Rust)

MVP should expose equivalents of the `less-db-js` CRDT wrapper surface from:
- `/Users/nchapman/Code/lessisbetter/less-platform/less-db-js/src/crdt/model-manager.ts`
- `/Users/nchapman/Code/lessisbetter/less-platform/less-db-js/src/crdt/patch-log.ts`

Suggested Rust API (names can differ, behavior must match):

- `generate_session_id() -> u64`
  - Must produce valid logical session IDs (`>= 65536`).
- `is_valid_session_id(sid: u64) -> bool`
- `Model::create(data: JsonValue, sid: u64) -> Model`
- `Model::diff(&self, new_data: &JsonValue) -> Option<Patch>`
  - Returns `None` on no-op diff.
- `Model::apply_patch(&mut self, patch: &Patch)`
- `Model::view(&self) -> JsonValue`
- `Model::fork(&self, sid: Option<u64>) -> Model`
- `Model::to_binary(&self) -> Vec<u8>`
- `Model::from_binary(data: &[u8]) -> Result<Model>`
- `Model::load(data: &[u8], sid: u64) -> Result<Model>`
  - Load + clock/session rebinding for local edits.

Patch API:
- `Patch::to_binary(&self) -> Vec<u8>`
- `Patch::from_binary(data: &[u8]) -> Result<Patch>`

Pending patch log API:
- `serialize_patches(patches: &[Patch]) -> Vec<u8>`
- `deserialize_patches(bytes: &[u8]) -> Result<Vec<Patch>>`
- `append_patch(existing: &[u8], patch: &Patch) -> Vec<u8>`
- `EMPTY_PATCH_LOG` equivalent (`&[]` or `Vec::new()` helper)

## 2. Required behavior semantics

These are required because `less-db-js` logic depends on them in:
- `/Users/nchapman/Code/lessisbetter/less-platform/less-db-js/src/storage/record-manager.ts`
- `/Users/nchapman/Code/lessisbetter/less-platform/less-db-js/tests/scenarios/conflict.test.ts`
- `/Users/nchapman/Code/lessisbetter/less-platform/less-db-js/tests/storage/correctness.test.ts`

- CRDT model is authoritative; `view()` materializes data.
- Diffs must be structural and minimal enough for repeated local updates.
- Merge via replaying pending patches onto remote model must be idempotent.
  - Reapplying already-seen ops must be safely ignored via clock/vector semantics.
- Type-level merge behavior used by tests/docs:
  - Strings: character-level CRDT behavior (RGA-like), deterministic convergence.
  - Objects: per-key LWW behavior.
  - Arrays: positional sequence CRDT behavior.
  - Scalars: LWW replacement behavior.
- `fork` must preserve history and allow divergent sessions.
- `load` must support editing with caller-provided session id.

## 3. Binary compatibility requirements (MVP-critical)

`less-db-js` persists and syncs opaque CRDT blobs and serialized patches. For interoperability, MVP should support json-joy wire formats used by:

- Model binary codec (used by `Model.toBinary/fromBinary/load`)
- Patch binary codec (used by `Patch.toBinary/fromBinary`)

And local patch log framing used by less-db-js:

- Format v1: `[0x01][len: u32-be][patch-bytes]...`
- Empty log: zero-length byte array.
- Defensive limits:
  - Max model binary size: `10 * 1024 * 1024`
  - Max single patch size while decoding log: `10 * 1024 * 1024`

Error cases expected by tests:
- Unsupported patch log version.
- Truncated patch log length header.
- Declared patch length exceeds max.
- Truncated patch payload.

## 4. Out of scope for MVP

Not required by observed `less-db-js` usage:

- JSON type system/schema builder from json-joy.
- Event/fanout/reactive APIs.
- Extensions/peritext.
- Server clock mode.
- Proxy/node-path editing APIs.
- Patch compaction/rewrite/rebase authoring utilities (unless needed internally).

## 5. Suggested MVP milestones

1. Core data model + patch application
- Implement core nodes and operation application sufficient for object/string/array/scalar docs.
- Ensure `view()`, `apply_patch()`, `fork()` correctness.

2. Diff engine parity for used types
- Implement `diff(model, value) -> Option<Patch>` with required semantics for object/string/array/scalar.

3. Binary codecs
- Implement model binary decode/encode compatible with json-joy blobs used in less-db-js.
- Implement patch binary decode/encode compatible with json-joy patches.

4. Session/clock behavior
- Implement sid generation/validation and vector/causal handling needed for idempotent merge replay.

5. Patch log framing + limits
- Implement `serialize/deserialize/append` for pending patch log v1 and all defensive validations.

## 6. MVP acceptance tests (minimum)

Port these behavior checks first (or equivalent):

- Create -> serialize model -> deserialize -> `view` equality.
- Fork preserves base history.
- Divergent edits from two sessions converge after patch replay.
- No-op diff returns `None`.
- Replaying already-applied patch does not corrupt state (idempotent behavior).
- Patch log roundtrip and append behavior.
- Patch log corruption/size-limit errors.
- Model binary size limit errors.

## 7. Practical note on rollout

If full binary compatibility is too large for first cut, use a two-step deliverable:

- MVP-A (local correctness): full semantics + deterministic convergence + local codec.
- MVP-B (interop): json-joy binary compatibility for model/patch codecs.

For `less-db-js` integration in production, MVP-B is the true required milestone.
