//! Record Marshalling (RM) framing format.
//!
//! Upstream reference: `json-pack/src/rm/`

mod decoder;
mod encoder;

pub use decoder::RmRecordDecoder;
pub use encoder::RmRecordEncoder;
