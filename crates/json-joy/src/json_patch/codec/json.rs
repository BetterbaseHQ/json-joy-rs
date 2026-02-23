//! JSON codec for JSON Patch operations.
//!
//! Converts operations to/from `serde_json::Value` in RFC 6902 + extensions format.
//!
//! Mirrors `packages/json-joy/src/json-patch/codec/json/`.

use serde_json::{json, Value};

use crate::json_patch::types::{JsonPatchType, Op, PatchError};

// ── Path helpers ──────────────────────────────────────────────────────────

fn encode_path(path: &[String]) -> Value {
    Value::String(if path.is_empty() {
        String::new()
    } else {
        format!(
            "/{}",
            path.iter()
                .map(|s| s.replace('~', "~0").replace('/', "~1"))
                .collect::<Vec<_>>()
                .join("/")
        )
    })
}

fn decode_path(v: &Value) -> Result<Vec<String>, PatchError> {
    let s = v
        .as_str()
        .ok_or_else(|| PatchError::InvalidOp("path must be a string".into()))?;
    Ok(json_joy_json_pointer::parse_json_pointer(s))
}

fn decode_type(v: &Value) -> Result<JsonPatchType, PatchError> {
    let s = v
        .as_str()
        .ok_or_else(|| PatchError::InvalidOp("type must be a string".into()))?;
    JsonPatchType::parse_str(s)
}

fn encode_type(t: &JsonPatchType) -> Value {
    Value::String(t.as_str().to_string())
}

fn decode_ops(arr: &Value) -> Result<Vec<Op>, PatchError> {
    let arr = arr
        .as_array()
        .ok_or_else(|| PatchError::InvalidOp("ops must be array".into()))?;
    arr.iter().map(from_json).collect()
}

// ── Serialization ─────────────────────────────────────────────────────────

