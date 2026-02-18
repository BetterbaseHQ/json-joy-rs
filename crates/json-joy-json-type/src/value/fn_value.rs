//! FnValue — a callable value.
//!
//! Upstream reference: json-type/src/value/FnValue.ts

use serde_json::Value as JsonValue;

/// A callable function value (takes a request and returns a response).
pub type FnValue = Box<dyn Fn(JsonValue) -> JsonValue + Send + Sync>;
