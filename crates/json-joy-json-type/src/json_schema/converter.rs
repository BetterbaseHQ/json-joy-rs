//! Converts TypeNode to JSON Schema representation.
//!
//! Upstream reference: json-type/src/json-schema/converter.ts

use serde_json::{json, Map, Value};

use crate::schema::NumFormat;
use crate::type_def::{
    AnyType, ArrType, BinType, BoolType, ConType, FnRxType, FnType, KeyType, MapType, NumType,
    ObjType, OrType, RefType, StrType, TypeNode,
};

fn base_schema(node: &TypeNode) -> Map<String, Value> {
    let base = node.base();
    let mut m = Map::new();
    if let Some(t) = &base.title {
        m.insert("title".into(), json!(t));
    }
    if let Some(d) = &base.description {
        m.insert("description".into(), json!(d));
    }
    if !base.examples.is_empty() {
        m.insert("examples".into(), Value::Array(base.examples.clone()));
    }
    m
}

fn merge(base: Map<String, Value>, extra: Map<String, Value>) -> Value {
    let mut out = base;
    out.extend(extra);
    Value::Object(out)
}

fn any_to_json_schema(t: &AnyType, base: Map<String, Value>) -> Value {
    let _ = t;
    let mut m = Map::new();
    m.insert(
        "type".into(),
        json!(["string", "number", "boolean", "null", "array", "object"]),
    );
    merge(base, m)
}

fn bool_to_json_schema(t: &BoolType, base: Map<String, Value>) -> Value {
    let _ = t;
    let mut m = Map::new();
    m.insert("type".into(), json!("boolean"));
    merge(base, m)
}

fn num_to_json_schema(t: &NumType, base: Map<String, Value>) -> Value {
    let integer_formats = [
        NumFormat::I8,
        NumFormat::I16,
        NumFormat::I32,
        NumFormat::U8,
        NumFormat::U16,
        NumFormat::U32,
    ];
    let type_str = if t
        .schema
        .format
        .map(|f| integer_formats.contains(&f))
        .unwrap_or(false)
    {
        "integer"
    } else {
        "number"
    };
    let mut m = Map::new();
    m.insert("type".into(), json!(type_str));
    if let Some(v) = t.schema.gt {
        m.insert("exclusiveMinimum".into(), json!(v));
    }
    if let Some(v) = t.schema.gte {
        m.insert("minimum".into(), json!(v));
    }
    if let Some(v) = t.schema.lt {
        m.insert("exclusiveMaximum".into(), json!(v));
    }
    if let Some(v) = t.schema.lte {
        m.insert("maximum".into(), json!(v));
    }
    merge(base, m)
}

fn str_to_json_schema(t: &StrType, base: Map<String, Value>) -> Value {
    let mut m = Map::new();
    m.insert("type".into(), json!("string"));
    if let Some(min) = t.schema.min {
        m.insert("minLength".into(), json!(min));
    }
    if let Some(max) = t.schema.max {
        m.insert("maxLength".into(), json!(max));
    }
    let is_ascii = t.schema.ascii.unwrap_or(false)
        || t.schema
            .format
            .map(|f| matches!(f, crate::schema::StrFormat::Ascii))
            .unwrap_or(false);
    if is_ascii {
        m.insert("pattern".into(), json!("^[\\x00-\\x7F]*$"));
    }
    merge(base, m)
}

fn bin_to_json_schema(t: &BinType, base: Map<String, Value>) -> Value {
    let _ = t;
    let mut m = Map::new();
    m.insert("type".into(), json!("binary"));
    merge(base, m)
}

fn arr_to_json_schema(t: &ArrType, base: Map<String, Value>) -> Value {
    let mut m = Map::new();
    m.insert("type".into(), json!("array"));
    if let Some(item_type) = &t.type_ {
        m.insert("items".into(), type_to_json_schema(item_type));
    }
    if let Some(min) = t.schema.min {
        m.insert("minItems".into(), json!(min));
    }
    if let Some(max) = t.schema.max {
        m.insert("maxItems".into(), json!(max));
    }
    merge(base, m)
}

