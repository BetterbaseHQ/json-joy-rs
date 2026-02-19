/**
 * Profile: break out where time is actually spent.
 *
 * Decomposes each slow benchmark into its constituent steps to find the
 * dominant cost.  Key questions:
 *
 *  Q1. arr_ins: is the bottleneck JSON parsing, con_val allocation, or RGA
 *      insertion?  Does cost scale linearly with prior array size? (O(n) vs
 *      O(log n) RGA detection)
 *
 *  Q2. str_ins: same question — does cost grow with string length?
 *
 *  Q3. apply_patch: how much is patch binary decode vs CRDT application?
 *
 *  Q4. obj_set: path resolve vs ins_obj?  1 key vs 4 keys?
 *
 *  Q5. view: does cost grow linearly with document size?
 *
 * Run:  node bench/bench-profile.mjs
 */

import { createRequire } from 'module';
import { performance }   from 'perf_hooks';

const _require = createRequire(import.meta.url);
const { Model: WASM } = _require('../crates/json-joy-wasm/pkg/json_joy_wasm.js');

let _sid = 1_000_000n;
const nextSid = () => _sid++;

function bench(n, fn) {
  const warmup = Math.max(200, Math.floor(n / 5));
  for (let i = 0; i < warmup; i++) fn();
  const start = performance.now();
  for (let i = 0; i < n; i++) fn();
  const ms = performance.now() - start;
  return { ops: Math.round(n / (ms / 1000)), us: (ms / n) * 1000 };
}

function scaled(label, sizes, n, mkFn) {
  console.log(`\n  ${label}`);
  console.log(`  ${'size'.padEnd(8)} ${'op/s'.padStart(14)} ${'µs/op'.padStart(10)} ${'vs size-0'.padStart(12)}`);
  const base = { us: null };
  for (const sz of sizes) {
    const fn  = mkFn(sz);
    const r   = bench(n, fn);
    const rel = base.us == null ? '  (baseline)' : `  ${(r.us / base.us).toFixed(2)}× slower`;
    if (base.us == null) base.us = r.us;
    console.log(`  ${String(sz).padEnd(8)} ${r.ops.toLocaleString().padStart(14)} ${r.us.toFixed(2).padStart(10)}  ${rel}`);
  }
}

// ── Helper: build a model with N items already in arr/str ────────────────────

function buildArrDoc(size) {
  const m = new WASM(nextSid());
  if (size === 0) {
    m.apiSet('[]');
    return m;
  }
  const values = JSON.stringify(Array.from({ length: size }, (_, i) => i));
  m.apiSet('[]');
  m.apiArrIns('null', 0, values);
  m.apiFlush();
  return m;
}

function buildStrDoc(size) {
  const m = new WASM(nextSid());
  m.apiSet('""');
  if (size > 0) {
    const text = 'a'.repeat(size);
    m.apiStrIns('null', 0, text);
    m.apiFlush();
  }
  return m;
}

// ── Q1. arr_ins: RGA scale test ──────────────────────────────────────────────
// Insert ONE element at END of an array that already has 0 / 10 / 100 / 500
// elements.  If O(n), cost at 500 is ~50× cost at 10.

console.log('\n═══════════════════════════════════════════════════════════');
console.log('  Profile: where does the time go?');
console.log('═══════════════════════════════════════════════════════════');

console.log('\n────────────────────────────────────────────────────────────');
console.log('  Q1.  arr_ins: does cost grow with array size? (O(n) test)');
console.log('────────────────────────────────────────────────────────────');

