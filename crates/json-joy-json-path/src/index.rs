//! Upstream-parity re-export module (`index.ts`).

pub use crate::ast::Ast;
pub use crate::types::*;
pub use crate::{
    get_accessed_properties, json_path_equals, json_path_to_string, JsonPathCodegen, JsonPathEval,
    JsonPathParser,
};
