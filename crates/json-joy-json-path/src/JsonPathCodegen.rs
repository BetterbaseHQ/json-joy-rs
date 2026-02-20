//! Upstream-parity `JsonPathCodegen` API.
//!
//! Rust divergence note: this implementation does not emit dynamic code; it
//! returns a compiled closure that delegates to the evaluator.

use crate::{JSONPath, JsonPathEval, JsonPathParser, Value};
use serde_json::Value as JsonValue;

pub type JsonPathCompiledFn = Box<dyn Fn(&JsonValue) -> Vec<Value> + Send + Sync + 'static>;

pub struct JsonPathCodegen;

impl JsonPathCodegen {
    pub fn run(path: &str, data: &JsonValue) -> Vec<Value> {
        let fnc = Self::compile(path);
        fnc(data)
    }

    pub fn run_ast(path: &JSONPath, data: &JsonValue) -> Vec<Value> {
        let fnc = Self::compile_ast(path);
        fnc(data)
    }

    pub fn compile(path: &str) -> JsonPathCompiledFn {
        let parsed = JsonPathParser::parse(path);
        if !parsed.success || parsed.path.is_none() || parsed.error.is_some() {
            panic!(
                "Invalid JSONPath: {} [position = {:?}, path = {}]",
                parsed
                    .error
                    .unwrap_or_else(|| "unknown parse error".to_string()),
                parsed.position,
                path
            );
        }
        Self::compile_ast(&parsed.path.expect("parse result path unexpectedly missing"))
    }

    pub fn compile_ast(path: &JSONPath) -> JsonPathCompiledFn {
        let ast = path.clone();
        Box::new(move |data| JsonPathEval::run_ast(&ast, data))
    }
}
