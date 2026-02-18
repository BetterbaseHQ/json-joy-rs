//! Backward-compatible conversion helpers.
//!
//! Provides `json_to_cbor`, `cbor_to_json`, `cbor_to_json_owned` using our
//! own `PackValue` type instead of `ciborium::Value`.

use serde_json::Value as JsonValue;

use super::error::CborError;
use crate::PackValue;

/// Convert `serde_json::Value` to `PackValue`.
pub fn json_to_cbor(v: &JsonValue) -> PackValue {
    PackValue::from(v.clone())
}

/// Convert `PackValue` to `serde_json::Value`.
pub fn cbor_to_json(v: &PackValue) -> Result<JsonValue, CborError> {
    Ok(cbor_to_json_owned(v.clone()))
}

/// Convert owned `PackValue` to `serde_json::Value`.
pub fn cbor_to_json_owned(v: PackValue) -> JsonValue {
    super::decoder::pack_to_json(v)
}
