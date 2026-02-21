//! json-binary branded string aliases.
//!
//! Upstream reference: `json-pack/src/json-binary/types.ts`

/// Base64 payload string alias.
pub type Base64String = String;

/// Data URI containing base64-encoded binary payload.
pub type BinaryString = String;

/// Data URI containing CBOR payload.
pub type CborString = String;

/// Data URI containing MessagePack payload.
pub type MsgpackString = String;
