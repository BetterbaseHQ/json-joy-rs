//! Byte slice copy utility.

/// Creates a copy of a byte slice.
///
/// # Example
///
/// ```
/// use json_joy_buffers::copy_slice;
///
/// let original = vec![1, 2, 3];
/// let duplicate = copy_slice(&original);
/// assert_eq!(original, duplicate);
/// ```
pub fn copy_slice(arr: &[u8]) -> Vec<u8> {
    arr.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy() {
        let original = vec![1, 2, 3];
        let duplicate = copy_slice(&original);
        assert_eq!(original, duplicate);
        assert_ne!(original.as_ptr(), duplicate.as_ptr());
    }
}
