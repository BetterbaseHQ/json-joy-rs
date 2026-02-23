//! SSH encoder/decoder error type.

use thiserror::Error;

/// Error type for SSH 2.0 binary protocol encoding and decoding operations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SshError {
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("invalid UTF-8")]
    InvalidUtf8,
    #[error("unsupported value type for SSH encoding: {0}")]
    UnsupportedType(&'static str),
    #[error("name-list elements must be strings")]
    InvalidNameList,
}
