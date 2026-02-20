//! Runtime discriminator evaluator.
//!
//! Upstream reference: `json-type/src/codegen/discriminator/index.ts`.

use std::sync::Arc;

use json_expression::{evaluate, operators_map, EvalCtx, Vars};
use serde_json::Value;

use crate::type_def::OrType;

pub type DiscriminatorFn = Box<dyn Fn(&Value) -> Result<i64, String> + Send + Sync>;

pub struct DiscriminatorCodegen;

impl DiscriminatorCodegen {
    pub fn get(or: &OrType) -> Result<DiscriminatorFn, String> {
        let expr = or.discriminator.clone();
        if is_no_discriminator(&expr) {
            return Err("NO_DISCRIMINATOR".to_string());
        }

        // Upstream memoizes generated functions by union identity. We skip caching
        // here and reuse shared operator tables through `Arc`.
        let operators = Arc::new(operators_map());
        Ok(Box::new(move |data: &Value| {
            let mut vars = Vars::new(data.clone());
            let mut ctx = EvalCtx::new(&mut vars, operators.clone());
            let out = evaluate(&expr, &mut ctx).map_err(|e| e.to_string())?;
            let n = json_expression::util::num(&out);
            if !n.is_finite() {
                return Err("NON_FINITE_DISCRIMINATOR".to_string());
            }
            Ok(n as i64)
        }))
    }
}

fn is_no_discriminator(expr: &Value) -> bool {
    let Value::Array(arr) = expr else {
        return false;
    };
    if arr.len() != 2 {
        return false;
    }
    matches!(
        (&arr[0], &arr[1]),
        (Value::String(op), Value::Number(num)) if op == "num" && num.as_i64() == Some(0)
    )
}
