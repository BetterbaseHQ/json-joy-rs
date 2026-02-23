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
}
