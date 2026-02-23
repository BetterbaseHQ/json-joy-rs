/// Performs a deep equality check between two values that may contain binary data.
///
/// This function extends [`json_joy_json_equal::deep_equal`] to support
/// comparing binary data (Vec<u8>) via the [`JsonBinary`] enum.
///
/// # Examples
///
/// ```
/// use json_joy_util::json_equal::{deep_equal_binary, JsonBinary};
///
/// let a = JsonBinary::Binary(vec![1, 2, 3]);
/// let b = JsonBinary::Binary(vec![1, 2, 3]);
/// let c = JsonBinary::Binary(vec![1, 2, 4]);
///
/// assert!(deep_equal_binary(&a, &b));
/// assert!(!deep_equal_binary(&a, &c));
/// ```
pub fn deep_equal_binary(
    a: &crate::json_clone::JsonBinary,
    b: &crate::json_clone::JsonBinary,
) -> bool {
    use crate::json_clone::JsonBinary;

    match (a, b) {
        (JsonBinary::Null, JsonBinary::Null) => true,
        (JsonBinary::Bool(a), JsonBinary::Bool(b)) => a == b,
        (JsonBinary::Number(a), JsonBinary::Number(b)) => a == b,
        (JsonBinary::String(a), JsonBinary::String(b)) => a == b,

        (JsonBinary::Binary(a), JsonBinary::Binary(b)) => a == b,

        (JsonBinary::Array(arr_a), JsonBinary::Array(arr_b)) => {
            if arr_a.len() != arr_b.len() {
                return false;
            }
            for i in 0..arr_a.len() {
                if !deep_equal_binary(&arr_a[i], &arr_b[i]) {
                    return false;
                }
            }
            true
        }

        (JsonBinary::Object(obj_a), JsonBinary::Object(obj_b)) => {
            if obj_a.len() != obj_b.len() {
                return false;
            }
            for (key, val_a) in obj_a {
                match obj_b.get(key) {
                    Some(val_b) => {
                        if !deep_equal_binary(val_a, val_b) {
                            return false;
                        }
                    }
                    None => return false,
                }
            }
            true
        }

        // Different types are never equal
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json_clone::JsonBinary;

    #[test]
    fn test_binary_equal() {
        let a = JsonBinary::Binary(vec![1, 2, 3]);
        let b = JsonBinary::Binary(vec![1, 2, 3]);
        assert!(deep_equal_binary(&a, &b));
    }

    #[test]
    fn test_binary_not_equal() {
        let a = JsonBinary::Binary(vec![1, 2, 3]);
        let b = JsonBinary::Binary(vec![1, 2, 4]);
        assert!(!deep_equal_binary(&a, &b));
    }

    #[test]
    fn test_empty_binary_equal() {
        let a = JsonBinary::Binary(vec![]);
        let b = JsonBinary::Binary(vec![]);
        assert!(deep_equal_binary(&a, &b));
    }

    #[test]
    fn test_binary_and_array_not_equal() {
        let a = JsonBinary::Binary(vec![]);
        let b = JsonBinary::Array(vec![]);
        assert!(!deep_equal_binary(&a, &b));
    }

    // --- Null ---

    #[test]
    fn test_null_equal() {
        assert!(deep_equal_binary(&JsonBinary::Null, &JsonBinary::Null));
    }

    #[test]
    fn test_null_not_equal_to_bool() {
        assert!(!deep_equal_binary(
            &JsonBinary::Null,
            &JsonBinary::Bool(false)
        ));
    }

    // --- Bool ---

    #[test]
    fn test_bool_equal() {
        assert!(deep_equal_binary(
            &JsonBinary::Bool(true),
            &JsonBinary::Bool(true)
        ));
    }

    #[test]
    fn test_bool_not_equal() {
        assert!(!deep_equal_binary(
            &JsonBinary::Bool(true),
            &JsonBinary::Bool(false)
        ));
    }

    // --- Number ---

    #[test]
    fn test_number_equal() {
        let n1 = JsonBinary::Number(serde_json::Number::from(42));
        let n2 = JsonBinary::Number(serde_json::Number::from(42));
        assert!(deep_equal_binary(&n1, &n2));
    }

    #[test]
    fn test_number_not_equal() {
        let n1 = JsonBinary::Number(serde_json::Number::from(1));
        let n2 = JsonBinary::Number(serde_json::Number::from(2));
        assert!(!deep_equal_binary(&n1, &n2));
    }

    // --- String ---

    #[test]
    fn test_string_equal() {
        let a = JsonBinary::String("hello".into());
        let b = JsonBinary::String("hello".into());
        assert!(deep_equal_binary(&a, &b));
    }

    #[test]
    fn test_string_not_equal() {
        let a = JsonBinary::String("hello".into());
        let b = JsonBinary::String("world".into());
        assert!(!deep_equal_binary(&a, &b));
    }

    // --- Array ---

    #[test]
    fn test_array_equal() {
        let a = JsonBinary::Array(vec![JsonBinary::Null, JsonBinary::Bool(true)]);
        let b = JsonBinary::Array(vec![JsonBinary::Null, JsonBinary::Bool(true)]);
        assert!(deep_equal_binary(&a, &b));
    }

    #[test]
    fn test_array_different_lengths() {
        let a = JsonBinary::Array(vec![JsonBinary::Null]);
        let b = JsonBinary::Array(vec![JsonBinary::Null, JsonBinary::Null]);
        assert!(!deep_equal_binary(&a, &b));
    }

    #[test]
    fn test_array_different_elements() {
        let a = JsonBinary::Array(vec![JsonBinary::Bool(true)]);
        let b = JsonBinary::Array(vec![JsonBinary::Bool(false)]);
        assert!(!deep_equal_binary(&a, &b));
    }

    #[test]
    fn test_array_empty() {
        let a = JsonBinary::Array(vec![]);
        let b = JsonBinary::Array(vec![]);
        assert!(deep_equal_binary(&a, &b));
    }

    // --- Object ---

    #[test]
    fn test_object_equal() {
        use std::collections::BTreeMap;
        let mut m1 = BTreeMap::new();
        m1.insert("a".into(), JsonBinary::Null);
        m1.insert("b".into(), JsonBinary::Bool(true));
        let mut m2 = BTreeMap::new();
        m2.insert("a".into(), JsonBinary::Null);
        m2.insert("b".into(), JsonBinary::Bool(true));
        assert!(deep_equal_binary(
            &JsonBinary::Object(m1),
            &JsonBinary::Object(m2)
        ));
    }

    #[test]
    fn test_object_different_sizes() {
        use std::collections::BTreeMap;
        let mut m1 = BTreeMap::new();
        m1.insert("a".into(), JsonBinary::Null);
        let m2 = BTreeMap::new();
        assert!(!deep_equal_binary(
            &JsonBinary::Object(m1),
            &JsonBinary::Object(m2)
        ));
    }

    #[test]
    fn test_object_different_values() {
        use std::collections::BTreeMap;
        let mut m1 = BTreeMap::new();
        m1.insert("a".into(), JsonBinary::Bool(true));
        let mut m2 = BTreeMap::new();
        m2.insert("a".into(), JsonBinary::Bool(false));
        assert!(!deep_equal_binary(
            &JsonBinary::Object(m1),
            &JsonBinary::Object(m2)
        ));
    }

    #[test]
    fn test_object_missing_key() {
        use std::collections::BTreeMap;
        let mut m1 = BTreeMap::new();
        m1.insert("a".into(), JsonBinary::Null);
        let mut m2 = BTreeMap::new();
        m2.insert("b".into(), JsonBinary::Null);
        assert!(!deep_equal_binary(
            &JsonBinary::Object(m1),
            &JsonBinary::Object(m2)
        ));
    }

    // --- Nested ---

    #[test]
    fn test_nested_array_in_object() {
        use std::collections::BTreeMap;
        let inner = JsonBinary::Array(vec![JsonBinary::Binary(vec![1, 2])]);
        let mut m1 = BTreeMap::new();
        m1.insert("arr".into(), inner.clone());
        let mut m2 = BTreeMap::new();
        m2.insert("arr".into(), inner);
        assert!(deep_equal_binary(
            &JsonBinary::Object(m1),
            &JsonBinary::Object(m2)
        ));
    }

    // --- Cross-type ---

    #[test]
    fn test_string_vs_number() {
        assert!(!deep_equal_binary(
            &JsonBinary::String("42".into()),
            &JsonBinary::Number(serde_json::Number::from(42))
        ));
    }

    #[test]
    fn test_null_vs_string() {
        assert!(!deep_equal_binary(
            &JsonBinary::Null,
            &JsonBinary::String(String::new())
        ));
    }
}
