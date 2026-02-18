//! Amazon Ion binary encoder/decoder.
//!
//! Upstream reference: `json-pack/src/ion/`

pub mod constants;
pub mod decoder;
pub mod encoder;
pub mod symbols;

pub use decoder::{IonDecodeError, IonDecoder};
pub use encoder::IonEncoder;
pub use symbols::IonSymbols;

/// Alias for the fast Ion encoder (matches upstream `IonEncoderFast` class name).
pub type IonEncoderFast = IonEncoder;
