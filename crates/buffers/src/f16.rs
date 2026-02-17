//! Half-precision (16-bit) floating point utilities.

/// Decodes a half-precision (16-bit) floating point value.
///
/// The input is the raw binary representation (u16) of an IEEE 754 half-precision float.
///
/// # Example
///
/// ```
/// use json_joy_buffers::decode_f16;
///
/// // Positive zero
/// assert_eq!(decode_f16(0x0000), 0.0);
///
/// // Negative zero
/// assert_eq!(decode_f16(0x8000), -0.0);
///
/// // One
/// assert_eq!(decode_f16(0x3C00), 1.0);
///
/// // Positive infinity
/// assert!(decode_f16(0x7C00).is_infinite() && decode_f16(0x7C00).is_sign_positive());
///
/// // Negative infinity
/// assert!(decode_f16(0xFC00).is_infinite() && decode_f16(0xFC00).is_sign_negative());
///
/// // NaN
/// assert!(decode_f16(0x7C01).is_nan());
/// ```
pub fn decode_f16(binary: u16) -> f64 {
    let exponent = ((binary & 0x7C00) >> 10) as i32;
    let fraction = (binary & 0x03FF) as f64;
    let sign = if (binary >> 15) & 1 == 1 { -1.0 } else { 1.0 };

    if exponent == 0 {
        // Subnormal or zero
        sign * 6.103515625e-5 * (fraction / 1024.0)
    } else if exponent == 0x1F {
        // Infinity or NaN
        if fraction != 0.0 {
            f64::NAN
        } else {
            sign * f64::INFINITY
        }
    } else {
        // Normalized
        sign * 2f64.powi(exponent - 15) * (1.0 + fraction / 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_f16_zero() {
        assert_eq!(decode_f16(0x0000), 0.0);
        assert_eq!(decode_f16(0x8000).abs(), 0.0);
    }

    #[test]
    fn test_decode_f16_one() {
        assert_eq!(decode_f16(0x3C00), 1.0);
        assert_eq!(decode_f16(0xBC00), -1.0);
    }

    #[test]
    fn test_decode_f16_two() {
        assert_eq!(decode_f16(0x4000), 2.0);
    }

    #[test]
    fn test_decode_f16_infinity() {
        assert!(decode_f16(0x7C00).is_infinite());
        assert!(decode_f16(0x7C00).is_sign_positive());
        assert!(decode_f16(0xFC00).is_infinite());
        assert!(decode_f16(0xFC00).is_sign_negative());
    }

    #[test]
    fn test_decode_f16_nan() {
        assert!(decode_f16(0x7C01).is_nan());
        assert!(decode_f16(0xFC01).is_nan());
    }
}
