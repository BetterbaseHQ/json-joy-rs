//! JSON encoder/decoder error type.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum JsonError {
    #[error("invalid JSON at byte {0}")]
    Invalid(usize),
    #[error("invalid UTF-8")]
    InvalidUtf8,
    #[error("invalid key `__proto__`")]
    InvalidKey,
    #[error("parse error: {0}")]
    Parse(#[from] serde_json::Error),
}
