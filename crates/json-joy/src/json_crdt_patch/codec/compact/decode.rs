//! Compact JSON codec decoder.
//!
//! Mirrors `packages/json-joy/src/json-crdt-patch/codec/compact/decode.ts`.

use crate::json_crdt_patch::clock::{ts, tss, ClockVector, ServerClockVector, Ts};
use crate::json_crdt_patch::enums::{JsonCrdtPatchOpcode, SESSION};
use crate::json_crdt_patch::patch::Patch;
use crate::json_crdt_patch::patch_builder::PatchBuilder;
use json_joy_json_pack::PackValue;
use serde_json::Value;

fn decode_id(v: &Value, patch_sid: u64) -> Ts {
    match v {
        Value::Number(n) => ts(patch_sid, n.as_u64().unwrap_or(0)),
        Value::Array(arr) if arr.len() >= 2 => {
            let sid = arr[0].as_u64().unwrap_or(0);
            let time = arr[1].as_u64().unwrap_or(0);
            ts(sid, time)
        }
        _ => ts(patch_sid, 0),
    }
}

fn decode_tss(v: &Value, patch_sid: u64) -> Option<crate::json_crdt_patch::clock::Tss> {
    let a = v.as_array()?;
    match a.len() {
        3 => {
            let sid = a.first()?.as_u64()?;
            let time = a.get(1)?.as_u64()?;
            let span = a.get(2)?.as_u64()?;
            Some(tss(sid, time, span))
        }
        2 => {
            let time = a.first()?.as_u64()?;
            let span = a.get(1)?.as_u64()?;
            Some(tss(patch_sid, time, span))
        }
        _ => None,
    }
}

