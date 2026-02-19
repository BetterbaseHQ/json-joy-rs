/**
 * Performance benchmark: upstream json-joy (TypeScript) vs json-joy-wasm (Rust/WASM)
 *
 * Run:  node bench/bench.mjs
 *
 * Operations measured:
 *   1. model_create    — Model.create() (no schema)
 *   2. set_flush       — create + api.set({...}) + api.flush()
 *   3. str_ins×100     — 100 single-char insertions into a CRDT string + flush
 *   4. binary_rt       — toBinary() + fromBinary() roundtrip
 *   5. apply_patch     — decode + apply a pre-computed patch
 *   6. obj_set         — set 4 keys on an object + flush
 *   7. arr_ins×20      — insert 20 elements into an array + flush
 *   8. view            — read the JSON view of a steady-state document
 */

import { createRequire } from 'module';
import { performance } from 'perf_hooks';

// ── Load upstream json-joy (ESM-compatible CJS) ────────────────────────────
const _require = createRequire(import.meta.url);
const { Model: TS, Patch: TSPatch } =
  _require('./node_modules/json-joy/lib/json-crdt/index.js');

// ── Load WASM model ────────────────────────────────────────────────────────
const { Model: WASM } =
  _require('../crates/json-joy-wasm/pkg/json_joy_wasm.js');

// Note: getrandom's JS feature does not initialise in Node.js CJS require
// context, so we must always pass an explicit sid to new WASM(sid).  We use a
// simple counter — uniqueness within a single process is all we need here.
let _sidCounter = 65_536n;
const nextSid = () => _sidCounter++;

// ── Benchmark harness ──────────────────────────────────────────────────────

/**
 * @param {string} name
 * @param {number} n
 * @param {() => void} fn
 */
function bench(name, n, fn) {
  const warmup = Math.max(50, Math.floor(n / 10));
  for (let i = 0; i < warmup; i++) fn();

  const start = performance.now();
  for (let i = 0; i < n; i++) fn();
  const ms = performance.now() - start;

  return { name, opsPerSec: Math.round(n / (ms / 1000)), ms, n };
}

function printRow(label, ts, wasm) {
  const ratio = wasm.opsPerSec / ts.opsPerSec;
  const marker = ratio >= 0.9 ? '✓' : ratio >= 0.5 ? '~' : '✗';
  const dir =
    ratio >= 1
      ? `wasm ${ratio.toFixed(2)}× faster`
      : `wasm ${(1 / ratio).toFixed(2)}× slower`;
  console.log(
    `  ${marker} ${label.padEnd(18)}` +
    `${ts.opsPerSec.toLocaleString().padStart(13)} op/s  →` +
    `${wasm.opsPerSec.toLocaleString().padStart(13)} op/s   (${dir})`
  );
}

// ── Fixtures ───────────────────────────────────────────────────────────────

// Pre-compute a patch to use in the apply_patch benchmark
const _tsSender = TS.create(BigInt(1));
_tsSender.api.set({ title: 'Hello world', count: 42, tags: ['a', 'b', 'c'] });
const tsPatchBytes = _tsSender.api.flush().toBinary();
const tsPrebuiltPatch = TSPatch.fromBinary(tsPatchBytes);

const _wasmSender = new WASM(BigInt(1));
_wasmSender.apiSet(JSON.stringify({ title: 'Hello world', count: 42, tags: ['a', 'b', 'c'] }));
const wasmPatchBytes = _wasmSender.apiFlush();

// Pre-built steady-state documents for the view() benchmark
const _tsViewDoc = TS.create(BigInt(2));
_tsViewDoc.api.set({ name: 'Alice', age: 30, items: [1, 2, 3], active: true });

const _wasmViewDoc = new WASM(BigInt(2));
_wasmViewDoc.apiSet(JSON.stringify({ name: 'Alice', age: 30, items: [1, 2, 3], active: true }));

// ── Run ────────────────────────────────────────────────────────────────────

console.log('\n  json-joy  TypeScript vs  Rust/WASM\n');
console.log('  Legend: ✓ within 10%   ~ within 2×   ✗ >2× slower (WASM vs TS)');
console.log(`  ${'operation'.padEnd(18)}${'typescript'.padStart(13)} op/s    ${'wasm'.padStart(12)} op/s\n`);

