//! Native baseline port of `json-hash/*`.

use crate::model_runtime::types::{ConCell, Id, RuntimeNode};
use crate::model_runtime::RuntimeModel;
use crate::patch::Timestamp;
use crate::schema::SchemaNode;
use serde_json::{Map, Number, Value};
use std::collections::BTreeMap;

const START_STATE: i64 = 5381;
const CONST_NULL: i64 = 982_452_847;
const CONST_TRUE: i64 = 982_453_247;
const CONST_FALSE: i64 = 982_454_243;
const CONST_ARRAY: i64 = 982_452_259;
const CONST_STRING: i64 = 982_453_601;
const CONST_OBJECT: i64 = 982_454_533;
const CONST_BINARY: i64 = 982_454_837;

#[inline]
fn update_num(state: i64, num: i64) -> i64 {
    (state << 5) + state + num
}

fn update_str(mut state: i64, s: &str) -> i64 {
    let units: Vec<u16> = s.encode_utf16().collect();
    state = update_num(state, CONST_STRING);
    state = update_num(state, units.len() as i64);
    for u in units.iter().rev() {
        state = (state << 5) + state + (*u as i64);
    }
    state
}

fn update_bin(mut state: i64, bytes: &[u8]) -> i64 {
    state = update_num(state, CONST_BINARY);
    state = update_num(state, bytes.len() as i64);
    for b in bytes.iter().rev() {
        state = (state << 5) + state + (*b as i64);
    }
    state
}

fn update_json(mut state: i64, json: &Value) -> i64 {
    match json {
        Value::Null => update_num(state, CONST_NULL),
        Value::Bool(v) => update_num(state, if *v { CONST_TRUE } else { CONST_FALSE }),
        Value::Number(v) => {
            if let Some(i) = v.as_i64() {
                update_num(state, i)
            } else if let Some(u) = v.as_u64() {
                update_num(state, u as i64)
            } else if let Some(f) = v.as_f64() {
                update_num(state, f as i64)
            } else {
                state
            }
        }
        Value::String(s) => {
            state = update_num(state, CONST_STRING);
            update_str(state, s)
        }
        Value::Array(arr) => {
            state = update_num(state, CONST_ARRAY);
            for v in arr {
                state = update_json(state, v);
            }
            state
        }
        Value::Object(obj) => {
            state = update_num(state, CONST_OBJECT);
            let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
            keys.sort_unstable();
            for key in keys {
                state = update_str(state, key);
                state = update_json(state, &obj[key]);
            }
            state
        }
    }
}

pub fn hash_json(json: &Value) -> u32 {
    (update_json(START_STATE, json) as u64 & 0xffff_ffff) as u32
}

pub fn hash_str(s: &str) -> u32 {
    (update_str(START_STATE, s) as u64 & 0xffff_ffff) as u32
}

pub fn hash_bin(bytes: &[u8]) -> u32 {
    (update_bin(START_STATE, bytes) as u64 & 0xffff_ffff) as u32
}

fn to_base36_u64(mut n: u64) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let mut out = String::new();
    while n > 0 {
        let d = (n % 36) as u8;
        out.push(if d < 10 {
            (b'0' + d) as char
        } else {
            (b'a' + (d - 10)) as char
        });
        n /= 36;
    }
    out.chars().rev().collect()
}

fn to_base36_i64(n: i64) -> String {
    if n < 0 {
        format!("-{}", to_base36_u64((-n) as u64))
    } else {
        to_base36_u64(n as u64)
    }
}

fn number_to_base36(n: &Number) -> String {
    if let Some(i) = n.as_i64() {
        return to_base36_i64(i);
    }
    if let Some(u) = n.as_u64() {
        return to_base36_u64(u);
    }
    if let Some(f) = n.as_f64() {
        if f.fract() == 0.0 {
            return to_base36_i64(f as i64);
        }
        return f.to_string();
    }
    "0".to_string()
}

pub fn struct_hash_json(value: &Value) -> String {
    match value {
        Value::String(s) => to_base36_u64(hash_str(s) as u64),
        Value::Number(n) => number_to_base36(n),
        Value::Bool(v) => {
            if *v {
                "T".to_string()
            } else {
                "F".to_string()
            }
        }
        Value::Null => "N".to_string(),
        Value::Array(values) => {
            let mut out = String::from("[");
            for v in values {
                out.push_str(&struct_hash_json(v));
                out.push(';');
            }
            out.push(']');
            out
        }
        Value::Object(fields) => {
            let mut keys: Vec<&str> = fields.keys().map(String::as_str).collect();
            keys.sort_unstable();
            let mut out = String::from("{");
            for key in keys {
                out.push_str(&to_base36_u64(hash_str(key) as u64));
                out.push(':');
                out.push_str(&struct_hash_json(&fields[key]));
                out.push(',');
            }
            out.push('}');
            out
        }
    }
}

