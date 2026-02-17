const {Model} = require('../../../tools/oracle-node/node_modules/json-joy/lib/json-crdt/index.js');
const {Patch} = require('../../../tools/oracle-node/node_modules/json-joy/lib/json-crdt-patch/index.js');
const {
  CrdtEngine,
  appendPatchLog,
  encodePatchLog,
  emptyPatchLog,
  decodePatchLog,
  patchLogToBatch,
} = require('../sdk/index.cjs');

const SID = 65536;
const WARMUP = Number.parseInt(process.env.WASM_REAL_WARMUP || '20', 10);
const RUNS = Number.parseInt(process.env.WASM_REAL_RUNS || '120', 10);
const UPDATE_STEPS = Number.parseInt(process.env.WASM_REAL_STEPS || '120', 10);
const PROFILE_ONCE = process.env.WASM_REAL_PROFILE === '1';

function bench(name, fn) {
  for (let i = 0; i < WARMUP; i++) fn();
  const start = process.hrtime.bigint();
  for (let i = 0; i < RUNS; i++) fn();
  const end = process.hrtime.bigint();
  const elapsedMs = Number(end - start) / 1e6;
  return {name, elapsedMs, avgMs: elapsedMs / RUNS};
}

function deepClone(value) {
  return JSON.parse(JSON.stringify(value));
}

function makeInitialDoc() {
  return {
    id: 'rec-1',
    title: 'Draft',
    body: 'hello',
    tags: ['a', 'b'],
    counters: {views: 0, edits: 0},
    flags: {archived: false, starred: false},
    items: [{id: 1, done: false}, {id: 2, done: true}],
    nested: {s: 'ab', v: [1, 2, 3]},
  };
}

function mutateDoc(prev, step) {
  const next = deepClone(prev);
  next.title = `Draft-${step % 17}`;
  next.body = `${next.body.slice(0, 20)}${String.fromCharCode(97 + (step % 26))}`;
  next.counters.views += 1;
  next.counters.edits += step % 3;
  next.flags.starred = step % 2 === 0;
  if (step % 5 === 0) next.flags.archived = !next.flags.archived;
  next.tags.push(`t${step % 9}`);
  if (next.tags.length > 8) next.tags.shift();
  next.items.push({id: 1000 + step, done: step % 2 === 1});
  if (next.items.length > 10) next.items.shift();
  next.nested.s = `${next.nested.s}${step % 10}`.slice(-20);
  next.nested.v[(step + 1) % next.nested.v.length] = (step * 7) % 101;
  return next;
}

function makeUpdateSeries() {
  const docs = [];
  let cur = makeInitialDoc();
  for (let i = 0; i < UPDATE_STEPS; i++) {
    cur = mutateDoc(cur, i + 1);
    docs.push(cur);
  }
  return docs;
}

const updates = makeUpdateSeries();
const finalExpected = updates[updates.length - 1];

function upstreamPrepareUpdateFlow() {
  const create = Model.create(undefined, SID);
  create.api.set(makeInitialDoc());
  create.api.flush();
  let crdtBinary = create.toBinary();
  let patchLog = emptyPatchLog();

  for (const next of updates) {
    const model = Model.load(crdtBinary, SID);
    const patch = model.api.diff(next);
    if (patch) {
      const patchBinary = patch.toBinary();
      model.applyPatch(patch);
      crdtBinary = model.toBinary();
      patchLog = appendPatchLog(patchLog, patchBinary);
    }
  }

  const finalModel = Model.fromBinary(crdtBinary);
  return {crdtBinary, patchLog, view: finalModel.view()};
}

function wasmPrepareUpdateFlow() {
  const init = CrdtEngine.createEmpty(SID);
  init.diffApplyJson(makeInitialDoc());
  let crdtBinary = init.exportModel();
  init.dispose();

  let patchLog = emptyPatchLog();
  for (const next of updates) {
    const engine = CrdtEngine.fromModel(crdtBinary, SID);
    const patchBinary = engine.diffApplyJson(next);
    if (patchBinary.length > 0) {
      crdtBinary = engine.exportModel();
      patchLog = appendPatchLog(patchLog, patchBinary);
    }
    engine.dispose();
  }

  const finalEngine = CrdtEngine.fromModel(crdtBinary, SID);
  const view = finalEngine.viewJson();
  finalEngine.dispose();
  return {crdtBinary, patchLog, view};
}

