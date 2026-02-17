const fs = require('node:fs');
const path = require('node:path');
const {Model} = require('../../../tools/oracle-node/node_modules/json-joy/lib/json-crdt/index.js');
const {Patch} = require('../../../tools/oracle-node/node_modules/json-joy/lib/json-crdt-patch/index.js');
const wasm = require('../../../crates/json-joy-wasm/pkg/json_joy_wasm.js');

const repoRoot = path.resolve(__dirname, '..', '..', '..');
const fixturesDir = path.join(repoRoot, 'tests', 'compat', 'fixtures');
const manifest = JSON.parse(fs.readFileSync(path.join(fixturesDir, 'manifest.json'), 'utf8'));
const diffFixtures = manifest.fixtures.filter((f) => f.scenario === 'model_diff_parity');

function hexToBytes(hex) {
  return new Uint8Array(Buffer.from(hex, 'hex'));
}

function bytesToHex(bytes) {
  return Buffer.from(bytes).toString('hex');
}

const limit = Number.parseInt(process.env.WASM_INTEROP_LIMIT || '80', 10);
const selected = diffFixtures.slice(0, Math.min(limit, diffFixtures.length));

let total = 0;
let pass = 0;

for (const entry of selected) {
  total += 1;
  const fixture = JSON.parse(fs.readFileSync(path.join(fixturesDir, entry.file), 'utf8'));
  const base = hexToBytes(fixture.input.base_model_binary_hex);
  const nextJson = fixture.input.next_view_json;
  const sid = BigInt(fixture.input.sid);
  const expectedView = fixture.expected.view_after_apply_json;

  try {
    // Upstream patch -> WASM apply
    const upstreamModel = Model.load(base, Number(sid));
    const upstreamPatchObj = upstreamModel.api.diff(nextJson);
    const upstreamPatchBin = upstreamPatchObj ? upstreamPatchObj.toBinary() : new Uint8Array(0);

    const wasmEngineA = wasm.engine_create_from_model(base, sid);
    if (upstreamPatchBin.length > 0) wasm.engine_apply_patch(wasmEngineA, upstreamPatchBin);
    const wasmViewA = JSON.parse(Buffer.from(wasm.engine_export_view_json(wasmEngineA)).toString('utf8'));
    wasm.engine_free(wasmEngineA);

    // WASM patch -> upstream apply
    const wasmEngineB = wasm.engine_create_from_model(base, sid);
    const wasmPatchBin = wasm.engine_diff_json(wasmEngineB, Buffer.from(JSON.stringify(nextJson)));
    wasm.engine_free(wasmEngineB);

    const upstreamApplyModel = Model.load(base, Number(sid));
    if (wasmPatchBin.length > 0) {
      upstreamApplyModel.applyPatch(Patch.fromBinary(wasmPatchBin));
    }
    const upstreamViewB = upstreamApplyModel.view();

    const okA = JSON.stringify(wasmViewA) === JSON.stringify(expectedView);
    const okB = JSON.stringify(upstreamViewB) === JSON.stringify(expectedView);

    // Optional binary-level signal: ensure non-empty patches from both sides decode upstream.
    if (wasmPatchBin.length > 0) {
      Patch.fromBinary(wasmPatchBin);
    }
    if (upstreamPatchBin.length > 0) {
      Patch.fromBinary(upstreamPatchBin);
    }

    if (okA && okB) {
      pass += 1;
    } else {
      console.log(`FAIL ${fixture.name}`);
      console.log(`  upstream->wasm view ok: ${okA}`);
      console.log(`  wasm->upstream view ok: ${okB}`);
      console.log(`  wasm patch bytes: ${bytesToHex(wasmPatchBin).slice(0, 64)}...`);
      console.log(`  upstream patch bytes: ${bytesToHex(upstreamPatchBin).slice(0, 64)}...`);
    }
  } catch (err) {
    console.log(`ERROR ${fixture.name}: ${err instanceof Error ? err.message : String(err)}`);
  }
}

console.log(`interop passes: ${pass}/${total}`);
if (pass !== total) process.exit(1);