// Pre-build steady-state models at each size (exclude setup from measurement)
for (const sz of [0, 10, 50, 100, 500]) {
  const template = buildArrDoc(sz);
  // Clone via binary roundtrip to avoid shared state across iterations
  const bin = template.toBinary();
  const one = JSON.stringify([99]);

  const r = bench(5_000, () => {
    const m = WASM.fromBinary(bin);
    m.apiArrIns('null', sz, one);   // insert at END (requires walking all sz elements)
    // no flush — we only care about the RGA insertion cost
  });
  console.log(`  arr size ${String(sz).padEnd(4)}  insert at end:  ${r.ops.toLocaleString().padStart(12)} op/s  (${r.us.toFixed(2)} µs)`);
}

// Also test insert at BEGINNING (after = ORIGIN, no position scan)
console.log();
for (const sz of [0, 10, 100, 500]) {
  const template = buildArrDoc(sz);
  const bin = template.toBinary();
  const one = JSON.stringify([99]);

  const r = bench(5_000, () => {
    const m = WASM.fromBinary(bin);
    m.apiArrIns('null', 0, one);   // insert at BEGINNING (no position scan)
  });
  console.log(`  arr size ${String(sz).padEnd(4)}  insert at start: ${r.ops.toLocaleString().padStart(12)} op/s  (${r.us.toFixed(2)} µs)`);
}

// ── Q2. str_ins: RGA scale test ──────────────────────────────────────────────

console.log('\n────────────────────────────────────────────────────────────');
console.log('  Q2.  str_ins: does cost grow with string length? (O(n) test)');
console.log('────────────────────────────────────────────────────────────');

for (const sz of [0, 10, 50, 100, 500, 1000]) {
  const template = buildStrDoc(sz);
  const bin = template.toBinary();

  const r = bench(5_000, () => {
    const m = WASM.fromBinary(bin);
    m.apiStrIns('null', sz, 'x');   // insert at END (requires finding after=char[sz-1])
  });
  console.log(`  str len ${String(sz).padEnd(5)}  insert at end:   ${r.ops.toLocaleString().padStart(12)} op/s  (${r.us.toFixed(2)} µs)`);
}

console.log();
for (const sz of [0, 10, 100, 500, 1000]) {
  const template = buildStrDoc(sz);
  const bin = template.toBinary();

  const r = bench(5_000, () => {
    const m = WASM.fromBinary(bin);
    m.apiStrIns('null', 0, 'x');   // insert at BEGINNING (no position scan)
  });
  console.log(`  str len ${String(sz).padEnd(5)}  insert at start: ${r.ops.toLocaleString().padStart(12)} op/s  (${r.us.toFixed(2)} µs)`);
}

// ── Q3. apply_patch: decode vs apply ────────────────────────────────────────

console.log('\n────────────────────────────────────────────────────────────');
console.log('  Q3.  apply_patch: decode cost vs application cost');
console.log('────────────────────────────────────────────────────────────');

// Build patches of different sizes
function makePatch(numKeys) {
  const m = new WASM(nextSid());
  const obj = Object.fromEntries(
    Array.from({ length: numKeys }, (_, i) => [`k${i}`, i])
  );
  m.apiSet(JSON.stringify(obj));
  return m.apiFlush();
}

function makeStrPatch(chars) {
  const m = new WASM(nextSid());
  m.apiSet('""');
  m.apiStrIns('null', 0, 'a'.repeat(chars));
  return m.apiFlush();
}

const patches = {
  'obj-1key':   makePatch(1),
  'obj-4keys':  makePatch(4),
  'obj-10keys': makePatch(10),
  'str-10ch':   makeStrPatch(10),
  'str-100ch':  makeStrPatch(100),
};

for (const [label, bytes] of Object.entries(patches)) {
  // Full apply (decode + apply)
  const rFull = bench(20_000, () => {
    const m = new WASM(nextSid());
    m.applyPatch(bytes);
  });
  // Model create alone (to subtract)
  const rCreate = bench(20_000, () => new WASM(nextSid()));

  // Rough estimate: decode+apply ≈ rFull.us - rCreate.us
  const applyUs = Math.max(0, rFull.us - rCreate.us);
  console.log(`  patch ${label.padEnd(12)}  full: ${rFull.us.toFixed(2)} µs  (create: ${rCreate.us.toFixed(2)} µs  decode+apply est: ${applyUs.toFixed(2)} µs)`);
}

