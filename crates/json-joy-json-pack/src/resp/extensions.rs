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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resp_push_creates_extension_with_tag_1() {
        let elements = vec![PackValue::Str("hello".into()), PackValue::Integer(42)];
        let result = resp_push(elements.clone());
        match result {
            PackValue::Extension(ext) => {
                assert_eq!(ext.tag, 1);
                assert!(matches!(*ext.val, PackValue::Array(_)));
            }
            _ => panic!("Expected Extension variant"),
        }
    }

    #[test]
    fn test_resp_push_empty() {
        let result = resp_push(vec![]);
        match result {
            PackValue::Extension(ext) => {
                assert_eq!(ext.tag, 1);
                assert_eq!(*ext.val, PackValue::Array(vec![]));
            }
            _ => panic!("Expected Extension variant"),
        }
    }

    #[test]
    fn test_resp_attributes_creates_extension_with_tag_2() {
        let fields = vec![("key".into(), PackValue::Str("value".into()))];
        let result = resp_attributes(fields);
        match result {
            PackValue::Extension(ext) => {
                assert_eq!(ext.tag, 2);
                assert!(matches!(*ext.val, PackValue::Object(_)));
            }
            _ => panic!("Expected Extension variant"),
        }
    }

    #[test]
    fn test_resp_attributes_empty() {
        let result = resp_attributes(vec![]);
        match result {
            PackValue::Extension(ext) => {
                assert_eq!(ext.tag, 2);
                assert_eq!(*ext.val, PackValue::Object(vec![]));
            }
            _ => panic!("Expected Extension variant"),
        }
    }

    #[test]
    fn test_resp_verbatim_string_creates_extension_with_tag_3() {
        let result = resp_verbatim_string("txt:hello".into());
        match result {
            PackValue::Extension(ext) => {
                assert_eq!(ext.tag, 3);
                assert_eq!(*ext.val, PackValue::Str("txt:hello".into()));
            }
            _ => panic!("Expected Extension variant"),
        }
    }

    #[test]
    fn test_is_resp_push_true() {
        let ext = JsonPackExtension::new(1, PackValue::Null);
        assert!(is_resp_push(&ext));
    }

    #[test]
    fn test_is_resp_push_false() {
        let ext = JsonPackExtension::new(2, PackValue::Null);
        assert!(!is_resp_push(&ext));
    }

    #[test]
    fn test_is_resp_attributes_true() {
        let ext = JsonPackExtension::new(2, PackValue::Null);
        assert!(is_resp_attributes(&ext));
    }

    #[test]
    fn test_is_resp_attributes_false() {
        let ext = JsonPackExtension::new(0, PackValue::Null);
        assert!(!is_resp_attributes(&ext));
    }

    #[test]
    fn test_is_resp_verbatim_string_true() {
        let ext = JsonPackExtension::new(3, PackValue::Null);
        assert!(is_resp_verbatim_string(&ext));
    }

    #[test]
    fn test_is_resp_verbatim_string_false() {
        let ext = JsonPackExtension::new(1, PackValue::Null);
        assert!(!is_resp_verbatim_string(&ext));
    }
}