fn struct_hash_timestamp(ts: Timestamp) -> String {
    format!("{}.{}", to_base36_u64(ts.sid % 2_000_000), to_base36_u64(ts.time))
}

fn runtime_struct_hash(runtime: &RuntimeModel, id: Id) -> String {
    match runtime.nodes.get(&id) {
        Some(RuntimeNode::Con(ConCell::Json(v))) => struct_hash_json(v),
        Some(RuntimeNode::Con(ConCell::Ref(rid))) => struct_hash_timestamp((*rid).into()),
        Some(RuntimeNode::Con(ConCell::Undef)) => "U".to_string(),
        Some(RuntimeNode::Val(child)) => runtime_struct_hash(runtime, *child),
        Some(RuntimeNode::Str(atoms)) => {
            let s: String = atoms.iter().filter_map(|a| a.ch).collect();
            to_base36_u64(hash_str(&s) as u64)
        }
        Some(RuntimeNode::Bin(atoms)) => {
            let bytes: Vec<u8> = atoms.iter().filter_map(|a| a.byte).collect();
            to_base36_u64(hash_bin(&bytes) as u64)
        }
        Some(RuntimeNode::Arr(atoms)) => {
            let mut out = String::from("[");
            for a in atoms {
                if let Some(v) = a.value {
                    out.push_str(&runtime_struct_hash(runtime, v));
                    out.push(';');
                }
            }
            out.push(']');
            out
        }
        Some(RuntimeNode::Vec(map)) => {
            let mut out = String::from("[");
            for child in map.values() {
                out.push_str(&runtime_struct_hash(runtime, *child));
                out.push(';');
            }
            out.push(']');
            out
        }
        Some(RuntimeNode::Obj(entries)) => {
            let mut latest: BTreeMap<String, Id> = BTreeMap::new();
            for (k, v) in entries {
                match runtime.nodes.get(v) {
                    Some(RuntimeNode::Con(ConCell::Undef)) | None => {
                        latest.remove(k);
                    }
                    _ => {
                        latest.insert(k.clone(), *v);
                    }
                }
            }
            let mut out = String::from("{");
            for (k, v) in latest {
                out.push_str(&to_base36_u64(hash_str(&k) as u64));
                out.push(':');
                out.push_str(&runtime_struct_hash(runtime, v));
                out.push(',');
            }
            out.push('}');
            out
        }
        None => "U".to_string(),
    }
}

pub fn struct_hash_crdt(runtime: &RuntimeModel, node: Option<Timestamp>) -> String {
    node.map_or_else(|| "U".to_string(), |id| runtime_struct_hash(runtime, Id::from(id)))
}

pub fn struct_hash_schema(node: Option<&SchemaNode>) -> String {
    match node {
        None => "U".to_string(),
        Some(SchemaNode::Con(c)) => match c {
            crate::patch::ConValue::Json(v) => struct_hash_json(v),
            crate::patch::ConValue::Ref(ts) => struct_hash_timestamp(*ts),
            crate::patch::ConValue::Undef => "U".to_string(),
        },
        Some(SchemaNode::Str(raw)) => struct_hash_json(&Value::String(raw.clone())),
        Some(SchemaNode::Bin(raw)) => to_base36_u64(hash_bin(raw) as u64),
        Some(SchemaNode::Val(inner)) => struct_hash_schema(Some(inner)),
        Some(SchemaNode::Obj { req, opt }) => {
            let mut fields: BTreeMap<String, &SchemaNode> = BTreeMap::new();
            for (k, v) in req {
                fields.insert(k.clone(), v);
            }
            for (k, v) in opt {
                fields.insert(k.clone(), v);
            }
            let mut out = String::from("{");
            for (k, v) in fields {
                out.push_str(&to_base36_u64(hash_str(&k) as u64));
                out.push(':');
                out.push_str(&struct_hash_schema(Some(v)));
                out.push(',');
            }
            out.push('}');
            out
        }
        Some(SchemaNode::Arr(items)) => {
            let mut out = String::from("[");
            for v in items {
                out.push_str(&struct_hash_schema(Some(v)));
                out.push(';');
            }
            out.push(']');
            out
        }
        Some(SchemaNode::Vec(items)) => {
            let mut out = String::from("[");
            for v in items {
                out.push_str(&struct_hash_schema(v.as_ref()));
                out.push(';');
            }
            out.push(']');
            out
        }
        Some(SchemaNode::Ext { .. }) => "U".to_string(),
    }
}

pub fn struct_hash_map(map: &Map<String, Value>) -> String {
    struct_hash_json(&Value::Object(map.clone()))
}