/// Serialize an `Op` to a `serde_json::Value` in the JSON Patch format.
pub fn to_json(op: &Op) -> Value {
    match op {
        Op::Add { path, value } => json!({
            "op": "add",
            "path": encode_path(path),
            "value": value
        }),
        Op::Remove { path, old_value } => {
            let mut m = serde_json::Map::new();
            m.insert("op".into(), json!("remove"));
            m.insert("path".into(), encode_path(path));
            if let Some(ov) = old_value {
                m.insert("oldValue".into(), ov.clone());
            }
            Value::Object(m)
        }
        Op::Replace {
            path,
            value,
            old_value,
        } => {
            let mut m = serde_json::Map::new();
            m.insert("op".into(), json!("replace"));
            m.insert("path".into(), encode_path(path));
            m.insert("value".into(), value.clone());
            if let Some(ov) = old_value {
                m.insert("oldValue".into(), ov.clone());
            }
            Value::Object(m)
        }
        Op::Copy { path, from } => json!({
            "op": "copy",
            "path": encode_path(path),
            "from": encode_path(from)
        }),
        Op::Move { path, from } => json!({
            "op": "move",
            "path": encode_path(path),
            "from": encode_path(from)
        }),
        Op::Test { path, value, not } => {
            let mut m = serde_json::Map::new();
            m.insert("op".into(), json!("test"));
            m.insert("path".into(), encode_path(path));
            m.insert("value".into(), value.clone());
            if *not {
                m.insert("not".into(), json!(true));
            }
            Value::Object(m)
        }
        Op::StrIns { path, pos, str_val } => json!({
            "op": "str_ins",
            "path": encode_path(path),
            "pos": pos,
            "str": str_val
        }),
        Op::StrDel {
            path,
            pos,
            str_val,
            len,
        } => {
            let mut m = serde_json::Map::new();
            m.insert("op".into(), json!("str_del"));
            m.insert("path".into(), encode_path(path));
            m.insert("pos".into(), json!(pos));
            if let Some(s) = str_val {
                m.insert("str".into(), json!(s));
            }
            if let Some(l) = len {
                m.insert("len".into(), json!(l));
            }
            Value::Object(m)
        }
        Op::Flip { path } => json!({ "op": "flip", "path": encode_path(path) }),
        Op::Inc { path, inc } => json!({
            "op": "inc",
            "path": encode_path(path),
            "inc": inc
        }),
        Op::Split { path, pos, props } => {
            let mut m = serde_json::Map::new();
            m.insert("op".into(), json!("split"));
            m.insert("path".into(), encode_path(path));
            m.insert("pos".into(), json!(pos));
            if let Some(p) = props {
                m.insert("props".into(), p.clone());
            }
            Value::Object(m)
        }
        Op::Merge { path, pos, props } => {
            let mut m = serde_json::Map::new();
            m.insert("op".into(), json!("merge"));
            m.insert("path".into(), encode_path(path));
            m.insert("pos".into(), json!(pos));
            if let Some(p) = props {
                m.insert("props".into(), p.clone());
            }
            Value::Object(m)
        }
        Op::Extend {
            path,
            props,
            delete_null,
        } => {
            let mut m = serde_json::Map::new();
            m.insert("op".into(), json!("extend"));
            m.insert("path".into(), encode_path(path));
            m.insert("props".into(), Value::Object(props.clone()));
            if *delete_null {
                m.insert("deleteNull".into(), json!(true));
            }
            Value::Object(m)
        }
        Op::Defined { path } => json!({ "op": "defined",   "path": encode_path(path) }),
        Op::Undefined { path } => json!({ "op": "undefined", "path": encode_path(path) }),
        Op::Contains {
            path,
            value,
            ignore_case,
        } => {
            let mut m = serde_json::Map::new();
            m.insert("op".into(), json!("contains"));
            m.insert("path".into(), encode_path(path));
            m.insert("value".into(), json!(value));
            if *ignore_case {
                m.insert("ignore_case".into(), json!(true));
            }
            Value::Object(m)
        }
        Op::Ends {
            path,
            value,
            ignore_case,
        } => {
            let mut m = serde_json::Map::new();
            m.insert("op".into(), json!("ends"));
            m.insert("path".into(), encode_path(path));
            m.insert("value".into(), json!(value));
            if *ignore_case {
                m.insert("ignore_case".into(), json!(true));
            }
            Value::Object(m)
        }
        Op::Starts {
            path,
            value,
            ignore_case,
        } => {
            let mut m = serde_json::Map::new();
            m.insert("op".into(), json!("starts"));
            m.insert("path".into(), encode_path(path));
            m.insert("value".into(), json!(value));
            if *ignore_case {
                m.insert("ignore_case".into(), json!(true));
            }
            Value::Object(m)
        }
        Op::In { path, value } => json!({
            "op": "in",
            "path": encode_path(path),
            "value": value
        }),
        Op::Less { path, value } => json!({
            "op": "less",
            "path": encode_path(path),
            "value": value
        }),
        Op::More { path, value } => json!({
            "op": "more",
            "path": encode_path(path),
            "value": value
        }),
        Op::Matches {
            path,
            value,
            ignore_case,
        } => {
            let mut m = serde_json::Map::new();
            m.insert("op".into(), json!("matches"));
            m.insert("path".into(), encode_path(path));
            m.insert("value".into(), json!(value));
            if *ignore_case {
                m.insert("ignore_case".into(), json!(true));
            }
            Value::Object(m)
        }
        Op::TestType { path, type_vals } => json!({
            "op": "test_type",
            "path": encode_path(path),
            "type": type_vals.iter().map(encode_type).collect::<Vec<_>>()
        }),
        Op::TestString {
            path,
            pos,
            str_val,
            not,
        } => {
            let mut m = serde_json::Map::new();
            m.insert("op".into(), json!("test_string"));
            m.insert("path".into(), encode_path(path));
            m.insert("pos".into(), json!(pos));
            m.insert("str".into(), json!(str_val));
            if *not {
                m.insert("not".into(), json!(true));
            }
            Value::Object(m)
        }
        Op::TestStringLen { path, len, not } => {
            let mut m = serde_json::Map::new();
            m.insert("op".into(), json!("test_string_len"));
            m.insert("path".into(), encode_path(path));
            m.insert("len".into(), json!(len));
            if *not {
                m.insert("not".into(), json!(true));
            }
            Value::Object(m)
        }
        Op::Type { path, value } => json!({
            "op": "type",
            "path": encode_path(path),
            "value": encode_type(value)
        }),
        Op::And { path, ops } => json!({
            "op": "and",
            "path": encode_path(path),
            "apply": ops.iter().map(to_json).collect::<Vec<_>>()
        }),
        Op::Not { path, ops } => json!({
            "op": "not",
            "path": encode_path(path),
            "apply": ops.iter().map(to_json).collect::<Vec<_>>()
        }),
        Op::Or { path, ops } => json!({
            "op": "or",
            "path": encode_path(path),
            "apply": ops.iter().map(to_json).collect::<Vec<_>>()
        }),
    }
}

