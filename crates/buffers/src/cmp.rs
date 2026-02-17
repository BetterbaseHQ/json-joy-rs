//! Byte slice comparison utilities.

/// Compares two byte slices for equality.
///
/// # Example
///
/// ```
/// use json_joy_buffers::cmp_uint8_array;
///
/// assert!(cmp_uint8_array(&[1, 2, 3], &[1, 2, 3]));
/// assert!(!cmp_uint8_array(&[1, 2, 3], &[1, 2, 4]));
/// assert!(!cmp_uint8_array(&[1, 2], &[1, 2, 3]));
/// ```
pub fn cmp_uint8_array(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a == b
}

/// Compares two byte slices lexicographically by byte values.
///
/// Returns a negative number if `a` is less than `b`, a positive number if `a`
/// is greater than `b`, or 0 if `a` is equal to `b`.
///
/// # Example
///
/// ```
/// use json_joy_buffers::cmp_uint8_array2;
///
/// assert!(cmp_uint8_array2(&[1, 2], &[1, 2, 3]) < 0);
/// assert!(cmp_uint8_array2(&[1, 2, 3], &[1, 2]) > 0);
/// assert_eq!(cmp_uint8_array2(&[1, 2, 3], &[1, 2, 3]), 0);
/// assert!(cmp_uint8_array2(&[1, 2, 3], &[1, 3, 2]) < 0);
/// ```
pub fn cmp_uint8_array2(a: &[u8], b: &[u8]) -> i32 {
    let len1 = a.len() as i64;
    let len2 = b.len() as i64;
    let len = a.len().min(b.len());

    for i in 0..len {
        let diff = a[i] as i32 - b[i] as i32;
        if diff != 0 {
            return diff;
        }
    }
    (len1 - len2) as i32
}

/// Compares two byte slices, first by length, then by each byte.
///
/// Returns a negative number if `a` is less than `b`, a positive number if `a`
/// is greater than `b`, or 0 if `a` is equal to `b`.
///
/// # Example
///
/// ```
/// use json_joy_buffers::cmp_uint8_array3;
///
/// assert!(cmp_uint8_array3(&[1, 2], &[1, 2, 3]) < 0);
/// assert!(cmp_uint8_array3(&[1, 2, 3], &[1, 2]) > 0);
/// assert_eq!(cmp_uint8_array3(&[1, 2, 3], &[1, 2, 3]), 0);
/// ```
pub fn cmp_uint8_array3(a: &[u8], b: &[u8]) -> i32 {
    let len1 = a.len() as i64;
    let len2 = b.len() as i64;
    let diff = (len1 - len2) as i32;
    if diff != 0 {
        return diff;
    }
    for i in 0..a.len() {
        let diff = a[i] as i32 - b[i] as i32;
        if diff != 0 {
            return diff;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmp_uint8_array() {
        assert!(cmp_uint8_array(&[1, 2, 3], &[1, 2, 3]));
        assert!(!cmp_uint8_array(&[1, 2, 3], &[1, 2, 4]));
        assert!(!cmp_uint8_array(&[1, 2], &[1, 2, 3]));
        assert!(cmp_uint8_array(&[], &[]));
    }

    #[test]
    fn test_cmp_uint8_array2() {
        assert_eq!(cmp_uint8_array2(&[1, 2, 3], &[1, 2, 3]), 0);
        assert!(cmp_uint8_array2(&[1, 2, 3], &[1, 3, 2]) < 0);
        assert!(cmp_uint8_array2(&[1, 3, 2], &[1, 2, 3]) > 0);
        assert!(cmp_uint8_array2(&[1, 2], &[1, 2, 3]) < 0);
        assert!(cmp_uint8_array2(&[1, 2, 3], &[1, 2]) > 0);
    }

    #[test]
    fn test_cmp_uint8_array3() {
        assert_eq!(cmp_uint8_array3(&[1, 2, 3], &[1, 2, 3]), 0);
        assert!(cmp_uint8_array3(&[1, 2], &[1, 2, 3]) < 0);
        assert!(cmp_uint8_array3(&[1, 2, 3], &[1, 2]) > 0);
    }
}
