use json_joy::json_crdt::codec::structural::binary as structural_binary;
use json_joy::json_crdt::constants::ORIGIN;
use json_joy::json_crdt::model::Model;
use json_joy::json_crdt::nodes::{CrdtNode, TsKey, ValNode};
use json_joy::json_crdt_diff::diff_node;
use json_joy::json_crdt_patch::clock::{ts, tss, Ts};
use json_joy::json_crdt_patch::codec::clock::ClockTable;
use json_joy::json_crdt_patch::operations::{ConValue, Op};
use json_joy::json_crdt_patch::patch::Patch;
use json_joy::json_crdt_patch::patch_builder::PatchBuilder;
use json_joy::json_crdt_patch::util::binary::{CrdtReader, CrdtWriter};
use json_joy_json_pack::PackValue;
use serde_json::{json, Map, Value};

use crate::common::assertions::{decode_hex, encode_hex, op_to_opcode};

pub(super) fn build_json_val(builder: &mut PatchBuilder, v: &Value) -> Ts {
    let val_id = builder.val();
    let con_id = builder.con_val(PackValue::from_json_scalar(v));
    builder.set_val(val_id, con_id);
    val_id
}

pub(super) fn build_json(builder: &mut PatchBuilder, v: &Value) -> Ts {
    match v {
        // Mirrors PatchBuilder.json(): scalars are val(con).
        Value::Null | Value::Bool(_) | Value::Number(_) => build_json_val(builder, v),
        Value::String(s) => {
            let str_id = builder.str_node();
            if !s.is_empty() {
                builder.ins_str(str_id, str_id, s.clone());
            }
            str_id
        }
        Value::Array(items) => {
            let arr_id = builder.arr();
            if !items.is_empty() {
                let ids: Vec<Ts> = items.iter().map(|item| build_json(builder, item)).collect();
                builder.ins_arr(arr_id, arr_id, ids);
            }
            arr_id
        }
        Value::Object(map) => {
            let obj_id = builder.obj();
            if !map.is_empty() {
                let pairs: Vec<(String, Ts)> = map
                    .iter()
                    .map(|(k, v)| {
                        let id = match v {
                            // Mirrors PatchBuilder.jsonObj(): object scalar fields are con.
                            Value::Null | Value::Bool(_) | Value::Number(_) => {
                                builder.con_val(PackValue::from_json_scalar(v))
                            }
                            _ => build_json(builder, v),
                        };
                        (k.clone(), id)
                    })
                    .collect();
                builder.ins_obj(obj_id, pairs);
            }
            obj_id
        }
    }
}

pub(super) fn build_const_or_json(builder: &mut PatchBuilder, v: &Value) -> Ts {
    match v {
        // Mirrors PatchBuilder.constOrJson(): root scalar values are con.
        Value::Null | Value::Bool(_) | Value::Number(_) => {
            builder.con_val(PackValue::from_json_scalar(v))
        }
        _ => build_json(builder, v),
    }
}

pub(super) fn model_from_json(data: &Value, sid: u64) -> Model {
    let mut model = Model::new(sid);
    let mut builder = PatchBuilder::new(sid, model.clock.time);
    let root = build_const_or_json(&mut builder, data);
    builder.root(root);
    let patch = builder.flush();
    if !patch.ops.is_empty() {
        model.apply_patch(&patch);
    }
    model
}

pub(super) fn patch_stats(patch: &Patch) -> Value {
    let opcodes: Vec<Value> = patch
        .ops
        .iter()
        .map(|op| Value::from(op_to_opcode(op) as u64))
        .collect();
    let id = patch.get_id();
    json!({
        "patch_present": true,
        "patch_binary_hex": encode_hex(&patch.to_binary()),
        "patch_op_count": patch.ops.len(),
        "patch_opcodes": opcodes,
        "patch_span": patch.span(),
        "patch_id_sid": id.map(|x| x.sid),
        "patch_id_time": id.map(|x| x.time),
        "patch_next_time": patch.next_time(),
    })
}

pub(super) fn set_model_sid(model: &mut Model, sid: u64) {
    if model.clock.sid != sid {
        model.clock = model.clock.fork(sid);
    }
}

