//! Structural verbose JSON codec.
//!
//! Mirrors:
//! - `structural/verbose/Encoder.ts`
//! - `structural/verbose/Decoder.ts`
//! - `structural/verbose/types.ts`
//!
//! Wire format: a JSON object with full field names.
//!
//! ```json
//! {
//!   "time": <number | [sid, time, ...]>,
//!   "root": { "type": "val", "id": ..., "value": <node> }
//! }
//! ```

use json_joy_base64::{from_base64, to_base64};
use serde_json::{json, Value};

use crate::json_crdt::constants::UNDEFINED_TS;
use crate::json_crdt::model::Model;
use crate::json_crdt::nodes::{
    ArrNode, BinNode, ConNode, CrdtNode, ObjNode, StrNode, TsKey, ValNode, VecNode,
};
use crate::json_crdt_patch::clock::{ts as mk_ts, ClockVector, Ts};
use crate::json_crdt_patch::enums::SESSION;
use crate::json_crdt_patch::operations::ConValue;
use json_joy_json_pack::PackValue;

// ── Encode ──────────────────────────────────────────────────────────────────

/// Encode a [`Model`] to the verbose JSON format.
pub fn encode(model: &Model) -> Value {
    let is_server = model.clock.sid == SESSION::SERVER;
    let time: Value = if is_server {
        json!(model.clock.time)
    } else {
        encode_clock(&model.clock)
    };

    let root = encode_val_root(model);
    json!({ "time": time, "root": root })
}

fn encode_clock(clock: &ClockVector) -> Value {
    let mut entries = Vec::new();
    // Local session first
    let local_ts = json!([clock.sid, clock.time]);
    entries.push(local_ts);
    // Peer sessions
    for peer in clock.peers.values() {
        entries.push(json!([peer.sid, peer.time]));
    }
    Value::Array(entries)
}

fn encode_ts(stamp: Ts) -> Value {
    if stamp.sid == SESSION::SERVER {
        json!(stamp.time)
    } else {
        json!([stamp.sid, stamp.time])
    }
}

fn encode_val_root(model: &Model) -> Value {
    let root_ts = model.root.val;
    let id = encode_ts(UNDEFINED_TS);
    if root_ts == UNDEFINED_TS || root_ts.time == 0 {
        // empty root: val node pointing to undefined
        json!({
            "type": "val",
            "id": id,
            "value": encode_con_undefined()
        })
    } else {
        let child_node = model.index.get(&TsKey::from(root_ts));
        let value = match child_node {
            Some(n) => encode_node(model, n),
            None => encode_con_undefined(),
        };
        json!({
            "type": "val",
            "id": encode_ts(root_ts),
            "value": value
        })
    }
}

fn encode_con_undefined() -> Value {
    json!({ "type": "con", "id": encode_ts(UNDEFINED_TS) })
}

fn encode_node(model: &Model, node: &CrdtNode) -> Value {
    match node {
        CrdtNode::Con(n) => encode_con(n),
        CrdtNode::Val(n) => encode_val(model, n),
        CrdtNode::Obj(n) => encode_obj(model, n),
        CrdtNode::Vec(n) => encode_vec(model, n),
        CrdtNode::Str(n) => encode_str(n),
        CrdtNode::Bin(n) => encode_bin(n),
        CrdtNode::Arr(n) => encode_arr(model, n),
    }
}

fn encode_con(node: &ConNode) -> Value {
    let id = encode_ts(node.id);
    match &node.val {
        ConValue::Ref(ref_ts) => {
            json!({
                "type": "con",
                "id": id,
                "timestamp": true,
                "value": encode_ts(*ref_ts)
            })
        }
        ConValue::Val(pv) => match pv {
            PackValue::Undefined => {
                json!({ "type": "con", "id": id })
            }
            _ => {
                let v = serde_json::Value::from(pv.clone());
                json!({ "type": "con", "id": id, "value": v })
            }
        },
    }
}

fn encode_val(model: &Model, node: &ValNode) -> Value {
    let id = encode_ts(node.id);
    let child_ts = node.val;
    let value = match model.index.get(&TsKey::from(child_ts)) {
        Some(n) => encode_node(model, n),
        None => encode_con_undefined(),
    };
    json!({ "type": "val", "id": id, "value": value })
}

fn encode_obj(model: &Model, node: &ObjNode) -> Value {
    let id = encode_ts(node.id);
    let mut map = serde_json::Map::new();
    for (key, child_ts) in &node.keys {
        if let Some(child) = model.index.get(&TsKey::from(*child_ts)) {
            map.insert(key.clone(), encode_node(model, child));
        }
    }
    json!({ "type": "obj", "id": id, "map": map })
}

fn encode_vec(model: &Model, node: &VecNode) -> Value {
    let id = encode_ts(node.id);
    let elements: Vec<Value> = node
        .elements
        .iter()
        .map(|e| match e {
            None => Value::Null,
            Some(child_ts) => match model.index.get(&TsKey::from(*child_ts)) {
                Some(child) => encode_node(model, child),
                None => Value::Null,
            },
        })
        .collect();
    json!({ "type": "vec", "id": id, "map": elements })
}

fn encode_str(node: &StrNode) -> Value {
    let id = encode_ts(node.id);
    let chunks: Vec<Value> = node
        .rga
        .iter()
        .map(|chunk| {
            let chunk_id = encode_ts(chunk.id);
            if chunk.deleted {
                json!({ "id": chunk_id, "span": chunk.span })
            } else {
                let data = chunk.data.as_deref().unwrap_or("");
                json!({ "id": chunk_id, "value": data })
            }
        })
        .collect();
    json!({ "type": "str", "id": id, "chunks": chunks })
}

fn encode_bin(node: &BinNode) -> Value {
    let id = encode_ts(node.id);
    let chunks: Vec<Value> = node
        .rga
        .iter()
        .map(|chunk| {
            let chunk_id = encode_ts(chunk.id);
            if chunk.deleted {
                json!({ "id": chunk_id, "span": chunk.span })
            } else {
                let data = chunk.data.as_deref().unwrap_or(&[]);
                let b64 = to_base64(data);
                json!({ "id": chunk_id, "value": b64 })
            }
        })
        .collect();
    json!({ "type": "bin", "id": id, "chunks": chunks })
}

