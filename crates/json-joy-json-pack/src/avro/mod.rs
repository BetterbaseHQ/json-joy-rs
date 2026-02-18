//! Apache Avro encoder/decoder.
//!
//! Upstream reference: `json-pack/src/avro/`
//! Reference: Apache Avro 1.12.0 specification

pub mod decoder;
pub mod encoder;
pub mod schema_decoder;
pub mod schema_encoder;
pub mod types;

pub use decoder::{AvroDecodeError, AvroDecoder};
pub use encoder::AvroEncoder;
pub use schema_decoder::AvroSchemaDecoder;
pub use schema_encoder::{AvroEncodeError, AvroSchemaEncoder};
pub use types::{AvroField, AvroSchema, AvroValue};