function wasmPrepareUpdateFlowResident() {
  const engine = CrdtEngine.createEmpty(SID);
  engine.diffApplyJson(makeInitialDoc());
  let crdtBinary = engine.exportModel();
  let patchLog = emptyPatchLog();

  for (const next of updates) {
    const patchBinary = engine.diffApplyJson(next);
    if (patchBinary.length > 0) {
      crdtBinary = engine.exportModel();
      patchLog = appendPatchLog(patchLog, patchBinary);
    }
  }

  const view = engine.viewJson();
  engine.dispose();
  return {crdtBinary, patchLog, view};
}

function wasmPrepareUpdateFlowResidentCoarse() {
  const engine = CrdtEngine.createEmpty(SID);
  engine.diffApplyJson(makeInitialDoc());
  let crdtBinary = engine.exportModel();
  let patchLog = emptyPatchLog();

  for (const next of updates) {
    const out = engine.diffApplyExportJson(next, {includeModel: true, includeView: false});
    if (out.patchBinary.length > 0) {
      crdtBinary = out.modelBinary;
      patchLog = appendPatchLog(patchLog, out.patchBinary);
    }
  }

  const view = engine.viewJson();
  engine.dispose();
  return {crdtBinary, patchLog, view};
}

function wasmPrepareUpdateFlowResidentLazyExport() {
  const engine = CrdtEngine.createEmpty(SID);
  engine.diffApplyJson(makeInitialDoc());
  let patchLog = emptyPatchLog();

  for (const next of updates) {
    const patchBinary = engine.diffApplyJson(next);
    if (patchBinary.length > 0) {
      patchLog = appendPatchLog(patchLog, patchBinary);
    }
  }

  const crdtBinary = engine.exportModel();
  const view = engine.viewJson();
  engine.dispose();
  return {crdtBinary, patchLog, view};
}

function wasmPrepareUpdateFlowResidentLazyExportPatchArray() {
  const engine = CrdtEngine.createEmpty(SID);
  engine.diffApplyJson(makeInitialDoc());
  const patches = [];

  for (const next of updates) {
    const patchBinary = engine.diffApplyJson(next);
    if (patchBinary.length > 0) patches.push(patchBinary);
  }

  const crdtBinary = engine.exportModel();
  const view = engine.viewJson();
  engine.dispose();
  return {crdtBinary, patches, view};
}

function wasmPrepareUpdateFlowResidentLazyExportLogBuilder() {
  const engine = CrdtEngine.createEmpty(SID);
  engine.diffApplyJson(makeInitialDoc());
  const patches = [];

  for (const next of updates) {
    const patchBinary = engine.diffApplyJson(next);
    if (patchBinary.length > 0) patches.push(patchBinary);
  }

  const crdtBinary = engine.exportModel();
  const view = engine.viewJson();
  const patchLog = encodePatchLog(patches);
  engine.dispose();
  return {crdtBinary, patchLog, view};
}

function upstreamMergeRecordsLike(remoteCrdt, localPatchLog) {
  const remote = Model.fromBinary(remoteCrdt);
  for (const patchBytes of decodePatchLog(localPatchLog)) {
    remote.applyPatch(Patch.fromBinary(patchBytes));
  }
  return {crdtBinary: remote.toBinary(), view: remote.view()};
}

function wasmMergeRecordsLike(remoteCrdt, localPatchLog) {
  const engine = CrdtEngine.fromModel(remoteCrdt, SID);
  engine.applyPatchLog(localPatchLog);
  const out = {crdtBinary: engine.exportModel(), view: engine.viewJson()};
  engine.dispose();
  return out;
}

