//! Binary codegen adapters (CBOR, MessagePack, JSON text bytes).
//!
//! Upstream reference:
//! - `json-type/src/codegen/binary/`
//!
//! This port keeps upstream naming and exposes runtime encoder builders backed
//! by the Rust `json-pack` encoders.

use std::sync::Arc;

use serde_json::Value;
use thiserror::Error;

use crate::codegen::json::{JsonTextCodegen, JsonTextCodegenError};
use crate::type_def::TypeNode;
use json_joy_json_pack::cbor::encode_json_to_cbor_bytes;
use json_joy_json_pack::msgpack::MsgPackEncoderFast;
use json_joy_json_pack::PackValue;

/// A compiled binary encoder function.
pub type BinaryEncoderFn = Arc<dyn Fn(&Value) -> Result<Vec<u8>, BinaryCodegenError> + Send + Sync>;

/// Binary codegen errors.
#[derive(Debug, Error)]
pub enum BinaryCodegenError {
    #[error("{0}")]
    JsonText(#[from] JsonTextCodegenError),
    #[error("Invalid generated JSON text: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("Failed to encode CBOR: {0}")]
    Cbor(String),
}

/// Runtime equivalent of upstream `CborCodegen`.
pub struct CborCodegen;

impl CborCodegen {
    pub fn get(type_: &TypeNode) -> BinaryEncoderFn {
        let encode_json = JsonTextCodegen::get(type_);
        Arc::new(move |value: &Value| {
            let json_text = encode_json(value)?;
            let json_value: Value = serde_json::from_str(&json_text)?;
            encode_json_to_cbor_bytes(&json_value)
                .map_err(|e| BinaryCodegenError::Cbor(e.to_string()))
        })
    }
}

/// Runtime equivalent of upstream `MsgPackCodegen`.
pub struct MsgPackCodegen;

impl MsgPackCodegen {
    pub fn get(type_: &TypeNode) -> BinaryEncoderFn {
        let encode_json = JsonTextCodegen::get(type_);
        Arc::new(move |value: &Value| {
            let json_text = encode_json(value)?;
            let json_value: Value = serde_json::from_str(&json_text)?;
            let pack = PackValue::from(json_value);
            let mut encoder = MsgPackEncoderFast::new();
            Ok(encoder.encode(&pack))
        })
    }
}

/// Runtime equivalent of upstream `JsonCodegen`.
pub struct JsonCodegen;

impl JsonCodegen {
    pub fn get(type_: &TypeNode) -> BinaryEncoderFn {
        let encode_json = JsonTextCodegen::get(type_);
        Arc::new(move |value: &Value| {
            let text = encode_json(value)?;
            Ok(text.into_bytes())
        })
    }
}
