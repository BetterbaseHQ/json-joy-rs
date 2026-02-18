use thiserror::Error;

/// Error type for CBOR encoding/decoding operations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CborError {
    #[error("invalid cbor payload")]
    InvalidPayload,
    #[error("unsupported cbor feature")]
    Unsupported,
    #[error("unexpected major type")]
    UnexpectedMajor,
    #[error("unexpected minor value")]
    UnexpectedMinor,
    #[error("unexpected binary chunk major type")]
    UnexpectedBinChunkMajor,
    #[error("unexpected binary chunk minor value")]
    UnexpectedBinChunkMinor,
    #[error("unexpected string chunk major type")]
    UnexpectedStrChunkMajor,
    #[error("unexpected string chunk minor value")]
    UnexpectedStrChunkMinor,
    #[error("unexpected object key")]
    UnexpectedObjKey,
    #[error("unexpected object break")]
    UnexpectedObjBreak,
    #[error("invalid size")]
    InvalidSize,
    #[error("key not found")]
    KeyNotFound,
    #[error("index out of bounds")]
    IndexOutOfBounds,
    #[error("unexpected string major type")]
    UnexpectedStrMajor,
}