function wasmMergeRecordsLikeBatch(remoteCrdt, localPatchLog) {
  const engine = CrdtEngine.fromModel(remoteCrdt, SID);
  const batch = patchLogToBatch(localPatchLog);
  engine.applyPatchBatch(batch);
  const out = {crdtBinary: engine.exportModel(), view: engine.viewJson()};
  engine.dispose();
  return out;
}

function runScenarioUpstream() {
  const local = upstreamPrepareUpdateFlow();

  const remoteCreate = Model.create(undefined, SID + 1);
  remoteCreate.api.set(makeInitialDoc());
  remoteCreate.api.flush();
  const remotePatch = remoteCreate.api.diff(mutateDoc(makeInitialDoc(), 999));
  if (remotePatch) remoteCreate.applyPatch(remotePatch);
  const remoteCrdt = remoteCreate.toBinary();

  const merged = upstreamMergeRecordsLike(remoteCrdt, local.patchLog);
  return {local, merged};
}

function runScenarioWasm() {
  const local = wasmPrepareUpdateFlow();

  const remote = CrdtEngine.createEmpty(SID + 1);
  remote.diffApplyJson(makeInitialDoc());
  remote.diffApplyJson(mutateDoc(makeInitialDoc(), 999));
  const remoteCrdt = remote.exportModel();
  remote.dispose();

  const merged = wasmMergeRecordsLike(remoteCrdt, local.patchLog);
  return {local, merged};
}

function runScenarioWasmResident() {
  const local = wasmPrepareUpdateFlowResident();

  const remote = CrdtEngine.createEmpty(SID + 1);
  remote.diffApplyJson(makeInitialDoc());
  remote.diffApplyJson(mutateDoc(makeInitialDoc(), 999));
  const remoteCrdt = remote.exportModel();
  remote.dispose();

  const merged = wasmMergeRecordsLike(remoteCrdt, local.patchLog);
  return {local, merged};
}

function runScenarioWasmResidentCoarse() {
  const local = wasmPrepareUpdateFlowResidentCoarse();

  const remote = CrdtEngine.createEmpty(SID + 1);
  remote.diffApplyJson(makeInitialDoc());
  remote.diffApplyJson(mutateDoc(makeInitialDoc(), 999));
  const remoteCrdt = remote.exportModel();
  remote.dispose();

  const merged = wasmMergeRecordsLike(remoteCrdt, local.patchLog);
  return {local, merged};
}

function runScenarioWasmResidentLazyExport() {
  const local = wasmPrepareUpdateFlowResidentLazyExport();

  const remote = CrdtEngine.createEmpty(SID + 1);
  remote.diffApplyJson(makeInitialDoc());
  remote.diffApplyJson(mutateDoc(makeInitialDoc(), 999));
  const remoteCrdt = remote.exportModel();
  remote.dispose();

  const merged = wasmMergeRecordsLike(remoteCrdt, local.patchLog);
  return {local, merged};
}

function runScenarioWasmResidentLazyExportBatchMerge() {
  const local = wasmPrepareUpdateFlowResidentLazyExport();

  const remote = CrdtEngine.createEmpty(SID + 1);
  remote.diffApplyJson(makeInitialDoc());
  remote.diffApplyJson(mutateDoc(makeInitialDoc(), 999));
  const remoteCrdt = remote.exportModel();
  remote.dispose();

  const merged = wasmMergeRecordsLikeBatch(remoteCrdt, local.patchLog);
  return {local, merged};
}

function runScenarioWasmResidentLazyExportDirectBatchMerge() {
  const local = wasmPrepareUpdateFlowResidentLazyExportPatchArray();

  const remote = CrdtEngine.createEmpty(SID + 1);
  remote.diffApplyJson(makeInitialDoc());
  remote.diffApplyJson(mutateDoc(makeInitialDoc(), 999));
  const remoteCrdt = remote.exportModel();
  remote.dispose();

  const engine = CrdtEngine.fromModel(remoteCrdt, SID);
  engine.applyPatchBatch(local.patches);
  const merged = {crdtBinary: engine.exportModel(), view: engine.viewJson()};
  engine.dispose();
  return {local, merged};
}

