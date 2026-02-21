//! MessagePack codec wrapper.
//!
//! Upstream reference: `json-pack/src/codecs/msgpack.ts`

use crate::{msgpack::MsgPackDecoder, msgpack::MsgPackEncoder, EncodingFormat, PackValue};

use super::types::{CodecError, JsonValueCodec};

pub struct MsgPackJsonValueCodec {
    pub encoder: MsgPackEncoder,
    pub decoder: MsgPackDecoder,
}

impl Default for MsgPackJsonValueCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl MsgPackJsonValueCodec {
    pub fn new() -> Self {
        Self {
            encoder: MsgPackEncoder::new(),
            decoder: MsgPackDecoder::new(),
        }
    }

    pub fn id(&self) -> &'static str {
        "msgpack"
    }

    pub fn format(&self) -> EncodingFormat {
        EncodingFormat::MsgPack
    }

    pub fn encode(&mut self, value: &PackValue) -> Result<Vec<u8>, CodecError> {
        Ok(self.encoder.encode(value))
    }

    pub fn decode(&mut self, bytes: &[u8]) -> Result<PackValue, CodecError> {
        Ok(self.decoder.decode(bytes)?)
    }
}

impl JsonValueCodec for MsgPackJsonValueCodec {
    fn id(&self) -> &'static str {
        self.id()
    }

    fn format(&self) -> EncodingFormat {
        self.format()
    }

    fn encode(&mut self, value: &PackValue) -> Result<Vec<u8>, CodecError> {
        self.encode(value)
    }

    fn decode(&mut self, bytes: &[u8]) -> Result<PackValue, CodecError> {
        self.decode(bytes)
    }
}
