//! JSON utility functions.
//!
//! Direct port of `json/util.ts` from upstream.

use super::error::JsonError;

/// Find the position of the closing `"` of a JSON string starting at `x`.
///
/// `x` must point to the first character after the opening `"`. The returned
/// index is the position of the closing `"` (exclusive of the contents).
///
/// Handles backslash escaping: `\"` inside the string does not terminate it.
pub fn find_ending_quote(data: &[u8], mut x: usize) -> Result<usize, JsonError> {
    let len = data.len();
    let mut prev: u8 = 0;
    while x < len {
        let ch = data[x];
        if ch == b'"' && prev != b'\\' {
            return Ok(x);
        }
        // double-backslash cancels the escape
        if ch == b'\\' && prev == b'\\' {
            prev = 0;
        } else {
            prev = ch;
        }
        x += 1;
    }
    Err(JsonError::Invalid(x))
}
