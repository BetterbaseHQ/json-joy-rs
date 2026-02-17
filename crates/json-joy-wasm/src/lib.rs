use json_joy_core::diff_runtime::diff_runtime_to_patch_bytes;
use json_joy_core::model_runtime::RuntimeModel;
use json_joy_core::patch::Patch;
use json_joy_core::{generate_session_id, is_valid_session_id};
use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

const U32_SIZE: usize = 4;
const PATCH_LOG_VERSION: u8 = 1;
const EXPORT_MODEL: u32 = 1;
const EXPORT_VIEW_JSON: u32 = 2;

#[derive(Clone)]
struct EngineState {
    runtime: RuntimeModel,
    sid: u64,
}

#[derive(Default)]
struct EngineStore {
    next_id: u32,
    models: HashMap<u32, EngineState>,
}

thread_local! {
    static ENGINE_STORE: RefCell<EngineStore> = RefCell::new(EngineStore {
        next_id: 1,
        models: HashMap::new(),
    });
}

fn ensure_valid_sid(sid: u64) -> Result<(), String> {
    if is_valid_session_id(sid) {
        Ok(())
    } else {
        Err(format!("invalid session id: {sid}"))
    }
}

fn read_u32_le(input: &[u8], cursor: &mut usize) -> Result<u32, String> {
    let end = cursor
        .checked_add(U32_SIZE)
        .ok_or_else(|| "cursor overflow".to_string())?;
    let bytes = input
        .get(*cursor..end)
        .ok_or_else(|| "input truncated while reading u32".to_string())?;
    *cursor = end;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_u32_be(input: &[u8], cursor: &mut usize) -> Result<u32, String> {
    let end = cursor
        .checked_add(U32_SIZE)
        .ok_or_else(|| "cursor overflow".to_string())?;
    let bytes = input
        .get(*cursor..end)
        .ok_or_else(|| "input truncated while reading u32".to_string())?;
    *cursor = end;
    Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn decode_patch_batch(batch: &[u8]) -> Result<Vec<&[u8]>, String> {
    let mut cursor = 0usize;
    let count = read_u32_le(batch, &mut cursor)? as usize;
    let mut out = Vec::with_capacity(count);

    for _ in 0..count {
        let len = read_u32_le(batch, &mut cursor)? as usize;
        let end = cursor
            .checked_add(len)
            .ok_or_else(|| "cursor overflow".to_string())?;
        let bytes = batch
            .get(cursor..end)
            .ok_or_else(|| "batch truncated while reading patch bytes".to_string())?;
        out.push(bytes);
        cursor = end;
    }

    if cursor != batch.len() {
        return Err("batch has trailing bytes".to_string());
    }

    Ok(out)
}

fn decode_patch_log_v1(log: &[u8]) -> Result<Vec<&[u8]>, String> {
    if log.is_empty() {
        return Ok(Vec::new());
    }
    if log[0] != PATCH_LOG_VERSION {
        return Err(format!(
            "unsupported patch log version: {} (expected {})",
            log[0], PATCH_LOG_VERSION
        ));
    }

    let mut cursor = 1usize;
    let mut out = Vec::new();
    while cursor < log.len() {
        let len = read_u32_be(log, &mut cursor)? as usize;
        let end = cursor
            .checked_add(len)
            .ok_or_else(|| "cursor overflow".to_string())?;
        let bytes = log
            .get(cursor..end)
            .ok_or_else(|| "patch log truncated while reading patch bytes".to_string())?;
        out.push(bytes);
        cursor = end;
    }

    Ok(out)
}

fn encode_patch_batch_from_slices(slices: &[&[u8]]) -> Result<Vec<u8>, String> {
    let mut total = U32_SIZE;
    for s in slices {
        total = total
            .checked_add(U32_SIZE)
            .and_then(|v| v.checked_add(s.len()))
            .ok_or_else(|| "batch size overflow".to_string())?;
    }

    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(&(slices.len() as u32).to_le_bytes());
    for s in slices {
        let len_u32 = u32::try_from(s.len()).map_err(|_| "patch too large".to_string())?;
        out.extend_from_slice(&len_u32.to_le_bytes());
        out.extend_from_slice(s);
    }
    Ok(out)
}

fn append_patch_log_bytes(existing: &[u8], patch_binary: &[u8]) -> Result<Vec<u8>, String> {
    let patch_len = u32::try_from(patch_binary.len()).map_err(|_| "patch too large".to_string())?;
    if existing.is_empty() {
        let mut out = Vec::with_capacity(1 + U32_SIZE + patch_binary.len());
        out.push(PATCH_LOG_VERSION);
        out.extend_from_slice(&patch_len.to_be_bytes());
        out.extend_from_slice(patch_binary);
        return Ok(out);
    }
    if existing[0] != PATCH_LOG_VERSION {
        return Err(format!(
            "unsupported patch log version: {} (expected {})",
            existing[0], PATCH_LOG_VERSION
        ));
    }

    let mut out = Vec::with_capacity(existing.len() + U32_SIZE + patch_binary.len());
    out.extend_from_slice(existing);
    out.extend_from_slice(&patch_len.to_be_bytes());
    out.extend_from_slice(patch_binary);
    Ok(out)
}

fn parse_json_bytes(json_utf8: &[u8]) -> Result<serde_json::Value, String> {
    serde_json::from_slice(json_utf8).map_err(|e| format!("invalid json: {e}"))
}

fn to_json_bytes(value: &serde_json::Value) -> Result<Vec<u8>, String> {
    serde_json::to_vec(value).map_err(|e| format!("json encode failed: {e}"))
}

fn runtime_from_binary_with_sid(model_binary: &[u8], sid: u64) -> Result<RuntimeModel, String> {
    let mut runtime = RuntimeModel::from_model_binary(model_binary)
        .map_err(|e| format!("model decode failed: {e}"))?;
    if model_binary.first().is_some_and(|b| (b & 0x80) == 0) {
        runtime = runtime.fork_with_sid(sid);
    }
    Ok(runtime)
}

fn apply_patch_to_runtime(runtime: &mut RuntimeModel, patch_binary: &[u8]) -> Result<(), String> {
    if patch_binary.is_empty() {
        return Ok(());
    }
    let patch =
        Patch::from_binary(patch_binary).map_err(|e| format!("patch decode failed: {e}"))?;
    runtime
        .apply_patch(&patch)
        .map_err(|e| format!("patch apply failed: {e}"))
}

fn engine_create_with_runtime(runtime: RuntimeModel, sid: u64) -> Result<u32, String> {
    ENGINE_STORE.with(|store| {
        let mut store = store.borrow_mut();
        let mut id = store.next_id;
        while id == 0 || store.models.contains_key(&id) {
            id = id.wrapping_add(1);
            if id == store.next_id {
                return Err("engine id space exhausted".to_string());
            }
        }
        store.next_id = id.wrapping_add(1);
        store.models.insert(id, EngineState { runtime, sid });
        Ok(id)
    })
}

fn with_engine_mut<T>(
    engine_id: u32,
    f: impl FnOnce(&mut EngineState) -> Result<T, String>,
) -> Result<T, String> {
    ENGINE_STORE.with(|store| {
        let mut store = store.borrow_mut();
        let engine = store
            .models
            .get_mut(&engine_id)
            .ok_or_else(|| format!("engine not found: {engine_id}"))?;
        f(engine)
    })
}

fn with_engine<T>(
    engine_id: u32,
    f: impl FnOnce(&EngineState) -> Result<T, String>,
) -> Result<T, String> {
    ENGINE_STORE.with(|store| {
        let store = store.borrow();
        let engine = store
            .models
            .get(&engine_id)
            .ok_or_else(|| format!("engine not found: {engine_id}"))?;
        f(engine)
    })
}

fn engine_create_empty_internal(sid: u64) -> Result<u32, String> {
    ensure_valid_sid(sid)?;
    engine_create_with_runtime(RuntimeModel::new_logical_empty(sid), sid)
}

fn engine_create_from_model_internal(model_binary: &[u8], sid: u64) -> Result<u32, String> {
    ensure_valid_sid(sid)?;
    let runtime = runtime_from_binary_with_sid(model_binary, sid)?;
    engine_create_with_runtime(runtime, sid)
}

fn engine_fork_internal(engine_id: u32, sid: u64) -> Result<u32, String> {
    ensure_valid_sid(sid)?;
    let runtime = with_engine(engine_id, |engine| Ok(engine.runtime.fork_with_sid(sid)))?;
    engine_create_with_runtime(runtime, sid)
}

fn engine_set_sid_internal(engine_id: u32, sid: u64) -> Result<(), String> {
    ensure_valid_sid(sid)?;
    with_engine_mut(engine_id, |engine| {
        engine.sid = sid;
        Ok(())
    })
}

fn engine_export_model_internal(engine_id: u32) -> Result<Vec<u8>, String> {
    with_engine(engine_id, |engine| {
        engine
            .runtime
            .to_model_binary_like()
            .map_err(|e| format!("model encode failed: {e}"))
    })
}

fn engine_export_view_json_internal(engine_id: u32) -> Result<Vec<u8>, String> {
    with_engine(engine_id, |engine| {
        to_json_bytes(&engine.runtime.view_json())
    })
}

fn engine_apply_patch_internal(engine_id: u32, patch_binary: &[u8]) -> Result<(), String> {
    with_engine_mut(engine_id, |engine| {
        apply_patch_to_runtime(&mut engine.runtime, patch_binary)
    })
}

fn engine_apply_patch_batch_internal(
    engine_id: u32,
    patch_batch_binary: &[u8],
) -> Result<u32, String> {
    with_engine_mut(engine_id, |engine| {
        let slices = decode_patch_batch(patch_batch_binary)?;
        for s in &slices {
            apply_patch_to_runtime(&mut engine.runtime, s)?;
        }
        Ok(slices.len() as u32)
    })
}

fn engine_apply_patch_log_internal(engine_id: u32, patch_log_binary: &[u8]) -> Result<u32, String> {
    with_engine_mut(engine_id, |engine| {
        let slices = decode_patch_log_v1(patch_log_binary)?;
        for s in &slices {
            apply_patch_to_runtime(&mut engine.runtime, s)?;
        }
        Ok(slices.len() as u32)
    })
}

fn engine_diff_json_internal(engine_id: u32, next_json_utf8: &[u8]) -> Result<Vec<u8>, String> {
    with_engine_mut(engine_id, |engine| {
        let next = parse_json_bytes(next_json_utf8)?;
        let patch = diff_runtime_to_patch_bytes(&engine.runtime, &next, engine.sid)
            .map_err(|e| format!("diff failed: {e}"))?;
        Ok(patch.unwrap_or_default())
    })
}

fn engine_diff_apply_json_internal(
    engine_id: u32,
    next_json_utf8: &[u8],
) -> Result<Vec<u8>, String> {
    with_engine_mut(engine_id, |engine| {
        let next = parse_json_bytes(next_json_utf8)?;
        let patch = diff_runtime_to_patch_bytes(&engine.runtime, &next, engine.sid)
            .map_err(|e| format!("diff failed: {e}"))?;
        if let Some(patch_binary) = patch {
            apply_patch_to_runtime(&mut engine.runtime, &patch_binary)?;
            Ok(patch_binary)
        } else {
            Ok(Vec::new())
        }
    })
}

fn encode_diff_apply_export_result(
    patch: &[u8],
    model: Option<&[u8]>,
    view_json: Option<&[u8]>,
) -> Result<Vec<u8>, String> {
    let model_len = model.map_or(0usize, |m| m.len());
    let view_len = view_json.map_or(0usize, |v| v.len());

    let total = U32_SIZE
        .checked_add(patch.len())
        .and_then(|v| v.checked_add(U32_SIZE))
        .and_then(|v| v.checked_add(model_len))
        .and_then(|v| v.checked_add(U32_SIZE))
        .and_then(|v| v.checked_add(view_len))
        .ok_or_else(|| "result envelope overflow".to_string())?;

    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(&(patch.len() as u32).to_le_bytes());
    out.extend_from_slice(patch);
    out.extend_from_slice(&(model_len as u32).to_le_bytes());
    if let Some(m) = model {
        out.extend_from_slice(m);
    }
    out.extend_from_slice(&(view_len as u32).to_le_bytes());
    if let Some(v) = view_json {
        out.extend_from_slice(v);
    }
    Ok(out)
}

fn engine_diff_apply_export_json_internal(
    engine_id: u32,
    next_json_utf8: &[u8],
    flags: u32,
) -> Result<Vec<u8>, String> {
    with_engine_mut(engine_id, |engine| {
        let next = parse_json_bytes(next_json_utf8)?;
        let patch = diff_runtime_to_patch_bytes(&engine.runtime, &next, engine.sid)
            .map_err(|e| format!("diff failed: {e}"))?;

        let patch_binary = if let Some(patch_binary) = patch {
            apply_patch_to_runtime(&mut engine.runtime, &patch_binary)?;
            patch_binary
        } else {
            Vec::new()
        };

        let model = if (flags & EXPORT_MODEL) != 0 {
            Some(
                engine
                    .runtime
                    .to_model_binary_like()
                    .map_err(|e| format!("model encode failed: {e}"))?,
            )
        } else {
            None
        };
        let view_json = if (flags & EXPORT_VIEW_JSON) != 0 {
            Some(to_json_bytes(&engine.runtime.view_json())?)
        } else {
            None
        };

        encode_diff_apply_export_result(&patch_binary, model.as_deref(), view_json.as_deref())
    })
}

#[wasm_bindgen]
pub fn session_generate() -> u64 {
    generate_session_id()
}

#[wasm_bindgen]
pub fn session_is_valid(sid: u64) -> bool {
    is_valid_session_id(sid)
}

#[wasm_bindgen]
pub fn engine_create_empty(sid: u64) -> Result<u32, JsValue> {
    engine_create_empty_internal(sid).map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn engine_create_from_model(model_binary: &[u8], sid: u64) -> Result<u32, JsValue> {
    engine_create_from_model_internal(model_binary, sid).map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn engine_fork(engine_id: u32, sid: u64) -> Result<u32, JsValue> {
    engine_fork_internal(engine_id, sid).map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn engine_set_sid(engine_id: u32, sid: u64) -> Result<(), JsValue> {
    engine_set_sid_internal(engine_id, sid).map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn engine_free(engine_id: u32) -> bool {
    ENGINE_STORE.with(|store| {
        let mut store = store.borrow_mut();
        store.models.remove(&engine_id).is_some()
    })
}

#[wasm_bindgen]
pub fn engine_export_model(engine_id: u32) -> Result<Vec<u8>, JsValue> {
    engine_export_model_internal(engine_id).map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn engine_export_view_json(engine_id: u32) -> Result<Vec<u8>, JsValue> {
    engine_export_view_json_internal(engine_id).map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn engine_apply_patch(engine_id: u32, patch_binary: &[u8]) -> Result<(), JsValue> {
    engine_apply_patch_internal(engine_id, patch_binary).map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn engine_apply_patch_batch(engine_id: u32, patch_batch_binary: &[u8]) -> Result<u32, JsValue> {
    engine_apply_patch_batch_internal(engine_id, patch_batch_binary)
        .map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn engine_apply_patch_log(engine_id: u32, patch_log_binary: &[u8]) -> Result<u32, JsValue> {
    engine_apply_patch_log_internal(engine_id, patch_log_binary).map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn engine_diff_json(engine_id: u32, next_json_utf8: &[u8]) -> Result<Vec<u8>, JsValue> {
    engine_diff_json_internal(engine_id, next_json_utf8).map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn engine_diff_apply_json(engine_id: u32, next_json_utf8: &[u8]) -> Result<Vec<u8>, JsValue> {
    engine_diff_apply_json_internal(engine_id, next_json_utf8).map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn engine_diff_apply_export_json(
    engine_id: u32,
    next_json_utf8: &[u8],
    flags: u32,
) -> Result<Vec<u8>, JsValue> {
    engine_diff_apply_export_json_internal(engine_id, next_json_utf8, flags)
        .map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn patch_log_append(
    existing_patch_log: &[u8],
    patch_binary: &[u8],
) -> Result<Vec<u8>, JsValue> {
    append_patch_log_bytes(existing_patch_log, patch_binary).map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn patch_log_to_batch(patch_log_binary: &[u8]) -> Result<Vec<u8>, JsValue> {
    let slices = decode_patch_log_v1(patch_log_binary).map_err(|e| JsValue::from_str(&e))?;
    encode_patch_batch_from_slices(&slices).map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn patch_batch_apply_to_model(
    base_model_binary: &[u8],
    patch_batch_binary: &[u8],
    sid_for_empty_model: u64,
) -> Result<Vec<u8>, JsValue> {
    ensure_valid_sid(sid_for_empty_model).map_err(|e| JsValue::from_str(&e))?;
    let mut runtime = if base_model_binary.is_empty() {
        RuntimeModel::new_logical_empty(sid_for_empty_model)
    } else {
        runtime_from_binary_with_sid(base_model_binary, sid_for_empty_model)
            .map_err(|e| JsValue::from_str(&e))?
    };

    let slices = decode_patch_batch(patch_batch_binary).map_err(|e| JsValue::from_str(&e))?;
    for s in slices {
        apply_patch_to_runtime(&mut runtime, s).map_err(|e| JsValue::from_str(&e))?;
    }

    runtime
        .to_model_binary_like()
        .map_err(|e| JsValue::from_str(&format!("model encode failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::{
        decode_patch_log_v1, engine_apply_patch_batch_internal, engine_apply_patch_log_internal,
        engine_create_empty_internal, engine_create_from_model_internal,
        engine_diff_apply_export_json_internal, engine_diff_apply_json_internal,
        engine_export_model_internal, engine_export_view_json_internal, engine_free,
        patch_batch_apply_to_model, patch_log_append, patch_log_to_batch, EXPORT_MODEL,
        EXPORT_VIEW_JSON,
    };
    use json_joy_core::model_runtime::RuntimeModel;
    use serde_json::Value;
    use std::fs;
    use std::path::{Path, PathBuf};

    const SID: u64 = 65_536;

    fn fixtures_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("tests")
            .join("compat")
            .join("fixtures")
    }

    fn decode_hex(s: &str) -> Vec<u8> {
        assert!(
            s.len().is_multiple_of(2),
            "hex string must have even length"
        );
        let mut out = Vec::with_capacity(s.len() / 2);
        let bytes = s.as_bytes();
        for i in (0..bytes.len()).step_by(2) {
            let hi = (bytes[i] as char).to_digit(16).expect("invalid hex") as u8;
            let lo = (bytes[i + 1] as char).to_digit(16).expect("invalid hex") as u8;
            out.push((hi << 4) | lo);
        }
        out
    }

    fn encode_batch(chunks: &[Vec<u8>]) -> Vec<u8> {
        let mut out = Vec::with_capacity(4 + chunks.iter().map(|c| 4 + c.len()).sum::<usize>());
        out.extend_from_slice(&(chunks.len() as u32).to_le_bytes());
        for c in chunks {
            out.extend_from_slice(&(c.len() as u32).to_le_bytes());
            out.extend_from_slice(c);
        }
        out
    }

    #[test]
    fn patch_log_roundtrip_decode_and_batch_conversion() {
        let a = vec![1, 2, 3];
        let b = vec![9, 8];
        let log1 = patch_log_append(&[], &a).expect("append 1");
        let log2 = patch_log_append(&log1, &b).expect("append 2");
        let slices = decode_patch_log_v1(&log2).expect("decode log");
        assert_eq!(slices.len(), 2);
        assert_eq!(slices[0], a.as_slice());
        assert_eq!(slices[1], b.as_slice());

        let batch = patch_log_to_batch(&log2).expect("to batch");
        let expected_batch = encode_batch(&[a, b]);
        assert_eq!(batch, expected_batch);
    }

    #[test]
    fn engine_diff_apply_and_export_view() {
        let eid = engine_create_empty_internal(SID).expect("create");
        let patch =
            engine_diff_apply_json_internal(eid, br#"{"k":1,"s":"ab"}"#).expect("diff+apply");
        assert!(!patch.is_empty());
        let view = engine_export_view_json_internal(eid).expect("view");
        let parsed: Value = serde_json::from_slice(&view).expect("json");
        assert_eq!(parsed, serde_json::json!({"k":1,"s":"ab"}));
        assert!(engine_free(eid));
    }

    #[test]
    fn diff_apply_export_envelope_contains_requested_sections() {
        let eid = engine_create_empty_internal(SID).expect("create");
        let out = engine_diff_apply_export_json_internal(
            eid,
            br#"{"a":1}"#,
            EXPORT_MODEL | EXPORT_VIEW_JSON,
        )
        .expect("diff apply export");

        let mut cursor = 0usize;
        let patch_len = u32::from_le_bytes([out[0], out[1], out[2], out[3]]) as usize;
        cursor += 4;
        assert!(patch_len > 0, "patch should be present");
        cursor += patch_len;

        let model_len = u32::from_le_bytes([
            out[cursor],
            out[cursor + 1],
            out[cursor + 2],
            out[cursor + 3],
        ]) as usize;
        cursor += 4;
        assert!(model_len > 0, "model should be present");
        cursor += model_len;

        let view_len = u32::from_le_bytes([
            out[cursor],
            out[cursor + 1],
            out[cursor + 2],
            out[cursor + 3],
        ]) as usize;
        cursor += 4;
        assert!(view_len > 0, "view should be present");
        let view_json = &out[cursor..cursor + view_len];
        let parsed: Value = serde_json::from_slice(view_json).expect("view json");
        assert_eq!(parsed, serde_json::json!({"a": 1}));
        assert!(engine_free(eid));
    }

    #[test]
    fn engine_apply_patch_log_matches_batch_apply() {
        let fixture = fixtures_dir().join("model_apply_replay_116_vec_in_order_v1.json");
        let raw = fs::read_to_string(&fixture).expect("fixture read");
        let parsed: Value = serde_json::from_str(&raw).expect("fixture parse");

        let base = decode_hex(
            parsed["input"]["base_model_binary_hex"]
                .as_str()
                .expect("base hex"),
        );
        let patch_hexes = parsed["input"]["patches_binary_hex"]
            .as_array()
            .expect("patches");
        let replay = parsed["input"]["replay_pattern"]
            .as_array()
            .expect("replay");
        let patches: Vec<Vec<u8>> = patch_hexes
            .iter()
            .map(|v| decode_hex(v.as_str().expect("patch hex")))
            .collect();
        let replay_chunks: Vec<Vec<u8>> = replay
            .iter()
            .map(|idx| {
                let i = idx.as_u64().expect("idx") as usize;
                patches[i].clone()
            })
            .collect();

        let mut log = Vec::new();
        for p in &replay_chunks {
            log = patch_log_append(&log, p).expect("append log");
        }

        let eid = engine_create_from_model_internal(&base, SID).expect("engine create");
        let n = engine_apply_patch_log_internal(eid, &log).expect("apply log");
        assert_eq!(n as usize, replay_chunks.len());
        let model_from_log = engine_export_model_internal(eid).expect("export model");
        assert!(engine_free(eid));

        let batch = encode_batch(&replay_chunks);
        let model_from_batch = patch_batch_apply_to_model(&base, &batch, SID).expect("batch apply");

        let view_log = RuntimeModel::from_model_binary(&model_from_log)
            .expect("decode log model")
            .view_json();
        let view_batch = RuntimeModel::from_model_binary(&model_from_batch)
            .expect("decode batch model")
            .view_json();
        assert_eq!(view_log, view_batch);
    }

    #[test]
    fn engine_apply_patch_batch_matches_stateless_batch_apply() {
        let fixture = fixtures_dir().join("model_apply_replay_01_obj_dup_single_v1.json");
        let raw = fs::read_to_string(&fixture).expect("fixture read");
        let parsed: Value = serde_json::from_str(&raw).expect("fixture parse");

        let base = decode_hex(
            parsed["input"]["base_model_binary_hex"]
                .as_str()
                .expect("base hex"),
        );
        let patch_hexes = parsed["input"]["patches_binary_hex"]
            .as_array()
            .expect("patches");
        let replay = parsed["input"]["replay_pattern"]
            .as_array()
            .expect("replay");
        let patches: Vec<Vec<u8>> = patch_hexes
            .iter()
            .map(|v| decode_hex(v.as_str().expect("patch hex")))
            .collect();
        let replay_chunks: Vec<Vec<u8>> = replay
            .iter()
            .map(|idx| {
                let i = idx.as_u64().expect("idx") as usize;
                patches[i].clone()
            })
            .collect();

        let batch = encode_batch(&replay_chunks);
        let eid = engine_create_from_model_internal(&base, SID).expect("engine create");
        let n = engine_apply_patch_batch_internal(eid, &batch).expect("engine apply batch");
        assert_eq!(n as usize, replay_chunks.len());
        let model_engine = engine_export_model_internal(eid).expect("engine export");
        assert!(engine_free(eid));

        let model_stateless = patch_batch_apply_to_model(&base, &batch, SID).expect("stateless");

        let engine_view = RuntimeModel::from_model_binary(&model_engine)
            .expect("decode engine")
            .view_json();
        let stateless_view = RuntimeModel::from_model_binary(&model_stateless)
            .expect("decode stateless")
            .view_json();
        assert_eq!(engine_view, stateless_view);
    }
}
