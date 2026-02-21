//! CBOR codec wrapper.
//!
//! Upstream reference: `json-pack/src/codecs/cbor.ts`

use crate::{cbor::CborDecoder, cbor::CborEncoder, EncodingFormat, PackValue};

use super::types::{CodecError, JsonValueCodec};

pub struct CborJsonValueCodec {
    pub encoder: CborEncoder,
    pub decoder: CborDecoder,
}

impl Default for CborJsonValueCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl CborJsonValueCodec {
    pub fn new() -> Self {
        Self {
            encoder: CborEncoder::new(),
            decoder: CborDecoder::new(),
        }
    }

    pub fn id(&self) -> &'static str {
        "cbor"
    }

    pub fn format(&self) -> EncodingFormat {
        EncodingFormat::Cbor
    }

    pub fn encode(&mut self, value: &PackValue) -> Result<Vec<u8>, CodecError> {
        Ok(self.encoder.encode(value))
    }

    pub fn decode(&mut self, bytes: &[u8]) -> Result<PackValue, CodecError> {
        Ok(self.decoder.decode(bytes)?)
    }
}

impl JsonValueCodec for CborJsonValueCodec {
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
