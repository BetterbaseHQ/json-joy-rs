//! Upstream path-parity shim for `src/splay/util.ts`.
//!
//! Rust keeps the position-tree splay implementation in `splay/mod.rs`;
//! this module re-exports that API so file layout mirrors upstream.

pub use super::{l_splay, ll_splay, lr_splay, r_splay, rl_splay, rr_splay, splay};
