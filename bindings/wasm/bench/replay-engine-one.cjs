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
const runs = 4000;
const sid = 65536n;

for (let i = 0; i < warmup; i++) {
  wasmPkg.patch_batch_apply_to_model(baseModel, batch, sid);
}
for (let i = 0; i < warmup; i++) {
  const id = wasmPkg.engine_create_from_model(baseModel, sid);
  wasmPkg.engine_apply_patch_batch(id, batch);
  wasmPkg.engine_free(id);
}
for (let i = 0; i < warmup; i++) {
  const id = wasmPkg.engine_create_from_model(baseModel, sid);
  wasmPkg.engine_apply_patch_batch(id, batch);
  wasmPkg.engine_export_model(id);
  wasmPkg.engine_free(id);
}
for (let i = 0; i < warmup; i++) {
  const model = Model.load(baseModel, Number(sid));
  for (const patchBytes of replayPatches) {
    model.applyPatch(Patch.fromBinary(patchBytes));
  }
  model.toBinary();
}

const stateless = bench('wasm_stateless_patch_batch_apply_to_model', runs, () => {
  wasmPkg.patch_batch_apply_to_model(baseModel, batch, sid);
});

const engineApplyOnly = bench('wasm_engine_create_apply_free', runs, () => {
  const id = wasmPkg.engine_create_from_model(baseModel, sid);
  wasmPkg.engine_apply_patch_batch(id, batch);
  wasmPkg.engine_free(id);
});

const engineApplyExport = bench('wasm_engine_create_apply_export_free', runs, () => {
  const id = wasmPkg.engine_create_from_model(baseModel, sid);
  wasmPkg.engine_apply_patch_batch(id, batch);
  wasmPkg.engine_export_model(id);
  wasmPkg.engine_free(id);
});

const upstream = bench('upstream_model_load_apply_toBinary', runs, () => {
  const model = Model.load(baseModel, Number(sid));
  for (const patchBytes of replayPatches) {
    model.applyPatch(Patch.fromBinary(patchBytes));
  }
  model.toBinary();
});

const precreatedIds1 = [];
for (let i = 0; i < runs; i++) precreatedIds1.push(wasmPkg.engine_create_from_model(baseModel, sid));
const enginePrecreatedApplyOnly = bench('wasm_engine_precreated_apply_only', runs, () => {
  const id = precreatedIds1.pop();
  wasmPkg.engine_apply_patch_batch(id, batch);
  wasmPkg.engine_free(id);
});

const precreatedIds2 = [];
for (let i = 0; i < runs; i++) precreatedIds2.push(wasmPkg.engine_create_from_model(baseModel, sid));
const enginePrecreatedApplyExport = bench('wasm_engine_precreated_apply_export', runs, () => {
  const id = precreatedIds2.pop();
  wasmPkg.engine_apply_patch_batch(id, batch);
  wasmPkg.engine_export_model(id);
  wasmPkg.engine_free(id);
});

const replayOps = replayPatches.length;
const results = [
  stateless,
  engineApplyOnly,
  engineApplyExport,
  enginePrecreatedApplyOnly,
  enginePrecreatedApplyExport,
  upstream,
].map((result) => {
  const opsPerSecond = (runs * replayOps * 1000) / result.elapsedMs;
  return {...result, opsPerSecond};
});

for (const result of results) {
  console.log(`${result.name}:`);
  console.log(`  total ms: ${result.elapsedMs.toFixed(2)}`);
  console.log(`  avg ms/call: ${result.avgMs.toFixed(4)}`);
  console.log(`  replay ops/s: ${result.opsPerSecond.toFixed(0)}`);
  console.log('');
}

function ratio(a, b) {
  return ((a.opsPerSecond / b.opsPerSecond) * 100).toFixed(1);
}

const byName = Object.fromEntries(results.map((r) => [r.name, r]));

console.log(
  `engine create+apply+free / stateless: ${ratio(byName.wasm_engine_create_apply_free, byName.wasm_stateless_patch_batch_apply_to_model)}%`,
);
console.log(
  `engine precreated apply-only / stateless: ${ratio(byName.wasm_engine_precreated_apply_only, byName.wasm_stateless_patch_batch_apply_to_model)}%`,
);
console.log(
  `engine precreated apply+export / stateless: ${ratio(byName.wasm_engine_precreated_apply_export, byName.wasm_stateless_patch_batch_apply_to_model)}%`,
);
console.log(
  `engine precreated apply+export / upstream: ${ratio(byName.wasm_engine_precreated_apply_export, byName.upstream_model_load_apply_toBinary)}%`,
);
