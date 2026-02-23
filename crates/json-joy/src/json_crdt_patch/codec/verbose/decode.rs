//! Verbose JSON codec decoder.
//!
//! Mirrors `packages/json-joy/src/json-crdt-patch/codec/verbose/decode.ts`.

use crate::json_crdt_patch::clock::{ts, tss, ClockVector, ServerClockVector, Ts};
use crate::json_crdt_patch::enums::SESSION;
use crate::json_crdt_patch::patch::Patch;
use crate::json_crdt_patch::patch_builder::PatchBuilder;
use json_joy_json_pack::PackValue;
use serde_json::Value;

fn decode_id(v: &Value) -> Ts {
    match v {
        Value::Number(n) => ts(SESSION::SERVER, n.as_u64().unwrap_or(0)),
        Value::Array(arr) if arr.len() >= 2 => {
            let sid = arr[0].as_u64().unwrap_or(0);
            let time = arr[1].as_u64().unwrap_or(0);
            ts(sid, time)
        }
        _ => ts(SESSION::SERVER, 0),
    }
}

/// Decodes a verbose-format JSON value into a [`Patch`].
pub fn decode(data: &Value) -> Patch {
    let obj = match data.as_object() {
        Some(o) => o,
        None => panic!("INVALID_PATCH"),
    };

    let id_val = obj.get("id").expect("missing id");
    let clock = match id_val {
        Value::Number(n) => {
            let time = n.as_u64().unwrap_or(0);
            let cv = ServerClockVector::new(time);
            PatchBuilder::from_server_clock(cv)
        }
        Value::Array(arr) if arr.len() >= 2 => {
            let sid = arr[0].as_u64().unwrap_or(0);
            let time = arr[1].as_u64().unwrap_or(0);
            let cv = ClockVector::new(sid, time);
            PatchBuilder::from_clock_vector(cv)
        }
        _ => panic!("INVALID_ID"),
    };
    let mut builder = clock;

    let ops = obj
        .get("ops")
        .and_then(|v| v.as_array())
        .map(|a| a.as_slice())
        .unwrap_or(&[]);
    for op_val in ops {
        let op_obj = match op_val.as_object() {
            Some(o) => o,
            None => continue,
        };
        let op_name = match op_obj.get("op").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => continue,
        };

        match op_name {
            "new_con" => {
                if op_obj
                    .get("timestamp")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    let ref_id = decode_id(op_obj.get("value").unwrap_or(&Value::Null));
                    builder.con_ref(ref_id);
                } else {
                    let val = PackValue::from(op_obj.get("value").unwrap_or(&Value::Null));
                    builder.con_val(val);
                }
            }
            "new_val" => {
                builder.val();
            }
            "new_obj" => {
                builder.obj();
            }
            "new_vec" => {
                builder.vec();
            }
            "new_str" => {
                builder.str_node();
            }
            "new_bin" => {
                builder.bin();
            }
            "new_arr" => {
                builder.arr();
            }
            "ins_val" => {
                let obj = decode_id(op_obj.get("obj").unwrap_or(&Value::Null));
                let val = decode_id(op_obj.get("value").unwrap_or(&Value::Null));
                builder.set_val(obj, val);
            }
            "ins_obj" => {
                let obj = decode_id(op_obj.get("obj").unwrap_or(&Value::Null));
                let value = op_obj
                    .get("value")
                    .and_then(|v| v.as_array())
                    .map(|a| a.as_slice())
                    .unwrap_or(&[]);
                let tuples: Vec<(String, Ts)> = value
                    .iter()
                    .filter_map(|pair| {
                        let arr = pair.as_array()?;
                        let key = arr.first()?.as_str()?.to_owned();
                        let id = decode_id(arr.get(1)?);
                        Some((key, id))
                    })
                    .collect();
                if !tuples.is_empty() {
                    builder.ins_obj(obj, tuples);
                }
            }
            "ins_vec" => {
                let obj = decode_id(op_obj.get("obj").unwrap_or(&Value::Null));
                let value = op_obj
                    .get("value")
                    .and_then(|v| v.as_array())
                    .map(|a| a.as_slice())
                    .unwrap_or(&[]);
                let tuples: Vec<(u8, Ts)> = value
                    .iter()
                    .filter_map(|pair| {
                        let arr = pair.as_array()?;
                        let idx = arr.first()?.as_u64()? as u8;
                        let id = decode_id(arr.get(1)?);
                        Some((idx, id))
                    })
                    .collect();
                if !tuples.is_empty() {
                    builder.ins_vec(obj, tuples);
                }
            }
            "ins_str" => {
                let obj = decode_id(op_obj.get("obj").unwrap_or(&Value::Null));
                let after = op_obj.get("after").map(decode_id).unwrap_or(obj);
                let data = op_obj
                    .get("value")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_owned();
                if !data.is_empty() {
                    builder.ins_str(obj, after, data);
                }
            }
            "ins_bin" => {
                let obj = decode_id(op_obj.get("obj").unwrap_or(&Value::Null));
                let after = op_obj.get("after").map(decode_id).unwrap_or(obj);
                let b64 = op_obj.get("value").and_then(|v| v.as_str()).unwrap_or("");
                let data = json_joy_base64::from_base64(b64).unwrap_or_default();
                if !data.is_empty() {
                    builder.ins_bin(obj, after, data);
                }
            }
            "ins_arr" => {
                let obj = decode_id(op_obj.get("obj").unwrap_or(&Value::Null));
                let after = op_obj.get("after").map(decode_id).unwrap_or(obj);
                let values = op_obj
                    .get("values")
                    .and_then(|v| v.as_array())
                    .map(|a| a.as_slice())
                    .unwrap_or(&[]);
                let elems: Vec<Ts> = values.iter().map(decode_id).collect();
                builder.ins_arr(obj, after, elems);
            }
            "upd_arr" => {
                let obj = decode_id(op_obj.get("obj").unwrap_or(&Value::Null));
                let after = decode_id(op_obj.get("ref").unwrap_or(&Value::Null));
                let val = decode_id(op_obj.get("value").unwrap_or(&Value::Null));
                builder.upd_arr(obj, after, val);
            }
            "del" => {
                let obj = decode_id(op_obj.get("obj").unwrap_or(&Value::Null));
                let what_arr = op_obj
                    .get("what")
                    .and_then(|v| v.as_array())
                    .map(|a| a.as_slice())
                    .unwrap_or(&[]);
                let what: Vec<crate::json_crdt_patch::clock::Tss> = what_arr
                    .iter()
                    .filter_map(|s| {
                        let arr = s.as_array()?;
                        let sid = arr.first()?.as_u64()?;
                        let time = arr.get(1)?.as_u64()?;
                        let span = arr.get(2)?.as_u64()?;
                        Some(tss(sid, time, span))
                    })
                    .collect();
                builder.del(obj, what);
            }
            "nop" => {
                let len = op_obj.get("len").and_then(|v| v.as_u64()).unwrap_or(1);
                builder.nop(len);
            }
            _ => {}
        }
    }

    let mut patch = builder.flush();
    if let Some(meta_val) = obj.get("meta") {
        patch.meta = Some(PackValue::from(meta_val));
    }
    patch
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json_crdt_patch::codec::verbose::encode;
    use crate::json_crdt_patch::operations::{ConValue, Op};
    use crate::json_crdt_patch::patch::Patch;
    use crate::json_crdt_patch::patch_builder::PatchBuilder;
    use json_joy_json_pack::PackValue;
    use serde_json::json;

    /// Helper: build a patch, encode to verbose JSON, decode back, assert ops match.
    fn roundtrip(patch: &Patch) -> Patch {
        let encoded = encode::encode(patch);
        decode(&encoded)
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
    fn roundtrip_new_con_ref() {
        let mut b = PatchBuilder::new(1, 0);
        b.con_ref(ts(2, 5));
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        assert!(matches!(
            &decoded.ops[0],
            Op::NewCon {
                val: ConValue::Ref(ref_id),
                ..
            } if ref_id.sid == 2 && ref_id.time == 5
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
        assert!(matches!(&decoded.ops[0], Op::NewVal { .. }));
        assert!(matches!(&decoded.ops[1], Op::NewObj { .. }));
        assert!(matches!(&decoded.ops[2], Op::NewVec { .. }));
        assert!(matches!(&decoded.ops[3], Op::NewStr { .. }));
        assert!(matches!(&decoded.ops[4], Op::NewBin { .. }));
        assert!(matches!(&decoded.ops[5], Op::NewArr { .. }));
    }

    #[test]
    fn roundtrip_ins_val() {
        let mut b = PatchBuilder::new(1, 0);
        let val_id = b.val();
        let con_id = b.con_val(PackValue::Null);
        b.set_val(val_id, con_id);
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        assert_eq!(decoded.ops.len(), 3);
        assert!(matches!(&decoded.ops[2], Op::InsVal { .. }));
    }

    #[test]
    fn roundtrip_ins_str() {
        let mut b = PatchBuilder::new(1, 0);
        let str_id = b.str_node();
        b.ins_str(str_id, str_id, "hello".into());
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
        let bin_id = b.bin();
        b.ins_bin(bin_id, bin_id, vec![0xDE, 0xAD]);
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        if let Op::InsBin { data, .. } = &decoded.ops[1] {
            assert_eq!(data, &[0xDE, 0xAD]);
        } else {
            panic!("expected InsBin");
        }
    }

    #[test]
    fn roundtrip_ins_obj() {
        let mut b = PatchBuilder::new(1, 0);
        let obj_id = b.obj();
        let con_id = b.con_val(PackValue::Bool(true));
        b.ins_obj(obj_id, vec![("key".into(), con_id)]);
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        if let Op::InsObj { data, .. } = &decoded.ops[2] {
            assert_eq!(data.len(), 1);
            assert_eq!(data[0].0, "key");
        } else {
            panic!("expected InsObj");
        }
    }

    #[test]
    fn roundtrip_ins_vec() {
        let mut b = PatchBuilder::new(1, 0);
        let vec_id = b.vec();
        let con_id = b.con_val(PackValue::Null);
        b.ins_vec(vec_id, vec![(0, con_id)]);
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        if let Op::InsVec { data, .. } = &decoded.ops[2] {
            assert_eq!(data.len(), 1);
            assert_eq!(data[0].0, 0);
        } else {
            panic!("expected InsVec");
        }
    }

    #[test]
    fn roundtrip_ins_arr() {
        let mut b = PatchBuilder::new(1, 0);
        let arr_id = b.arr();
        let con_id = b.con_val(PackValue::Null);
        b.ins_arr(arr_id, arr_id, vec![con_id]);
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        assert!(matches!(&decoded.ops[2], Op::InsArr { .. }));
    }

    #[test]
    fn roundtrip_upd_arr() {
        let mut b = PatchBuilder::new(1, 0);
        let arr_id = b.arr();
        let c1 = b.con_val(PackValue::Integer(1));
        b.ins_arr(arr_id, arr_id, vec![c1]);
        let c2 = b.con_val(PackValue::Integer(2));
        b.upd_arr(arr_id, c1, c2);
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        assert!(matches!(&decoded.ops[4], Op::UpdArr { .. }));
    }

    #[test]
    fn roundtrip_del() {
        let mut b = PatchBuilder::new(1, 0);
        let str_id = b.str_node();
        b.ins_str(str_id, str_id, "abc".into());
        b.del(str_id, vec![tss(1, 1, 2)]);
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        if let Op::Del { what, .. } = &decoded.ops[2] {
            assert_eq!(what.len(), 1);
            assert_eq!(what[0].span, 2);
        } else {
            panic!("expected Del");
        }
    }

    #[test]
    fn roundtrip_nop() {
        let mut b = PatchBuilder::new(1, 0);
        b.nop(5);
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        assert!(matches!(&decoded.ops[0], Op::Nop { len: 5, .. }));
    }

    #[test]
    fn roundtrip_nop_default_len() {
        let mut b = PatchBuilder::new(1, 0);
        b.nop(1);
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        assert!(matches!(&decoded.ops[0], Op::Nop { len: 1, .. }));
    }

    #[test]
    fn roundtrip_with_meta() {
        let mut b = PatchBuilder::new(1, 0);
        b.con_val(PackValue::Null);
        let mut patch = b.flush();
        patch.meta = Some(PackValue::Str("test-meta".into()));
        let decoded = roundtrip(&patch);
        assert!(decoded.meta.is_some());
    }

    #[test]
    fn decode_server_id() {
        // When id is a plain number, it's a server-session patch.
        let data = json!({
            "id": 10,
            "ops": [{"op": "new_val"}]
        });
        let patch = decode(&data);
        assert_eq!(patch.ops.len(), 1);
    }

    #[test]
    fn decode_skips_non_object_ops() {
        let data = json!({
            "id": [1, 0],
            "ops": [42, "not_an_op", {"op": "new_obj"}]
        });
        let patch = decode(&data);
        // Only the valid op should be decoded.
        assert_eq!(patch.ops.len(), 1);
    }

    #[test]
    fn decode_skips_unknown_op() {
        let data = json!({
            "id": [1, 0],
            "ops": [{"op": "unknown_op_type"}]
        });
        let patch = decode(&data);
        assert_eq!(patch.ops.len(), 0);
    }

    #[test]
    fn decode_empty_ops() {
        let data = json!({
            "id": [1, 0],
            "ops": []
        });
        let patch = decode(&data);
        assert_eq!(patch.ops.len(), 0);
    }

    #[test]
    fn decode_no_ops_key() {
        let data = json!({
            "id": [1, 0]
        });
        let patch = decode(&data);
        assert_eq!(patch.ops.len(), 0);
    }
}