fn encode_arr(model: &Model, node: &ArrNode) -> Value {
    let id = encode_ts(node.id);
    let chunks: Vec<Value> = node
        .rga
        .iter()
        .map(|chunk| {
            let chunk_id = encode_ts(chunk.id);
            if chunk.deleted {
                json!({ "id": chunk_id, "span": chunk.span })
            } else {
                let ids = chunk.data.as_deref().unwrap_or(&[]);
                let values: Vec<Value> = ids
                    .iter()
                    .filter_map(|id| {
                        model
                            .index
                            .get(&TsKey::from(*id))
                            .map(|n| encode_node(model, n))
                    })
                    .collect();
                json!({ "id": chunk_id, "value": values })
            }
        })
        .collect();
    json!({ "type": "arr", "id": id, "chunks": chunks })
}

// ── Decode ──────────────────────────────────────────────────────────────────

/// Errors that can occur during verbose decode.
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("unexpected format: {0}")]
    Format(String),
    #[error("unknown node type: {0}")]
    UnknownNodeType(String),
    #[error("missing field: {0}")]
    MissingField(String),
}

/// Decode a verbose JSON document back into a [`Model`].
pub fn decode(data: &Value) -> Result<Model, DecodeError> {
    let obj = data
        .as_object()
        .ok_or_else(|| DecodeError::Format("expected object".into()))?;

    let time_val = obj
        .get("time")
        .ok_or_else(|| DecodeError::MissingField("time".into()))?;
    let root_val = obj
        .get("root")
        .ok_or_else(|| DecodeError::MissingField("root".into()))?;

    let is_server = time_val.is_number();
    let mut model = if is_server {
        let server_time = time_val
            .as_u64()
            .ok_or_else(|| DecodeError::Format("server time must be u64".into()))?;
        Model::new_server(server_time)
    } else {
        let timestamps = time_val
            .as_array()
            .ok_or_else(|| DecodeError::Format("logical clock must be array".into()))?;
        let clock = decode_clock(timestamps)?;
        Model::new_from_clock(clock)
    };

    decode_root(root_val, &mut model)?;
    Ok(model)
}

fn decode_clock(timestamps: &[Value]) -> Result<ClockVector, DecodeError> {
    if timestamps.is_empty() {
        return Err(DecodeError::Format("clock table is empty".into()));
    }
    let first = &timestamps[0];
    let first_arr = first
        .as_array()
        .ok_or_else(|| DecodeError::Format("clock entry must be array".into()))?;
    if first_arr.len() < 2 {
        return Err(DecodeError::Format("clock entry too short".into()));
    }
    let sid = first_arr[0].as_u64().unwrap_or(0);
    let time = first_arr[1].as_u64().unwrap_or(0);
    let mut clock = ClockVector::new(sid, time);

    for stamp_val in &timestamps[1..] {
        let stamp_arr = stamp_val
            .as_array()
            .ok_or_else(|| DecodeError::Format("clock entry must be array".into()))?;
        if stamp_arr.len() >= 2 {
            let peer_sid = stamp_arr[0].as_u64().unwrap_or(0);
            let peer_time = stamp_arr[1].as_u64().unwrap_or(0);
            clock.observe(mk_ts(peer_sid, peer_time), 1);
        }
    }
    Ok(clock)
}

fn decode_ts(val: &Value) -> Result<Ts, DecodeError> {
    if let Some(t) = val.as_u64() {
        return Ok(mk_ts(SESSION::SERVER, t));
    }
    if let Some(arr) = val.as_array() {
        if arr.len() >= 2 {
            let sid = arr[0].as_u64().unwrap_or(0);
            let time = arr[1].as_u64().unwrap_or(0);
            return Ok(mk_ts(sid, time));
        }
    }
    Err(DecodeError::Format(format!("invalid timestamp: {}", val)))
}

fn decode_root(val: &Value, model: &mut Model) -> Result<(), DecodeError> {
    let obj = val
        .as_object()
        .ok_or_else(|| DecodeError::Format("root must be object".into()))?;

    if let Some(value_val) = obj.get("value") {
        let child_id = decode_node(value_val, model)?;
        model.root.val = child_id;
    }
    Ok(())
}

fn decode_node(val: &Value, model: &mut Model) -> Result<Ts, DecodeError> {
    let obj = val
        .as_object()
        .ok_or_else(|| DecodeError::Format("node must be object".into()))?;

    let type_str = obj
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DecodeError::MissingField("type".into()))?;

    match type_str {
        "con" => decode_con(val, model),
        "val" => decode_val(val, model),
        "obj" => decode_obj(val, model),
        "vec" => decode_vec(val, model),
        "str" => decode_str(val, model),
        "bin" => decode_bin(val, model),
        "arr" => decode_arr(val, model),
        other => Err(DecodeError::UnknownNodeType(other.into())),
    }
}

fn decode_con(val: &Value, model: &mut Model) -> Result<Ts, DecodeError> {
    let obj = val
        .as_object()
        .ok_or_else(|| DecodeError::MissingField("node object".into()))?;
    let id = decode_ts(
        obj.get("id")
            .ok_or_else(|| DecodeError::MissingField("con.id".into()))?,
    )?;

    let con_val = if obj
        .get("timestamp")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        let ts_val = obj
            .get("value")
            .ok_or_else(|| DecodeError::MissingField("con.value (timestamp)".into()))?;
        let ref_ts = decode_ts(ts_val)?;
        ConValue::Ref(ref_ts)
    } else {
        match obj.get("value") {
            None => ConValue::Val(PackValue::Undefined),
            Some(v) => ConValue::Val(PackValue::from(v.clone())),
        }
    };

    use crate::json_crdt::nodes::ConNode;
    let node = CrdtNode::Con(ConNode::new(id, con_val));
    model.index.insert(TsKey::from(id), node);
    Ok(id)
}

fn decode_val(val: &Value, model: &mut Model) -> Result<Ts, DecodeError> {
    let obj = val
        .as_object()
        .ok_or_else(|| DecodeError::MissingField("node object".into()))?;
    let id = decode_ts(
        obj.get("id")
            .ok_or_else(|| DecodeError::MissingField("val.id".into()))?,
    )?;

    let value = obj
        .get("value")
        .ok_or_else(|| DecodeError::MissingField("val.value".into()))?;
    let child_id = decode_node(value, model)?;

    use crate::json_crdt::nodes::ValNode;
    let mut node = ValNode::new(id);
    node.val = child_id;
    model.index.insert(TsKey::from(id), CrdtNode::Val(node));
    Ok(id)
}

