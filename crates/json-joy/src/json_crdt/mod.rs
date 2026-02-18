//! JSON CRDT document model and node types.
//!
//! Mirrors `packages/json-joy/src/json-crdt/`.
//!
//! This module provides:
//! - A simplified in-memory CRDT document model ([`model::Model`])
//! - All JSON CRDT node types ([`nodes`])
//! - The UNDEFINED_TS / ORIGIN sentinel constants ([`constants`])

pub mod constants;
pub mod nodes;
pub mod model;
pub mod equal;
pub mod schema;
pub mod extensions;
pub mod log;
pub mod draft;
pub mod codec;
pub mod partial_edit;
pub mod json_patch_apply;

pub use model::Model;
pub use model::ModelApi;
pub use nodes::{CrdtNode, NodeIndex};
pub use constants::{ORIGIN, UNDEFINED_TS};
pub use extensions::{AnyExtension, ExtApi, ExtNode, Extensions};