// ── Deserialization ───────────────────────────────────────────────────────

/// Deserialize a `serde_json::Value` into an `Op`.
pub fn from_json(v: &Value) -> Result<Op, PatchError> {
    let obj = v
        .as_object()
        .ok_or_else(|| PatchError::InvalidOp("operation must be an object".into()))?;
    let op_str = obj
        .get("op")
        .and_then(|v| v.as_str())
        .ok_or_else(|| PatchError::InvalidOp("missing 'op' field".into()))?;

    let path = decode_path(obj.get("path").unwrap_or(&Value::String(String::new())))?;

    match op_str {
        "add" => {
            let value = obj
                .get("value")
                .ok_or_else(|| PatchError::InvalidOp("add requires 'value'".into()))?
                .clone();
            Ok(Op::Add { path, value })
        }
        "remove" => {
            let old_value = obj.get("oldValue").cloned();
            Ok(Op::Remove { path, old_value })
        }
        "replace" => {
            let value = obj
                .get("value")
                .ok_or_else(|| PatchError::InvalidOp("replace requires 'value'".into()))?
                .clone();
            let old_value = obj.get("oldValue").cloned();
            Ok(Op::Replace {
                path,
                value,
                old_value,
            })
        }
        "copy" => {
            let from = decode_path(
                obj.get("from")
                    .ok_or_else(|| PatchError::InvalidOp("copy requires 'from'".into()))?,
            )?;
            Ok(Op::Copy { path, from })
        }
        "move" => {
            let from = decode_path(
                obj.get("from")
                    .ok_or_else(|| PatchError::InvalidOp("move requires 'from'".into()))?,
            )?;
            Ok(Op::Move { path, from })
        }
        "test" => {
            let value = obj
                .get("value")
                .ok_or_else(|| PatchError::InvalidOp("test requires 'value'".into()))?
                .clone();
            let not = obj.get("not").and_then(|v| v.as_bool()).unwrap_or(false);
            Ok(Op::Test { path, value, not })
        }
        "str_ins" => {
            let pos = obj
                .get("pos")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| PatchError::InvalidOp("str_ins requires 'pos'".into()))?
                as usize;
            let str_val = obj
                .get("str")
                .and_then(|v| v.as_str())
                .ok_or_else(|| PatchError::InvalidOp("str_ins requires 'str'".into()))?
                .to_string();
            Ok(Op::StrIns { path, pos, str_val })
        }
        "str_del" => {
            let pos = obj
                .get("pos")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| PatchError::InvalidOp("str_del requires 'pos'".into()))?
                as usize;
            let str_val = obj
                .get("str")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let len = obj.get("len").and_then(|v| v.as_u64()).map(|l| l as usize);
            Ok(Op::StrDel {
                path,
                pos,
                str_val,
                len,
            })
        }
        "flip" => Ok(Op::Flip { path }),
        "inc" => {
            let inc = obj
                .get("inc")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| PatchError::InvalidOp("inc requires 'inc'".into()))?;
            Ok(Op::Inc { path, inc })
        }
        "split" => {
            let pos = obj
                .get("pos")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| PatchError::InvalidOp("split requires 'pos'".into()))?
                as usize;
            let props = obj.get("props").cloned();
            Ok(Op::Split { path, pos, props })
        }
        "merge" => {
            let pos = obj
                .get("pos")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| PatchError::InvalidOp("merge requires 'pos'".into()))?
                as usize;
            let props = obj.get("props").cloned();
            Ok(Op::Merge { path, pos, props })
        }
        "extend" => {
            let props = obj
                .get("props")
                .and_then(|v| v.as_object())
                .ok_or_else(|| PatchError::InvalidOp("extend requires 'props'".into()))?
                .clone();
            let delete_null = obj
                .get("deleteNull")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(Op::Extend {
                path,
                props,
                delete_null,
            })
        }
        "defined" => Ok(Op::Defined { path }),
        "undefined" => Ok(Op::Undefined { path }),
        "contains" => {
            let value = obj
                .get("value")
                .and_then(|v| v.as_str())
                .ok_or_else(|| PatchError::InvalidOp("contains requires 'value'".into()))?
                .to_string();
            let ignore_case = obj
                .get("ignore_case")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(Op::Contains {
                path,
                value,
                ignore_case,
            })
        }
        "ends" => {
            let value = obj
                .get("value")
                .and_then(|v| v.as_str())
                .ok_or_else(|| PatchError::InvalidOp("ends requires 'value'".into()))?
                .to_string();
            let ignore_case = obj
                .get("ignore_case")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(Op::Ends {
                path,
                value,
                ignore_case,
            })
        }
        "starts" => {
            let value = obj
                .get("value")
                .and_then(|v| v.as_str())
                .ok_or_else(|| PatchError::InvalidOp("starts requires 'value'".into()))?
                .to_string();
            let ignore_case = obj
                .get("ignore_case")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(Op::Starts {
                path,
                value,
                ignore_case,
            })
        }
        "in" => {
            let value = obj
                .get("value")
                .and_then(|v| v.as_array())
                .ok_or_else(|| PatchError::InvalidOp("in requires 'value' array".into()))?
                .clone();
            Ok(Op::In { path, value })
        }
        "less" => {
            let value = obj
                .get("value")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| PatchError::InvalidOp("less requires 'value'".into()))?;
            Ok(Op::Less { path, value })
        }
        "more" => {
            let value = obj
                .get("value")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| PatchError::InvalidOp("more requires 'value'".into()))?;
            Ok(Op::More { path, value })
        }
        "matches" => {
            let value = obj
                .get("value")
                .and_then(|v| v.as_str())
                .ok_or_else(|| PatchError::InvalidOp("matches requires 'value'".into()))?
                .to_string();
            let ignore_case = obj
                .get("ignore_case")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(Op::Matches {
                path,
                value,
                ignore_case,
            })
        }
        "test_type" => {
            let types = obj
                .get("type")
                .and_then(|v| v.as_array())
                .ok_or_else(|| PatchError::InvalidOp("test_type requires 'type' array".into()))?;
            let type_vals: Result<Vec<_>, _> = types.iter().map(decode_type).collect();
            Ok(Op::TestType {
                path,
                type_vals: type_vals?,
            })
        }
        "test_string" => {
            let pos = obj
                .get("pos")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| PatchError::InvalidOp("test_string requires 'pos'".into()))?
                as usize;
            let str_val = obj
                .get("str")
                .and_then(|v| v.as_str())
                .ok_or_else(|| PatchError::InvalidOp("test_string requires 'str'".into()))?
                .to_string();
            let not = obj.get("not").and_then(|v| v.as_bool()).unwrap_or(false);
            Ok(Op::TestString {
                path,
                pos,
                str_val,
                not,
            })
        }
        "test_string_len" => {
            let len = obj
                .get("len")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| PatchError::InvalidOp("test_string_len requires 'len'".into()))?
                as usize;
            let not = obj.get("not").and_then(|v| v.as_bool()).unwrap_or(false);
            Ok(Op::TestStringLen { path, len, not })
        }
        "type" => {
            let value = decode_type(
                obj.get("value")
                    .ok_or_else(|| PatchError::InvalidOp("type requires 'value'".into()))?,
            )?;
            Ok(Op::Type { path, value })
        }
        "and" => {
            let ops = decode_ops(
                obj.get("apply")
                    .ok_or_else(|| PatchError::InvalidOp("and requires 'apply'".into()))?,
            )?;
            Ok(Op::And { path, ops })
        }
        "not" => {
            let ops = decode_ops(
                obj.get("apply")
                    .ok_or_else(|| PatchError::InvalidOp("not requires 'apply'".into()))?,
            )?;
            Ok(Op::Not { path, ops })
        }
        "or" => {
            let ops = decode_ops(
                obj.get("apply")
                    .ok_or_else(|| PatchError::InvalidOp("or requires 'apply'".into()))?,
            )?;
            Ok(Op::Or { path, ops })
        }
        other => Err(PatchError::InvalidOp(format!("unknown op: {other}"))),
    }
}