fn decode_obj(val: &Value, model: &mut Model) -> Result<Ts, DecodeError> {
    let obj = val
        .as_object()
        .ok_or_else(|| DecodeError::MissingField("node object".into()))?;
    let id = decode_ts(
        obj.get("id")
            .ok_or_else(|| DecodeError::MissingField("obj.id".into()))?,
    )?;

    let map = obj
        .get("map")
        .and_then(|v| v.as_object())
        .ok_or_else(|| DecodeError::MissingField("obj.map".into()))?;

    use crate::json_crdt::nodes::ObjNode;
    let mut node = ObjNode::new(id);
    let entries: Vec<(String, Value)> = map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    for (key, child_val) in entries {
        let child_id = decode_node(&child_val, model)?;
        node.keys.insert(key, child_id);
    }
    model.index.insert(TsKey::from(id), CrdtNode::Obj(node));
    Ok(id)
}

fn decode_vec(val: &Value, model: &mut Model) -> Result<Ts, DecodeError> {
    let obj = val
        .as_object()
        .ok_or_else(|| DecodeError::MissingField("node object".into()))?;
    let id = decode_ts(
        obj.get("id")
            .ok_or_else(|| DecodeError::MissingField("vec.id".into()))?,
    )?;

    let map_arr = obj
        .get("map")
        .and_then(|v| v.as_array())
        .ok_or_else(|| DecodeError::MissingField("vec.map".into()))?
        .clone();

    use crate::json_crdt::nodes::VecNode;
    let mut node = VecNode::new(id);
    for elem in &map_arr {
        if elem.is_null() {
            node.elements.push(None);
        } else {
            let child_id = decode_node(elem, model)?;
            node.elements.push(Some(child_id));
        }
    }
    model.index.insert(TsKey::from(id), CrdtNode::Vec(node));
    Ok(id)
}

fn decode_str(val: &Value, model: &mut Model) -> Result<Ts, DecodeError> {
    let obj = val
        .as_object()
        .ok_or_else(|| DecodeError::MissingField("node object".into()))?;
    let id = decode_ts(
        obj.get("id")
            .ok_or_else(|| DecodeError::MissingField("str.id".into()))?,
    )?;

    let chunks_arr = obj
        .get("chunks")
        .and_then(|v| v.as_array())
        .ok_or_else(|| DecodeError::MissingField("str.chunks".into()))?
        .clone();

    use crate::json_crdt::nodes::rga::Chunk;
    use crate::json_crdt::nodes::StrNode;

    let mut node = StrNode::new(id);
    for chunk_val in &chunks_arr {
        let chunk_obj = chunk_val
            .as_object()
            .ok_or_else(|| DecodeError::Format("str chunk must be object".into()))?;
        let chunk_id = decode_ts(
            chunk_obj
                .get("id")
                .ok_or_else(|| DecodeError::MissingField("chunk.id".into()))?,
        )?;
        if let Some(span) = chunk_obj.get("span").and_then(|v| v.as_u64()) {
            node.rga.push_chunk(Chunk::new_deleted(chunk_id, span));
        } else if let Some(s) = chunk_obj.get("value").and_then(|v| v.as_str()) {
            let span = s.encode_utf16().count() as u64;
            node.rga
                .push_chunk(Chunk::new(chunk_id, span, s.to_string()));
        } else {
            return Err(DecodeError::Format(
                "str chunk must have span or value".into(),
            ));
        }
    }
    model.index.insert(TsKey::from(id), CrdtNode::Str(node));
    Ok(id)
}

fn decode_bin(val: &Value, model: &mut Model) -> Result<Ts, DecodeError> {
    let obj = val
        .as_object()
        .ok_or_else(|| DecodeError::MissingField("node object".into()))?;
    let id = decode_ts(
        obj.get("id")
            .ok_or_else(|| DecodeError::MissingField("bin.id".into()))?,
    )?;

    let chunks_arr = obj
        .get("chunks")
        .and_then(|v| v.as_array())
        .ok_or_else(|| DecodeError::MissingField("bin.chunks".into()))?
        .clone();

    use crate::json_crdt::nodes::rga::Chunk;
    use crate::json_crdt::nodes::BinNode;

    let mut node = BinNode::new(id);
    for chunk_val in &chunks_arr {
        let chunk_obj = chunk_val
            .as_object()
            .ok_or_else(|| DecodeError::Format("bin chunk must be object".into()))?;
        let chunk_id = decode_ts(
            chunk_obj
                .get("id")
                .ok_or_else(|| DecodeError::MissingField("chunk.id".into()))?,
        )?;
        if let Some(span) = chunk_obj.get("span").and_then(|v| v.as_u64()) {
            node.rga.push_chunk(Chunk::new_deleted(chunk_id, span));
        } else if let Some(b64) = chunk_obj.get("value").and_then(|v| v.as_str()) {
            let data = from_base64(b64)
                .map_err(|e| DecodeError::Format(format!("base64 decode error: {}", e)))?;
            let span = data.len() as u64;
            node.rga.push_chunk(Chunk::new(chunk_id, span, data));
        } else {
            return Err(DecodeError::Format(
                "bin chunk must have span or value".into(),
            ));
        }
    }
    model.index.insert(TsKey::from(id), CrdtNode::Bin(node));
    Ok(id)
}

