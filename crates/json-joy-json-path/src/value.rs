//! Query value wrapper with normalized path helpers.

use serde_json::Value;

use crate::types::PathComponent;

/// A matched JSON value paired with its normalized path components.
#[derive(Debug, Clone, PartialEq)]
pub struct ValueNode<'a> {
    pub data: &'a Value,
    pub path: Vec<PathComponent>,
}

impl<'a> ValueNode<'a> {
    pub fn new(data: &'a Value, path: Vec<PathComponent>) -> Self {
        Self { data, path }
    }

    /// Return normalized path components from root to this node.
    pub fn normalized_path(&self) -> &[PathComponent] {
        &self.path
    }

    /// Return this node path as a JSON Pointer.
    pub fn pointer(&self) -> String {
        if self.path.is_empty() {
            return String::new();
        }

        let mut out = String::new();
        for component in &self.path {
            out.push('/');
            match component {
                PathComponent::Key(key) => out.push_str(&escape_json_pointer_key(key)),
                PathComponent::Index(index) => out.push_str(&index.to_string()),
            }
        }
        out
    }

    /// Return this node path as a bracketed JSONPath string.
    pub fn json_path(&self) -> String {
        let mut out = String::from("$");
        for component in &self.path {
            match component {
                PathComponent::Key(key) => {
                    out.push_str("['");
                    out.push_str(&escape_single_quoted(key));
                    out.push_str("']");
                }
                PathComponent::Index(index) => {
                    out.push('[');
                    out.push_str(&index.to_string());
                    out.push(']');
                }
            }
        }
        out
    }
}

fn escape_json_pointer_key(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

fn escape_single_quoted(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn pointer_formats_root_and_nested_paths() {
        let doc = json!({"a": [{"b/c": 1}]});
        let root = ValueNode::new(&doc, vec![]);
        assert_eq!(root.pointer(), "");

        let nested = ValueNode::new(
            &doc["a"][0]["b/c"],
            vec![
                PathComponent::Key("a".into()),
                PathComponent::Index(0),
                PathComponent::Key("b/c".into()),
            ],
        );
        assert_eq!(nested.pointer(), "/a/0/b~1c");
    }

    #[test]
    fn json_path_formats_bracket_notation() {
        let doc = json!(1);
        let node = ValueNode::new(
            &doc,
            vec![
                PathComponent::Key("hello world".into()),
                PathComponent::Index(2),
                PathComponent::Key("quote'key".into()),
            ],
        );
        assert_eq!(node.json_path(), "$['hello world'][2]['quote\\'key']");
    }
}
