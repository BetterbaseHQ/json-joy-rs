//! UBJSON decoder error type.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum UbjsonError {
    #[error("unexpected byte 0x{0:02x} at position {1}")]
    UnexpectedByte(u8, usize),
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("invalid UTF-8 in string")]
    InvalidUtf8,
    #[error("invalid key `__proto__`")]
    InvalidKey,
}