/// Decodes a compact-format array into a [`Patch`].
pub fn decode(data: &[Value]) -> Patch {
    if data.is_empty() {
        panic!("INVALID_PATCH");
    }

    // First element is the header: [id, meta?]
    let header = data[0].as_array().expect("INVALID_HEADER");
    let id_val = header.first().expect("MISSING_ID");

    let (patch_sid, patch_time) = match id_val {
        Value::Number(n) => (SESSION::SERVER, n.as_u64().unwrap_or(0)),
        Value::Array(arr) if arr.len() >= 2 => {
            (arr[0].as_u64().unwrap_or(0), arr[1].as_u64().unwrap_or(0))
        }
        _ => panic!("INVALID_ID"),
    };

    let mut builder = if patch_sid == SESSION::SERVER {
        PatchBuilder::from_server_clock(ServerClockVector::new(patch_time))
    } else {
        PatchBuilder::from_clock_vector(ClockVector::new(patch_sid, patch_time))
    };

    if let Some(meta_val) = header.get(1) {
        builder.patch.meta = Some(PackValue::from(meta_val));
    }

    // Remaining elements are operations
    for op_val in &data[1..] {
        let arr = match op_val.as_array() {
            Some(a) => a,
            None => continue,
        };
        let opcode_num = match arr.first().and_then(|v| v.as_u64()) {
            Some(n) => n as u8,
            None => continue,
        };

        match JsonCrdtPatchOpcode::from_u8(opcode_num) {
            Some(JsonCrdtPatchOpcode::NewCon) => {
                // [0] or [0, value] or [0, ts_ref, true]
                let is_ts_ref = arr.get(2).and_then(Value::as_bool).unwrap_or(false);
                if is_ts_ref {
                    let ref_id = decode_id(arr.get(1).unwrap_or(&Value::Null), patch_sid);
                    builder.con_ref(ref_id);
                } else {
                    let val = arr
                        .get(1)
                        .map(PackValue::from)
                        .unwrap_or(json_joy_json_pack::PackValue::Undefined);
                    builder.con_val(val);
                }
            }
            Some(JsonCrdtPatchOpcode::NewVal) => {
                builder.val();
            }
            Some(JsonCrdtPatchOpcode::NewObj) => {
                builder.obj();
            }
            Some(JsonCrdtPatchOpcode::NewVec) => {
                builder.vec();
            }
            Some(JsonCrdtPatchOpcode::NewStr) => {
                builder.str_node();
            }
            Some(JsonCrdtPatchOpcode::NewBin) => {
                builder.bin();
            }
            Some(JsonCrdtPatchOpcode::NewArr) => {
                builder.arr();
            }
            Some(JsonCrdtPatchOpcode::InsVal) => {
                let obj = decode_id(arr.get(1).unwrap_or(&Value::Null), patch_sid);
                let val = decode_id(arr.get(2).unwrap_or(&Value::Null), patch_sid);
                builder.set_val(obj, val);
            }
            Some(JsonCrdtPatchOpcode::InsObj) => {
                let obj = decode_id(arr.get(1).unwrap_or(&Value::Null), patch_sid);
                let mut tuples = Vec::new();
                if let Some(items) = arr.get(2).and_then(Value::as_array) {
                    for item in items {
                        if let Some(pair) = item.as_array() {
                            if pair.len() >= 2 {
                                if let Some(key) = pair[0].as_str() {
                                    let val_id = decode_id(&pair[1], patch_sid);
                                    tuples.push((key.to_owned(), val_id));
                                }
                            }
                        }
                    }
                }
                if !tuples.is_empty() {
                    builder.ins_obj(obj, tuples);
                }
            }
            Some(JsonCrdtPatchOpcode::InsVec) => {
                let obj = decode_id(arr.get(1).unwrap_or(&Value::Null), patch_sid);
                let mut tuples = Vec::new();
                if let Some(items) = arr.get(2).and_then(Value::as_array) {
                    for item in items {
                        if let Some(pair) = item.as_array() {
                            if pair.len() >= 2 {
                                if let Some(idx) = pair[0].as_u64() {
                                    let val_id = decode_id(&pair[1], patch_sid);
                                    tuples.push((idx as u8, val_id));
                                }
                            }
                        }
                    }
                }
                if !tuples.is_empty() {
                    builder.ins_vec(obj, tuples);
                }
            }
            Some(JsonCrdtPatchOpcode::InsStr) => {
                let obj = decode_id(arr.get(1).unwrap_or(&Value::Null), patch_sid);
                let after = decode_id(arr.get(2).unwrap_or(&Value::Null), patch_sid);
                let data = arr.get(3).and_then(|v| v.as_str()).unwrap_or("").to_owned();
                if !data.is_empty() {
                    builder.ins_str(obj, after, data);
                }
            }
            Some(JsonCrdtPatchOpcode::InsBin) => {
                let obj = decode_id(arr.get(1).unwrap_or(&Value::Null), patch_sid);
                let after = decode_id(arr.get(2).unwrap_or(&Value::Null), patch_sid);
                let b64 = arr.get(3).and_then(|v| v.as_str()).unwrap_or("");
                let data = json_joy_base64::from_base64(b64).unwrap_or_default();
                if !data.is_empty() {
                    builder.ins_bin(obj, after, data);
                }
            }
            Some(JsonCrdtPatchOpcode::InsArr) => {
                let obj = decode_id(arr.get(1).unwrap_or(&Value::Null), patch_sid);
                let after = decode_id(arr.get(2).unwrap_or(&Value::Null), patch_sid);
                let elems: Vec<Ts> = arr
                    .get(3)
                    .and_then(Value::as_array)
                    .map(|items| items.iter().map(|e| decode_id(e, patch_sid)).collect())
                    .unwrap_or_default();
                builder.ins_arr(obj, after, elems);
            }
            Some(JsonCrdtPatchOpcode::UpdArr) => {
                let obj = decode_id(arr.get(1).unwrap_or(&Value::Null), patch_sid);
                let after = decode_id(arr.get(2).unwrap_or(&Value::Null), patch_sid);
                let val = decode_id(arr.get(3).unwrap_or(&Value::Null), patch_sid);
                builder.upd_arr(obj, after, val);
            }
            Some(JsonCrdtPatchOpcode::Del) => {
                let obj = decode_id(arr.get(1).unwrap_or(&Value::Null), patch_sid);
                let what: Vec<crate::json_crdt_patch::clock::Tss> = arr
                    .get(2)
                    .and_then(Value::as_array)
                    .map(|spans| {
                        spans
                            .iter()
                            .filter_map(|span| decode_tss(span, patch_sid))
                            .collect()
                    })
                    .unwrap_or_default();
                builder.del(obj, what);
            }
            Some(JsonCrdtPatchOpcode::Nop) => {
                let len = arr.get(1).and_then(|v| v.as_u64()).unwrap_or(1);
                builder.nop(len);
            }
            _ => {}
        }
    }

    builder.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json_crdt_patch::codec::compact::encode;
    use crate::json_crdt_patch::operations::{ConValue, Op};
    use crate::json_crdt_patch::patch::Patch;
    use crate::json_crdt_patch::patch_builder::PatchBuilder;
    use json_joy_json_pack::PackValue;

    fn roundtrip(patch: &Patch) -> Patch {
        let encoded = encode::encode(patch);
        decode(&encoded)
    }

    #[test]
    fn roundtrip_new_con_val() {
        let mut b = PatchBuilder::new(1, 0);
        b.con_val(PackValue::Integer(99));
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        assert_eq!(decoded.ops.len(), 1);
        assert!(matches!(
            &decoded.ops[0],
            Op::NewCon {
                val: ConValue::Val(PackValue::Integer(99)),
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
    fn roundtrip_new_con_undefined() {
        let mut b = PatchBuilder::new(1, 0);
        b.con_val(PackValue::Undefined);
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        assert_eq!(decoded.ops.len(), 1);
    }

    #[test]
    fn roundtrip_all_creation_ops() {
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
        let v = b.val();
        let c = b.con_val(PackValue::Null);
        b.set_val(v, c);
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        assert!(matches!(&decoded.ops[2], Op::InsVal { .. }));
    }

    #[test]
    fn roundtrip_ins_obj() {
        let mut b = PatchBuilder::new(1, 0);
        let obj = b.obj();
        let c = b.con_val(PackValue::Bool(true));
        b.ins_obj(obj, vec![("k".into(), c)]);
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        if let Op::InsObj { data, .. } = &decoded.ops[2] {
            assert_eq!(data[0].0, "k");
        } else {
            panic!("expected InsObj");
        }
    }

    #[test]
    fn roundtrip_ins_vec() {
        let mut b = PatchBuilder::new(1, 0);
        let v = b.vec();
        let c = b.con_val(PackValue::Null);
        b.ins_vec(v, vec![(0, c)]);
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        assert!(matches!(&decoded.ops[2], Op::InsVec { .. }));
    }

    #[test]
    fn roundtrip_ins_str() {
        let mut b = PatchBuilder::new(1, 0);
        let s = b.str_node();
        b.ins_str(s, s, "test".into());
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        if let Op::InsStr { data, .. } = &decoded.ops[1] {
            assert_eq!(data, "test");
        } else {
            panic!("expected InsStr");
        }
    }

    #[test]
    fn roundtrip_ins_bin() {
        let mut b = PatchBuilder::new(1, 0);
        let bin = b.bin();
        b.ins_bin(bin, bin, vec![0xAB, 0xCD]);
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        if let Op::InsBin { data, .. } = &decoded.ops[1] {
            assert_eq!(data, &[0xAB, 0xCD]);
        } else {
            panic!("expected InsBin");
        }
    }

    #[test]
    fn roundtrip_ins_arr() {
        let mut b = PatchBuilder::new(1, 0);
        let arr = b.arr();
        let c = b.con_val(PackValue::Null);
        b.ins_arr(arr, arr, vec![c]);
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        assert!(matches!(&decoded.ops[2], Op::InsArr { .. }));
    }

    #[test]
    fn roundtrip_upd_arr() {
        let mut b = PatchBuilder::new(1, 0);
        let arr = b.arr();
        let c1 = b.con_val(PackValue::Integer(1));
        b.ins_arr(arr, arr, vec![c1]);
        let c2 = b.con_val(PackValue::Integer(2));
        b.upd_arr(arr, c1, c2);
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        assert!(matches!(&decoded.ops[4], Op::UpdArr { .. }));
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
    fn roundtrip_del_cross_session() {
        let mut b = PatchBuilder::new(1, 0);
        let s = b.str_node();
        b.ins_str(s, s, "abc".into());
        b.del(s, vec![tss(2, 100, 3)]); // different session
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        if let Op::Del { what, .. } = &decoded.ops[2] {
            assert_eq!(what[0].sid, 2);
            assert_eq!(what[0].time, 100);
            assert_eq!(what[0].span, 3);
        } else {
            panic!("expected Del");
        }
    }

    #[test]
    fn roundtrip_nop() {
        let mut b = PatchBuilder::new(1, 0);
        b.nop(7);
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        assert!(matches!(&decoded.ops[0], Op::Nop { len: 7, .. }));
    }

    #[test]
    fn roundtrip_with_meta() {
        let mut b = PatchBuilder::new(1, 0);
        b.con_val(PackValue::Null);
        let mut patch = b.flush();
        patch.meta = Some(PackValue::Integer(123));
        let decoded = roundtrip(&patch);
        assert!(decoded.meta.is_some());
    }

    #[test]
    fn roundtrip_server_session() {
        let mut b = PatchBuilder::from_server_clock(ServerClockVector::new(10));
        b.con_val(PackValue::Bool(false));
        let patch = b.flush();
        let decoded = roundtrip(&patch);
        assert_eq!(decoded.ops.len(), 1);
    }

    #[test]
    fn decode_skips_non_array_ops() {
        let data = vec![
            serde_json::json!([[1, 0]]),
            serde_json::json!("not_an_array"),
            serde_json::json!([1, 42]),
        ];
        let patch = decode(&data);
        // "not_an_array" should be skipped, [1, 42] is NewVal with value 42
        assert!(!patch.ops.is_empty());
    }

    #[test]
    fn decode_skips_unknown_opcode() {
        let data = vec![
            serde_json::json!([[1, 0]]),
            serde_json::json!([255]), // unknown opcode
        ];
        let patch = decode(&data);
        assert_eq!(patch.ops.len(), 0);
    }
}
