//! Compact-binary codec: compact JSON encoded as CBOR bytes.
//!
//! Mirrors `packages/json-joy/src/json-crdt-patch/codec/compact-binary/`.

use crate::json_crdt_patch::patch::Patch;

/// Encodes a patch to compact-binary (CBOR-encoded compact format).
pub fn encode(patch: &Patch) -> Vec<u8> {
    let compact = super::compact::encode(patch);
    // Encode the Vec<Value> as a CBOR array using json-pack's CBOR encoder
    let pack_val = json_array_to_pack(compact);
    let mut enc = json_joy_json_pack::cbor::CborEncoder::new();
    enc.encode(&pack_val)
}

/// Decodes a compact-binary blob into a patch.
pub fn decode(data: &[u8]) -> Patch {
    let pack_val = json_joy_json_pack::cbor::decode_cbor_value(data).expect("CBOR decode failed");
    let json_val = pack_to_json_value(pack_val);
    let arr = json_val.as_array().expect("expected array").clone();
    super::compact::decode(&arr)
}

fn json_array_to_pack(vals: Vec<serde_json::Value>) -> json_joy_json_pack::PackValue {
    json_joy_json_pack::PackValue::Array(vals.into_iter().map(json_val_to_pack).collect())
}

fn json_val_to_pack(v: serde_json::Value) -> json_joy_json_pack::PackValue {
    use json_joy_json_pack::PackValue;
    use serde_json::Value;
    match v {
        Value::Null => PackValue::Null,
        Value::Bool(b) => PackValue::Bool(b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                PackValue::Integer(i)
            } else if let Some(u) = n.as_u64() {
                PackValue::UInteger(u)
            } else {
                PackValue::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        Value::String(s) => PackValue::Str(s),
        Value::Array(arr) => PackValue::Array(arr.into_iter().map(json_val_to_pack).collect()),
        Value::Object(obj) => PackValue::Object(
            obj.into_iter()
                .map(|(k, v)| (k, json_val_to_pack(v)))
                .collect(),
        ),
    }
}

fn pack_to_json_value(v: json_joy_json_pack::PackValue) -> serde_json::Value {
    use json_joy_json_pack::PackValue;
    use serde_json::Value;
    match v {
        PackValue::Null | PackValue::Undefined | PackValue::Blob(_) => Value::Null,
        PackValue::Bool(b) => Value::Bool(b),
        PackValue::Integer(i) => serde_json::json!(i),
        PackValue::UInteger(u) => serde_json::json!(u),
        PackValue::Float(f) => serde_json::Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        PackValue::BigInt(i) => serde_json::json!(i),
        PackValue::Str(s) => Value::String(s),
        PackValue::Bytes(b) => Value::String(json_joy_base64::to_base64(&b)),
        PackValue::Array(arr) => Value::Array(arr.into_iter().map(pack_to_json_value).collect()),
        PackValue::Object(obj) => {
            let map: serde_json::Map<_, _> = obj
                .into_iter()
                .map(|(k, v)| (k, pack_to_json_value(v)))
                .collect();
            Value::Object(map)
        }
        PackValue::Extension(_) => Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json_crdt_patch::clock::tss;
    use crate::json_crdt_patch::operations::{ConValue, Op};
    use crate::json_crdt_patch::patch_builder::PatchBuilder;
    use json_joy_json_pack::PackValue;

    fn roundtrip(patch: &Patch) -> Patch {
        let bytes = encode(patch);
        decode(&bytes)
    }

    #[test]
    fn roundtrip_new_con_val() {
        let mut b = PatchBuilder::new(1, 0);
        b.con_val(PackValue::Integer(42));
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        assert_eq!(decoded.ops.len(), 1);
        assert!(matches!(
            &decoded.ops[0],
            Op::NewCon {
                val: ConValue::Val(PackValue::Integer(42)),
                ..
            }
        ));
    }

    #[test]
    fn roundtrip_creation_ops() {
        let mut b = PatchBuilder::new(1, 0);
        b.val();
        b.obj();
        b.vec();
        b.str_node();
        b.bin();
        b.arr();
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        assert_eq!(decoded.ops.len(), 6);
    }

    #[test]
    fn roundtrip_ins_str() {
        let mut b = PatchBuilder::new(1, 0);
        let s = b.str_node();
        b.ins_str(s, s, "hello".into());
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        if let Op::InsStr { data, .. } = &decoded.ops[1] {
            assert_eq!(data, "hello");
        } else {
            panic!("expected InsStr");
        }
    }

    #[test]
    fn roundtrip_ins_bin() {
        let mut b = PatchBuilder::new(1, 0);
        let bin = b.bin();
        b.ins_bin(bin, bin, vec![1, 2, 3]);
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        if let Op::InsBin { data, .. } = &decoded.ops[1] {
            assert_eq!(data, &[1, 2, 3]);
        } else {
            panic!("expected InsBin");
        }
    }

    #[test]
    fn roundtrip_del() {
        let mut b = PatchBuilder::new(1, 0);
        let s = b.str_node();
        b.ins_str(s, s, "abc".into());
        b.del(s, vec![tss(1, 1, 2)]);
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        if let Op::Del { what, .. } = &decoded.ops[2] {
            assert_eq!(what[0].span, 2);
        } else {
            panic!("expected Del");
        }
    }

    #[test]
    fn roundtrip_nop() {
        let mut b = PatchBuilder::new(1, 0);
        b.nop(3);
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        assert!(matches!(&decoded.ops[0], Op::Nop { len: 3, .. }));
    }

    #[test]
    fn roundtrip_multi_op() {
        let mut b = PatchBuilder::new(1, 0);
        let s = b.str_node();
        b.ins_str(s, s, "hi".into());
        b.nop(1);
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        assert_eq!(decoded.ops.len(), 3);
    }
}
