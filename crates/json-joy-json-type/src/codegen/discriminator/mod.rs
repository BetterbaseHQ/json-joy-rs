//! Discriminator codegen.
//!
//! Upstream reference:
//! - `json-type/src/codegen/discriminator/index.ts`
//!
//! This runtime port evaluates the discriminator JSON expression with the
//! `json-expression` crate.

use std::sync::Arc;

use json_expression::util::num;
use json_expression::{JsError, JsonExpressionCodegen, Vars};
use serde_json::Value;
use thiserror::Error;

use crate::type_def::OrType;

/// A compiled union discriminator function.
pub type DiscriminatorFn = Arc<dyn Fn(&Value) -> i64 + Send + Sync>;

/// Errors while building a discriminator.
#[derive(Debug, Error)]
pub enum DiscriminatorCodegenError {
    #[error("NO_DISCRIMINATOR")]
    NoDiscriminator,
    #[error("Failed to evaluate discriminator expression: {0}")]
    Evaluate(#[from] JsError),
}

/// Runtime equivalent of upstream `DiscriminatorCodegen`.
pub struct DiscriminatorCodegen;

impl DiscriminatorCodegen {
    /// Build a discriminator from an `or` type's schema discriminator expression.
    pub fn get(or: &OrType) -> Result<DiscriminatorFn, DiscriminatorCodegenError> {
        let expr = &or.discriminator;
        if !json_truthy(expr) || is_num_zero_expression(expr) {
            return Err(DiscriminatorCodegenError::NoDiscriminator);
        }

        let codegen = JsonExpressionCodegen::with_expression(expr.clone());
        let compiled = codegen.compile();

        Ok(Arc::new(move |data: &Value| {
            let mut vars = Vars::new(data.clone());
            match compiled.call(&mut vars) {
                Ok(result) => num(&result).trunc() as i64,
                Err(_) => 0,
            }
        }))
    }
}

fn is_num_zero_expression(expr: &Value) -> bool {
    match expr {
        Value::Array(items) if items.len() == 2 => {
            items[0].as_str() == Some("num") && items[1].as_i64() == Some(0)
        }
        _ => false,
    }
}

fn json_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|v| v != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}