/// Serialize a list of operations to a JSON array.
pub fn to_json_patch(ops: &[Op]) -> Value {
    Value::Array(ops.iter().map(to_json).collect())
}

/// Deserialize a JSON array into a list of operations.
pub fn from_json_patch(v: &Value) -> Result<Vec<Op>, PatchError> {
    let arr = v
        .as_array()
        .ok_or_else(|| PatchError::InvalidOp("patch must be an array".into()))?;
    arr.iter().map(from_json).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn roundtrip(op: Op) -> Op {
        let v = to_json(&op);
        from_json(&v).expect("roundtrip failed")
    }

    #[test]
    fn roundtrip_add() {
        let op = Op::Add {
            path: vec!["a".to_string()],
            value: json!(42),
        };
        let rt = roundtrip(op);
        assert_eq!(rt.op_name(), "add");
    }

    #[test]
    fn roundtrip_remove() {
        let op = Op::Remove {
            path: vec!["a".to_string()],
            old_value: None,
        };
        let rt = roundtrip(op);
        assert_eq!(rt.op_name(), "remove");
    }

    #[test]
    fn roundtrip_replace() {
        let op = Op::Replace {
            path: vec!["x".to_string()],
            value: json!("new"),
            old_value: Some(json!("old")),
        };
        let v = to_json(&op);
        assert_eq!(v["op"], "replace");
        assert_eq!(v["oldValue"], "old");
    }

    #[test]
    fn roundtrip_test_type() {
        let op = Op::TestType {
            path: vec!["n".to_string()],
            type_vals: vec![JsonPatchType::Number, JsonPatchType::Integer],
        };
        let rt = roundtrip(op);
        assert_eq!(rt.op_name(), "test_type");
    }

    #[test]
    fn decode_rfc6902_patch() {
        let patch_json = json!([
            {"op": "add", "path": "/foo", "value": 1},
            {"op": "remove", "path": "/bar"},
            {"op": "replace", "path": "/baz", "value": "new"},
        ]);
        let ops = from_json_patch(&patch_json).unwrap();
        assert_eq!(ops.len(), 3);
        assert_eq!(ops[0].op_name(), "add");
        assert_eq!(ops[1].op_name(), "remove");
        assert_eq!(ops[2].op_name(), "replace");
    }

    #[test]
    fn roundtrip_and_predicate() {
        let op = Op::And {
            path: vec![],
            ops: vec![
                Op::Defined {
                    path: vec!["a".to_string()],
                },
                Op::Less {
                    path: vec!["a".to_string()],
                    value: 100.0,
                },
            ],
        };
        let rt = roundtrip(op);
        assert_eq!(rt.op_name(), "and");
    }

    // ── Path encoding/decoding ──────────────────────────────────────────

    #[test]
    fn encode_path_empty() {
        let v = to_json(&Op::Add {
            path: vec![],
            value: json!(1),
        });
        assert_eq!(v["path"], "");
    }

    #[test]
    fn encode_path_with_tilde_and_slash() {
        let op = Op::Add {
            path: vec!["a/b".to_string(), "c~d".to_string()],
            value: json!(1),
        };
        let v = to_json(&op);
        // '/' -> '~1', '~' -> '~0'
        assert_eq!(v["path"], "/a~1b/c~0d");
    }

    #[test]
    fn decode_path_not_a_string() {
        let v = json!({"op": "add", "path": 42, "value": 1});
        let err = from_json(&v).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    // ── Error paths for from_json ───────────────────────────────────────

    #[test]
    fn from_json_not_object() {
        let err = from_json(&json!("not an object")).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_missing_op_field() {
        let err = from_json(&json!({"path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_unknown_op() {
        let err = from_json(&json!({"op": "bogus"})).unwrap_err();
        match err {
            PatchError::InvalidOp(s) => assert!(s.contains("unknown op")),
            _ => panic!("expected InvalidOp"),
        }
    }

    #[test]
    fn from_json_add_missing_value() {
        let err = from_json(&json!({"op": "add", "path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_replace_missing_value() {
        let err = from_json(&json!({"op": "replace", "path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_test_missing_value() {
        let err = from_json(&json!({"op": "test", "path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_copy_missing_from() {
        let err = from_json(&json!({"op": "copy", "path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_move_missing_from() {
        let err = from_json(&json!({"op": "move", "path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_str_ins_missing_pos() {
        let err = from_json(&json!({"op": "str_ins", "path": "/a", "str": "x"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_str_ins_missing_str() {
        let err = from_json(&json!({"op": "str_ins", "path": "/a", "pos": 0})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_str_del_missing_pos() {
        let err = from_json(&json!({"op": "str_del", "path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_inc_missing_inc() {
        let err = from_json(&json!({"op": "inc", "path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_split_missing_pos() {
        let err = from_json(&json!({"op": "split", "path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_merge_missing_pos() {
        let err = from_json(&json!({"op": "merge", "path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_extend_missing_props() {
        let err = from_json(&json!({"op": "extend", "path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_contains_missing_value() {
        let err = from_json(&json!({"op": "contains", "path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_ends_missing_value() {
        let err = from_json(&json!({"op": "ends", "path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_starts_missing_value() {
        let err = from_json(&json!({"op": "starts", "path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_in_missing_value() {
        let err = from_json(&json!({"op": "in", "path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_less_missing_value() {
        let err = from_json(&json!({"op": "less", "path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_more_missing_value() {
        let err = from_json(&json!({"op": "more", "path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_matches_missing_value() {
        let err = from_json(&json!({"op": "matches", "path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_test_type_missing_type() {
        let err = from_json(&json!({"op": "test_type", "path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_test_string_missing_fields() {
        let err = from_json(&json!({"op": "test_string", "path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_test_string_len_missing_len() {
        let err = from_json(&json!({"op": "test_string_len", "path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_type_missing_value() {
        let err = from_json(&json!({"op": "type", "path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_and_missing_apply() {
        let err = from_json(&json!({"op": "and", "path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_not_missing_apply() {
        let err = from_json(&json!({"op": "not", "path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_or_missing_apply() {
        let err = from_json(&json!({"op": "or", "path": "/a"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    // ── Roundtrip for all op types ──────────────────────────────────────

    #[test]
    fn roundtrip_copy() {
        let op = Op::Copy {
            path: vec!["a".to_string()],
            from: vec!["b".to_string()],
        };
        let rt = roundtrip(op);
        assert_eq!(rt.op_name(), "copy");
    }

    #[test]
    fn roundtrip_move() {
        let op = Op::Move {
            path: vec!["a".to_string()],
            from: vec!["b".to_string()],
        };
        let rt = roundtrip(op);
        assert_eq!(rt.op_name(), "move");
    }

    #[test]
    fn roundtrip_test_not() {
        let op = Op::Test {
            path: vec!["a".to_string()],
            value: json!(42),
            not: true,
        };
        let v = to_json(&op);
        assert_eq!(v["not"], true);
        let rt = from_json(&v).unwrap();
        match rt {
            Op::Test { not, .. } => assert!(not),
            _ => panic!("expected Test"),
        }
    }

    #[test]
    fn roundtrip_test_no_not() {
        let op = Op::Test {
            path: vec!["a".to_string()],
            value: json!(42),
            not: false,
        };
        let v = to_json(&op);
        assert!(v.get("not").is_none());
    }

    #[test]
    fn roundtrip_str_ins() {
        let op = Op::StrIns {
            path: vec!["a".to_string()],
            pos: 5,
            str_val: "hello".to_string(),
        };
        let rt = roundtrip(op);
        assert_eq!(rt.op_name(), "str_ins");
    }

    #[test]
    fn roundtrip_str_del_with_str_and_len() {
        let op = Op::StrDel {
            path: vec!["a".to_string()],
            pos: 3,
            str_val: Some("abc".to_string()),
            len: Some(3),
        };
        let v = to_json(&op);
        assert_eq!(v["str"], "abc");
        assert_eq!(v["len"], 3);
        let rt = from_json(&v).unwrap();
        assert_eq!(rt.op_name(), "str_del");
    }

    #[test]
    fn roundtrip_str_del_no_optional() {
        let op = Op::StrDel {
            path: vec!["a".to_string()],
            pos: 0,
            str_val: None,
            len: None,
        };
        let v = to_json(&op);
        assert!(v.get("str").is_none());
        assert!(v.get("len").is_none());
    }

    #[test]
    fn roundtrip_flip() {
        let op = Op::Flip {
            path: vec!["x".to_string()],
        };
        let rt = roundtrip(op);
        assert_eq!(rt.op_name(), "flip");
    }

    #[test]
    fn roundtrip_inc() {
        let op = Op::Inc {
            path: vec!["c".to_string()],
            inc: 3.5,
        };
        let rt = roundtrip(op);
        assert_eq!(rt.op_name(), "inc");
    }

    #[test]
    fn roundtrip_split_with_props() {
        let op = Op::Split {
            path: vec!["a".to_string()],
            pos: 5,
            props: Some(json!({"bold": true})),
        };
        let v = to_json(&op);
        assert_eq!(v["props"]["bold"], true);
        let rt = from_json(&v).unwrap();
        assert_eq!(rt.op_name(), "split");
    }

    #[test]
    fn roundtrip_split_no_props() {
        let op = Op::Split {
            path: vec!["a".to_string()],
            pos: 5,
            props: None,
        };
        let v = to_json(&op);
        assert!(v.get("props").is_none());
    }

    #[test]
    fn roundtrip_merge_with_props() {
        let op = Op::Merge {
            path: vec!["a".to_string()],
            pos: 2,
            props: Some(json!({"style": "italic"})),
        };
        let v = to_json(&op);
        assert!(v.get("props").is_some());
        let rt = from_json(&v).unwrap();
        assert_eq!(rt.op_name(), "merge");
    }

    #[test]
    fn roundtrip_merge_no_props() {
        let op = Op::Merge {
            path: vec!["a".to_string()],
            pos: 2,
            props: None,
        };
        let v = to_json(&op);
        assert!(v.get("props").is_none());
    }

    #[test]
    fn roundtrip_extend_with_delete_null() {
        let mut props = serde_json::Map::new();
        props.insert("key".into(), json!("val"));
        let op = Op::Extend {
            path: vec!["a".to_string()],
            props,
            delete_null: true,
        };
        let v = to_json(&op);
        assert_eq!(v["deleteNull"], true);
        let rt = from_json(&v).unwrap();
        assert_eq!(rt.op_name(), "extend");
    }

    #[test]
    fn roundtrip_extend_no_delete_null() {
        let mut props = serde_json::Map::new();
        props.insert("key".into(), json!("val"));
        let op = Op::Extend {
            path: vec!["a".to_string()],
            props,
            delete_null: false,
        };
        let v = to_json(&op);
        assert!(v.get("deleteNull").is_none());
    }

    #[test]
    fn roundtrip_defined() {
        let rt = roundtrip(Op::Defined {
            path: vec!["a".to_string()],
        });
        assert_eq!(rt.op_name(), "defined");
    }

    #[test]
    fn roundtrip_undefined() {
        let rt = roundtrip(Op::Undefined {
            path: vec!["a".to_string()],
        });
        assert_eq!(rt.op_name(), "undefined");
    }

    #[test]
    fn roundtrip_contains_ignore_case() {
        let op = Op::Contains {
            path: vec!["a".to_string()],
            value: "test".to_string(),
            ignore_case: true,
        };
        let v = to_json(&op);
        assert_eq!(v["ignore_case"], true);
        let rt = from_json(&v).unwrap();
        match rt {
            Op::Contains { ignore_case, .. } => assert!(ignore_case),
            _ => panic!("expected Contains"),
        }
    }

    #[test]
    fn roundtrip_contains_case_sensitive() {
        let op = Op::Contains {
            path: vec![],
            value: "x".to_string(),
            ignore_case: false,
        };
        let v = to_json(&op);
        assert!(v.get("ignore_case").is_none());
    }

    #[test]
    fn roundtrip_ends_ignore_case() {
        let op = Op::Ends {
            path: vec![],
            value: "xyz".to_string(),
            ignore_case: true,
        };
        let v = to_json(&op);
        assert_eq!(v["ignore_case"], true);
        let rt = from_json(&v).unwrap();
        assert_eq!(rt.op_name(), "ends");
    }

    #[test]
    fn roundtrip_starts_ignore_case() {
        let op = Op::Starts {
            path: vec![],
            value: "abc".to_string(),
            ignore_case: true,
        };
        let v = to_json(&op);
        assert_eq!(v["ignore_case"], true);
        let rt = from_json(&v).unwrap();
        assert_eq!(rt.op_name(), "starts");
    }

    #[test]
    fn roundtrip_in() {
        let op = Op::In {
            path: vec!["x".to_string()],
            value: vec![json!(1), json!(2)],
        };
        let rt = roundtrip(op);
        assert_eq!(rt.op_name(), "in");
    }

    #[test]
    fn roundtrip_less() {
        let op = Op::Less {
            path: vec![],
            value: 42.0,
        };
        let rt = roundtrip(op);
        assert_eq!(rt.op_name(), "less");
    }

    #[test]
    fn roundtrip_more() {
        let op = Op::More {
            path: vec![],
            value: 99.0,
        };
        let rt = roundtrip(op);
        assert_eq!(rt.op_name(), "more");
    }

    #[test]
    fn roundtrip_matches_ignore_case() {
        let op = Op::Matches {
            path: vec![],
            value: "^abc$".to_string(),
            ignore_case: true,
        };
        let v = to_json(&op);
        assert_eq!(v["ignore_case"], true);
        let rt = from_json(&v).unwrap();
        assert_eq!(rt.op_name(), "matches");
    }

    #[test]
    fn roundtrip_test_string() {
        let op = Op::TestString {
            path: vec!["a".to_string()],
            pos: 3,
            str_val: "abc".to_string(),
            not: false,
        };
        let rt = roundtrip(op);
        assert_eq!(rt.op_name(), "test_string");
    }

    #[test]
    fn roundtrip_test_string_with_not() {
        let op = Op::TestString {
            path: vec![],
            pos: 0,
            str_val: "x".to_string(),
            not: true,
        };
        let v = to_json(&op);
        assert_eq!(v["not"], true);
    }

    #[test]
    fn roundtrip_test_string_len() {
        let op = Op::TestStringLen {
            path: vec![],
            len: 10,
            not: false,
        };
        let rt = roundtrip(op);
        assert_eq!(rt.op_name(), "test_string_len");
    }

    #[test]
    fn roundtrip_test_string_len_with_not() {
        let op = Op::TestStringLen {
            path: vec![],
            len: 5,
            not: true,
        };
        let v = to_json(&op);
        assert_eq!(v["not"], true);
    }

    #[test]
    fn roundtrip_type() {
        let op = Op::Type {
            path: vec![],
            value: JsonPatchType::Array,
        };
        let v = to_json(&op);
        assert_eq!(v["value"], "array");
        let rt = from_json(&v).unwrap();
        assert_eq!(rt.op_name(), "type");
    }

    #[test]
    fn roundtrip_not_predicate() {
        let op = Op::Not {
            path: vec![],
            ops: vec![Op::Defined {
                path: vec!["a".to_string()],
            }],
        };
        let rt = roundtrip(op);
        assert_eq!(rt.op_name(), "not");
    }

    #[test]
    fn roundtrip_or_predicate() {
        let op = Op::Or {
            path: vec![],
            ops: vec![Op::Less {
                path: vec![],
                value: 1.0,
            }],
        };
        let rt = roundtrip(op);
        assert_eq!(rt.op_name(), "or");
    }

    #[test]
    fn roundtrip_remove_with_old_value() {
        let op = Op::Remove {
            path: vec!["a".to_string()],
            old_value: Some(json!("old")),
        };
        let v = to_json(&op);
        assert_eq!(v["oldValue"], "old");
        let rt = from_json(&v).unwrap();
        match rt {
            Op::Remove { old_value, .. } => assert_eq!(old_value, Some(json!("old"))),
            _ => panic!("expected Remove"),
        }
    }

    // ── to_json_patch / from_json_patch ─────────────────────────────────

    #[test]
    fn to_json_patch_produces_array() {
        let ops = vec![
            Op::Add {
                path: vec!["a".to_string()],
                value: json!(1),
            },
            Op::Remove {
                path: vec!["b".to_string()],
                old_value: None,
            },
        ];
        let v = to_json_patch(&ops);
        assert!(v.is_array());
        assert_eq!(v.as_array().unwrap().len(), 2);
    }

    #[test]
    fn from_json_patch_not_array() {
        let err = from_json_patch(&json!({"op": "add"})).unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_no_path_defaults_empty() {
        let v = json!({"op": "flip"});
        let op = from_json(&v).unwrap();
        assert!(op.path().is_empty());
    }

    // ── decode_type error path ──────────────────────────────────────────

    #[test]
    fn from_json_test_type_invalid_type_string() {
        let err = from_json(&json!({
            "op": "test_type",
            "path": "/a",
            "type": ["bogus"]
        }))
        .unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }

    #[test]
    fn from_json_type_invalid_value_type() {
        let err = from_json(&json!({
            "op": "type",
            "path": "/a",
            "value": 42
        }))
        .unwrap_err();
        assert!(matches!(err, PatchError::InvalidOp(_)));
    }
}
