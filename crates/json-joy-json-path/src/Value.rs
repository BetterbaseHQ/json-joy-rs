//! Upstream-parity query value node (`Value.ts`).

use crate::{NormalizedPath, NormalizedPathSegment, PathComponent};
use serde_json::Value as JsonValue;

/// A matched JSONPath value with normalized path metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    pub data: JsonValue,
    path: NormalizedPath,
    pointer: String,
}

impl Value {
    pub(crate) fn from_query_path(components: &[PathComponent], data: JsonValue) -> Self {
        let mut path = Vec::with_capacity(components.len() + 1);
        path.push(NormalizedPathSegment::Key("$".to_string()));

        for component in components {
            match component {
                PathComponent::Key(key) => path.push(NormalizedPathSegment::Key(key.clone())),
                PathComponent::Index(index) => {
                    path.push(NormalizedPathSegment::Index(*index as i64))
                }
            }
        }

        let pointer = pointer_from_path(&path);
        Self {
            data,
            path,
            pointer,
        }
    }

    pub fn path(&self) -> NormalizedPath {
        self.path.clone()
    }

    pub fn pointer(&self) -> String {
        self.pointer.clone()
    }
}

fn pointer_from_path(path: &[NormalizedPathSegment]) -> String {
    let mut out = String::new();
    for (idx, segment) in path.iter().enumerate() {
        match segment {
            NormalizedPathSegment::Key(key) if idx == 0 && key == "$" => out.push('$'),
            NormalizedPathSegment::Key(key) => {
                let encoded = serde_json::to_string(key).unwrap_or_else(|_| format!("\"{key}\""));
                let inner = encoded
                    .strip_prefix('"')
                    .and_then(|s| s.strip_suffix('"'))
                    .unwrap_or(&encoded);
                out.push_str("['");
                out.push_str(inner);
                out.push_str("']");
            }
            NormalizedPathSegment::Index(index) => {
                out.push('[');
                out.push_str(&index.to_string());
                out.push(']');
            }
        }
    }
    if out.is_empty() {
        "$".to_string()
    } else {
        out
    }
}
