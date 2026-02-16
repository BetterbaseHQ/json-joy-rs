//! Indexed binary codec aligned to upstream `json-crdt/codec/indexed/binary`.

mod decode;
mod encode;
mod types;

pub use decode::decode_fields_to_model_binary;
pub use encode::encode_model_binary_to_fields;
pub use types::{IndexedBinaryCodecError, IndexedFields};
