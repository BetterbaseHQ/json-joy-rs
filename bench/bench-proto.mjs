/**
 * Prototype benchmark: alternative "Doc" API shape
 *
 * Hypothesis: the primary cost of the current API is crossing the WASM boundary
 * for view() on every read.  If we keep the view in JS (as a plain value) and
 * only call WASM for diff/merge/encode, reads become O(1) and the overall
 * throughput profile looks very different.
 *
 * This file implements a minimal Doc class over the existing WASM bindings (no
 * new Rust code required) and benchmarks the key operations side-by-side with
 * the current API and the upstream TypeScript implementation.
 *
 * Run:  node bench/bench-proto.mjs
 */

import { createRequire } from 'module';
import { performance }   from 'perf_hooks';

const _require = createRequire(import.meta.url);
const { Model: TS, Patch: TSPatch } = _require('./node_modules/json-joy/lib/json-crdt/index.js');
const { Model: WASM }               = _require('../crates/json-joy-wasm/pkg/json_joy_wasm.js');

let _sid = 100_000n;
const nextSid = () => _sid++;

// ── Benchmark harness ──────────────────────────────────────────────────────

function bench(name, n, fn) {
  const warmup = Math.max(100, Math.floor(n / 5));
  for (let i = 0; i < warmup; i++) fn();
  const start = performance.now();
  for (let i = 0; i < n; i++) fn();
  const ms = performance.now() - start;
  return { name, ops: Math.round(n / (ms / 1000)), ms, n };
}

function row(label, ref, cmp, unit = '') {
  const ratio = cmp.ops / ref.ops;
  const mark  = ratio >= 0.9 ? '✓' : ratio >= 0.5 ? '~' : '✗';
  const dir   = ratio >= 1
    ? `${ratio.toFixed(1)}× faster`
    : `${(1/ratio).toFixed(1)}× slower`;
  console.log(
    `  ${mark} ${label.padEnd(26)}` +
    `${ref.ops.toLocaleString().padStart(13)} op/s →` +
    `${cmp.ops.toLocaleString().padStart(13)} op/s  (${dir})`
  );
}

// ── Proto: Doc ─────────────────────────────────────────────────────────────
//
// Design principles:
//
//  1. The view is always a plain JS value.  It is cached on write and only
//     refreshed from WASM when applying remote patches.  This moves the hot
//     read path from WASM (700K op/s) to native JS property access (200M op/s).
//
//  2. Writes use diffApply: the user provides the next plain-JS state, Rust
//     computes the CRDT diff and applies it, returning a binary patch.
//     On a fresh document this is identical to apiSet+flush; on an existing
//     document only the changed paths become patch ops.
//
//  3. Remote sync: applyPatch (WASM) + JSON.parse(viewJson()) to refresh.
//     This is one WASM call + one JSON.parse per incoming patch — cheap.
//
//  4. For fine-grained text/CRDT ops (where diffApply degrades), a raw-ops
//     escape hatch delegates directly to the underlying WASM model.

class Doc {
  /**
   * @param {import('../crates/json-joy-wasm/pkg/json_joy_wasm.js').Model} wasm
   * @param {unknown} initialView
   */
  constructor(wasm, initialView = null) {
    this._m    = wasm;
    this._view = initialView;
  }

  // ── Factory methods ─────────────────────────────────────────────────────

  static create(sid) {
    return new Doc(new WASM(sid ?? nextSid()));
  }

  static fromBinary(bytes) {
    const wasm = WASM.fromBinary(bytes);
    return new Doc(wasm, JSON.parse(wasm.viewJson()));
  }

  // ── Read (O(1) — pure JS, no WASM crossing) ────────────────────────────

  /** Current document state as a plain JS value. */
  get view() { return this._view; }

  // ── Write (diff + cache) ───────────────────────────────────────────────

  /**
   * Replace the document with `nextValue`.
   *
   * Rust computes the CRDT diff against the current state, applies it, and
   * returns a binary patch suitable for broadcasting to peers.
   * The patch is empty if the value hasn't changed.
   *
   * The JS-side cache is updated to `nextValue` immediately — no WASM read.
   *
   * @param {unknown} nextValue
   * @returns {Uint8Array} binary patch
   */
  set(nextValue) {
    const patch = this._m.diffApply(JSON.stringify(nextValue));
    this._view  = nextValue;   // cache: no WASM crossing on subsequent reads
    return patch;
  }

  // ── Remote sync ─────────────────────────────────────────────────────────

  /**
   * Apply a remote patch received from a peer.
   *
   * Updates the WASM CRDT state, then refreshes the JS-side cache via a
   * single viewJson() call.  This is the *only* time we cross the WASM
   * boundary for a read.
   *
   * @param {Uint8Array} patchBytes
   */
  apply(patchBytes) {
    this._m.applyPatch(patchBytes);
    this._view = JSON.parse(this._m.viewJson());
  }

