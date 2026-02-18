//! json-binary: embed binary data in JSON using data URI strings.
//!
//! Upstream: `packages/json-pack/src/json-binary/`

pub mod constants;
mod codec;

pub use codec::{parse, stringify, stringify_binary, unwrap_binary, wrap_binary};
