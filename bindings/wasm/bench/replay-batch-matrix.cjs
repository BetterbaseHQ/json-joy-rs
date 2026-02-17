const fs = require('node:fs');
const path = require('node:path');

const {Model} = require('../../../tools/oracle-node/node_modules/json-joy/lib/json-crdt/index.js');
const {Patch} = require('../../../tools/oracle-node/node_modules/json-joy/lib/json-crdt-patch/index.js');
const wasmPkg = require('../../../crates/json-joy-wasm/pkg/json_joy_wasm.js');

const repoRoot = path.resolve(__dirname, '..', '..', '..');
const fixturesDir = path.join(repoRoot, 'tests', 'compat', 'fixtures');
const manifestPath = path.join(fixturesDir, 'manifest.json');

const sid = 65536n;
const DEFAULT_FIXTURE_LIMIT = 24;
const DEFAULT_WARMUP = 40;
const DEFAULT_RUNS = 250;
const DEFAULT_TRIALS = 5;
const DEFAULT_MIN_BENCH_MS = 120;

function readIntEnv(name, defaultValue) {
  const raw = process.env[name];
  if (!raw) return defaultValue;
  const parsed = Number.parseInt(raw, 10);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    throw new Error(`Invalid ${name}: ${raw}`);
  }
  return parsed;
}

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

function bench(runs, fn) {
  const start = process.hrtime.bigint();
  for (let i = 0; i < runs; i++) fn();
  const end = process.hrtime.bigint();
  return Number(end - start) / 1e6;
}

function selectEvenly(items, limit) {
  if (items.length <= limit) return items.slice();
  const chosen = [];
  const used = new Set();
  for (let i = 0; i < limit; i++) {
    const idx = Math.round((i * (items.length - 1)) / (limit - 1));
    if (!used.has(idx)) {
      used.add(idx);
      chosen.push(items[idx]);
    }
  }
  if (chosen.length < limit) {
    for (let i = 0; i < items.length && chosen.length < limit; i++) {
      if (!used.has(i)) chosen.push(items[i]);
    }
  }
  return chosen;
}

function parseFixture(entry) {
  const raw = JSON.parse(fs.readFileSync(path.join(fixturesDir, entry.file), 'utf8'));
  const baseModel = hexToBytes(raw.input.base_model_binary_hex);
  const patches = raw.input.patches_binary_hex.map(hexToBytes);
  const replayPatches = raw.input.replay_pattern.map((idx) => patches[idx]);
  return {
    name: raw.name,
    baseModel,
    replayOps: replayPatches.length,
    batch: encodeBatch(replayPatches),
    replayPatches,
  };
}

function calibrateRuns(caseData, warmup, baseRuns, minBenchMs) {
  let runs = baseRuns;
  while (true) {
    for (let i = 0; i < warmup; i++) {
      wasmPkg.patch_batch_apply_to_model(caseData.baseModel, caseData.batch, sid);
    }
    for (let i = 0; i < warmup; i++) {
      const model = Model.fromBinary(caseData.baseModel);
      for (const patchBytes of caseData.replayPatches) {
        model.applyPatch(Patch.fromBinary(patchBytes));
      }
      model.toBinary();
    }

    const wasmMs = bench(runs, () => {
      wasmPkg.patch_batch_apply_to_model(caseData.baseModel, caseData.batch, sid);
    });
    const upstreamMs = bench(runs, () => {
      const model = Model.fromBinary(caseData.baseModel);
      for (const patchBytes of caseData.replayPatches) {
        model.applyPatch(Patch.fromBinary(patchBytes));
      }
      model.toBinary();
    });
    if (wasmMs >= minBenchMs && upstreamMs >= minBenchMs) return runs;
    if (runs >= 1_000_000) return runs;
    runs *= 2;
  }
}