fn obj_to_json_schema(t: &ObjType, base: Map<String, Value>) -> Value {
    let mut m = Map::new();
    m.insert("type".into(), json!("object"));
    let mut properties = Map::new();
    let mut required: Vec<String> = Vec::new();
    for key in &t.keys {
        let key_schema = type_to_json_schema(&key.val);
        properties.insert(key.key.clone(), key_schema);
        if !key.optional {
            required.push(key.key.clone());
        }
    }
    if !properties.is_empty() {
        m.insert("properties".into(), Value::Object(properties));
    }
    if !required.is_empty() {
        m.insert("required".into(), json!(required));
    }
    if t.schema.decode_unknown_keys == Some(false) {
        m.insert("additionalProperties".into(), json!(false));
    }
    merge(base, m)
}

fn map_to_json_schema(t: &MapType, base: Map<String, Value>) -> Value {
    let mut m = Map::new();
    m.insert("type".into(), json!("object"));
    let value_schema = type_to_json_schema(&t.value);
    let mut pattern_props = Map::new();
    pattern_props.insert(".*".into(), value_schema);
    m.insert("patternProperties".into(), Value::Object(pattern_props));
    merge(base, m)
}

fn con_to_json_schema(t: &ConType, base: Map<String, Value>) -> Value {
    let value = &t.value;
    let mut m = Map::new();
    match value {
        Value::String(s) => {
            m.insert("type".into(), json!("string"));
            m.insert("const".into(), json!(s));
        }
        Value::Number(n) => {
            m.insert("type".into(), json!("number"));
            m.insert("const".into(), value.clone());
            let _ = n;
        }
        Value::Bool(b) => {
            m.insert("type".into(), json!("boolean"));
            m.insert("const".into(), json!(b));
        }
        Value::Null => {
            m.insert("type".into(), json!("null"));
            m.insert("const".into(), Value::Null);
        }
        Value::Array(_) => {
            m.insert("type".into(), json!("array"));
            m.insert("const".into(), value.clone());
        }
        Value::Object(_) => {
            m.insert("type".into(), json!("object"));
            m.insert("const".into(), value.clone());
        }
    }
    merge(base, m)
}

fn or_to_json_schema(t: &OrType, base: Map<String, Value>) -> Value {
    let any_of: Vec<Value> = t.types.iter().map(type_to_json_schema).collect();
    let mut m = Map::new();
    m.insert("anyOf".into(), Value::Array(any_of));
    merge(base, m)
}

fn ref_to_json_schema(t: &RefType, base: Map<String, Value>) -> Value {
    let mut m = Map::new();
    m.insert("$ref".into(), json!(format!("#/$defs/{}", t.ref_)));
    merge(base, m)
}

fn key_to_json_schema(t: &KeyType, base: Map<String, Value>) -> Value {
    let _ = t;
    // KeyType is a field descriptor, not a standalone schema
    merge(base, Map::new())
}

fn fn_to_json_schema(_t: &FnType, base: Map<String, Value>) -> Value {
    merge(base, Map::new())
}

fn fn_rx_to_json_schema(_t: &FnRxType, base: Map<String, Value>) -> Value {
    merge(base, Map::new())
}

/// Convert a `TypeNode` to a JSON Schema `Value`.
///
/// Ports `typeToJsonSchema` from `json-type/src/json-schema/converter.ts`.
pub fn type_to_json_schema(type_: &TypeNode) -> Value {
    let base = base_schema(type_);
    match type_ {
        TypeNode::Any(t) => any_to_json_schema(t, base),
        TypeNode::Bool(t) => bool_to_json_schema(t, base),
        TypeNode::Num(t) => num_to_json_schema(t, base),
        TypeNode::Str(t) => str_to_json_schema(t, base),
        TypeNode::Bin(t) => bin_to_json_schema(t, base),
        TypeNode::Con(t) => con_to_json_schema(t, base),
        TypeNode::Arr(t) => arr_to_json_schema(t, base),
        TypeNode::Obj(t) => obj_to_json_schema(t, base),
        TypeNode::Map(t) => map_to_json_schema(t, base),
        TypeNode::Ref(t) => ref_to_json_schema(t, base),
        TypeNode::Or(t) => or_to_json_schema(t, base),
        TypeNode::Fn(t) => fn_to_json_schema(t, base),
        TypeNode::FnRx(t) => fn_rx_to_json_schema(t, base),
        TypeNode::Key(t) => key_to_json_schema(t, base),
        TypeNode::Alias(alias) => type_to_json_schema(alias.get_type()),
    }
}