// ── Q4. obj_set: resolve cost vs ins_obj ────────────────────────────────────

console.log('\n────────────────────────────────────────────────────────────');
console.log('  Q4.  obj_set: resolve path vs ins_obj; 1 key vs 4 keys');
console.log('────────────────────────────────────────────────────────────');

const objBase = new WASM(nextSid());
objBase.apiSet('{}');
const objBin = objBase.toBinary();

// Vary number of keys written per call
for (const numKeys of [1, 2, 4, 8]) {
  const entries = JSON.stringify(
    Object.fromEntries(Array.from({ length: numKeys }, (_, i) => [`k${i}`, i]))
  );
  const r = bench(20_000, () => {
    const m = WASM.fromBinary(objBin);
    m.apiObjSet('null', entries);
  });
  console.log(`  obj_set ${numKeys} key(s):  ${r.ops.toLocaleString().padStart(12)} op/s  (${r.us.toFixed(2)} µs)`);
}

// Resolve overhead: fromBinary alone (model decode cost = lower bound for any op)
const r_decode = bench(50_000, () => WASM.fromBinary(objBin));
const r_1key   = bench(20_000, () => { const m = WASM.fromBinary(objBin); m.apiObjSet('null', '{"k":1}'); });
const r_8keys  = bench(20_000, () => { const m = WASM.fromBinary(objBin); m.apiObjSet('null', '{"k0":0,"k1":1,"k2":2,"k3":3,"k4":4,"k5":5,"k6":6,"k7":7}'); });
console.log(`\n  fromBinary only:          ${r_decode.ops.toLocaleString().padStart(12)} op/s  (${r_decode.us.toFixed(2)} µs)`);
console.log(`  obj_set 1 key:            ${r_1key.ops.toLocaleString().padStart(12)} op/s  (${r_1key.us.toFixed(2)} µs)`);
console.log(`  → CRDT overhead est:      ${Math.max(0, r_1key.us - r_decode.us).toFixed(2)} µs per op  (excl. fromBinary)`);

// ── Q5. view: does cost grow linearly with doc size? ────────────────────────
// Decode each model once, then benchmark view() on the in-memory model.

console.log('\n────────────────────────────────────────────────────────────');
console.log('  Q5.  view: cost vs document size (in-memory model, no decode)');
console.log('────────────────────────────────────────────────────────────');

for (const numFields of [1, 4, 10, 20, 50]) {
  const obj = Object.fromEntries(
    Array.from({ length: numFields }, (_, i) => [`key${i}`, i])
  );
  const m = new WASM(nextSid());
  m.apiSet(JSON.stringify(obj));

  const rObj  = bench(200_000, () => m.view());
  const rJson = bench(200_000, () => JSON.parse(m.viewJson()));
  console.log(`  ${String(numFields).padEnd(2)} fields:  view()=${rObj.us.toFixed(2)} µs  viewJson+parse=${rJson.us.toFixed(2)} µs`);
}

console.log('\n────────────────────────────────────────────────────────────');
console.log('  Q5b.  fromBinary: decode cost vs doc size');
console.log('────────────────────────────────────────────────────────────');

for (const numFields of [1, 4, 10, 20, 50]) {
  const obj = Object.fromEntries(
    Array.from({ length: numFields }, (_, i) => [`key${i}`, i])
  );
  const m = new WASM(nextSid());
  m.apiSet(JSON.stringify(obj));
  const bin = m.toBinary();
  const r = bench(20_000, () => WASM.fromBinary(bin));
  console.log(`  ${String(numFields).padEnd(2)} fields:  fromBinary=${r.us.toFixed(2)} µs  (${bin.byteLength} bytes)`);
}

console.log('\n');
