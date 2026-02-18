//! UBJSON (Universal Binary JSON) encoding/decoding.
//!
//! Upstream: `packages/json-pack/src/ubjson/`

mod decoder;
mod encoder;
mod error;

pub use decoder::UbjsonDecoder;
pub use encoder::UbjsonEncoder;
pub use error::UbjsonError;
