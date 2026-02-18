//! MessagePack decoder error type.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MsgPackError {
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("invalid key `__proto__`")]
    InvalidKey,
    #[error("invalid UTF-8")]
    InvalidUtf8,
    #[error("invalid size")]
    InvalidSize,
    #[error("not an object")]
    NotObj,
    #[error("not an array")]
    NotArr,
    #[error("not a string")]
    NotStr,
    #[error("key not found")]
    KeyNotFound,
    #[error("index out of bounds")]
    IndexOutOfBounds,
    #[error("invalid MessagePack byte at offset {0}")]
    InvalidByte(usize),
}
