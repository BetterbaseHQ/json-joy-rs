//! URL-safe base64 decoding function.

use crate::create_from_base64;

/// Decodes a URL-safe base64 string to bytes.
///
/// This expects the URL-safe alphabet (`-` and `_` instead of `+` and `/`)
/// and handles missing padding automatically.
///
/// # Arguments
///
/// * `encoded` - The URL-safe base64-encoded string to decode.
///
/// # Returns
///
/// The decoded bytes, or an error if the input is invalid.
///
/// # Example
///
/// ```
/// use json_joy_base64::from_base64_url;
///
/// let decoded = from_base64_url("aGVsbG8gd29ybGQ").unwrap();
/// assert_eq!(decoded, b"hello world");
/// ```
pub fn from_base64_url(encoded: &str) -> Result<Vec<u8>, crate::Base64Error> {
    let decoder = create_from_base64(
        Some("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_"),
        true, // no_padding - will add padding if needed
    )?;
    decoder(encoded)
}
