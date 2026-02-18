//! Top-level constants for json-pack.
//!
//! Mirrors `constants.ts` from upstream.

/// Binary encoding format identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodingFormat {
    Cbor = 0,
    MsgPack = 1,
    Json = 2,
}
