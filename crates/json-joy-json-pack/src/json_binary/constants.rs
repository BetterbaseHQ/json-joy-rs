//! Data URI prefix constants for json-binary codec.
//!
//! Mirrors `json-binary/constants.ts` from upstream.

/// Binary data URI prefix: `data:application/octet-stream;base64,`
pub const BIN_URI_START: &str = "data:application/octet-stream;base64,";

/// MsgPack URI prefix (base): `data:application/msgpack;base64`
pub const MSGPACK_URI_HEADER: &str = "data:application/msgpack;base64";

/// MsgPack value URI prefix: `data:application/msgpack;base64,`
pub const MSGPACK_URI_START: &str = "data:application/msgpack;base64,";

/// MsgPack extension URI prefix: `data:application/msgpack;base64;ext=`
pub const MSGPACK_EXT_START: &str = "data:application/msgpack;base64;ext=";