function runOne(caseData, warmup, baseRuns, trials, minBenchMs) {
  const runs = calibrateRuns(caseData, warmup, baseRuns, minBenchMs);
  const ratios = [];
  const wasmMsTrials = [];
  const upstreamMsTrials = [];

  for (let trial = 0; trial < trials; trial++) {
    for (let i = 0; i < warmup; i++) {
      wasmPkg.patch_batch_apply_to_model(caseData.baseModel, caseData.batch, sid);
    }
    for (let i = 0; i < warmup; i++) {
      const model = Model.fromBinary(caseData.baseModel);
      for (const patchBytes of caseData.replayPatches) {
        model.applyPatch(Patch.fromBinary(patchBytes));
      }
      model.toBinary();
    }

    let wasmMs;
    let upstreamMs;
    if (trial % 2 === 0) {
      wasmMs = bench(runs, () => {
        wasmPkg.patch_batch_apply_to_model(caseData.baseModel, caseData.batch, sid);
      });
      upstreamMs = bench(runs, () => {
        const model = Model.fromBinary(caseData.baseModel);
        for (const patchBytes of caseData.replayPatches) {
          model.applyPatch(Patch.fromBinary(patchBytes));
        }
        model.toBinary();
      });
    } else {
      upstreamMs = bench(runs, () => {
        const model = Model.fromBinary(caseData.baseModel);
        for (const patchBytes of caseData.replayPatches) {
          model.applyPatch(Patch.fromBinary(patchBytes));
        }
        model.toBinary();
      });
      wasmMs = bench(runs, () => {
        wasmPkg.patch_batch_apply_to_model(caseData.baseModel, caseData.batch, sid);
      });
    }

    const opsTotal = runs * caseData.replayOps;
    const wasmOpsPerSecond = (opsTotal * 1000) / wasmMs;
    const upstreamOpsPerSecond = (opsTotal * 1000) / upstreamMs;
    ratios.push(wasmOpsPerSecond / upstreamOpsPerSecond);
    wasmMsTrials.push(wasmMs);
    upstreamMsTrials.push(upstreamMs);
  }

  const wasmMs = median(wasmMsTrials);
  const upstreamMs = median(upstreamMsTrials);
  const opsTotal = runs * caseData.replayOps;
  const wasmOpsPerSecond = (opsTotal * 1000) / wasmMs;
  const upstreamOpsPerSecond = (opsTotal * 1000) / upstreamMs;

  return {
    name: caseData.name,
    replayOps: caseData.replayOps,
    runs,
    trials,
    wasmMs,
    upstreamMs,
    wasmOpsPerSecond,
    upstreamOpsPerSecond,
    ratio: median(ratios),
  };
}

function median(values) {
  if (values.length === 0) return 0;
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1 ? sorted[mid] : (sorted[mid - 1] + sorted[mid]) / 2;
}

function main() {
  const fixtureLimit = readIntEnv('WASM_BENCH_FIXTURE_LIMIT', DEFAULT_FIXTURE_LIMIT);
  const warmup = readIntEnv('WASM_BENCH_WARMUP', DEFAULT_WARMUP);
  const runs = readIntEnv('WASM_BENCH_RUNS', DEFAULT_RUNS);
  const trials = readIntEnv('WASM_BENCH_TRIALS', DEFAULT_TRIALS);
  const minBenchMs = readIntEnv('WASM_BENCH_MIN_MS', DEFAULT_MIN_BENCH_MS);

  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  const applyReplayFixtures = manifest.fixtures.filter((f) => f.scenario === 'model_apply_replay');
  const selected = selectEvenly(applyReplayFixtures, fixtureLimit);
  const cases = selected.map(parseFixture);

  const results = [];
  for (const caseData of cases) {
    const r = runOne(caseData, warmup, runs, trials, minBenchMs);
    results.push(r);
    console.log(
      `${r.name}: wasm ${r.wasmOpsPerSecond.toFixed(0)} ops/s | upstream ${r.upstreamOpsPerSecond.toFixed(0)} ops/s | ratio ${(r.ratio * 100).toFixed(1)}% (runs ${r.runs}, trials ${r.trials})`,
    );
  }

  const totalOps = results.reduce((sum, r) => sum + r.runs * r.replayOps, 0);
  const wasmTotalMs = results.reduce((sum, r) => sum + r.wasmMs, 0);
  const upstreamTotalMs = results.reduce((sum, r) => sum + r.upstreamMs, 0);
  const wasmAggregateOpsPerSecond = (totalOps * 1000) / wasmTotalMs;
  const upstreamAggregateOpsPerSecond = (totalOps * 1000) / upstreamTotalMs;
  const aggregateRatio = wasmAggregateOpsPerSecond / upstreamAggregateOpsPerSecond;
  const medianRatio = median(results.map((r) => r.ratio));

  console.log('');
  console.log(`fixtures: ${results.length} (from ${applyReplayFixtures.length} model_apply_replay fixtures)`);
  console.log(`warmup: ${warmup}, base runs: ${runs}, trials: ${trials}, min bench ms: ${minBenchMs}`);
  console.log(`aggregate wasm ops/s: ${wasmAggregateOpsPerSecond.toFixed(0)}`);
  console.log(`aggregate upstream ops/s: ${upstreamAggregateOpsPerSecond.toFixed(0)}`);
  console.log(`aggregate wasm/upstream ratio: ${(aggregateRatio * 100).toFixed(1)}%`);
  console.log(`median per-fixture ratio: ${(medianRatio * 100).toFixed(1)}%`);
}

main();
