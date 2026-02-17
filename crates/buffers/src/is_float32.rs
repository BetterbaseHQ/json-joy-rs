//! Float32 precision checking utility.

/// Checks if a number can be exactly represented as a 32-bit floating point value.
///
/// Returns `true` if the number can be represented without precision loss
/// when stored as an `f32`, `false` otherwise.
///
/// # Example
///
/// ```
/// use json_joy_buffers::is_float32;
///
/// assert!(is_float32(1.0));
/// assert!(is_float32(0.5));
/// assert!(is_float32(0.25));
/// assert!(!is_float32(0.1));  // 0.1 cannot be exactly represented in f32
/// ```
pub fn is_float32(n: f64) -> bool {
    // Convert to f32 and back, check if the value is preserved
    (n as f32) as f64 == n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_float32() {
        assert!(is_float32(1.0));
        assert!(is_float32(0.5));
        assert!(is_float32(0.25));
        assert!(is_float32(-1.0));
        assert!(is_float32(0.0));
    }
}
