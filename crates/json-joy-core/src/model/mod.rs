//! JSON CRDT Model binary handling (M2).
//!
//! Compatibility notes:
//! - This implementation decodes logical-clock model binaries into materialized
//!   JSON views for fixture-covered data types.
//! - Malformed payload handling is intentionally fixture-driven to match
//!   upstream `json-joy@17.67.0` behavior (including permissive quirks).

use json_joy_json_pack::PackValue;
use serde_json::{Map, Number, Value};
use thiserror::Error;

include!("error.rs");
include!("view.rs");
include!("decode.rs");
include!("encode.rs");