  // ── Serialisation ───────────────────────────────────────────────────────

  /** Binary-encode the full CRDT model for persistence / sync handshake. */
  encode() { return this._m.toBinary(); }

  // ── Escape hatch: raw WASM ops (for fine-grained text editing) ──────────
  //
  // Using the CRDT ops directly avoids the diffApply overhead for single-char
  // insertions into large strings (diffApply must re-scan the whole string to
  // compute the diff; direct CRDT is O(log n) per insert).
  //
  // The view cache is invalidated on each direct op and refreshed lazily on
  // the next flush() call.

  get wasm() { return this._m; }  // raw escape hatch

  strIns(pathArr, index, text) {
    const pathJson = pathArr.length ? JSON.stringify(pathArr) : 'null';
    this._m.apiStrIns(pathJson, index, text);
    this._dirty = true;
  }

  strDel(pathArr, index, count) {
    const pathJson = pathArr.length ? JSON.stringify(pathArr) : 'null';
    this._m.apiStrDel(pathJson, index, count);
    this._dirty = true;
  }

  /**
   * Flush pending direct CRDT ops.
   * Returns the binary patch and refreshes the JS view cache.
   *
   * @returns {Uint8Array}
   */
  flush() {
    const patch = this._m.apiFlush();
    if (this._dirty) {
      this._view = JSON.parse(this._m.viewJson());
      this._dirty = false;
    }
    return patch;
  }
}

// ── Fixtures ───────────────────────────────────────────────────────────────

// Patch for apply_patch benchmark
const _tsSender = TS.create(1n);
_tsSender.api.set({ title: 'Hello world', count: 42, tags: ['a','b','c'] });
const tsPatchBytes   = _tsSender.api.flush().toBinary();
const tsPrebuiltPatch = TSPatch.fromBinary(tsPatchBytes);

const _wasmSender = Doc.create(1n);
const wasmPatchBytes = _wasmSender.set({ title: 'Hello world', count: 42, tags: ['a','b','c'] });

// Steady-state view doc
const _tsViewDoc   = TS.create(2n);
_tsViewDoc.api.set({ name: 'Alice', age: 30, items: [1,2,3], active: true });

const _wasmViewCur = new WASM(3n);
_wasmViewCur.apiSet(JSON.stringify({ name: 'Alice', age: 30, items: [1,2,3], active: true }));

const _protoViewDoc = Doc.create(3n);
_protoViewDoc.set({ name: 'Alice', age: 30, items: [1,2,3], active: true });

// ── Run ────────────────────────────────────────────────────────────────────

console.log('\n  Proto Doc API  vs  current WASM API  vs  TypeScript\n');
console.log('  Legend: ✓ within 10%   ~ within 2×   ✗ >2× slower');
console.log(`  ${'operation'.padEnd(26)}${'reference'.padStart(13)} op/s    ${'compare'.padStart(12)} op/s\n`);

// 1. model_create
{
  const ts       = bench('model_create', 50_000, () => TS.create());
  const wasmCur  = bench('model_create', 50_000, () => new WASM(nextSid()));
  const proto    = bench('model_create', 50_000, () => Doc.create());
  console.log('── model_create');
  row('ts vs wasm (current)', ts, wasmCur);
  row('ts vs proto',          ts, proto);
  console.log();
}

// 2. set + flush — the write path
{
  const obj    = { x: 1, y: 2, active: false };
  const objStr = JSON.stringify(obj);

  const ts = bench('set_flush', 10_000, () => {
    const m = TS.create(); m.api.set(obj); m.api.flush();
  });
  const wasmCur = bench('set_flush', 10_000, () => {
    const m = new WASM(nextSid()); m.apiSet(objStr); m.apiFlush();
  });
  const proto = bench('set_flush', 10_000, () => {
    const d = Doc.create(); d.set(obj);
    // view access is free — just verifying cache is set
    void d.view;
  });

  console.log('── set + flush  (write a simple object)');
  row('ts vs wasm (current)',  ts, wasmCur);
  row('ts vs proto (doc.set)', ts, proto);
  console.log();
}

// 3. view — the read path (this is the headline)
{
  const ts = bench('view', 200_000, () => _tsViewDoc.view());

  // current WASM: crosses boundary every time
  const wasmCur = bench('view', 200_000, () => _wasmViewCur.view());
  const wasmJson = bench('view', 200_000, () => JSON.parse(_wasmViewCur.viewJson()));

  // proto: pure JS property access
  const proto = bench('view', 200_000, () => _protoViewDoc.view);

  console.log('── view  (read the current document state)');
  row('ts vs wasm obj',    ts, wasmCur);
  row('ts vs wasm JSON',   ts, wasmJson);
  row('ts vs proto.view',  ts, proto);
  console.log();
}