function runScenarioWasmResidentLazyExportLogBuilderBatchMerge() {
  const local = wasmPrepareUpdateFlowResidentLazyExportLogBuilder();

  const remote = CrdtEngine.createEmpty(SID + 1);
  remote.diffApplyJson(makeInitialDoc());
  remote.diffApplyJson(mutateDoc(makeInitialDoc(), 999));
  const remoteCrdt = remote.exportModel();
  remote.dispose();

  const merged = wasmMergeRecordsLikeBatch(remoteCrdt, local.patchLog);
  return {local, merged};
}

function nsToMs(ns) {
  return Number(ns) / 1e6;
}

function profileUpstreamOnce() {
  const stats = {
    diffNs: 0n,
    applyNs: 0n,
    exportNs: 0n,
    patchLogNs: 0n,
    mergeApplyNs: 0n,
    mergeExportNs: 0n,
  };

  const create = Model.create(undefined, SID);
  create.api.set(makeInitialDoc());
  create.api.flush();
  let crdtBinary = create.toBinary();
  let patchLog = emptyPatchLog();

  for (const next of updates) {
    const model = Model.load(crdtBinary, SID);

    const t0 = process.hrtime.bigint();
    const patch = model.api.diff(next);
    const t1 = process.hrtime.bigint();
    stats.diffNs += t1 - t0;

    if (patch) {
      const patchBinary = patch.toBinary();

      const t2 = process.hrtime.bigint();
      model.applyPatch(patch);
      const t3 = process.hrtime.bigint();
      stats.applyNs += t3 - t2;

      const t4 = process.hrtime.bigint();
      crdtBinary = model.toBinary();
      const t5 = process.hrtime.bigint();
      stats.exportNs += t5 - t4;

      const t6 = process.hrtime.bigint();
      patchLog = appendPatchLog(patchLog, patchBinary);
      const t7 = process.hrtime.bigint();
      stats.patchLogNs += t7 - t6;
    }
  }

  const remoteCreate = Model.create(undefined, SID + 1);
  remoteCreate.api.set(makeInitialDoc());
  remoteCreate.api.flush();
  const remotePatch = remoteCreate.api.diff(mutateDoc(makeInitialDoc(), 999));
  if (remotePatch) remoteCreate.applyPatch(remotePatch);
  const remoteCrdt = remoteCreate.toBinary();
  const remote = Model.fromBinary(remoteCrdt);

  const mergePatches = decodePatchLog(patchLog);
  const t8 = process.hrtime.bigint();
  for (const p of mergePatches) remote.applyPatch(Patch.fromBinary(p));
  const t9 = process.hrtime.bigint();
  stats.mergeApplyNs += t9 - t8;

  const t10 = process.hrtime.bigint();
  remote.toBinary();
  remote.view();
  const t11 = process.hrtime.bigint();
  stats.mergeExportNs += t11 - t10;

  return stats;
}

function profileWasmResidentLazyExportBatchMergeOnce() {
  const stats = {
    diffApplyNs: 0n,
    exportNs: 0n,
    patchLogNs: 0n,
    mergeApplyNs: 0n,
    mergeExportNs: 0n,
  };

  const engine = CrdtEngine.createEmpty(SID);
  engine.diffApplyJson(makeInitialDoc());
  let patchLog = emptyPatchLog();

  for (const next of updates) {
    const t0 = process.hrtime.bigint();
    const patchBinary = engine.diffApplyJson(next);
    const t1 = process.hrtime.bigint();
    stats.diffApplyNs += t1 - t0;

    if (patchBinary.length > 0) {
      const t2 = process.hrtime.bigint();
      patchLog = appendPatchLog(patchLog, patchBinary);
      const t3 = process.hrtime.bigint();
      stats.patchLogNs += t3 - t2;
    }
  }

  const t4 = process.hrtime.bigint();
  engine.exportModel();
  engine.dispose();
  const t5 = process.hrtime.bigint();
  stats.exportNs += t5 - t4;

  const remote = CrdtEngine.createEmpty(SID + 1);
  remote.diffApplyJson(makeInitialDoc());
  remote.diffApplyJson(mutateDoc(makeInitialDoc(), 999));
  const remoteCrdt = remote.exportModel();
  remote.dispose();

  const mergeEngine = CrdtEngine.fromModel(remoteCrdt, SID);
  const batch = patchLogToBatch(patchLog);
  const t6 = process.hrtime.bigint();
  mergeEngine.applyPatchBatch(batch);
  const t7 = process.hrtime.bigint();
  stats.mergeApplyNs += t7 - t6;

  const t8 = process.hrtime.bigint();
  mergeEngine.exportModel();
  mergeEngine.viewJson();
  mergeEngine.dispose();
  const t9 = process.hrtime.bigint();
  stats.mergeExportNs += t9 - t8;

  return stats;
}

