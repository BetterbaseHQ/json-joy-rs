'use strict';
/**
 * Two-way interop correctness check.
 *
 * For each model_diff_parity fixture:
 *
 *   Direction A — upstream TS generates patch, Rust/WASM applies it:
 *     TS:   Model.fromBinary(base) → api.diff(nextJson) → toBinary()
 *     WASM: Model.fromBinary(base) → applyPatch(tsPatch) → viewJson()
 *     assert view == expected
 *
 *   Direction B — Rust/WASM generates patch, upstream TS applies it:
 *     WASM: Model.fromBinary(base) → diffApply(nextJson) → patchBytes
 *     TS:   Model.fromBinary(base) → applyPatch(Patch.fromBinary(wasmPatch)) → view()
 *     assert view == expected
 *
 * Run:  node bench/interop.cjs
 */

const fs = require('node:fs');
const path = require('node:path');

const { Model: TSModel } =
  require('./node_modules/json-joy/lib/json-crdt/index.js');
const { Patch: TSPatch } =
  require('./node_modules/json-joy/lib/json-crdt-patch/index.js');
const { Model: WASMModel } =
  require('../crates/json-joy-wasm/pkg/json_joy_wasm.js');

const repoRoot = path.resolve(__dirname, '..');
const fixturesDir = path.join(repoRoot, 'tests', 'compat', 'fixtures');
const manifest = JSON.parse(fs.readFileSync(path.join(fixturesDir, 'manifest.json'), 'utf8'));

const fixtures = manifest.fixtures.filter((f) => f.scenario === 'model_diff_parity');
const limit = Number.parseInt(process.env.INTEROP_LIMIT || String(fixtures.length), 10);
const selected = fixtures.slice(0, Math.min(limit, fixtures.length));

let total = 0, passA = 0, passB = 0, errors = 0;

for (const entry of selected) {
  total++;
  const fix = JSON.parse(fs.readFileSync(path.join(fixturesDir, entry.file), 'utf8'));
  const baseBytes   = Buffer.from(fix.input.base_model_binary_hex, 'hex');
  const nextJson    = fix.input.next_view_json;
  const nextJsonStr = JSON.stringify(nextJson);
  const expected    = JSON.stringify(fix.expected.view_after_apply_json);

  try {
    // ── Direction A: TS diff → WASM apply ──────────────────────────────────
    const tsModelA = TSModel.fromBinary(baseBytes);
    const tsPatch  = tsModelA.api.diff(nextJson);
    const tsPatchBytes = tsPatch ? tsPatch.toBinary() : new Uint8Array(0);

    const wasmModelA = WASMModel.fromBinary(baseBytes);
    if (tsPatchBytes.length > 0) wasmModelA.applyPatch(tsPatchBytes);
    const viewA = JSON.stringify(wasmModelA.view());

    const okA = viewA === expected;
    if (okA) passA++;

    // ── Direction B: WASM diff → TS apply ──────────────────────────────────
    const wasmModelB  = WASMModel.fromBinary(baseBytes);
    const wasmPatchBytes = wasmModelB.diffApply(nextJsonStr);

    const tsModelB = TSModel.fromBinary(baseBytes);
    if (wasmPatchBytes.length > 0) {
      tsModelB.applyPatch(TSPatch.fromBinary(wasmPatchBytes));
    }
    const viewB = JSON.stringify(tsModelB.view());

    const okB = viewB === expected;
    if (okB) passB++;

    if (!okA || !okB) {
      console.log(`FAIL  ${fix.name}`);
      if (!okA) console.log(`  A (ts→wasm):  got ${viewA.slice(0, 120)}`);
      if (!okB) console.log(`  B (wasm→ts):  got ${viewB.slice(0, 120)}`);
      console.log(`  expected:     ${expected.slice(0, 120)}`);
    }
  } catch (err) {
    errors++;
    console.log(`ERROR ${fix.name}: ${err instanceof Error ? err.message : String(err)}`);
  }
}

const bothPass = selected.filter((_, i) => i < total).length;  // placeholder
console.log(`\n  ts→wasm  (A): ${passA}/${total} pass`);
console.log(`  wasm→ts  (B): ${passB}/${total} pass`);
if (errors) console.log(`  errors:       ${errors}`);
console.log(`  total:        ${total} fixtures\n`);

if (passA !== total || passB !== total) process.exit(1);
