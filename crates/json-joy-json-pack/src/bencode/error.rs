//! Bencode decoder error type.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BencodeError {
    #[error("invalid bencode: unexpected byte at position {0}")]
    InvalidByte(usize),
    #[error("invalid bencode: unexpected end of input")]
    UnexpectedEof,
    #[error("invalid bencode: integer overflow")]
    IntegerOverflow,
    #[error("invalid bencode: invalid UTF-8 in string")]
    InvalidUtf8,
    #[error("invalid bencode: invalid key `__proto__`")]
    InvalidKey,
}