function assertJsonEqual(name, a, b) {
  const sa = JSON.stringify(a);
  const sb = JSON.stringify(b);
  if (sa !== sb) throw new Error(`${name} mismatch`);
}

// correctness pre-check
{
  const upstream = runScenarioUpstream();
  const wasm = runScenarioWasm();
  const wasmResident = runScenarioWasmResident();
  const wasmResidentCoarse = runScenarioWasmResidentCoarse();
  const wasmResidentLazy = runScenarioWasmResidentLazyExport();
  const wasmResidentLazyBatch = runScenarioWasmResidentLazyExportBatchMerge();
  const wasmResidentLazyDirectBatch = runScenarioWasmResidentLazyExportDirectBatchMerge();
  const wasmResidentLazyLogBuilderBatch = runScenarioWasmResidentLazyExportLogBuilderBatchMerge();
  assertJsonEqual('local final', upstream.local.view, wasm.local.view);
  assertJsonEqual('merged final', upstream.merged.view, wasm.merged.view);
  assertJsonEqual('local final resident', upstream.local.view, wasmResident.local.view);
  assertJsonEqual('merged final resident', upstream.merged.view, wasmResident.merged.view);
  assertJsonEqual('local final resident coarse', upstream.local.view, wasmResidentCoarse.local.view);
  assertJsonEqual('merged final resident coarse', upstream.merged.view, wasmResidentCoarse.merged.view);
  assertJsonEqual('local final resident lazy', upstream.local.view, wasmResidentLazy.local.view);
  assertJsonEqual('merged final resident lazy', upstream.merged.view, wasmResidentLazy.merged.view);
  assertJsonEqual('local final resident lazy batch', upstream.local.view, wasmResidentLazyBatch.local.view);
  assertJsonEqual('merged final resident lazy batch', upstream.merged.view, wasmResidentLazyBatch.merged.view);
  assertJsonEqual('local final resident lazy direct-batch', upstream.local.view, wasmResidentLazyDirectBatch.local.view);
  assertJsonEqual('merged final resident lazy direct-batch', upstream.merged.view, wasmResidentLazyDirectBatch.merged.view);
  assertJsonEqual('local final resident lazy logbuilder+batch', upstream.local.view, wasmResidentLazyLogBuilderBatch.local.view);
  assertJsonEqual('merged final resident lazy logbuilder+batch', upstream.merged.view, wasmResidentLazyLogBuilderBatch.merged.view);
  assertJsonEqual('expected final', wasm.local.view, finalExpected);
}

