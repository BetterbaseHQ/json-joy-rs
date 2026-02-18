//! [`JsonPackValue`] — pre-encoded binary value wrapper.
//!
//! Mirrors `JsonPackValue.ts` from upstream.

/// A wrapper for a pre-encoded MessagePack or CBOR value.
///
/// The contents of `val` will be written as-is to the output document.
/// Also serves as CBOR simple value container (val is the simple value number).
#[derive(Debug, Clone, PartialEq)]
pub struct JsonPackValue {
    pub val: Vec<u8>,
}

impl JsonPackValue {
    pub fn new(val: Vec<u8>) -> Self {
        Self { val }
    }
}
