//! JSON text codegen.
//!
//! Upstream reference:
//! - `json-type/src/codegen/json/JsonTextCodegen.ts`
//!
//! This Rust port keeps upstream naming and behavior intent while using a
//! runtime recursive encoder instead of JS source generation.

use std::collections::HashSet;
use std::sync::Arc;

use serde_json::Value;
use thiserror::Error;

use crate::codegen::discriminator::DiscriminatorCodegen;
use crate::codegen::validator::{validate, ErrorMode, ValidatorOptions};
use crate::type_def::{OrType, TypeBuilder, TypeNode};

/// A compiled JSON text encoder function.
pub type JsonEncoderFn = Arc<dyn Fn(&Value) -> Result<String, JsonTextCodegenError> + Send + Sync>;

/// Errors while encoding values with `JsonTextCodegen`.
#[derive(Debug, Error)]
pub enum JsonTextCodegenError {
    #[error("NO_SYSTEM")]
    NoSystem,
    #[error("Failed to resolve reference: {0}")]
    ResolveRef(String),
    #[error("Failed to serialize JSON text: {0}")]
    Serialize(#[from] serde_json::Error),
}

/// Runtime equivalent of upstream `JsonTextCodegen`.
pub struct JsonTextCodegen;

impl JsonTextCodegen {
    /// Build a reusable encoder for a type.
    pub fn get(type_: &TypeNode) -> JsonEncoderFn {
        let type_ = type_.clone();
        Arc::new(move |value: &Value| encode_node(&type_, value))
    }
}

fn encode_node(type_: &TypeNode, value: &Value) -> Result<String, JsonTextCodegenError> {
    match type_ {
        TypeNode::Any(_) => serde_json::to_string(value).map_err(Into::into),
        TypeNode::Bool(_) => Ok(if js_truthy(value) {
            "true".to_string()
        } else {
            "false".to_string()
        }),
        TypeNode::Num(_) => Ok(js_to_string(value)),
        TypeNode::Str(t) => {
            let s = value
                .as_str()
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| js_to_string(value));
            if t.schema.no_json_escape == Some(true) {
                Ok(format!("\"{s}\""))
            } else {
                serde_json::to_string(&s).map_err(Into::into)
            }
        }
        TypeNode::Bin(_) => encode_bin(value),
        TypeNode::Con(t) => serde_json::to_string(t.literal()).map_err(Into::into),
        TypeNode::Arr(t) => encode_arr(t, value),
        TypeNode::Obj(t) => encode_obj(t, value),
        TypeNode::Map(t) => encode_map(t, value),
        TypeNode::Ref(t) => {
            let Some(system) = &t.base.system else {
                return Err(JsonTextCodegenError::NoSystem);
            };
            let alias = system
                .resolve(&t.ref_)
                .map_err(JsonTextCodegenError::ResolveRef)?;
            let builder = TypeBuilder::with_system(Arc::clone(system));
            let resolved = builder.import(&alias.schema);
            encode_node(&resolved, value)
        }
        TypeNode::Or(t) => encode_or(t, value),
        TypeNode::Fn(_) | TypeNode::FnRx(_) => Ok("null".to_string()),
        TypeNode::Key(t) => encode_node(&t.val, value),
        TypeNode::Alias(t) => encode_node(&t.type_, value),
    }
}

fn encode_bin(value: &Value) -> Result<String, JsonTextCodegenError> {
    let bytes = match value {
        Value::Array(items) => items
            .iter()
            .map(|v| v.as_u64().and_then(|n| u8::try_from(n).ok()).unwrap_or(0))
            .collect::<Vec<u8>>(),
        _ => Vec::new(),
    };
    let uri = json_joy_json_pack::json_binary::stringify_binary(&bytes);
    serde_json::to_string(&uri).map_err(Into::into)
}

fn encode_arr(t: &crate::type_def::ArrType, value: &Value) -> Result<String, JsonTextCodegenError> {
    let Some(items) = value.as_array() else {
        return Ok("[]".to_string());
    };

    let mut out = String::from("[");
    let len = items.len();
    let tail_len = t.tail.len();

    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }

        let encoded = if i < t.head.len() {
            encode_node(&t.head[i], item)?
        } else if tail_len > 0 && i >= len.saturating_sub(tail_len) {
            let tail_index = i - (len - tail_len);
            encode_node(&t.tail[tail_index], item)?
        } else if let Some(body) = &t.type_ {
            encode_node(body, item)?
        } else {
            serde_json::to_string(item)?
        };
        out.push_str(&encoded);
    }

    out.push(']');
    Ok(out)
}

fn encode_obj(t: &crate::type_def::ObjType, value: &Value) -> Result<String, JsonTextCodegenError> {
    let map = value.as_object();
    let mut parts: Vec<String> = Vec::new();
    let mut known_keys: HashSet<&str> = HashSet::new();

    for field in &t.keys {
        known_keys.insert(field.key.as_str());
        if field.optional {
            let Some(obj) = map else {
                continue;
            };
            let Some(field_value) = obj.get(&field.key) else {
                continue;
            };
            parts.push(format!(
                "{}:{}",
                serde_json::to_string(&field.key)?,
                encode_node(&field.val, field_value)?
            ));
            continue;
        }

        let field_value = map
            .and_then(|obj| obj.get(&field.key))
            .unwrap_or(&Value::Null);
        parts.push(format!(
            "{}:{}",
            serde_json::to_string(&field.key)?,
            encode_node(&field.val, field_value)?
        ));
    }

    if t.schema.encode_unknown_keys == Some(true) {
        if let Some(obj) = map {
            for (key, val) in obj {
                if known_keys.contains(key.as_str()) {
                    continue;
                }
                parts.push(format!(
                    "{}:{}",
                    serde_json::to_string(key)?,
                    serde_json::to_string(val)?
                ));
            }
        }
    }

    Ok(format!("{{{}}}", parts.join(",")))
}

fn encode_map(t: &crate::type_def::MapType, value: &Value) -> Result<String, JsonTextCodegenError> {
    let Some(map) = value.as_object() else {
        return Ok("{}".to_string());
    };

    let mut parts: Vec<String> = Vec::with_capacity(map.len());
    for (key, val) in map {
        parts.push(format!(
            "{}:{}",
            serde_json::to_string(key)?,
            encode_node(&t.value, val)?
        ));
    }
    Ok(format!("{{{}}}", parts.join(",")))
}

fn encode_or(t: &OrType, value: &Value) -> Result<String, JsonTextCodegenError> {
    let index = if let Ok(discriminator) = DiscriminatorCodegen::get(t) {
        let idx = discriminator(value);
        if idx >= 0 && (idx as usize) < t.types.len() {
            idx as usize
        } else {
            first_matching_or_index(t, value)
        }
    } else {
        first_matching_or_index(t, value)
    };

    encode_node(&t.types[index], value)
}

fn first_matching_or_index(t: &OrType, value: &Value) -> usize {
    let opts = ValidatorOptions {
        errors: ErrorMode::Boolean,
        ..Default::default()
    };
    for (i, child) in t.types.iter().enumerate() {
        if validate(value, child, &opts, &[]).is_ok() {
            return i;
        }
    }
    0
}

fn js_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|v| v != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

fn js_to_string(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(true) => "true".to_string(),
        Value::Bool(false) => "false".to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(_) | Value::Object(_) => serde_json::to_string(value).unwrap_or_default(),
    }
}