const upstreamPerf = bench('upstream_lessdb_like', runScenarioUpstream);
const wasmPerf = bench('wasm_sdk_lessdb_like_stateless', runScenarioWasm);
const wasmResidentPerf = bench('wasm_sdk_lessdb_like_resident', runScenarioWasmResident);
const wasmResidentCoarsePerf = bench('wasm_sdk_lessdb_like_resident_coarse', runScenarioWasmResidentCoarse);
const wasmResidentLazyPerf = bench('wasm_sdk_lessdb_like_resident_lazy_export', runScenarioWasmResidentLazyExport);
const wasmResidentLazyBatchPerf = bench(
  'wasm_sdk_lessdb_like_resident_lazy_export_batch_merge',
  runScenarioWasmResidentLazyExportBatchMerge,
);
const wasmResidentLazyDirectBatchPerf = bench(
  'wasm_sdk_lessdb_like_resident_lazy_export_direct_batch_merge',
  runScenarioWasmResidentLazyExportDirectBatchMerge,
);
const wasmResidentLazyLogBuilderBatchPerf = bench(
  'wasm_sdk_lessdb_like_resident_lazy_export_logbuilder_batch_merge',
  runScenarioWasmResidentLazyExportLogBuilderBatchMerge,
);

const upstreamOpsPerSecond = RUNS * 1000 / upstreamPerf.elapsedMs;
const wasmOpsPerSecond = RUNS * 1000 / wasmPerf.elapsedMs;
const wasmResidentOpsPerSecond = RUNS * 1000 / wasmResidentPerf.elapsedMs;
const wasmResidentCoarseOpsPerSecond = RUNS * 1000 / wasmResidentCoarsePerf.elapsedMs;
const wasmResidentLazyOpsPerSecond = RUNS * 1000 / wasmResidentLazyPerf.elapsedMs;
const wasmResidentLazyBatchOpsPerSecond = RUNS * 1000 / wasmResidentLazyBatchPerf.elapsedMs;
const wasmResidentLazyDirectBatchOpsPerSecond = RUNS * 1000 / wasmResidentLazyDirectBatchPerf.elapsedMs;
const wasmResidentLazyLogBuilderBatchOpsPerSecond = RUNS * 1000 / wasmResidentLazyLogBuilderBatchPerf.elapsedMs;

