//! BSON (Binary JSON) encoding and decoding.
//!
//! Upstream reference: `json-pack/src/bson/`

pub mod decoder;
pub mod encoder;
pub mod values;

pub use decoder::BsonDecoder;
pub use encoder::BsonEncoder;
pub use values::{
    BsonBinary, BsonDbPointer, BsonDecimal128, BsonJavascriptCode, BsonJavascriptCodeWithScope,
    BsonMaxKey, BsonMinKey, BsonObjectId, BsonSymbol, BsonTimestamp, BsonValue,
};
