//! Upstream-parity JSONPath parser API (`JsonPathParser.ts`).

use crate::parser_impl::ParserImpl;
use crate::{JSONPath, ParseError, ParseResult};

/// Upstream-compatible parser facade.
#[derive(Debug, Default, Clone, Copy)]
pub struct JsonPathParser;

impl JsonPathParser {
    /// Parse a JSONPath expression and return upstream-style parse result.
    pub fn parse(path_str: &str) -> ParseResult {
        match ParserImpl::parse_with_position(path_str) {
            Ok(path) => ParseResult {
                success: true,
                path: Some(path),
                error: None,
                position: None,
            },
            Err((error, position)) => ParseResult {
                success: false,
                path: None,
                error: Some(error.to_string()),
                position: Some(position),
            },
        }
    }

    /// Rust compatibility path that preserves the old `Result` parser API.
    pub fn parse_strict(path_str: &str) -> Result<JSONPath, ParseError> {
        ParserImpl::parse(path_str)
    }
}
