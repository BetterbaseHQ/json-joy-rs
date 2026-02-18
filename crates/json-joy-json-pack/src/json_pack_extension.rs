//! [`JsonPackExtension`] — wrapper for MessagePack extensions and CBOR tags.
//!
//! Mirrors `JsonPackExtension.ts` from upstream.

use crate::PackValue;

/// A wrapper for MessagePack extension or CBOR tag value.
///
/// When an encoder encounters a [`JsonPackExtension`] it will encode it as a
/// MessagePack extension or CBOR tag. Likewise, the decoder will decode
/// extensions into [`JsonPackExtension`].
#[derive(Debug, Clone, PartialEq)]
pub struct JsonPackExtension {
    pub tag: u64,
    pub val: Box<PackValue>,
}

impl JsonPackExtension {
    pub fn new(tag: u64, val: PackValue) -> Self {
        Self {
            tag,
            val: Box::new(val),
        }
    }
}
