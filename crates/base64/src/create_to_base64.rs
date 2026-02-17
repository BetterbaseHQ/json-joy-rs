//! Factory function for creating base64 encoders with custom alphabets.

use crate::constants::ALPHABET;
use crate::Base64Error;

/// Creates a base64 encoder function with a custom alphabet and padding character.
///
/// # Arguments
///
/// * `chars` - A 64-character string representing the base64 alphabet. Defaults to standard base64.
/// * `pad` - The padding character. Defaults to '='. Use empty string for no padding.
///
/// # Returns
///
/// A function that encodes a `&[u8]` to a base64 `String`.
///
/// # Errors
///
/// Returns an error if `chars` is not exactly 64 characters long.
///
/// # Example
///
/// ```
/// use json_joy_base64::create_to_base64;
///
/// let encode = create_to_base64(None, None).unwrap();
/// let result = encode(b"hello", 5);
/// assert_eq!(result, "aGVsbG8=");
/// ```
pub fn create_to_base64(
    chars: Option<&str>,
    pad: Option<&str>,
) -> Result<impl Fn(&[u8], usize) -> String, Base64Error> {
    let chars = chars.unwrap_or(ALPHABET);
    let pad = pad.unwrap_or("=");

    if chars.len() != 64 {
        return Err(Base64Error::InvalidCharSetLength);
    }

    // Build single-character lookup table
    let table: Vec<char> = chars.chars().collect();

    // Build two-character lookup table (4096 entries for all 2-char combinations)
    let mut table2: Vec<String> = Vec::with_capacity(4096);
    for c1 in &table {
        for c2 in &table {
            table2.push(format!("{}{}", c1, c2));
        }
    }

    let do_padding = pad.len() == 1;
    let e: String = pad.to_string();
    let ee: String = if do_padding {
        format!("{}{}", pad, pad)
    } else {
        String::new()
    };

    Ok(move |uint8: &[u8], length: usize| -> String {
        let mut out = String::with_capacity((length * 4 / 3) + 4);
        let extra_length = length % 3;
        let base_length = length - extra_length;

        let mut i = 0;
        while i < base_length {
            let o1 = uint8[i];
            let o2 = uint8[i + 1];
            let o3 = uint8[i + 2];
            let v1 = ((o1 as u16) << 4) | ((o2 as u16) >> 4);
            let v2 = (((o2 & 0b1111) as u16) << 8) | (o3 as u16);
            out.push_str(&table2[v1 as usize]);
            out.push_str(&table2[v2 as usize]);
            i += 3;
        }

        if extra_length == 0 {
            return out;
        }

        if extra_length == 1 {
            let o1 = uint8[base_length];
            out.push_str(&table2[(o1 as usize) << 4]);
            if do_padding {
                out.push_str(&ee);
            }
        } else {
            // extra_length == 2
            let o1 = uint8[base_length];
            let o2 = uint8[base_length + 1];
            let v1 = ((o1 as u16) << 4) | ((o2 as u16) >> 4);
            let v2 = ((o2 & 0b1111) as u16) << 2;
            out.push_str(&table2[v1 as usize]);
            out.push(table[v2 as usize]);
            if do_padding {
                out.push_str(&e);
            }
        }

        out
    })
}
