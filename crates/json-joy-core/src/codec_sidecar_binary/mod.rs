//! Sidecar binary codec aligned to upstream `json-crdt/codec/sidecar/binary`.

mod decode;
mod encode;
mod types;

pub use decode::decode_sidecar_to_model_binary;
pub use encode::encode_model_binary_to_sidecar;
pub use types::SidecarBinaryCodecError;
