//! Discriminator computation for Or types.
//!
//! Upstream reference: json-type/src/type/discriminator.ts

use serde_json::Value;

use super::TypeNode;

/// Computes a discriminator expression for distinguishing between union type members.
pub struct Discriminator;

impl Discriminator {
    /// Create a discriminator expression for the given list of types.
    ///
    /// Returns a JSON expression that, when evaluated, yields the index of the
    /// matching type (0-based).
    pub fn create_expression(types: &[TypeNode]) -> Value {
        // Simple heuristic: use a discriminator based on the first distinguishing property
        // In Rust we don't do JIT codegen, so we return a marker that the runtime
        // validator will use to try each type in order.
        serde_json::json!({"$discriminator": "linear", "count": types.len()})
    }
}
