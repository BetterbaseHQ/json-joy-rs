//! JSON codec wrapper.
//!
//! Upstream reference: `json-pack/src/codecs/json.ts`

use crate::{json::JsonDecoder, json::JsonEncoder, EncodingFormat, PackValue};

use super::types::{CodecError, JsonValueCodec};

pub struct JsonJsonValueCodec {
    pub encoder: JsonEncoder,
    pub decoder: JsonDecoder,
}

impl Default for JsonJsonValueCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonJsonValueCodec {
    pub fn new() -> Self {
        Self {
            encoder: JsonEncoder::new(),
            decoder: JsonDecoder::new(),
        }
    }

    pub fn id(&self) -> &'static str {
        "json"
    }

    pub fn format(&self) -> EncodingFormat {
        EncodingFormat::Json
    }

    pub fn encode(&mut self, value: &PackValue) -> Result<Vec<u8>, CodecError> {
        Ok(self.encoder.encode(value))
    }

    pub fn decode(&mut self, bytes: &[u8]) -> Result<PackValue, CodecError> {
        Ok(self.decoder.decode(bytes)?)
    }
}

impl JsonValueCodec for JsonJsonValueCodec {
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
