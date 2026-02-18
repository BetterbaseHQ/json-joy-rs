//! RESP3 extension value types.
//!
//! Upstream reference: `json-pack/src/resp/extensions.ts`

use crate::{JsonPackExtension, PackValue};

/// RESP3 Push message (server-to-client unsolicited messages).
/// Encoded as `PackValue::Extension` with tag `1`.
pub fn resp_push(elements: Vec<PackValue>) -> PackValue {
    PackValue::Extension(Box::new(JsonPackExtension::new(
        1,
        PackValue::Array(elements),
    )))
}

/// RESP3 Attributes map (metadata attached to any response).
/// Encoded as `PackValue::Extension` with tag `2`.
pub fn resp_attributes(fields: Vec<(String, PackValue)>) -> PackValue {
    PackValue::Extension(Box::new(JsonPackExtension::new(
        2,
        PackValue::Object(fields),
    )))
}

/// RESP3 Verbatim string (typed string with encoding prefix).
/// Encoded as `PackValue::Extension` with tag `3`.
pub fn resp_verbatim_string(s: String) -> PackValue {
    PackValue::Extension(Box::new(JsonPackExtension::new(3, PackValue::Str(s))))
}

/// Returns `true` if the given tag belongs to a RESP extension.
pub fn is_resp_push(ext: &JsonPackExtension) -> bool {
    ext.tag == 1
}
pub fn is_resp_attributes(ext: &JsonPackExtension) -> bool {
    ext.tag == 2
}
pub fn is_resp_verbatim_string(ext: &JsonPackExtension) -> bool {
    ext.tag == 3
}
