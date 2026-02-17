const fs = require('node:fs');
const path = require('node:path');

const {Model} = require('../../../tools/oracle-node/node_modules/json-joy/lib/json-crdt/index.js');
const {Patch} = require('../../../tools/oracle-node/node_modules/json-joy/lib/json-crdt-patch/index.js');
const wasmPkg = require('../../../crates/json-joy-wasm/pkg/json_joy_wasm.js');

const repoRoot = path.resolve(__dirname, '..', '..', '..');
const fixturePath = path.join(
  repoRoot,
  'tests',
  'compat',
  'fixtures',
  'model_apply_replay_116_vec_in_order_v1.json',
);

function hexToBytes(hex) {
  if (hex.length % 2 !== 0) throw new Error('hex length must be even');
  return new Uint8Array(Buffer.from(hex, 'hex'));
}

function encodeBatch(chunks) {
  let total = 4;
  for (const chunk of chunks) total += 4 + chunk.length;
  const out = new Uint8Array(total);
  const view = new DataView(out.buffer, out.byteOffset, out.byteLength);
  let cursor = 0;
  view.setUint32(cursor, chunks.length, true);
  cursor += 4;
  for (const chunk of chunks) {
    view.setUint32(cursor, chunk.length, true);
    cursor += 4;
    out.set(chunk, cursor);
    cursor += chunk.length;
  }
  return out;
}

function bench(name, runs, fn) {
  const start = process.hrtime.bigint();
  for (let i = 0; i < runs; i++) fn();
  const end = process.hrtime.bigint();
  const elapsedMs = Number(end - start) / 1e6;
  return {name, elapsedMs, avgMs: elapsedMs / runs};
}

const fixture = JSON.parse(fs.readFileSync(fixturePath, 'utf8'));
const baseModel = hexToBytes(fixture.input.base_model_binary_hex);
const patches = fixture.input.patches_binary_hex.map(hexToBytes);
const replayPatches = fixture.input.replay_pattern.map((idx) => patches[idx]);
const batch = encodeBatch(replayPatches);

const warmup = 200;
const runs = 3000;
const sid = 65536n;

for (let i = 0; i < warmup; i++) {
  wasmPkg.patch_batch_apply_to_model(baseModel, batch, sid);
}
for (let i = 0; i < warmup; i++) {
  const model = Model.fromBinary(baseModel);
  for (const patchBytes of replayPatches) {
    model.applyPatch(Patch.fromBinary(patchBytes));
  }
  model.toBinary();
}

const wasmResult = bench('wasm_patch_batch_apply_to_model', runs, () => {
  wasmPkg.patch_batch_apply_to_model(baseModel, batch, sid);
});

const upstreamResult = bench('upstream_model_apply_loop', runs, () => {
  const model = Model.fromBinary(baseModel);
  for (const patchBytes of replayPatches) {
    model.applyPatch(Patch.fromBinary(patchBytes));
  }
  model.toBinary();
});

const replayOps = replayPatches.length;
const wasmOpsPerSecond = (runs * replayOps * 1000) / wasmResult.elapsedMs;
const upstreamOpsPerSecond = (runs * replayOps * 1000) / upstreamResult.elapsedMs;
const ratio = wasmOpsPerSecond / upstreamOpsPerSecond;

console.log(`fixture: ${fixture.name}`);
console.log(`replay patches/call: ${replayOps}`);
console.log(`runs: ${runs}`);
console.log('');
console.log(`${wasmResult.name}:`);
console.log(`  total ms: ${wasmResult.elapsedMs.toFixed(2)}`);
console.log(`  avg ms/call: ${wasmResult.avgMs.toFixed(4)}`);
console.log(`  replay ops/s: ${wasmOpsPerSecond.toFixed(0)}`);
console.log('');
console.log(`${upstreamResult.name}:`);
console.log(`  total ms: ${upstreamResult.elapsedMs.toFixed(2)}`);
console.log(`  avg ms/call: ${upstreamResult.avgMs.toFixed(4)}`);
console.log(`  replay ops/s: ${upstreamOpsPerSecond.toFixed(0)}`);
console.log('');
console.log(`wasm/upstream throughput ratio: ${(ratio * 100).toFixed(1)}%`);
