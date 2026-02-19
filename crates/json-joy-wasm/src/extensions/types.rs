use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub type JsonObject = Map<String, Value>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PmNodeType {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PmMark {
    #[serde(rename = "type")]
    pub mark_type: PmNodeType,
    #[serde(default)]
    pub attrs: Option<JsonObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PmFragment {
    #[serde(default)]
    pub content: Vec<PmNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PmNode {
    #[serde(rename = "type")]
    pub node_type: PmNodeType,
    #[serde(default)]
    pub attrs: Option<JsonObject>,
    #[serde(default)]
    pub content: Option<PmFragment>,
    #[serde(default)]
    pub marks: Option<Vec<PmMark>>,
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SlateTextNode {
    pub text: String,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SlateElementNode {
    #[serde(rename = "type")]
    pub element_type: String,
    #[serde(default)]
    pub children: Vec<SlateNode>,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum SlateNode {
    Element(SlateElementNode),
    Text(SlateTextNode),
}

pub type SlateDocument = Vec<SlateNode>;

pub type QuillDeltaAttributes = JsonObject;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum QuillInsert {
    Text(String),
    Embed(JsonObject),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuillOpInsert {
    pub insert: QuillInsert,
    #[serde(default)]
    pub attributes: Option<QuillDeltaAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuillOpDelete {
    pub delete: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuillOpRetain {
    pub retain: u64,
    #[serde(default)]
    pub attributes: Option<QuillDeltaAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum QuillOp {
    Insert(QuillOpInsert),
    Delete(QuillOpDelete),
    Retain(QuillOpRetain),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuillPatch {
    pub ops: Vec<QuillOp>,
}

#[cfg(test)]
mod tests {
    use super::{PmNode, QuillPatch, SlateDocument};

    #[test]
    fn parses_prosemirror_node_shape() {
        let s = r#"{"type":{"name":"doc"},"content":{"content":[{"type":{"name":"text"},"text":"x"}]}}"#;
        let node: PmNode = serde_json::from_str(s).unwrap();
        assert_eq!(node.node_type.name, "doc");
    }

    #[test]
    fn parses_slate_document_shape() {
        let s = r#"[{"type":"paragraph","children":[{"text":"hello","bold":true}]}]"#;
        let doc: SlateDocument = serde_json::from_str(s).unwrap();
        assert_eq!(doc.len(), 1);
    }

    #[test]
    fn parses_quill_patch_shape() {
        let s =
            r#"{"ops":[{"insert":"hello"},{"retain":2,"attributes":{"bold":true}},{"delete":1}]}"#;
        let patch: QuillPatch = serde_json::from_str(s).unwrap();
        assert_eq!(patch.ops.len(), 3);
    }
}