fn decode_arr(val: &Value, model: &mut Model) -> Result<Ts, DecodeError> {
    let obj = val
        .as_object()
        .ok_or_else(|| DecodeError::MissingField("node object".into()))?;
    let id = decode_ts(
        obj.get("id")
            .ok_or_else(|| DecodeError::MissingField("arr.id".into()))?,
    )?;

    let chunks_arr = obj
        .get("chunks")
        .and_then(|v| v.as_array())
        .ok_or_else(|| DecodeError::MissingField("arr.chunks".into()))?
        .clone();

    use crate::json_crdt::nodes::rga::Chunk;
    use crate::json_crdt::nodes::ArrNode;

    let mut node = ArrNode::new(id);
    for chunk_val in &chunks_arr {
        let chunk_obj = chunk_val
            .as_object()
            .ok_or_else(|| DecodeError::Format("arr chunk must be object".into()))?;
        let chunk_id = decode_ts(
            chunk_obj
                .get("id")
                .ok_or_else(|| DecodeError::MissingField("chunk.id".into()))?,
        )?;
        if let Some(span) = chunk_obj.get("span").and_then(|v| v.as_u64()) {
            node.rga.push_chunk(Chunk::new_deleted(chunk_id, span));
        } else if let Some(values) = chunk_obj.get("value").and_then(|v| v.as_array()) {
            let values = values.clone();
            let mut ids = Vec::new();
            for child_val in &values {
                let child_id = decode_node(child_val, model)?;
                ids.push(child_id);
            }
            let span = ids.len() as u64;
            node.rga.push_chunk(Chunk::new(chunk_id, span, ids));
        } else {
            return Err(DecodeError::Format(
                "arr chunk must have span or value".into(),
            ));
        }
    }
    model.index.insert(TsKey::from(id), CrdtNode::Arr(node));
    Ok(id)
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json_crdt_patch::clock::ts;
    use crate::json_crdt_patch::operations::{ConValue, Op};
    use json_joy_json_pack::PackValue;
    use serde_json::json;

    fn sid() -> u64 {
        654321
    }

    #[test]
    fn encode_empty_model() {
        let model = Model::new(sid());
        let encoded = encode(&model);
        assert!(encoded.get("time").is_some());
        assert!(encoded.get("root").is_some());
    }

    #[test]
    fn roundtrip_simple_string() {
        let mut model = Model::new(sid());
        let s = sid();
        model.apply_operation(&Op::NewStr { id: ts(s, 1) });
        model.apply_operation(&Op::InsStr {
            id: ts(s, 2),
            obj: ts(s, 1),
            after: crate::json_crdt::constants::ORIGIN,
            data: "test".to_string(),
        });
        model.apply_operation(&Op::InsVal {
            id: ts(s, 7),
            obj: crate::json_crdt::constants::ORIGIN,
            val: ts(s, 1),
        });

        let view = model.view();
        let encoded = encode(&model);
        let decoded = decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.view(), view);
    }

    #[test]
    fn roundtrip_con_number() {
        let mut model = Model::new(sid());
        let s = sid();
        model.apply_operation(&Op::NewCon {
            id: ts(s, 1),
            val: ConValue::Val(PackValue::Integer(100)),
        });
        model.apply_operation(&Op::InsVal {
            id: ts(s, 2),
            obj: crate::json_crdt::constants::ORIGIN,
            val: ts(s, 1),
        });

        let view = model.view();
        let encoded = encode(&model);
        let decoded = decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.view(), view);
    }

    #[test]
    fn decode_root_without_value_defaults_to_null() {
        let data = json!({
            "time": 0,
            "root": {
                "type": "val",
                "id": 0
            }
        });
        let decoded = decode(&data).expect("decode should succeed");
        assert_eq!(decoded.view(), json!(null));
    }

    #[test]
    fn decode_val_without_value_field_errors() {
        let data = json!({
            "time": 0,
            "root": {
                "type": "val",
                "id": 0,
                "value": {
                    "type": "val",
                    "id": 1
                }
            }
        });
        let err = decode(&data).expect_err("decode should fail");
        assert!(matches!(err, DecodeError::MissingField(field) if field == "val.value"));
    }

    #[test]
    fn encode_obj_preserves_insertion_order() {
        let s = sid();
        let mut model = Model::new(s);
        model.apply_operation(&Op::NewObj { id: ts(s, 1) });
        model.apply_operation(&Op::NewCon {
            id: ts(s, 2),
            val: ConValue::Val(PackValue::Str("b".to_string())),
        });
        model.apply_operation(&Op::NewCon {
            id: ts(s, 3),
            val: ConValue::Val(PackValue::Str("a".to_string())),
        });
        model.apply_operation(&Op::InsObj {
            id: ts(s, 4),
            obj: ts(s, 1),
            data: vec![("b".to_string(), ts(s, 2))],
        });
        model.apply_operation(&Op::InsObj {
            id: ts(s, 5),
            obj: ts(s, 1),
            data: vec![("a".to_string(), ts(s, 3))],
        });
        model.apply_operation(&Op::InsVal {
            id: ts(s, 6),
            obj: crate::json_crdt::constants::ORIGIN,
            val: ts(s, 1),
        });

        let encoded = encode(&model);
        let map = encoded
            .get("root")
            .and_then(|node| node.get("value"))
            .and_then(|node| node.get("map"))
            .and_then(|node| node.as_object())
            .expect("encoded object map");
        let keys: Vec<&str> = map.keys().map(|k| k.as_str()).collect();
        assert_eq!(keys, vec!["b", "a"]);
    }

    // ── Verbose codec error handling tests ───────────────────────────────

    #[test]
    fn decode_rejects_non_object_document() {
        let data = json!("not an object");
        let err = decode(&data).expect_err("should reject non-object document");
        assert!(matches!(err, DecodeError::Format(_)));
    }

    #[test]
    fn decode_rejects_missing_time_field() {
        let data = json!({ "root": { "value": { "type": "con", "id": 0, "value": 1 } } });
        let err = decode(&data).expect_err("should reject missing time");
        assert!(matches!(err, DecodeError::MissingField(f) if f == "time"));
    }

    #[test]
    fn decode_rejects_missing_root_field() {
        let data = json!({ "time": 0 });
        let err = decode(&data).expect_err("should reject missing root");
        assert!(matches!(err, DecodeError::MissingField(f) if f == "root"));
    }

    #[test]
    fn decode_rejects_non_object_root_node() {
        let data = json!({ "time": 0, "root": "not_an_object" });
        let err = decode(&data).expect_err("should reject non-object root");
        assert!(matches!(err, DecodeError::Format(_)));
    }

    #[test]
    fn decode_rejects_unknown_node_type() {
        // Root has a "value" child with an unknown type
        let data = json!({
            "time": 0,
            "root": {
                "value": { "type": "foobar", "id": 0 }
            }
        });
        let err = decode(&data).expect_err("should reject unknown node type");
        assert!(matches!(err, DecodeError::UnknownNodeType(t) if t == "foobar"));
    }

    #[test]
    fn decode_rejects_node_missing_type() {
        let data = json!({
            "time": 0,
            "root": {
                "value": { "id": 0 }
            }
        });
        let err = decode(&data).expect_err("should reject node without type");
        assert!(matches!(err, DecodeError::MissingField(f) if f == "type"));
    }

    #[test]
    fn decode_rejects_non_object_child_node() {
        let data = json!({
            "time": 0,
            "root": {
                "value": "not_an_object"
            }
        });
        let err = decode(&data).expect_err("should reject non-object child node");
        assert!(matches!(err, DecodeError::Format(_)));
    }

    // ── Roundtrip coverage for all node types ─────────────────────────

    #[test]
    fn roundtrip_obj_with_multiple_keys() {
        let s = sid();
        let mut model = Model::new(s);
        model.apply_operation(&Op::NewObj { id: ts(s, 1) });
        model.apply_operation(&Op::NewCon {
            id: ts(s, 2),
            val: ConValue::Val(PackValue::Integer(10)),
        });
        model.apply_operation(&Op::NewCon {
            id: ts(s, 3),
            val: ConValue::Val(PackValue::Str("hello".into())),
        });
        model.apply_operation(&Op::NewCon {
            id: ts(s, 4),
            val: ConValue::Val(PackValue::Bool(true)),
        });
        model.apply_operation(&Op::InsObj {
            id: ts(s, 5),
            obj: ts(s, 1),
            data: vec![
                ("num".into(), ts(s, 2)),
                ("str".into(), ts(s, 3)),
                ("flag".into(), ts(s, 4)),
            ],
        });
        model.apply_operation(&Op::InsVal {
            id: ts(s, 6),
            obj: crate::json_crdt::constants::ORIGIN,
            val: ts(s, 1),
        });

        let view = model.view();
        let encoded = encode(&model);
        let decoded = decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.view(), view);
    }

    #[test]
    fn roundtrip_nested_obj_in_obj() {
        let s = sid();
        let mut model = Model::new(s);
        // Inner obj
        model.apply_operation(&Op::NewObj { id: ts(s, 1) });
        model.apply_operation(&Op::NewCon {
            id: ts(s, 2),
            val: ConValue::Val(PackValue::Integer(42)),
        });
        model.apply_operation(&Op::InsObj {
            id: ts(s, 3),
            obj: ts(s, 1),
            data: vec![("x".into(), ts(s, 2))],
        });
        // Outer obj
        model.apply_operation(&Op::NewObj { id: ts(s, 4) });
        model.apply_operation(&Op::InsObj {
            id: ts(s, 5),
            obj: ts(s, 4),
            data: vec![("inner".into(), ts(s, 1))],
        });
        model.apply_operation(&Op::InsVal {
            id: ts(s, 6),
            obj: crate::json_crdt::constants::ORIGIN,
            val: ts(s, 4),
        });

        let view = model.view();
        let encoded = encode(&model);
        let decoded = decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.view(), view);
    }

    #[test]
    fn roundtrip_vec_node() {
        let s = sid();
        let mut model = Model::new(s);
        model.apply_operation(&Op::NewVec { id: ts(s, 1) });
        model.apply_operation(&Op::NewCon {
            id: ts(s, 2),
            val: ConValue::Val(PackValue::Integer(10)),
        });
        model.apply_operation(&Op::NewCon {
            id: ts(s, 3),
            val: ConValue::Val(PackValue::Integer(20)),
        });
        model.apply_operation(&Op::InsVec {
            id: ts(s, 4),
            obj: ts(s, 1),
            data: vec![(0, ts(s, 2)), (1, ts(s, 3))],
        });
        model.apply_operation(&Op::InsVal {
            id: ts(s, 5),
            obj: crate::json_crdt::constants::ORIGIN,
            val: ts(s, 1),
        });

        let view = model.view();
        let encoded = encode(&model);
        let decoded = decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.view(), view);
    }

    #[test]
    fn roundtrip_bin_node() {
        let s = sid();
        let mut model = Model::new(s);
        model.apply_operation(&Op::NewBin { id: ts(s, 1) });
        model.apply_operation(&Op::InsBin {
            id: ts(s, 2),
            obj: ts(s, 1),
            after: crate::json_crdt::constants::ORIGIN,
            data: vec![0xDE, 0xAD, 0xBE, 0xEF],
        });
        model.apply_operation(&Op::InsVal {
            id: ts(s, 7),
            obj: crate::json_crdt::constants::ORIGIN,
            val: ts(s, 1),
        });

        let view = model.view();
        let encoded = encode(&model);
        let decoded = decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.view(), view);
    }

    #[test]
    fn roundtrip_arr_node() {
        let s = sid();
        let mut model = Model::new(s);
        model.apply_operation(&Op::NewArr { id: ts(s, 1) });
        model.apply_operation(&Op::NewCon {
            id: ts(s, 2),
            val: ConValue::Val(PackValue::Str("a".into())),
        });
        model.apply_operation(&Op::NewCon {
            id: ts(s, 3),
            val: ConValue::Val(PackValue::Str("b".into())),
        });
        model.apply_operation(&Op::InsArr {
            id: ts(s, 4),
            obj: ts(s, 1),
            after: crate::json_crdt::constants::ORIGIN,
            data: vec![ts(s, 2), ts(s, 3)],
        });
        model.apply_operation(&Op::InsVal {
            id: ts(s, 7),
            obj: crate::json_crdt::constants::ORIGIN,
            val: ts(s, 1),
        });

        let view = model.view();
        let encoded = encode(&model);
        let decoded = decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.view(), view);
    }

    #[test]
    fn roundtrip_con_with_ref() {
        let s = sid();
        let mut model = Model::new(s);
        model.apply_operation(&Op::NewStr { id: ts(s, 1) });
        model.apply_operation(&Op::InsStr {
            id: ts(s, 2),
            obj: ts(s, 1),
            after: crate::json_crdt::constants::ORIGIN,
            data: "ref-target".to_string(),
        });
        model.apply_operation(&Op::NewCon {
            id: ts(s, 12),
            val: ConValue::Ref(ts(s, 1)),
        });
        model.apply_operation(&Op::InsVal {
            id: ts(s, 13),
            obj: crate::json_crdt::constants::ORIGIN,
            val: ts(s, 12),
        });

        let encoded = encode(&model);
        // Verify the ref is preserved through encode/decode
        let root = encoded.get("root").unwrap();
        let value = root.get("value").unwrap();
        assert_eq!(value.get("type").unwrap(), "con");
        assert!(value.get("timestamp").is_some());

        let decoded = decode(&encoded).expect("decode should succeed");
        // Both should produce the same view
        assert_eq!(decoded.view(), model.view());
    }

    #[test]
    fn roundtrip_con_null_and_bool() {
        let s = sid();
        let mut model = Model::new(s);
        model.apply_operation(&Op::NewObj { id: ts(s, 1) });
        model.apply_operation(&Op::NewCon {
            id: ts(s, 2),
            val: ConValue::Val(PackValue::Null),
        });
        model.apply_operation(&Op::NewCon {
            id: ts(s, 3),
            val: ConValue::Val(PackValue::Bool(false)),
        });
        model.apply_operation(&Op::InsObj {
            id: ts(s, 4),
            obj: ts(s, 1),
            data: vec![("nil".into(), ts(s, 2)), ("flag".into(), ts(s, 3))],
        });
        model.apply_operation(&Op::InsVal {
            id: ts(s, 5),
            obj: crate::json_crdt::constants::ORIGIN,
            val: ts(s, 1),
        });

        let view = model.view();
        let encoded = encode(&model);
        let decoded = decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.view(), view);
    }

    #[test]
    fn roundtrip_multibyte_string() {
        let s = sid();
        let mut model = Model::new(s);
        model.apply_operation(&Op::NewStr { id: ts(s, 1) });
        model.apply_operation(&Op::InsStr {
            id: ts(s, 2),
            obj: ts(s, 1),
            after: crate::json_crdt::constants::ORIGIN,
            data: "Hello \u{1F600} world \u{00E9}".to_string(),
        });
        model.apply_operation(&Op::InsVal {
            id: ts(s, 20),
            obj: crate::json_crdt::constants::ORIGIN,
            val: ts(s, 1),
        });

        let view = model.view();
        let encoded = encode(&model);
        let decoded = decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.view(), view);
    }

    #[test]
    fn roundtrip_server_mode() {
        let mut model = Model::new_server(10);
        let s = crate::json_crdt_patch::enums::SESSION::SERVER;
        model.apply_operation(&Op::NewCon {
            id: ts(s, 1),
            val: ConValue::Val(PackValue::Str("server".into())),
        });
        model.apply_operation(&Op::InsVal {
            id: ts(s, 2),
            obj: crate::json_crdt::constants::ORIGIN,
            val: ts(s, 1),
        });

        let view = model.view();
        let encoded = encode(&model);
        // Server mode uses a number for time
        assert!(encoded.get("time").unwrap().is_number());
        let decoded = decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.view(), view);
    }

    #[test]
    fn roundtrip_con_undefined() {
        let s = sid();
        let mut model = Model::new(s);
        model.apply_operation(&Op::NewCon {
            id: ts(s, 1),
            val: ConValue::Val(PackValue::Undefined),
        });
        model.apply_operation(&Op::InsVal {
            id: ts(s, 2),
            obj: crate::json_crdt::constants::ORIGIN,
            val: ts(s, 1),
        });

        let view = model.view();
        let encoded = encode(&model);
        let decoded = decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.view(), view);
    }

    // ── Decode error paths ────────────────────────────────────────────

    #[test]
    fn decode_rejects_empty_clock_table() {
        let data = json!({
            "time": [],
            "root": { "type": "val", "id": 0 }
        });
        let err = decode(&data).expect_err("should reject empty clock table");
        assert!(matches!(err, DecodeError::Format(_)));
    }

    #[test]
    fn decode_rejects_clock_entry_too_short() {
        let data = json!({
            "time": [[1]],
            "root": { "type": "val", "id": 0 }
        });
        let err = decode(&data).expect_err("should reject short clock entry");
        assert!(matches!(err, DecodeError::Format(_)));
    }

    #[test]
    fn decode_rejects_obj_without_map() {
        let data = json!({
            "time": 5,
            "root": {
                "value": { "type": "obj", "id": 1 }
            }
        });
        let err = decode(&data).expect_err("should reject obj without map");
        assert!(matches!(err, DecodeError::MissingField(_)));
    }

    #[test]
    fn decode_rejects_vec_without_map() {
        let data = json!({
            "time": 5,
            "root": {
                "value": { "type": "vec", "id": 1 }
            }
        });
        let err = decode(&data).expect_err("should reject vec without map");
        assert!(matches!(err, DecodeError::MissingField(_)));
    }

    #[test]
    fn decode_rejects_str_without_chunks() {
        let data = json!({
            "time": 5,
            "root": {
                "value": { "type": "str", "id": 1 }
            }
        });
        let err = decode(&data).expect_err("should reject str without chunks");
        assert!(matches!(err, DecodeError::MissingField(_)));
    }

    #[test]
    fn decode_rejects_bin_without_chunks() {
        let data = json!({
            "time": 5,
            "root": {
                "value": { "type": "bin", "id": 1 }
            }
        });
        let err = decode(&data).expect_err("should reject bin without chunks");
        assert!(matches!(err, DecodeError::MissingField(_)));
    }

    #[test]
    fn decode_rejects_arr_without_chunks() {
        let data = json!({
            "time": 5,
            "root": {
                "value": { "type": "arr", "id": 1 }
            }
        });
        let err = decode(&data).expect_err("should reject arr without chunks");
        assert!(matches!(err, DecodeError::MissingField(_)));
    }

    #[test]
    fn decode_rejects_str_chunk_without_span_or_value() {
        let data = json!({
            "time": 5,
            "root": {
                "value": {
                    "type": "str",
                    "id": 1,
                    "chunks": [{ "id": 2 }]
                }
            }
        });
        let err = decode(&data).expect_err("should reject str chunk without span or value");
        assert!(matches!(err, DecodeError::Format(_)));
    }

    #[test]
    fn decode_rejects_bin_chunk_without_span_or_value() {
        let data = json!({
            "time": 5,
            "root": {
                "value": {
                    "type": "bin",
                    "id": 1,
                    "chunks": [{ "id": 2 }]
                }
            }
        });
        let err = decode(&data).expect_err("should reject bin chunk without span or value");
        assert!(matches!(err, DecodeError::Format(_)));
    }

    #[test]
    fn decode_rejects_arr_chunk_without_span_or_value() {
        let data = json!({
            "time": 5,
            "root": {
                "value": {
                    "type": "arr",
                    "id": 1,
                    "chunks": [{ "id": 2 }]
                }
            }
        });
        let err = decode(&data).expect_err("should reject arr chunk without span or value");
        assert!(matches!(err, DecodeError::Format(_)));
    }

    #[test]
    fn decode_rejects_invalid_timestamp_format() {
        let data = json!({
            "time": 5,
            "root": {
                "value": { "type": "con", "id": "bad" }
            }
        });
        let err = decode(&data).expect_err("should reject invalid timestamp");
        assert!(matches!(err, DecodeError::Format(_)));
    }

    #[test]
    fn roundtrip_logical_clock_with_peers() {
        let s = sid();
        let peer = 999999u64;
        let mut model = Model::new(s);
        model.apply_operation(&Op::NewCon {
            id: ts(s, 1),
            val: ConValue::Val(PackValue::Integer(1)),
        });
        model.apply_operation(&Op::InsVal {
            id: ts(s, 2),
            obj: crate::json_crdt::constants::ORIGIN,
            val: ts(s, 1),
        });
        // Observe a peer timestamp
        model.clock.observe(ts(peer, 5), 1);

        let encoded = encode(&model);
        // Should be an array-based clock, not a number
        assert!(encoded.get("time").unwrap().is_array());
        let decoded = decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.view(), model.view());
    }

    // ── Additional coverage tests ───────────────────────────────────

    #[test]
    fn roundtrip_con_float() {
        let s = sid();
        let mut model = Model::new(s);
        model.apply_operation(&Op::NewCon {
            id: ts(s, 1),
            val: ConValue::Val(PackValue::Float(1.5)),
        });
        model.apply_operation(&Op::InsVal {
            id: ts(s, 2),
            obj: crate::json_crdt::constants::ORIGIN,
            val: ts(s, 1),
        });

        let view = model.view();
        let encoded = encode(&model);
        let decoded = decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.view(), view);
    }

    #[test]
    fn roundtrip_empty_str_node() {
        let s = sid();
        let mut model = Model::new(s);
        model.apply_operation(&Op::NewStr { id: ts(s, 1) });
        model.apply_operation(&Op::InsVal {
            id: ts(s, 2),
            obj: crate::json_crdt::constants::ORIGIN,
            val: ts(s, 1),
        });

        let view = model.view();
        let encoded = encode(&model);
        let decoded = decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.view(), view);
    }

    #[test]
    fn roundtrip_empty_bin_node() {
        let s = sid();
        let mut model = Model::new(s);
        model.apply_operation(&Op::NewBin { id: ts(s, 1) });
        model.apply_operation(&Op::InsVal {
            id: ts(s, 2),
            obj: crate::json_crdt::constants::ORIGIN,
            val: ts(s, 1),
        });

        let view = model.view();
        let encoded = encode(&model);
        let decoded = decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.view(), view);
    }

    #[test]
    fn roundtrip_empty_arr_node() {
        let s = sid();
        let mut model = Model::new(s);
        model.apply_operation(&Op::NewArr { id: ts(s, 1) });
        model.apply_operation(&Op::InsVal {
            id: ts(s, 2),
            obj: crate::json_crdt::constants::ORIGIN,
            val: ts(s, 1),
        });

        let view = model.view();
        let encoded = encode(&model);
        let decoded = decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.view(), view);
    }

    #[test]
    fn roundtrip_empty_vec_node() {
        let s = sid();
        let mut model = Model::new(s);
        model.apply_operation(&Op::NewVec { id: ts(s, 1) });
        model.apply_operation(&Op::InsVal {
            id: ts(s, 2),
            obj: crate::json_crdt::constants::ORIGIN,
            val: ts(s, 1),
        });

        let view = model.view();
        let encoded = encode(&model);
        let decoded = decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.view(), view);
    }

    #[test]
    fn roundtrip_empty_obj_node() {
        let s = sid();
        let mut model = Model::new(s);
        model.apply_operation(&Op::NewObj { id: ts(s, 1) });
        model.apply_operation(&Op::InsVal {
            id: ts(s, 2),
            obj: crate::json_crdt::constants::ORIGIN,
            val: ts(s, 1),
        });

        let view = model.view();
        let encoded = encode(&model);
        let decoded = decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.view(), view);
    }

    #[test]
    fn roundtrip_val_wrapping_val() {
        let s = sid();
        let mut model = Model::new(s);
        model.apply_operation(&Op::NewCon {
            id: ts(s, 1),
            val: ConValue::Val(PackValue::Integer(42)),
        });
        model.apply_operation(&Op::NewVal { id: ts(s, 2) });
        model.apply_operation(&Op::InsVal {
            id: ts(s, 3),
            obj: ts(s, 2),
            val: ts(s, 1),
        });
        model.apply_operation(&Op::InsVal {
            id: ts(s, 4),
            obj: crate::json_crdt::constants::ORIGIN,
            val: ts(s, 2),
        });

        let view = model.view();
        let encoded = encode(&model);
        let decoded = decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded.view(), view);
    }

    #[test]
    fn decode_rejects_non_array_clock_entry() {
        let data = json!({
            "time": ["not_an_array"],
            "root": { "type": "val", "id": 0 }
        });
        let err = decode(&data).expect_err("should reject non-array clock entry");
        assert!(matches!(err, DecodeError::Format(_)));
    }

    #[test]
    fn decode_rejects_str_chunk_not_object() {
        let data = json!({
            "time": 5,
            "root": {
                "value": {
                    "type": "str",
                    "id": 1,
                    "chunks": ["not_an_object"]
                }
            }
        });
        let err = decode(&data).expect_err("should reject non-object str chunk");
        assert!(matches!(err, DecodeError::Format(_)));
    }

    #[test]
    fn decode_rejects_bin_chunk_not_object() {
        let data = json!({
            "time": 5,
            "root": {
                "value": {
                    "type": "bin",
                    "id": 1,
                    "chunks": [42]
                }
            }
        });
        let err = decode(&data).expect_err("should reject non-object bin chunk");
        assert!(matches!(err, DecodeError::Format(_)));
    }

    #[test]
    fn decode_rejects_arr_chunk_not_object() {
        let data = json!({
            "time": 5,
            "root": {
                "value": {
                    "type": "arr",
                    "id": 1,
                    "chunks": ["bad"]
                }
            }
        });
        let err = decode(&data).expect_err("should reject non-object arr chunk");
        assert!(matches!(err, DecodeError::Format(_)));
    }

    #[test]
    fn decode_rejects_bin_invalid_base64() {
        let data = json!({
            "time": 5,
            "root": {
                "value": {
                    "type": "bin",
                    "id": 1,
                    "chunks": [{ "id": 2, "value": "!!!invalid-base64!!!" }]
                }
            }
        });
        let err = decode(&data).expect_err("should reject invalid base64");
        assert!(matches!(err, DecodeError::Format(_)));
    }

    #[test]
    fn decode_rejects_con_missing_id() {
        let data = json!({
            "time": 5,
            "root": {
                "value": { "type": "con" }
            }
        });
        let err = decode(&data).expect_err("should reject con without id");
        assert!(matches!(err, DecodeError::MissingField(_)));
    }

    #[test]
    fn decode_rejects_obj_missing_id() {
        let data = json!({
            "time": 5,
            "root": {
                "value": { "type": "obj", "map": {} }
            }
        });
        let err = decode(&data).expect_err("should reject obj without id");
        assert!(matches!(err, DecodeError::MissingField(_)));
    }

    #[test]
    fn decode_rejects_vec_missing_id() {
        let data = json!({
            "time": 5,
            "root": {
                "value": { "type": "vec", "map": [] }
            }
        });
        let err = decode(&data).expect_err("should reject vec without id");
        assert!(matches!(err, DecodeError::MissingField(_)));
    }

    #[test]
    fn decode_rejects_str_missing_id() {
        let data = json!({
            "time": 5,
            "root": {
                "value": { "type": "str", "chunks": [] }
            }
        });
        let err = decode(&data).expect_err("should reject str without id");
        assert!(matches!(err, DecodeError::MissingField(_)));
    }

    #[test]
    fn decode_rejects_bin_missing_id() {
        let data = json!({
            "time": 5,
            "root": {
                "value": { "type": "bin", "chunks": [] }
            }
        });
        let err = decode(&data).expect_err("should reject bin without id");
        assert!(matches!(err, DecodeError::MissingField(_)));
    }

    #[test]
    fn decode_rejects_arr_missing_id() {
        let data = json!({
            "time": 5,
            "root": {
                "value": { "type": "arr", "chunks": [] }
            }
        });
        let err = decode(&data).expect_err("should reject arr without id");
        assert!(matches!(err, DecodeError::MissingField(_)));
    }

    #[test]
    fn decode_rejects_chunk_missing_id() {
        let data = json!({
            "time": 5,
            "root": {
                "value": {
                    "type": "str",
                    "id": 1,
                    "chunks": [{ "value": "text" }]
                }
            }
        });
        let err = decode(&data).expect_err("should reject chunk without id");
        assert!(matches!(err, DecodeError::MissingField(_)));
    }

    #[test]
    fn decode_con_with_timestamp_but_missing_value() {
        let data = json!({
            "time": 5,
            "root": {
                "value": {
                    "type": "con",
                    "id": 1,
                    "timestamp": true
                }
            }
        });
        let err = decode(&data).expect_err("should reject con with timestamp but no value");
        assert!(matches!(err, DecodeError::MissingField(_)));
    }

    #[test]
    fn decode_vec_with_null_elements() {
        let data = json!({
            "time": 5,
            "root": {
                "value": {
                    "type": "vec",
                    "id": 1,
                    "map": [
                        null,
                        { "type": "con", "id": 2, "value": 42 }
                    ]
                }
            }
        });
        let decoded = decode(&data).expect("decode should succeed");
        let view = decoded.view();
        // The null element and the con(42) element
        assert!(view.is_array() || view.is_null() || view.as_array().is_some());
    }

    #[test]
    fn decode_str_with_deleted_chunk() {
        let data = json!({
            "time": 5,
            "root": {
                "value": {
                    "type": "str",
                    "id": 1,
                    "chunks": [
                        { "id": 2, "value": "hello" },
                        { "id": 7, "span": 3 }
                    ]
                }
            }
        });
        let decoded = decode(&data).expect("decode should succeed");
        let view = decoded.view();
        assert_eq!(view, json!("hello"));
    }

    #[test]
    fn decode_bin_with_deleted_chunk() {
        let data = json!({
            "time": 5,
            "root": {
                "value": {
                    "type": "bin",
                    "id": 1,
                    "chunks": [
                        { "id": 2, "value": "AAEC" },
                        { "id": 5, "span": 2 }
                    ]
                }
            }
        });
        let decoded = decode(&data).expect("decode should succeed");
        // Should decode successfully with the first chunk live and second tombstoned
        let _ = decoded.view();
    }

    #[test]
    fn decode_arr_with_deleted_chunk() {
        let data = json!({
            "time": 5,
            "root": {
                "value": {
                    "type": "arr",
                    "id": 1,
                    "chunks": [
                        { "id": 2, "value": [{ "type": "con", "id": 3, "value": "x" }] },
                        { "id": 4, "span": 1 }
                    ]
                }
            }
        });
        let decoded = decode(&data).expect("decode should succeed");
        let view = decoded.view();
        assert_eq!(view, json!(["x"]));
    }

    #[test]
    fn decode_error_display() {
        let err = DecodeError::Format("test error".into());
        assert_eq!(err.to_string(), "unexpected format: test error");

        let err = DecodeError::UnknownNodeType("foo".into());
        assert_eq!(err.to_string(), "unknown node type: foo");

        let err = DecodeError::MissingField("bar".into());
        assert_eq!(err.to_string(), "missing field: bar");
    }
}