// 1. model_create
{
  const ts   = bench('model_create', 50_000, () => TS.create());
  const wasm = bench('model_create', 50_000, () => new WASM(nextSid()));
  printRow('model_create', ts, wasm);
}

// 2. set + flush (simple object, no strings to CRDT-ify)
{
  const obj = { x: 1, y: 2, active: false };
  const objStr = JSON.stringify(obj);

  const ts = bench('set_flush', 10_000, () => {
    const m = TS.create();
    m.api.set(obj);
    m.api.flush();
  });

  const wasm = bench('set_flush', 10_000, () => {
    const m = new WASM(nextSid());
    m.apiSet(objStr);
    m.apiFlush();
  });

  printRow('set_flush', ts, wasm);
}

// 3. str_ins ×100 — simulates collaborative text editing
{
  const chars = 'abcdefghijklmnopqrstuvwxyz0123456789';
  const pathJson = 'null';

  const ts = bench('str_ins×100', 500, () => {
    const m = TS.create();
    m.api.set('');
    for (let i = 0; i < 100; i++) {
      m.api.str([]).ins(i, chars[i % chars.length]);
    }
    m.api.flush();
  });

  const wasm = bench('str_ins×100', 500, () => {
    const m = new WASM(nextSid());
    m.apiSet('""');
    for (let i = 0; i < 100; i++) {
      m.apiStrIns(pathJson, i, chars[i % chars.length]);
    }
    m.apiFlush();
  });

  printRow('str_ins×100', ts, wasm);
}

// 4. binary_rt — toBinary + fromBinary (serialisation roundtrip)
{
  const tsModel = TS.create(BigInt(3));
  tsModel.api.set({ name: 'test', items: [1, 2, 3], nested: { x: 0 } });
  const tsBin = tsModel.toBinary();

  const wasmModel = new WASM(BigInt(3));
  wasmModel.apiSet(JSON.stringify({ name: 'test', items: [1, 2, 3], nested: { x: 0 } }));
  const wasmBin = wasmModel.toBinary();

  const ts   = bench('binary_rt', 20_000, () => TS.fromBinary(tsBin));
  const wasm = bench('binary_rt', 20_000, () => WASM.fromBinary(wasmBin));
  printRow('binary_rt', ts, wasm);
}

// 5a. apply_patch (fair) — both sides decode from binary bytes each iteration
//     This simulates receiving a patch over the network.
{
  const ts   = bench('apply_patch', 10_000, () => {
    const m = TS.create();
    m.applyPatch(TSPatch.fromBinary(tsPatchBytes));
  });

  const wasm = bench('apply_patch', 10_000, () => {
    const m = new WASM(nextSid());
    m.applyPatch(wasmPatchBytes);
  });

  printRow('apply_patch', ts, wasm);
}

// 6. obj_set — partial update of 4 keys on an existing object
{
  const entries = { x: 1, y: 2, label: 'point', active: true };
  const entriesStr = JSON.stringify(entries);

  const ts = bench('obj_set', 20_000, () => {
    const m = TS.create();
    m.api.set({});
    m.api.obj([]).set(entries);
    m.api.flush();
  });

  const wasm = bench('obj_set', 20_000, () => {
    const m = new WASM(nextSid());
    m.apiSet('{}');
    m.apiObjSet('null', entriesStr);
    m.apiFlush();
  });

  printRow('obj_set', ts, wasm);
}

// 7. arr_ins ×20 — insert 20 elements at once
{
  const values = Array.from({ length: 20 }, (_, i) => i);
  const valuesStr = JSON.stringify(values);

  const ts = bench('arr_ins×20', 5_000, () => {
    const m = TS.create();
    m.api.set([]);
    m.api.arr([]).ins(0, values);
    m.api.flush();
  });

  const wasm = bench('arr_ins×20', 5_000, () => {
    const m = new WASM(nextSid());
    m.apiSet('[]');
    m.apiArrIns('null', 0, valuesStr);
    m.apiFlush();
  });

  printRow('arr_ins×20', ts, wasm);
}

