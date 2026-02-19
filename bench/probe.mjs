import { createRequire } from 'module';
import { performance } from 'perf_hooks';

const _require = createRequire(import.meta.url);
const { Model: WASM } = _require('../crates/json-joy-wasm/pkg/json_joy_wasm.js');

function bench(n, fn) {
  for (let i = 0; i < Math.max(50, Math.floor(n / 10)); i++) fn();
  const start = performance.now();
  for (let i = 0; i < n; i++) fn();
  const ms = performance.now() - start;
  return Math.round(n / (ms / 1000));
}

let sid = 65_536n;
const obj     = JSON.stringify({ x: 1, y: 2, active: false });
const bigObj  = JSON.stringify({ name: 'Alice', age: 30, items: [1,2,3], active: true });
const text100 = 'abcdefghijklmnopqrstuvwxyz0123456789'.repeat(3).slice(0, 100);

console.log('\n── Probing key primitives ──\n');

// 1. diffApply vs apiSet+flush on a fresh document
const r1 = bench(10_000, () => { const m = new WASM(sid++); m.diffApply(obj); });
const r2 = bench(10_000, () => { const m = new WASM(sid++); m.apiSet(obj); m.apiFlush(); });
console.log(`diffApply (fresh, small obj):   ${r1.toLocaleString()} op/s`);
console.log(`apiSet+flush (same):            ${r2.toLocaleString()} op/s`);
console.log(`  ratio: diffApply is ${(r2/r1).toFixed(2)}× ${r2 > r1 ? 'faster' : 'slower'} than set+flush\n`);

// 2. diffApply on steady-state doc (no-op re-set of same value)
const m_ss = new WASM(sid++); m_ss.diffApply(obj);
const r3 = bench(20_000, () => m_ss.diffApply(obj));
console.log(`diffApply (no-op, same state):  ${r3.toLocaleString()} op/s\n`);

// 3. view strategies
const m_view = new WASM(sid++); m_view.diffApply(bigObj);
const cached  = JSON.parse(bigObj);
const r4 = bench(200_000, () => m_view.view());
const r5 = bench(200_000, () => JSON.parse(m_view.viewJson()));
const r6 = bench(200_000, () => cached);
console.log(`view (wasm obj):                ${r4.toLocaleString()} op/s`);
console.log(`viewJson + JSON.parse:          ${r5.toLocaleString()} op/s`);
console.log(`plain JS object access:         ${r6.toLocaleString()} op/s`);
console.log(`  cached is ${Math.round(r6/r4)}× faster than wasm obj, ${Math.round(r6/r5)}× faster than viewJson\n`);

// 4. Text: diffApply for 100-char string insert (fresh doc)
const r7 = bench(500, () => { const m = new WASM(sid++); m.diffApply(JSON.stringify(text100)); });
console.log(`diffApply (100-char str, fresh): ${r7.toLocaleString()} op/s`);

// 5. Text: diffApply for incremental char-by-char additions (realistic text editing)
// Each call adds one char to an existing doc — measures the incremental diff cost
const m_txt = new WASM(sid++);
const chars = 'abcdefghijklmnopqrstuvwxyz0123456789';
let current = '';
// Warm up: build to length 100
for (let i = 0; i < 100; i++) {
  current += chars[i % chars.length];
  m_txt.diffApply(JSON.stringify(current));
}
// Measure: add one more char to an existing 100-char string
const r8 = bench(500, () => {
  current += 'x';
  m_txt.diffApply(JSON.stringify(current));
});
console.log(`diffApply (add 1 char to 100-char str): ${r8.toLocaleString()} op/s`);
console.log(`  (grows unbounded — realistic text editing scenario)\n`);

// 6. Apply remote patch
const m_sender = new WASM(sid++);
m_sender.apiSet(JSON.stringify({ title: 'Hello world', count: 42, tags: ['a','b','c'] }));
const patchBytes = m_sender.apiFlush();
const r9 = bench(10_000, () => {
  const m = new WASM(sid++);
  m.applyPatch(patchBytes);
  JSON.parse(m.viewJson()); // cache refresh
});
const r10 = bench(10_000, () => {
  const m = new WASM(sid++);
  m.applyPatch(patchBytes);
  // no view refresh
});
console.log(`apply + viewJson refresh:       ${r9.toLocaleString()} op/s`);
console.log(`apply (no view):                ${r10.toLocaleString()} op/s`);
console.log(`  view refresh adds: ${((1/r9 - 1/r10)*1e6).toFixed(2)}µs per apply\n`);
