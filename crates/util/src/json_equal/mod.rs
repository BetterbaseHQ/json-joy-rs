//! JSON equality utilities.
//!
//! Provides deep equality comparison functions for JSON values.
//!
//! The core [`deep_equal`] function is re-exported from the standalone
//! `json-joy-json-equal` crate (mirroring the upstream extraction of
//! `@jsonjoy.com/json-equal` in v18.0.0). The binary-aware variant
//! [`deep_equal_binary`] remains here since it depends on [`JsonBinary`].

mod deep_equal;

// Re-export from the standalone json-equal crate for backward compatibility.
pub use json_joy_json_equal::deep_equal;

pub use deep_equal::deep_equal_binary;

// Re-export JsonBinary for convenience
pub use crate::json_clone::JsonBinary;
