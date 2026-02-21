//! Common codec traits and errors.
//!
//! Upstream reference: `json-pack/src/codecs/types.ts`

use crate::{cbor::CborError, json::JsonError, msgpack::MsgPackError, EncodingFormat, PackValue};

#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("CBOR codec error: {0}")]
    Cbor(#[from] CborError),
    #[error("JSON codec error: {0}")]
    Json(#[from] JsonError),
    #[error("MessagePack codec error: {0}")]
    MsgPack(#[from] MsgPackError),
}

/// Trait for binary codecs that encode/decode [`PackValue`].
pub trait JsonValueCodec {
    fn id(&self) -> &'static str;
    fn format(&self) -> EncodingFormat;
    fn encode(&mut self, value: &PackValue) -> Result<Vec<u8>, CodecError>;
    fn decode(&mut self, bytes: &[u8]) -> Result<PackValue, CodecError>;
}
