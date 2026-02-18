//! Value — a typed JSON value.
//!
//! Upstream reference: json-type/src/value/Value.ts

use serde_json::Value as JsonValue;

/// A typed JSON value (pairs a runtime value with its type node).
#[derive(Debug, Clone)]
pub struct Value {
    pub data: JsonValue,
}

impl Value {
    pub fn new(data: JsonValue) -> Self {
        Self { data }
    }

    pub fn name(&self) -> &'static str {
        "Value"
    }
}

/// Create an untyped value wrapper.
pub fn unknown(data: JsonValue) -> Value {
    Value::new(data)
}
