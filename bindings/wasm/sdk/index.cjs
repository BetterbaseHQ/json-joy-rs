const wasm = require('../../../crates/json-joy-wasm/pkg/json_joy_wasm.js');

const PATCH_LOG_VERSION = 1;
const EXPORT_MODEL = 1;
const EXPORT_VIEW_JSON = 2;

function asSidBigInt(sid) {
  return typeof sid === 'bigint' ? sid : BigInt(sid);
}

function encodePatchBatch(patches) {
  let total = 4;
  for (const patch of patches) total += 4 + patch.length;
  const out = new Uint8Array(total);
  const view = new DataView(out.buffer, out.byteOffset, out.byteLength);
  let cursor = 0;
  view.setUint32(cursor, patches.length, true);
  cursor += 4;
  for (const patch of patches) {
    view.setUint32(cursor, patch.length, true);
    cursor += 4;
    out.set(patch, cursor);
    cursor += patch.length;
  }
  return out;
}

function appendPatchLog(existing, patchBinary) {
  return wasm.patch_log_append(existing, patchBinary);
}

function encodePatchLog(patches) {
  if (!patches || patches.length === 0) return new Uint8Array(0);
  let total = 1;
  for (const patch of patches) total += 4 + patch.length;
  const out = new Uint8Array(total);
  const view = new DataView(out.buffer, out.byteOffset, out.byteLength);
  out[0] = PATCH_LOG_VERSION;
  let cursor = 1;
  for (const patch of patches) {
    view.setUint32(cursor, patch.length, false);
    cursor += 4;
    out.set(patch, cursor);
    cursor += patch.length;
  }
  return out;
}

function patchLogToBatch(patchLog) {
  return wasm.patch_log_to_batch(patchLog);
}

function emptyPatchLog() {
  return new Uint8Array(0);
}

function decodePatchLog(patchLog) {
  if (patchLog.length === 0) return [];
  if (patchLog[0] !== PATCH_LOG_VERSION) {
    throw new Error(`Unsupported patch log version: ${patchLog[0]}`);
  }
  const view = new DataView(patchLog.buffer, patchLog.byteOffset, patchLog.byteLength);
  const out = [];
  let cursor = 1;
  while (cursor < patchLog.length) {
    if (cursor + 4 > patchLog.length) throw new Error('Corrupt patch log header');
    const len = view.getUint32(cursor, false);
    cursor += 4;
    const end = cursor + len;
    if (end > patchLog.length) throw new Error('Corrupt patch log payload');
    out.push(patchLog.subarray(cursor, end));
    cursor = end;
  }
  return out;
}

function decodeDiffApplyExportEnvelope(bytes) {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  let cursor = 0;
  if (cursor + 4 > bytes.length) throw new Error('Invalid envelope (patch length)');
  const patchLen = view.getUint32(cursor, true);
  cursor += 4;
  if (cursor + patchLen > bytes.length) throw new Error('Invalid envelope (patch payload)');
  const patchBinary = bytes.subarray(cursor, cursor + patchLen);
  cursor += patchLen;

  if (cursor + 4 > bytes.length) throw new Error('Invalid envelope (model length)');
  const modelLen = view.getUint32(cursor, true);
  cursor += 4;
  if (cursor + modelLen > bytes.length) throw new Error('Invalid envelope (model payload)');
  const modelBinary = modelLen > 0 ? bytes.subarray(cursor, cursor + modelLen) : null;
  cursor += modelLen;

  if (cursor + 4 > bytes.length) throw new Error('Invalid envelope (view length)');
  const viewLen = view.getUint32(cursor, true);
  cursor += 4;
  if (cursor + viewLen > bytes.length) throw new Error('Invalid envelope (view payload)');
  const viewJson =
    viewLen > 0
      ? JSON.parse(Buffer.from(bytes.subarray(cursor, cursor + viewLen)).toString('utf8'))
      : null;
  cursor += viewLen;
  if (cursor !== bytes.length) throw new Error('Invalid envelope (trailing bytes)');

  return {patchBinary, modelBinary, viewJson};
}

class CrdtEngine {
  constructor(engineId) {
    this.engineId = engineId;
  }

  static createEmpty(sessionId) {
    const id = wasm.engine_create_empty(asSidBigInt(sessionId));
    return new CrdtEngine(id);
  }

  static fromModel(modelBinary, sessionId) {
    const id = wasm.engine_create_from_model(modelBinary, asSidBigInt(sessionId));
    return new CrdtEngine(id);
  }

  fork(sessionId) {
    const id = wasm.engine_fork(this.engineId, asSidBigInt(sessionId));
    return new CrdtEngine(id);
  }

  setSession(sessionId) {
    wasm.engine_set_sid(this.engineId, asSidBigInt(sessionId));
  }

  diffJson(nextJson) {
    return wasm.engine_diff_json(this.engineId, Buffer.from(JSON.stringify(nextJson)));
  }

  diffApplyJson(nextJson) {
    return wasm.engine_diff_apply_json(this.engineId, Buffer.from(JSON.stringify(nextJson)));
  }

  diffApplyExportJson(nextJson, options = {}) {
    const includeModel = options.includeModel !== false;
    const includeView = options.includeView === true;
    let flags = 0;
    if (includeModel) flags |= EXPORT_MODEL;
    if (includeView) flags |= EXPORT_VIEW_JSON;
    const envelope = wasm.engine_diff_apply_export_json(
      this.engineId,
      Buffer.from(JSON.stringify(nextJson)),
      flags,
    );
    return decodeDiffApplyExportEnvelope(envelope);
  }

  applyPatch(patchBinary) {
    wasm.engine_apply_patch(this.engineId, patchBinary);
  }

  applyPatchBatch(patches) {
    const batch = Array.isArray(patches) ? encodePatchBatch(patches) : patches;
    return wasm.engine_apply_patch_batch(this.engineId, batch);
  }

  applyPatchLog(patchLog) {
    return wasm.engine_apply_patch_log(this.engineId, patchLog);
  }

  exportModel() {
    return wasm.engine_export_model(this.engineId);
  }

  viewJson() {
    return JSON.parse(Buffer.from(wasm.engine_export_view_json(this.engineId)).toString('utf8'));
  }

  dispose() {
    if (this.engineId !== null) {
      wasm.engine_free(this.engineId);
      this.engineId = null;
    }
  }
}

module.exports = {
  CrdtEngine,
  encodePatchBatch,
  appendPatchLog,
  encodePatchLog,
  patchLogToBatch,
  emptyPatchLog,
  decodePatchLog,
  decodeDiffApplyExportEnvelope,
};
