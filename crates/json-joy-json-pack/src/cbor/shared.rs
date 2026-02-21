//! Shared convenience wrappers for CBOR encode/decode.
//!
//! Upstream reference: `json-pack/src/cbor/shared.ts`

use crate::PackValue;

use super::{CborDecoder, CborEncoder, CborError};

/// Encode a [`PackValue`] into CBOR bytes.
pub fn encode(data: &PackValue) -> Vec<u8> {
    let mut encoder = CborEncoder::new();
    encoder.encode(data)
}

/// Decode CBOR bytes into a [`PackValue`].
pub fn decode(blob: &[u8]) -> Result<PackValue, CborError> {
    let decoder = CborDecoder::new();
    decoder.decode(blob)
}
