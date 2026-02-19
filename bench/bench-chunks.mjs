import { createRequire } from 'module';
import { performance } from 'perf_hooks';
const _require = createRequire(import.meta.url);
const { Model: WASM } = _require('../crates/json-joy-wasm/pkg/json_joy_wasm.js');

function bench(n, fn) {
  for (let i = 0; i < Math.max(100, Math.floor(n/5)); i++) fn();
  const t = performance.now();
  for (let i = 0; i < n; i++) fn();
  return { us: ((performance.now()-t)/n)*1000 };
}

let sid = 999_000n;
const next = () => sid++;

// Build a sz-length string as ONE chunk
function buildOneChunk(sz) {
  const m = new WASM(next());
  m.apiSet('""');
  if (sz > 0) { m.apiStrIns('null', 0, 'a'.repeat(sz)); m.apiFlush(); }
  return m.toBinary();
}
// Build a sz-length string as sz separate 1-char chunks (sequential inserts)
function buildManyChunks(sz) {
  const m = new WASM(next());
  m.apiSet('""');
  for (let i = 0; i < sz; i++) { m.apiStrIns('null', i, 'a'); m.apiFlush(); }
  return m.toBinary();
}

console.log('\n  str_ins at END: 1 big chunk vs N individual chunks');
console.log('  Hypothesis: O(n) is driven by chunk count, not string length.\n');
console.log('  size   1-chunk       N-chunks      ratio');

for (const sz of [10, 50, 100, 200]) {
  const bin1 = buildOneChunk(sz);
  const binN = buildManyChunks(sz);
  const r1 = bench(3000, () => { const m = WASM.fromBinary(bin1); m.apiStrIns('null', sz, 'x'); });
  const rN = bench(3000, () => { const m = WASM.fromBinary(binN); m.apiStrIns('null', sz, 'x'); });
  console.log(`  ${String(sz).padEnd(6)} ${r1.us.toFixed(1).padStart(8)} µs    ${rN.us.toFixed(1).padStart(8)} µs    ${(rN.us/r1.us).toFixed(1)}×`);
}
