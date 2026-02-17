//! Standard base64 decoding function.

use crate::create_from_base64;

/// Decodes a base64 string to bytes.
///
/// # Arguments
///
/// * `encoded` - The base64-encoded string to decode.
///
/// # Returns
///
/// The decoded bytes, or an error if the input is invalid.
///
/// # Example
///
/// ```
/// use json_joy_base64::from_base64;
///
/// let decoded = from_base64("aGVsbG8gd29ybGQ=").unwrap();
/// assert_eq!(decoded, b"hello world");
/// ```
pub fn from_base64(encoded: &str) -> Result<Vec<u8>, crate::Base64Error> {
    let decoder = create_from_base64(None, false)?;
    decoder(encoded)
}