// 4. apply remote patch + refresh view
{
  const ts = bench('apply_patch', 10_000, () => {
    const m = TS.create(); m.applyPatch(TSPatch.fromBinary(tsPatchBytes));
  });
  const wasmCur = bench('apply_patch', 10_000, () => {
    const m = new WASM(nextSid()); m.applyPatch(wasmPatchBytes);
    // current API: no automatic view refresh
  });
  const proto = bench('apply_patch', 10_000, () => {
    const d = Doc.create(); d.apply(wasmPatchBytes);
    // proto: apply + view refresh included
    void d.view;
  });

  console.log('── apply remote patch  (includes view refresh for proto)');
  row('ts vs wasm (no refresh)',    ts, wasmCur);
  row('ts vs proto (with refresh)', ts, proto);
  console.log();
}

// 5. str_ins × 100 — collaborative text editing
//    New shape: strIns() calls direct CRDT API (avoids diffApply overhead for
//    single-char insertions).  Same throughput as current, but view is cached.
{
  const chars    = 'abcdefghijklmnopqrstuvwxyz0123456789';
  const pathJson = 'null';

  const ts = bench('str_ins×100', 500, () => {
    const m = TS.create();
    m.api.set('');
    for (let i = 0; i < 100; i++) m.api.str([]).ins(i, chars[i % chars.length]);
    m.api.flush();
  });
  const wasmCur = bench('str_ins×100', 500, () => {
    const m = new WASM(nextSid());
    m.apiSet('""');
    for (let i = 0; i < 100; i++) m.apiStrIns(pathJson, i, chars[i % chars.length]);
    m.apiFlush();
  });
  const proto = bench('str_ins×100', 500, () => {
    const d = Doc.create();
    d.set('');
    for (let i = 0; i < 100; i++) d.strIns([], i, chars[i % chars.length]);
    d.flush();        // one WASM view refresh at the end
    void d.view;      // O(1) read
  });
  // Also try: single diffApply of the whole 100-char string
  const text100  = Array.from({ length: 100 }, (_, i) => chars[i % chars.length]).join('');
  const proto_diff = bench('str_ins×100 (diff)', 500, () => {
    const d = Doc.create();
    d.set(text100);   // one diffApply — Rust computes char-level diff
    void d.view;
  });

  console.log('── str_ins × 100  (collaborative text)');
  row('ts vs wasm (current)',    ts, wasmCur);
  row('ts vs proto (direct op)', ts, proto);
  row('ts vs proto (diffApply)', ts, proto_diff);
  console.log();
}

// 6. binary_rt — serialisation roundtrip
{
  const tsM  = TS.create(4n); tsM.api.set({ name: 'test', items: [1,2,3], nested: { x: 0 } });
  const tsBin = tsM.toBinary();
  const protoM = Doc.create(4n); protoM.set({ name: 'test', items: [1,2,3], nested: { x: 0 } });
  const protoBin = protoM.encode();

  const ts    = bench('binary_rt', 20_000, () => TS.fromBinary(tsBin));
  const proto = bench('binary_rt', 20_000, () => Doc.fromBinary(protoBin));

  console.log('── binary roundtrip  (decode includes view refresh)');
  row('ts vs proto', ts, proto);
  console.log();
}

// 7. Realistic usage pattern: write → read → write → read
//    Most apps interleave writes and view reads.
{
  const docData  = { name: 'Alice', score: 0, active: true };
  const docData2 = { name: 'Alice', score: 1, active: true };
  const docStr   = JSON.stringify(docData);
  const docStr2  = JSON.stringify(docData2);

  // TS: natural — always in native V8
  const ts = bench('write+read cycle', 20_000, () => {
    const m = TS.create();
    m.api.set(docData);
    m.api.flush();
    void m.view();        // JS: O(1) traverse
    m.api.set(docData2);
    m.api.flush();
    void m.view();
  });

  // Current WASM: view() crosses boundary every time
  const wasmCur = bench('write+read cycle', 20_000, () => {
    const m = new WASM(nextSid());
    m.apiSet(docStr);
    m.apiFlush();
    void m.view();        // WASM crossing
    m.apiSet(docStr2);
    m.apiFlush();
    void m.view();        // WASM crossing
  });

  // Proto: view is free after set
  const proto = bench('write+read cycle', 20_000, () => {
    const d = Doc.create();
    d.set(docData);       // write + cache
    void d.view;          // O(1) JS property
    d.set(docData2);      // write + cache
    void d.view;          // O(1) JS property
  });

  console.log('── write → read → write → read  (realistic interleaved cycle)');
  row('ts vs wasm (current)', ts, wasmCur);
  row('ts vs proto',          ts, proto);
  console.log();
}

console.log('  Note: proto.view is O(1) cached JS property access.');
console.log('  WASM binary_rt decode is slower because it includes viewJson refresh.\n');
