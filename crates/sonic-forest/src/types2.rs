//! Upstream path-parity shim for `src/types2.ts`.
//!
//! Rust keeps both traits in `types.rs` for idiomatic colocated trait defs;
//! this file re-exports `Node2` under the upstream path.

pub use crate::types::Node2;
