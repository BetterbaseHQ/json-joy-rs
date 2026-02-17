//! String encoding utilities for ASCII and UTF-8.

/// Converts a string to a vector of ASCII bytes.
///
/// Each character is converted to its ASCII byte value.
/// Characters with code points > 127 will have their lower 8 bits used.
///
/// # Example
///
/// ```
/// use json_joy_buffers::ascii;
///
/// assert_eq!(ascii("hello"), vec![b'h', b'e', b'l', b'l', b'o']);
/// ```
pub fn ascii(s: &str) -> Vec<u8> {
    s.bytes().collect()
}

/// Converts a string to a vector of UTF-8 bytes.
///
/// # Example
///
/// ```
/// use json_joy_buffers::utf8;
///
/// assert_eq!(utf8("hello"), b"hello".to_vec());
/// assert_eq!(utf8("日本"), vec![0xE6, 0x97, 0xA5, 0xE6, 0x9C, 0xAC]);
/// ```
pub fn utf8(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii() {
        assert_eq!(ascii("hello"), b"hello".to_vec());
        assert_eq!(ascii(""), Vec::<u8>::new());
    }

    #[test]
    fn test_utf8() {
        assert_eq!(utf8("hello"), b"hello".to_vec());
        assert_eq!(utf8(""), Vec::<u8>::new());
    }
}