pub(super) fn model_from_patches(patches: &[Patch]) -> Result<Model, String> {
    if patches.is_empty() {
        return Err("NO_PATCHES".to_string());
    }
    let sid = patches
        .first()
        .and_then(Patch::get_id)
        .map(|id| id.sid)
        .ok_or_else(|| "NO_SID".to_string())?;
    if sid == 0 {
        return Err("NO_SID".to_string());
    }
    let mut model = Model::new(sid);
    for patch in patches {
        model.apply_patch(patch);
    }
    Ok(model)
}

pub(super) fn append_patch_log(existing: &[u8], patch_binary: &[u8]) -> Vec<u8> {
    if existing.is_empty() {
        let mut out = Vec::with_capacity(1 + 4 + patch_binary.len());
        out.push(1);
        out.extend_from_slice(&(patch_binary.len() as u32).to_be_bytes());
        out.extend_from_slice(patch_binary);
        return out;
    }
    let mut out = Vec::with_capacity(existing.len() + 4 + patch_binary.len());
    out.extend_from_slice(existing);
    out.extend_from_slice(&(patch_binary.len() as u32).to_be_bytes());
    out.extend_from_slice(patch_binary);
    out
}

pub(super) fn decode_patch_log_count(data: &[u8]) -> Result<usize, String> {
    if data.is_empty() {
        return Ok(0);
    }
    if data[0] != 1 {
        return Err("Unsupported patch log version".to_string());
    }
    let mut offset = 1usize;
    let mut count = 0usize;
    while offset < data.len() {
        if offset + 4 > data.len() {
            return Err("Corrupt pending patches: truncated length header".to_string());
        }
        let len = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        offset += 4;
        if offset + len > data.len() {
            return Err("Corrupt pending patches: truncated patch data".to_string());
        }
        offset += len;
        count += 1;
    }
    Ok(count)
}

pub(super) fn parse_ts(v: &Value) -> Result<Ts, String> {
    let arr = v
        .as_array()
        .ok_or_else(|| "ts must be [sid,time]".to_string())?;
    if arr.len() != 2 {
        return Err("ts must have 2 elements".to_string());
    }
    let sid = arr[0]
        .as_u64()
        .ok_or_else(|| "sid must be u64".to_string())?;
    let time = arr[1]
        .as_u64()
        .ok_or_else(|| "time must be u64".to_string())?;
    Ok(ts(sid, time))
}

pub(super) fn encode_clock_table_binary(table: &ClockTable) -> Vec<u8> {
    let mut writer = CrdtWriter::new();
    writer.vu57(table.by_idx.len() as u64);
    for entry in &table.by_idx {
        writer.vu57(entry.sid);
        writer.vu57(entry.time);
    }
    writer.flush()
}

pub(super) fn decode_clock_table_binary(data: &[u8]) -> Result<ClockTable, String> {
    let mut reader = CrdtReader::new(data);
    let n = reader.vu57() as usize;
    if n == 0 {
        return Err("invalid clock table: empty".to_string());
    }
    let mut table = ClockTable::new();
    for _ in 0..n {
        let sid = reader.vu57();
        let time = reader.vu57();
        table.push(ts(sid, time));
    }
    Ok(table)
}