// 8. view — read the full JSON view of a steady-state document
{
  const ts       = bench('view', 200_000, () => _tsViewDoc.view());
  const wasm     = bench('view', 200_000, () => _wasmViewDoc.view());
  printRow('view', ts, wasm);
}

// ── Experiments ────────────────────────────────────────────────────────────────
console.log('\n  ── Experiments ──────────────────────────────────────────────────');
console.log('  Alternate strategies that may close the gap.\n');

// E1. view_json — return JSON string from WASM, JSON.parse on JS side
//     Hypothesis: serde_json::to_string (pure Rust) + one string copy +
//     V8 JSON.parse is much cheaper than per-field serde_wasm_bindgen crossings.
{
  const ts       = bench('view (TS)', 200_000, () => _tsViewDoc.view());
  const wasmObj  = bench('view (wasm-obj)', 200_000, () => _wasmViewDoc.view());
  const wasmJson = bench('view (wasm-json)', 200_000, () => JSON.parse(_wasmViewDoc.viewJson()));
  console.log('  E1. view strategies:');
  printRow('  ts.view()', ts, ts);  // baseline reference
  printRow('  wasm obj', ts, wasmObj);
  printRow('  wasm JSON', ts, wasmJson);
}

// E2. str_ins batch — build all 100 char ops in ONE patch vs 100 separate patches
//     Hypothesis: most of the str_ins×100 cost is 100× apply_patch overhead;
//     batching into 1 patch reveals the pure CRDT allocation cost.
{
  const chars   = 'abcdefghijklmnopqrstuvwxyz0123456789';
  const pathJson = 'null';
  // Pre-compute ops array: [[0,"a"],[1,"b"],...]
  const batchOps = JSON.stringify(
    Array.from({ length: 100 }, (_, i) => [i, chars[i % chars.length]])
  );

  const ts = bench('str_ins×100 (TS)', 500, () => {
    const m = TS.create();
    m.api.set('');
    for (let i = 0; i < 100; i++) {
      m.api.str([]).ins(i, chars[i % chars.length]);
    }
    m.api.flush();
  });

  const wasmInd = bench('str_ins×100 (ind)', 500, () => {
    const m = new WASM(nextSid());
    m.apiSet('""');
    for (let i = 0; i < 100; i++) {
      m.apiStrIns(pathJson, i, chars[i % chars.length]);
    }
    m.apiFlush();
  });

  const wasmBatch = bench('str_ins×100 (batch)', 500, () => {
    const m = new WASM(nextSid());
    m.apiSet('""');
    m.apiStrInsBatch(pathJson, batchOps);
    m.apiFlush();
  });

  console.log('\n  E2. str_ins×100 strategies:');
  printRow('  ts', ts, ts);
  printRow('  wasm individual', ts, wasmInd);
  printRow('  wasm batch', ts, wasmBatch);
}

// E3. arr_ins typed array — pass Int32Array instead of JSON string
//     Hypothesis: serde_json parse of [0..19] is non-trivial; typed array
//     crossing skips all JSON encode/decode overhead.
{
  const values     = Array.from({ length: 20 }, (_, i) => i);
  const valuesStr  = JSON.stringify(values);
  const valuesI32  = new Int32Array(values);

  const ts = bench('arr_ins×20 (TS)', 5_000, () => {
    const m = TS.create();
    m.api.set([]);
    m.api.arr([]).ins(0, values);
    m.api.flush();
  });

  const wasmJson = bench('arr_ins×20 (JSON)', 5_000, () => {
    const m = new WASM(nextSid());
    m.apiSet('[]');
    m.apiArrIns('null', 0, valuesStr);
    m.apiFlush();
  });

  const wasmTyped = bench('arr_ins×20 (i32[])', 5_000, () => {
    const m = new WASM(nextSid());
    m.apiSet('[]');
    m.apiArrInsInts('null', 0, valuesI32);
    m.apiFlush();
  });

  console.log('\n  E3. arr_ins×20 strategies:');
  printRow('  ts', ts, ts);
  printRow('  wasm JSON', ts, wasmJson);
  printRow('  wasm i32[]', ts, wasmTyped);
}

console.log('\n  Geometry: smaller documents, so WASM boundary overhead is most visible here.');
console.log('  Run 2–3 times for stable numbers (V8 JIT warms up on first run).\n');
