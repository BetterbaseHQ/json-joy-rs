use json_joy::json_crdt::codec::structural::binary as structural_binary;
use json_joy::json_crdt_patch::patch::Patch;
use serde_json::{json, Map, Value};

use crate::common::assertions::{decode_hex, encode_hex, op_to_opcode};

use super::helpers::{
    append_patch_log, decode_patch_log_count, model_api_diff_patch, model_from_json, set_model_sid,
};

pub(super) fn eval_lessdb(
    scenario: &str,
    input: &Map<String, Value>,
    _fixture: &Value,
) -> Result<Value, String> {
    match scenario {
        "lessdb_model_manager" => {
            let workflow = input
                .get("workflow")
                .and_then(Value::as_str)
                .ok_or_else(|| "input.workflow missing".to_string())?;
            match workflow {
                "create_diff_apply" => {
                    let sid = input
                        .get("sid")
                        .and_then(Value::as_u64)
                        .ok_or_else(|| "input.sid missing".to_string())?;
                    let initial = input
                        .get("initial_json")
                        .ok_or_else(|| "input.initial_json missing".to_string())?;
                    let ops = input
                        .get("ops")
                        .and_then(Value::as_array)
                        .ok_or_else(|| "input.ops missing".to_string())?;
                    let mut model = model_from_json(initial, sid);
                    let mut pending = Vec::<u8>::new();
                    let mut last_patch: Option<Patch> = None;
                    let mut steps = Vec::<Value>::with_capacity(ops.len());

                    for op in ops {
                        let kind = op
                            .get("kind")
                            .and_then(Value::as_str)
                            .ok_or_else(|| "lessdb op.kind missing".to_string())?;
                        match kind {
                            "diff" => {
                                let next = op
                                    .get("next_view_json")
                                    .ok_or_else(|| "diff.next_view_json missing".to_string())?;
                                let patch = model_api_diff_patch(&model, sid, next);
                                if let Some(p) = patch {
                                    let id = p.get_id();
                                    steps.push(json!({
                                        "kind": "diff",
                                        "patch_present": true,
                                        "patch_binary_hex": encode_hex(&p.to_binary()),
                                        "patch_op_count": p.ops.len(),
                                        "patch_opcodes": p.ops.iter().map(|op| Value::from(op_to_opcode(op) as u64)).collect::<Vec<_>>(),
                                        "patch_span": p.span(),
                                        "patch_id_sid": id.map(|x| x.sid),
                                        "patch_id_time": id.map(|x| x.time),
                                        "patch_next_time": p.next_time(),
                                    }));
                                    last_patch = Some(p);
                                } else {
                                    steps.push(json!({
                                        "kind": "diff",
                                        "patch_present": false,
                                        "patch_binary_hex": Value::Null,
                                    }));
                                    last_patch = None;
                                }
                            }
                            "apply_last_diff" => {
                                if let Some(p) = &last_patch {
                                    model.apply_patch(p);
                                }
                                steps.push(json!({
                                    "kind": "apply_last_diff",
                                    "view_json": model.view(),
                                    "model_binary_hex": encode_hex(&structural_binary::encode(&model)),
                                }));
                            }
                            "patch_log_append_last_diff" => {
                                if let Some(p) = &last_patch {
                                    pending = append_patch_log(&pending, &p.to_binary());
                                }
                                steps.push(json!({
                                    "kind": "patch_log_append_last_diff",
                                    "pending_patch_log_hex": encode_hex(&pending),
                                }));
                            }
                            "patch_log_deserialize" => {
                                let count = decode_patch_log_count(&pending)?;
                                steps.push(json!({
                                    "kind": "patch_log_deserialize",
                                    "patch_count": count,
                                }));
                            }
                            other => return Err(format!("unsupported lessdb op kind: {other}")),
                        }
                    }

                    Ok(json!({
                        "steps": steps,
                        "final_view_json": model.view(),
                        "final_model_binary_hex": encode_hex(&structural_binary::encode(&model)),
                        "final_pending_patch_log_hex": encode_hex(&pending),
                    }))
                }
                "fork_merge" => {
                    let sid = input
                        .get("sid")
                        .and_then(Value::as_u64)
                        .ok_or_else(|| "input.sid missing".to_string())?;
                    let initial = input
                        .get("initial_json")
                        .ok_or_else(|| "input.initial_json missing".to_string())?;
                    let ops = input
                        .get("ops")
                        .and_then(Value::as_array)
                        .ok_or_else(|| "input.ops missing".to_string())?;
                    let base = model_from_json(initial, sid);
                    let base_binary = structural_binary::encode(&base);
                    let mut fork: Option<json_joy::json_crdt::model::Model> = None;
                    let mut last_patch: Option<Patch> = None;
                    let mut merged =
                        structural_binary::decode(&base_binary).map_err(|e| format!("{e:?}"))?;
                    let mut steps = Vec::<Value>::with_capacity(ops.len());

                    for op in ops {
                        let kind = op
                            .get("kind")
                            .and_then(Value::as_str)
                            .ok_or_else(|| "lessdb op.kind missing".to_string())?;
                        match kind {
                            "fork" => {
                                let fork_sid = op
                                    .get("sid")
                                    .and_then(Value::as_u64)
                                    .ok_or_else(|| "fork.sid missing".to_string())?;
                                let mut f = structural_binary::decode(&base_binary)
                                    .map_err(|e| format!("{e:?}"))?;
                                set_model_sid(&mut f, fork_sid);
                                steps.push(json!({
                                    "kind": "fork",
                                    "view_json": f.view(),
                                }));
                                fork = Some(f);
                            }
                            "diff_on_fork" => {
                                let next = op.get("next_view_json").ok_or_else(|| {
                                    "diff_on_fork.next_view_json missing".to_string()
                                })?;
                                let f = fork
                                    .as_ref()
                                    .ok_or_else(|| "diff_on_fork called before fork".to_string())?;
                                let patch = model_api_diff_patch(f, f.clock.sid, next);
                                if let Some(p) = patch {
                                    steps.push(json!({
                                        "kind": "diff_on_fork",
                                        "patch_present": true,
                                        "patch_binary_hex": encode_hex(&p.to_binary()),
                                    }));
                                    last_patch = Some(p);
                                } else {
                                    steps.push(json!({
                                        "kind": "diff_on_fork",
                                        "patch_present": false,
                                        "patch_binary_hex": Value::Null,
                                    }));
                                    last_patch = None;
                                }
                            }
                            "apply_last_diff_on_fork" => {
                                let f = fork.as_mut().ok_or_else(|| {
                                    "apply_last_diff_on_fork called before fork".to_string()
                                })?;
                                if let Some(p) = &last_patch {
                                    f.apply_patch(p);
                                }
                                steps.push(json!({
                                    "kind": "apply_last_diff_on_fork",
                                    "view_json": f.view(),
                                    "model_binary_hex": encode_hex(&structural_binary::encode(f)),
                                }));
                            }
                            "merge_into_base" => {
                                merged = structural_binary::decode(&base_binary)
                                    .map_err(|e| format!("{e:?}"))?;
                                if let Some(p) = &last_patch {
                                    merged.apply_patch(p);
                                }
                                steps.push(json!({
                                    "kind": "merge_into_base",
                                    "view_json": merged.view(),
                                    "model_binary_hex": encode_hex(&structural_binary::encode(&merged)),
                                }));
                            }
                            other => return Err(format!("unsupported lessdb op kind: {other}")),
                        }
                    }

                    Ok(json!({
                        "steps": steps,
                        "final_view_json": merged.view(),
                        "final_model_binary_hex": encode_hex(&structural_binary::encode(&merged)),
                    }))
                }
                "merge_idempotent" => {
                    let base_binary = decode_hex(
                        input
                            .get("base_model_binary_hex")
                            .and_then(Value::as_str)
                            .ok_or_else(|| "input.base_model_binary_hex missing".to_string())?,
                    )?;
                    let ops = input
                        .get("ops")
                        .and_then(Value::as_array)
                        .ok_or_else(|| "input.ops missing".to_string())?;
                    let mut model =
                        structural_binary::decode(&base_binary).map_err(|e| format!("{e:?}"))?;
                    let mut first_patch_hex = String::new();
                    let mut steps = Vec::<Value>::new();
                    for op in ops {
                        let kind = op
                            .get("kind")
                            .and_then(Value::as_str)
                            .ok_or_else(|| "lessdb op.kind missing".to_string())?;
                        if kind != "merge" {
                            return Err(format!("unsupported merge_idempotent op kind: {kind}"));
                        }
                        let patches = op
                            .get("patches_binary_hex")
                            .and_then(Value::as_array)
                            .ok_or_else(|| "merge.patches_binary_hex missing".to_string())?;
                        for (i, phex) in patches.iter().enumerate() {
                            let phex = phex
                                .as_str()
                                .ok_or_else(|| "merge patch hex must be string".to_string())?;
                            if i == 0 && first_patch_hex.is_empty() {
                                first_patch_hex = phex.to_string();
                            }
                            let pb = decode_hex(phex)?;
                            let patch = Patch::from_binary(&pb).map_err(|e| e.to_string())?;
                            model.apply_patch(&patch);
                        }
                        steps.push(json!({
                            "kind": "merge",
                            "view_json": model.view(),
                            "model_binary_hex": encode_hex(&structural_binary::encode(&model)),
                        }));
                    }
                    Ok(json!({
                        "steps": steps,
                        "patch_binary_hex": first_patch_hex,
                        "final_view_json": model.view(),
                        "final_model_binary_hex": encode_hex(&structural_binary::encode(&model)),
                    }))
                }
                other => Err(format!("unsupported lessdb workflow: {other}")),
            }
        }
        _ => Err(format!("unknown lessdb scenario: {scenario}")),
    }
}