pub(super) fn parse_patch_ops(input_ops: &[Value]) -> Result<Vec<Op>, String> {
    let mut ops = Vec::<Op>::with_capacity(input_ops.len());
    for opv in input_ops {
        let obj = opv
            .as_object()
            .ok_or_else(|| "op must be object".to_string())?;
        let kind = obj
            .get("op")
            .and_then(Value::as_str)
            .ok_or_else(|| "op.op missing".to_string())?;
        let id = parse_ts(obj.get("id").ok_or_else(|| "op.id missing".to_string())?)?;
        let op = match kind {
            "new_con" => Op::NewCon {
                id,
                val: ConValue::Val(PackValue::from(obj.get("value").unwrap_or(&Value::Null))),
            },
            "new_con_ref" => Op::NewCon {
                id,
                val: ConValue::Ref(parse_ts(
                    obj.get("value_ref")
                        .ok_or_else(|| "value_ref missing".to_string())?,
                )?),
            },
            "new_val" => Op::NewVal { id },
            "new_obj" => Op::NewObj { id },
            "new_vec" => Op::NewVec { id },
            "new_str" => Op::NewStr { id },
            "new_bin" => Op::NewBin { id },
            "new_arr" => Op::NewArr { id },
            "ins_val" => Op::InsVal {
                id,
                obj: parse_ts(obj.get("obj").ok_or_else(|| "obj missing".to_string())?)?,
                val: parse_ts(obj.get("val").ok_or_else(|| "val missing".to_string())?)?,
            },
            "ins_obj" => {
                let data = obj
                    .get("data")
                    .and_then(Value::as_array)
                    .ok_or_else(|| "data missing".to_string())?
                    .iter()
                    .map(|pair| {
                        let arr = pair.as_array().ok_or_else(|| "ins_obj pair".to_string())?;
                        if arr.len() != 2 {
                            return Err("ins_obj pair len".to_string());
                        }
                        let key = arr[0].as_str().ok_or_else(|| "ins_obj key".to_string())?;
                        let id = parse_ts(&arr[1])?;
                        Ok((key.to_string(), id))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                Op::InsObj {
                    id,
                    obj: parse_ts(obj.get("obj").ok_or_else(|| "obj missing".to_string())?)?,
                    data,
                }
            }
            "ins_vec" => {
                let data = obj
                    .get("data")
                    .and_then(Value::as_array)
                    .ok_or_else(|| "data missing".to_string())?
                    .iter()
                    .map(|pair| {
                        let arr = pair.as_array().ok_or_else(|| "ins_vec pair".to_string())?;
                        if arr.len() != 2 {
                            return Err("ins_vec pair len".to_string());
                        }
                        let idx = arr[0].as_u64().ok_or_else(|| "ins_vec idx".to_string())?;
                        let id = parse_ts(&arr[1])?;
                        Ok((idx as u8, id))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                Op::InsVec {
                    id,
                    obj: parse_ts(obj.get("obj").ok_or_else(|| "obj missing".to_string())?)?,
                    data,
                }
            }
            "ins_str" => Op::InsStr {
                id,
                obj: parse_ts(obj.get("obj").ok_or_else(|| "obj missing".to_string())?)?,
                after: parse_ts(obj.get("ref").ok_or_else(|| "ref missing".to_string())?)?,
                data: obj
                    .get("data")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "data missing".to_string())?
                    .to_string(),
            },
            "ins_bin" => {
                let data = obj
                    .get("data")
                    .and_then(Value::as_array)
                    .ok_or_else(|| "data missing".to_string())?
                    .iter()
                    .map(|v| {
                        let x = v.as_u64().ok_or_else(|| "ins_bin byte".to_string())?;
                        Ok(x as u8)
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                Op::InsBin {
                    id,
                    obj: parse_ts(obj.get("obj").ok_or_else(|| "obj missing".to_string())?)?,
                    after: parse_ts(obj.get("ref").ok_or_else(|| "ref missing".to_string())?)?,
                    data,
                }
            }
            "ins_arr" => {
                let data = obj
                    .get("data")
                    .and_then(Value::as_array)
                    .ok_or_else(|| "data missing".to_string())?
                    .iter()
                    .map(parse_ts)
                    .collect::<Result<Vec<_>, String>>()?;
                Op::InsArr {
                    id,
                    obj: parse_ts(obj.get("obj").ok_or_else(|| "obj missing".to_string())?)?,
                    after: parse_ts(obj.get("ref").ok_or_else(|| "ref missing".to_string())?)?,
                    data,
                }
            }
            "upd_arr" => Op::UpdArr {
                id,
                obj: parse_ts(obj.get("obj").ok_or_else(|| "obj missing".to_string())?)?,
                after: parse_ts(obj.get("ref").ok_or_else(|| "ref missing".to_string())?)?,
                val: parse_ts(obj.get("val").ok_or_else(|| "val missing".to_string())?)?,
            },
            "del" => {
                let what = obj
                    .get("what")
                    .and_then(Value::as_array)
                    .ok_or_else(|| "what missing".to_string())?
                    .iter()
                    .map(|spanv| {
                        let arr = spanv
                            .as_array()
                            .ok_or_else(|| "span must be array".to_string())?;
                        if arr.len() != 3 {
                            return Err("span must have 3 values".to_string());
                        }
                        let sid = arr[0].as_u64().ok_or_else(|| "span sid".to_string())?;
                        let time = arr[1].as_u64().ok_or_else(|| "span time".to_string())?;
                        let span = arr[2].as_u64().ok_or_else(|| "span size".to_string())?;
                        Ok(tss(sid, time, span))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                Op::Del {
                    id,
                    obj: parse_ts(obj.get("obj").ok_or_else(|| "obj missing".to_string())?)?,
                    what,
                }
            }
            "nop" => Op::Nop {
                id,
                len: obj.get("len").and_then(Value::as_u64).unwrap_or(1),
            },
            _ => return Err(format!("unsupported op kind {kind}")),
        };
        ops.push(op);
    }
    Ok(ops)
}

pub(super) fn view_and_binary_after_apply(model: &mut Model, patch: &Patch) -> Value {
    model.apply_patch(patch);
    let mut view_json = model.view();
    if let Some(CrdtNode::Bin(_)) = model.index.get(&TsKey::from(model.root.val)) {
        if let Value::Array(items) = view_json {
            // JS serializes Uint8Array via JSON.stringify to {"0":...} shape.
            let mut obj = Map::new();
            for (i, v) in items.into_iter().enumerate() {
                obj.insert(i.to_string(), v);
            }
            view_json = Value::Object(obj);
        }
    }
    json!({
        "view_after_apply_json": view_json,
        "model_binary_after_apply_hex": encode_hex(&structural_binary::encode(model)),
    })
}

pub(super) fn parse_path(path: &Value) -> Result<Vec<Value>, String> {
    path.as_array()
        .cloned()
        .ok_or_else(|| "path must be array".to_string())
}

pub(super) fn path_step_to_index(step: &Value) -> Option<usize> {
    match step {
        Value::Number(n) => n
            .as_i64()
            .and_then(|v| {
                if v >= 0 {
                    usize::try_from(v).ok()
                } else {
                    None
                }
            })
            .or_else(|| n.as_u64().and_then(|v| usize::try_from(v).ok())),
        Value::String(s) => s.parse::<usize>().ok(),
        _ => None,
    }
}

pub(super) fn path_step_to_key(step: &Value) -> String {
    match step {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        _ => step.to_string(),
    }
}

fn clamped_insert_index(step: &Value, len: usize) -> usize {
    let raw = match step {
        Value::Number(n) => n
            .as_i64()
            .or_else(|| n.as_u64().and_then(|v| i64::try_from(v).ok()))
            .unwrap_or(0),
        Value::String(s) => s.parse::<i64>().unwrap_or(0),
        _ => 0,
    };
    if raw <= 0 {
        0
    } else {
        usize::try_from(raw).unwrap_or(usize::MAX).min(len)
    }
}

pub(super) fn find_at_path<'a>(root: &'a Value, path: &[Value]) -> Result<&'a Value, String> {
    if path.is_empty() {
        return Ok(root);
    }
    match root {
        Value::Array(items) => {
            let idx = path_step_to_index(&path[0])
                .ok_or_else(|| "invalid array index in path".to_string())?;
            let next = items
                .get(idx)
                .ok_or_else(|| "array path index out of bounds".to_string())?;
            find_at_path(next, &path[1..])
        }
        Value::Object(map) => {
            let key = path_step_to_key(&path[0]);
            let next = map
                .get(&key)
                .ok_or_else(|| "missing object key in path".to_string())?;
            find_at_path(next, &path[1..])
        }
        _ => Err("non-container in path".to_string()),
    }
}

pub(super) fn find_at_path_mut<'a>(
    root: &'a mut Value,
    path: &[Value],
) -> Result<&'a mut Value, String> {
    if path.is_empty() {
        return Ok(root);
    }
    match root {
        Value::Array(items) => {
            let idx = path_step_to_index(&path[0])
                .ok_or_else(|| "invalid array index in path".to_string())?;
            let next = items
                .get_mut(idx)
                .ok_or_else(|| "array path index out of bounds".to_string())?;
            find_at_path_mut(next, &path[1..])
        }
        Value::Object(map) => {
            let key = path_step_to_key(&path[0]);
            let next = map
                .get_mut(&key)
                .ok_or_else(|| "missing object key in path".to_string())?;
            find_at_path_mut(next, &path[1..])
        }
        _ => Err("non-container in path".to_string()),
    }
}

pub(super) fn set_at_path(root: &mut Value, path: &[Value], value: Value) -> Result<(), String> {
    if path.is_empty() {
        *root = value;
        return Ok(());
    }

    let parent_path = &path[..path.len() - 1];
    let leaf = &path[path.len() - 1];
    let parent = find_at_path_mut(root, parent_path)?;

    match parent {
        Value::Array(items) => {
            let idx = path_step_to_index(leaf).ok_or_else(|| "invalid array leaf".to_string())?;
            if idx < items.len() {
                items[idx] = value;
                Ok(())
            } else if idx == items.len() {
                items.push(value);
                Ok(())
            } else {
                Err("array leaf index out of bounds".to_string())
            }
        }
        Value::Object(map) => {
            map.insert(path_step_to_key(leaf), value);
            Ok(())
        }
        _ => Err("invalid leaf parent".to_string()),
    }
}

pub(super) fn add_at_path(root: &mut Value, path: &[Value], value: Value) -> Result<(), String> {
    if path.is_empty() {
        return Err("add path must not be empty".to_string());
    }
    let parent_path = &path[..path.len() - 1];
    let leaf = &path[path.len() - 1];
    let parent = find_at_path_mut(root, parent_path)?;

    match parent {
        Value::Array(items) => {
            let idx = clamped_insert_index(leaf, items.len());
            items.insert(idx, value);
            Ok(())
        }
        Value::Object(map) => {
            map.insert(path_step_to_key(leaf), value);
            Ok(())
        }
        _ => Err("add parent is not container".to_string()),
    }
}

pub(super) fn remove_at_path(root: &mut Value, path: &[Value]) -> Result<(), String> {
    if path.is_empty() {
        return Err("remove path must not be empty".to_string());
    }
    let parent_path = &path[..path.len() - 1];
    let leaf = &path[path.len() - 1];
    let parent = find_at_path_mut(root, parent_path)?;

    match parent {
        Value::Array(items) => {
            if let Some(idx) = path_step_to_index(leaf) {
                if idx < items.len() {
                    items.remove(idx);
                }
            }
            Ok(())
        }
        Value::Object(map) => {
            map.remove(&path_step_to_key(leaf));
            Ok(())
        }
        _ => Err("remove parent is not container".to_string()),
    }
}

pub(super) fn model_api_diff_patch(model: &Model, sid: u64, next: &Value) -> Option<Patch> {
    let root_val = CrdtNode::Val(ValNode {
        id: ORIGIN,
        val: model.root.val,
    });
    diff_node(&root_val, &model.index, sid, model.clock.time, next)
}

pub(super) fn parse_ts_pair(v: &Value) -> Result<(u64, u64), String> {
    let arr = v
        .as_array()
        .ok_or_else(|| "timestamp must be [sid,time]".to_string())?;
    if arr.len() != 2 {
        return Err("timestamp must have 2 elements".to_string());
    }
    let sid = arr[0]
        .as_u64()
        .ok_or_else(|| "timestamp sid must be u64".to_string())?;
    let time = arr[1]
        .as_u64()
        .ok_or_else(|| "timestamp time must be u64".to_string())?;
    Ok((sid, time))
}

pub(super) fn write_u32be(out: &mut Vec<u8>, n: u32) {
    out.extend_from_slice(&n.to_be_bytes());
}

pub(super) fn write_vu57(out: &mut Vec<u8>, n: u64) {
    let mut w = CrdtWriter::new();
    w.vu57(n);
    out.extend_from_slice(&w.flush());
}

pub(super) fn write_b1vu56(out: &mut Vec<u8>, flag: u8, n: u64) {
    let mut w = CrdtWriter::new();
    w.b1vu56(flag, n);
    out.extend_from_slice(&w.flush());
}

pub(super) fn write_cbor_major(out: &mut Vec<u8>, major: u8, n: u64) {
    if n < 24 {
        out.push((major << 5) | n as u8);
    } else if n < 256 {
        out.push((major << 5) | 24);
        out.push(n as u8);
    } else if n < 65536 {
        out.push((major << 5) | 25);
        out.extend_from_slice(&(n as u16).to_be_bytes());
    } else {
        out.push((major << 5) | 26);
        out.extend_from_slice(&(n as u32).to_be_bytes());
    }
}

pub(super) fn write_cbor_canonical(out: &mut Vec<u8>, v: &Value) -> Result<(), String> {
    match v {
        Value::Null => {
            out.push(0xf6);
            Ok(())
        }
        Value::Bool(false) => {
            out.push(0xf4);
            Ok(())
        }
        Value::Bool(true) => {
            out.push(0xf5);
            Ok(())
        }
        Value::Number(num) => {
            if let Some(i) = num.as_i64() {
                if i >= 0 {
                    write_cbor_major(out, 0, i as u64);
                } else {
                    write_cbor_major(out, 1, (-1 - i) as u64);
                }
                Ok(())
            } else if let Some(u) = num.as_u64() {
                write_cbor_major(out, 0, u);
                Ok(())
            } else if let Some(f) = num.as_f64() {
                out.push(0xfb);
                out.extend_from_slice(&f.to_be_bytes());
                Ok(())
            } else {
                Err("unsupported number".to_string())
            }
        }
        Value::String(s) => {
            let b = s.as_bytes();
            write_cbor_major(out, 3, b.len() as u64);
            out.extend_from_slice(b);
            Ok(())
        }
        _ => Err(format!("unsupported cbor value: {v}")),
    }
}

pub(super) fn encode_model_canonical(input: &Map<String, Value>) -> Result<Vec<u8>, String> {
    let mode = input
        .get("mode")
        .and_then(Value::as_str)
        .ok_or_else(|| "input.mode missing".to_string())?;
    let root = input
        .get("root")
        .ok_or_else(|| "input.root missing".to_string())?;

    let mut clock_table = Vec::<(u64, u64)>::new();
    let mut idx_by_sid = std::collections::HashMap::<u64, usize>::new();
    let mut base_by_sid = std::collections::HashMap::<u64, u64>::new();
    if mode == "logical" {
        let arr = input
            .get("clock_table")
            .and_then(Value::as_array)
            .ok_or_else(|| "input.clock_table missing".to_string())?;
        for (i, v) in arr.iter().enumerate() {
            let (sid, time) = parse_ts_pair(v)?;
            clock_table.push((sid, time));
            idx_by_sid.insert(sid, i);
            base_by_sid.insert(sid, time);
        }
    }

    fn write_type_len(out: &mut Vec<u8>, major: u8, len: usize) {
        if len < 31 {
            out.push((major << 5) | len as u8);
        } else {
            out.push((major << 5) | 31);
            write_vu57(out, len as u64);
        }
    }

    fn encode_id(
        out: &mut Vec<u8>,
        mode: &str,
        node_id: &Value,
        idx_by_sid: &std::collections::HashMap<u64, usize>,
        base_by_sid: &std::collections::HashMap<u64, u64>,
    ) -> Result<(), String> {
        let (sid, time) = parse_ts_pair(node_id)?;
        if mode == "server" {
            write_vu57(out, time);
            return Ok(());
        }
        let idx = *idx_by_sid
            .get(&sid)
            .ok_or_else(|| format!("sid {sid} missing from clock_table"))?;
        let base = *base_by_sid
            .get(&sid)
            .ok_or_else(|| format!("sid {sid} missing base clock"))?;
        let diff = time
            .checked_sub(base)
            .ok_or_else(|| "timestamp underflow".to_string())?;
        if idx <= 7 && diff <= 15 {
            out.push(((idx as u8) << 4) | (diff as u8));
        } else {
            write_b1vu56(out, 0, idx as u64);
            write_vu57(out, diff);
        }
        Ok(())
    }

    fn write_node(
        out: &mut Vec<u8>,
        mode: &str,
        node: &Value,
        idx_by_sid: &std::collections::HashMap<u64, usize>,
        base_by_sid: &std::collections::HashMap<u64, u64>,
    ) -> Result<(), String> {
        let obj = node
            .as_object()
            .ok_or_else(|| "node must be object".to_string())?;
        encode_id(
            out,
            mode,
            obj.get("id").ok_or_else(|| "node.id missing".to_string())?,
            idx_by_sid,
            base_by_sid,
        )?;
        let kind = obj
            .get("kind")
            .and_then(Value::as_str)
            .ok_or_else(|| "node.kind missing".to_string())?;
        match kind {
            "con" => {
                out.push(0b0000_0000);
                write_cbor_canonical(
                    out,
                    obj.get("value")
                        .ok_or_else(|| "con.value missing".to_string())?,
                )?;
            }
            "val" => {
                out.push(0b0010_0000);
                write_node(
                    out,
                    mode,
                    obj.get("child")
                        .ok_or_else(|| "val.child missing".to_string())?,
                    idx_by_sid,
                    base_by_sid,
                )?;
            }
            "obj" => {
                let entries = obj
                    .get("entries")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                write_type_len(out, 2, entries.len());
                for entry in entries {
                    let eobj = entry
                        .as_object()
                        .ok_or_else(|| "obj entry must be object".to_string())?;
                    write_cbor_canonical(
                        out,
                        eobj.get("key")
                            .ok_or_else(|| "obj entry key missing".to_string())?,
                    )?;
                    write_node(
                        out,
                        mode,
                        eobj.get("value")
                            .ok_or_else(|| "obj entry value missing".to_string())?,
                        idx_by_sid,
                        base_by_sid,
                    )?;
                }
            }
            "vec" => {
                let values = obj
                    .get("values")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                write_type_len(out, 3, values.len());
                for v in values {
                    if v.is_null() {
                        out.push(0);
                    } else {
                        write_node(out, mode, &v, idx_by_sid, base_by_sid)?;
                    }
                }
            }
            "str" => {
                let chunks = obj
                    .get("chunks")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                write_type_len(out, 4, chunks.len());
                for chunk in chunks {
                    let cobj = chunk
                        .as_object()
                        .ok_or_else(|| "str chunk must be object".to_string())?;
                    encode_id(
                        out,
                        mode,
                        cobj.get("id")
                            .ok_or_else(|| "str chunk id missing".to_string())?,
                        idx_by_sid,
                        base_by_sid,
                    )?;
                    if let Some(text) = cobj.get("text") {
                        write_cbor_canonical(out, text)?;
                    } else {
                        write_cbor_canonical(
                            out,
                            cobj.get("deleted")
                                .ok_or_else(|| "str chunk deleted missing".to_string())?,
                        )?;
                    }
                }
            }
            "bin" => {
                let chunks = obj
                    .get("chunks")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                write_type_len(out, 5, chunks.len());
                for chunk in chunks {
                    let cobj = chunk
                        .as_object()
                        .ok_or_else(|| "bin chunk must be object".to_string())?;
                    encode_id(
                        out,
                        mode,
                        cobj.get("id")
                            .ok_or_else(|| "bin chunk id missing".to_string())?,
                        idx_by_sid,
                        base_by_sid,
                    )?;
                    if let Some(deleted) = cobj.get("deleted") {
                        let n = deleted
                            .as_u64()
                            .ok_or_else(|| "bin deleted must be u64".to_string())?;
                        write_b1vu56(out, 1, n);
                    } else {
                        let hex = cobj
                            .get("bytes_hex")
                            .and_then(Value::as_str)
                            .ok_or_else(|| "bin bytes_hex missing".to_string())?;
                        let bytes = decode_hex(hex)?;
                        write_b1vu56(out, 0, bytes.len() as u64);
                        out.extend_from_slice(&bytes);
                    }
                }
            }
            "arr" => {
                let chunks = obj
                    .get("chunks")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                write_type_len(out, 6, chunks.len());
                for chunk in chunks {
                    let cobj = chunk
                        .as_object()
                        .ok_or_else(|| "arr chunk must be object".to_string())?;
                    encode_id(
                        out,
                        mode,
                        cobj.get("id")
                            .ok_or_else(|| "arr chunk id missing".to_string())?,
                        idx_by_sid,
                        base_by_sid,
                    )?;
                    if let Some(deleted) = cobj.get("deleted") {
                        let n = deleted
                            .as_u64()
                            .ok_or_else(|| "arr deleted must be u64".to_string())?;
                        write_b1vu56(out, 1, n);
                    } else {
                        let vals = cobj
                            .get("values")
                            .and_then(Value::as_array)
                            .cloned()
                            .unwrap_or_default();
                        write_b1vu56(out, 0, vals.len() as u64);
                        for v in vals {
                            write_node(out, mode, &v, idx_by_sid, base_by_sid)?;
                        }
                    }
                }
            }
            _ => return Err(format!("unsupported canonical model kind: {kind}")),
        }
        Ok(())
    }

    let mut root_bytes = Vec::<u8>::new();
    write_node(&mut root_bytes, mode, root, &idx_by_sid, &base_by_sid)?;

    if mode == "server" {
        let server_time = input
            .get("server_time")
            .and_then(Value::as_u64)
            .ok_or_else(|| "input.server_time missing".to_string())?;
        let mut out = Vec::<u8>::new();
        out.push(0x80);
        write_vu57(&mut out, server_time);
        out.extend_from_slice(&root_bytes);
        return Ok(out);
    }

    let mut out = Vec::<u8>::new();
    write_u32be(&mut out, root_bytes.len() as u32);
    out.extend_from_slice(&root_bytes);
    write_vu57(&mut out, clock_table.len() as u64);
    for (sid, time) in clock_table {
        write_vu57(&mut out, sid);
        write_vu57(&mut out, time);
    }
    Ok(out)
}
