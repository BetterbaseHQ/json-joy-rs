//! ObjValue — typed object value.
//!
//! Upstream reference: json-type/src/value/ObjValue.ts

use std::collections::HashMap;
use serde_json::Value as JsonValue;

/// A typed object value (key → JSON value map).
#[derive(Debug, Clone, Default)]
pub struct ObjValue {
    pub fields: HashMap<String, JsonValue>,
}

impl ObjValue {
    pub fn new() -> Self { Self::default() }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        self.fields.get(key)
    }

    pub fn set(&mut self, key: impl Into<String>, value: JsonValue) {
        self.fields.insert(key.into(), value);
    }
}
