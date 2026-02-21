//! Aggregate codec holder.
//!
//! Upstream reference: `json-pack/src/codecs/Codecs.ts`

use super::{CborJsonValueCodec, JsonJsonValueCodec, MsgPackJsonValueCodec};

pub struct Codecs {
    pub cbor: CborJsonValueCodec,
    pub msgpack: MsgPackJsonValueCodec,
    pub json: JsonJsonValueCodec,
}

impl Default for Codecs {
    fn default() -> Self {
        Self::new()
    }
}

impl Codecs {
    pub fn new() -> Self {
        Self {
            cbor: CborJsonValueCodec::new(),
            msgpack: MsgPackJsonValueCodec::new(),
            json: JsonJsonValueCodec::new(),
        }
    }
}
