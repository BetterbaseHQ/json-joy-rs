//! TypeScript type generator.
//!
//! Upstream reference: json-type/src/typescript/

pub mod converter;
pub mod to_text;
pub mod types;

pub use converter::to_typescript_ast;
pub use to_text::to_text;
pub use types::TsType;
