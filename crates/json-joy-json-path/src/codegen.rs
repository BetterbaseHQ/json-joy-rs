//! JSONPath compile/run helpers.
//!
//! Upstream maps this surface to runtime code generation.
//! Rust compiles to a reusable closure over parsed AST.

use serde_json::Value;

use crate::{JSONPath, JsonPathEval, JsonPathParser, ParseError};

pub type JsonPathCompiledFn =
    Box<dyn for<'a> Fn(&'a Value) -> Vec<&'a Value> + Send + Sync + 'static>;

pub struct JsonPathCodegen;

impl JsonPathCodegen {
    /// Parse, compile, and run one JSONPath query.
    pub fn run<'a>(path: &str, data: &'a Value) -> Result<Vec<&'a Value>, ParseError> {
        let compiled = Self::compile(path)?;
        Ok(compiled(data))
    }

    /// Parse and compile JSONPath into a reusable closure.
    pub fn compile(path: &str) -> Result<JsonPathCompiledFn, ParseError> {
        let ast = JsonPathParser::parse(path)?;
        Ok(Self::compile_ast(ast))
    }

    /// Compile a pre-parsed AST into a reusable closure.
    pub fn compile_ast(path: JSONPath) -> JsonPathCompiledFn {
        Box::new(move |data| JsonPathEval::eval(&path, data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn run_compiles_and_executes() {
        let doc = json!({"store": {"books": [{"title": "A"}, {"title": "B"}]}});
        let out = JsonPathCodegen::run("$.store.books[*].title", &doc).expect("compile/run");
        assert_eq!(out.len(), 2);
        assert_eq!(out[0], &json!("A"));
        assert_eq!(out[1], &json!("B"));
    }

    #[test]
    fn compile_ast_reuses_parsed_path() {
        let ast = JsonPathParser::parse("$[1]").expect("parse");
        let compiled = JsonPathCodegen::compile_ast(ast);
        let doc = json!([10, 20, 30]);
        let out = compiled(&doc);
        assert_eq!(out, vec![&json!(20)]);
    }
}