console.log(`updates per scenario: ${UPDATE_STEPS}`);
console.log(`warmup: ${WARMUP}, runs: ${RUNS}`);
console.log('');
console.log(`${upstreamPerf.name}:`);
console.log(`  total ms: ${upstreamPerf.elapsedMs.toFixed(2)}`);
console.log(`  avg ms/scenario: ${upstreamPerf.avgMs.toFixed(3)}`);
console.log(`  scenarios/s: ${upstreamOpsPerSecond.toFixed(2)}`);
console.log('');
console.log(`${wasmPerf.name}:`);
console.log(`  total ms: ${wasmPerf.elapsedMs.toFixed(2)}`);
console.log(`  avg ms/scenario: ${wasmPerf.avgMs.toFixed(3)}`);
console.log(`  scenarios/s: ${wasmOpsPerSecond.toFixed(2)}`);
console.log('');
console.log(`${wasmResidentPerf.name}:`);
console.log(`  total ms: ${wasmResidentPerf.elapsedMs.toFixed(2)}`);
console.log(`  avg ms/scenario: ${wasmResidentPerf.avgMs.toFixed(3)}`);
console.log(`  scenarios/s: ${wasmResidentOpsPerSecond.toFixed(2)}`);
console.log('');
console.log(`${wasmResidentCoarsePerf.name}:`);
console.log(`  total ms: ${wasmResidentCoarsePerf.elapsedMs.toFixed(2)}`);
console.log(`  avg ms/scenario: ${wasmResidentCoarsePerf.avgMs.toFixed(3)}`);
console.log(`  scenarios/s: ${wasmResidentCoarseOpsPerSecond.toFixed(2)}`);
console.log('');
console.log(`${wasmResidentLazyPerf.name}:`);
console.log(`  total ms: ${wasmResidentLazyPerf.elapsedMs.toFixed(2)}`);
console.log(`  avg ms/scenario: ${wasmResidentLazyPerf.avgMs.toFixed(3)}`);
console.log(`  scenarios/s: ${wasmResidentLazyOpsPerSecond.toFixed(2)}`);
console.log('');
console.log(`${wasmResidentLazyBatchPerf.name}:`);
console.log(`  total ms: ${wasmResidentLazyBatchPerf.elapsedMs.toFixed(2)}`);
console.log(`  avg ms/scenario: ${wasmResidentLazyBatchPerf.avgMs.toFixed(3)}`);
console.log(`  scenarios/s: ${wasmResidentLazyBatchOpsPerSecond.toFixed(2)}`);
console.log('');
console.log(`${wasmResidentLazyDirectBatchPerf.name}:`);
console.log(`  total ms: ${wasmResidentLazyDirectBatchPerf.elapsedMs.toFixed(2)}`);
console.log(`  avg ms/scenario: ${wasmResidentLazyDirectBatchPerf.avgMs.toFixed(3)}`);
console.log(`  scenarios/s: ${wasmResidentLazyDirectBatchOpsPerSecond.toFixed(2)}`);
console.log('');
console.log(`${wasmResidentLazyLogBuilderBatchPerf.name}:`);
console.log(`  total ms: ${wasmResidentLazyLogBuilderBatchPerf.elapsedMs.toFixed(2)}`);
console.log(`  avg ms/scenario: ${wasmResidentLazyLogBuilderBatchPerf.avgMs.toFixed(3)}`);
console.log(`  scenarios/s: ${wasmResidentLazyLogBuilderBatchOpsPerSecond.toFixed(2)}`);
console.log('');
console.log(`wasm stateless/upstream ratio: ${((wasmOpsPerSecond / upstreamOpsPerSecond) * 100).toFixed(1)}%`);
console.log(`wasm resident/upstream ratio: ${((wasmResidentOpsPerSecond / upstreamOpsPerSecond) * 100).toFixed(1)}%`);
console.log(`wasm resident coarse/upstream ratio: ${((wasmResidentCoarseOpsPerSecond / upstreamOpsPerSecond) * 100).toFixed(1)}%`);
console.log(`wasm resident lazy-export/upstream ratio: ${((wasmResidentLazyOpsPerSecond / upstreamOpsPerSecond) * 100).toFixed(1)}%`);
console.log(`wasm resident lazy-export+batch-merge/upstream ratio: ${((wasmResidentLazyBatchOpsPerSecond / upstreamOpsPerSecond) * 100).toFixed(1)}%`);
console.log(
  `wasm resident lazy-export+direct-batch-merge ratio: ${((wasmResidentLazyDirectBatchOpsPerSecond / upstreamOpsPerSecond) * 100).toFixed(1)}%`,
);
console.log(
  `wasm resident lazy-export+logbuilder+batch-merge ratio: ${((wasmResidentLazyLogBuilderBatchOpsPerSecond / upstreamOpsPerSecond) * 100).toFixed(1)}%`,
);

if (PROFILE_ONCE) {
  const u = profileUpstreamOnce();
  const w = profileWasmResidentLazyExportBatchMergeOnce();
  console.log('');
  console.log('phase profile (single scenario):');
  console.log(`  upstream diff ms: ${nsToMs(u.diffNs).toFixed(2)}`);
  console.log(`  upstream apply ms: ${nsToMs(u.applyNs).toFixed(2)}`);
  console.log(`  upstream export ms: ${nsToMs(u.exportNs).toFixed(2)}`);
  console.log(`  upstream patch-log ms: ${nsToMs(u.patchLogNs).toFixed(2)}`);
  console.log(`  upstream merge-apply ms: ${nsToMs(u.mergeApplyNs).toFixed(2)}`);
  console.log(`  upstream merge-export ms: ${nsToMs(u.mergeExportNs).toFixed(2)}`);
  console.log(`  wasm diff+apply ms: ${nsToMs(w.diffApplyNs).toFixed(2)}`);
  console.log(`  wasm export ms: ${nsToMs(w.exportNs).toFixed(2)}`);
  console.log(`  wasm patch-log ms: ${nsToMs(w.patchLogNs).toFixed(2)}`);
  console.log(`  wasm merge-apply ms: ${nsToMs(w.mergeApplyNs).toFixed(2)}`);
  console.log(`  wasm merge-export ms: ${nsToMs(w.mergeExportNs).toFixed(2)}`);
}
