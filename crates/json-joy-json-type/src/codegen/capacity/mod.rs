//! Capacity estimator codegen.
//!
//! Upstream reference:
//! - `json-type/src/codegen/capacity/CapacityEstimatorCodegen.ts`
//!
//! This Rust port keeps the same public naming and behavior intent while using
//! runtime traversal instead of JS source generation.

use std::sync::Arc;

use serde_json::Value;

use crate::codegen::discriminator::DiscriminatorCodegen;
use crate::codegen::validator::{validate, ErrorMode, ValidatorOptions};
use crate::type_def::{OrType, TypeBuilder, TypeNode};
use json_joy_util::json_size::{max_encoding_capacity, MaxEncodingOverhead};

/// Compiled capacity estimator function.
pub type CompiledCapacityEstimator = Arc<dyn Fn(&Value) -> usize + Send + Sync>;

/// Runtime equivalent of upstream `CapacityEstimatorCodegen`.
pub struct CapacityEstimatorCodegen;

impl CapacityEstimatorCodegen {
    /// Build a reusable estimator for a type.
    pub fn get(type_: &TypeNode) -> CompiledCapacityEstimator {
        let type_ = type_.clone();
        Arc::new(move |value: &Value| estimate_node(&type_, value))
    }
}

fn estimate_node(type_: &TypeNode, value: &Value) -> usize {
    match type_ {
        TypeNode::Any(_) => max_encoding_capacity(value),
        TypeNode::Con(t) => max_encoding_capacity(t.literal()),
        TypeNode::Bool(_) => MaxEncodingOverhead::BOOLEAN,
        TypeNode::Num(_) => MaxEncodingOverhead::NUMBER,
        TypeNode::Str(_) => {
            let len = value.as_str().map_or(0, |s| s.len());
            MaxEncodingOverhead::STRING + len * MaxEncodingOverhead::STRING_LENGTH_MULTIPLIER
        }
        TypeNode::Bin(_) => {
            let len = value.as_array().map_or(0, Vec::len);
            MaxEncodingOverhead::BINARY + len * MaxEncodingOverhead::BINARY_LENGTH_MULTIPLIER
        }
        TypeNode::Arr(t) => estimate_arr(t, value),
        TypeNode::Obj(t) => estimate_obj(t, value),
        TypeNode::Map(t) => estimate_map(t, value),
        TypeNode::Ref(t) => {
            if let Some(system) = &t.base.system {
                if let Ok(alias) = system.resolve(&t.ref_) {
                    let builder = TypeBuilder::with_system(Arc::clone(system));
                    let resolved = builder.import(&alias.schema);
                    return estimate_node(&resolved, value);
                }
            }
            max_encoding_capacity(value)
        }
        TypeNode::Or(t) => estimate_or(t, value),
        TypeNode::Fn(_) | TypeNode::FnRx(_) => max_encoding_capacity(value),
        TypeNode::Key(t) => estimate_node(&t.val, value),
        TypeNode::Alias(t) => estimate_node(&t.type_, value),
    }
}

fn estimate_arr(t: &crate::type_def::ArrType, value: &Value) -> usize {
    let arr = match value.as_array() {
        Some(arr) => arr,
        None => {
            return MaxEncodingOverhead::ARRAY;
        }
    };

    let mut size = MaxEncodingOverhead::ARRAY + arr.len() * MaxEncodingOverhead::ARRAY_ELEMENT;
    let len = arr.len();
    let tail_len = t.tail.len();

    for (i, item) in arr.iter().enumerate() {
        if i < t.head.len() {
            size += estimate_node(&t.head[i], item);
            continue;
        }
        if tail_len > 0 && i >= len.saturating_sub(tail_len) {
            let tail_index = i - (len - tail_len);
            size += estimate_node(&t.tail[tail_index], item);
            continue;
        }
        if let Some(body) = &t.type_ {
            size += estimate_node(body, item);
        }
    }

    size
}

fn estimate_obj(t: &crate::type_def::ObjType, value: &Value) -> usize {
    let Some(obj) = value.as_object() else {
        return MaxEncodingOverhead::OBJECT;
    };

    if t.schema.encode_unknown_keys == Some(true) {
        return max_encoding_capacity(value);
    }

    let mut size = MaxEncodingOverhead::OBJECT;
    for field in &t.keys {
        if !field.optional || obj.contains_key(&field.key) {
            size += MaxEncodingOverhead::OBJECT_ELEMENT;
            size += max_encoding_capacity(&Value::String(field.key.clone()));
            let field_value = obj.get(&field.key).unwrap_or(&Value::Null);
            size += estimate_node(&field.val, field_value);
        }
    }
    size
}

fn estimate_map(t: &crate::type_def::MapType, value: &Value) -> usize {
    let Some(obj) = value.as_object() else {
        return MaxEncodingOverhead::OBJECT;
    };

    let mut size = MaxEncodingOverhead::OBJECT + obj.len() * MaxEncodingOverhead::OBJECT_ELEMENT;
    for (key, val) in obj {
        size +=
            MaxEncodingOverhead::STRING + key.len() * MaxEncodingOverhead::STRING_LENGTH_MULTIPLIER;
        size += estimate_node(&t.value, val);
    }
    size
}

fn estimate_or(t: &OrType, value: &Value) -> usize {
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
    estimate_node(&t.types[index], value)
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
