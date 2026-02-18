//! [`JsonPackMpint`] — SSH multiprecision integer.
//!
//! Mirrors `JsonPackMpint.ts` from upstream.

/// Represents an SSH multiprecision integer (mpint).
///
/// Stored in two's complement format, 8 bits per byte, MSB first (RFC 4251).
#[derive(Debug, Clone, PartialEq)]
pub struct JsonPackMpint {
    /// Raw bytes in two's complement format, MSB first.
    pub data: Vec<u8>,
}

impl JsonPackMpint {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Create an mpint from a BigInt value.
    pub fn from_i128(value: i128) -> Self {
        if value == 0 {
            return Self::new(vec![]);
        }

        let negative = value < 0;
        let mut bytes: Vec<u8> = Vec::new();

        if negative {
            let abs_value: i128 = -value;
            let bit_length = (128 - abs_value.leading_zeros()) as usize;
            let byte_length = (bit_length + 8) / 8; // +1 for sign bit, rounded up

            let two_complement: u128 = (1u128 << (byte_length * 8)).wrapping_add(value as u128);

            for i in (0..byte_length).rev() {
                bytes.push(((two_complement >> (i * 8)) & 0xff) as u8);
            }

            // Strip redundant 0xff prefix bytes where MSB of next byte is set
            while bytes.len() > 1 && bytes[0] == 0xff && (bytes[1] & 0x80) != 0 {
                bytes.remove(0);
            }
        } else {
            let mut temp = value as u128;
            while temp > 0 {
                bytes.insert(0, (temp & 0xff) as u8);
                temp >>= 8;
            }

            // Add leading zero if MSB is set (to indicate positive)
            if !bytes.is_empty() && (bytes[0] & 0x80) != 0 {
                bytes.insert(0, 0);
            }
        }

        Self::new(bytes)
    }

    /// Convert the mpint to an i128.
    pub fn to_i128(&self) -> i128 {
        if self.data.is_empty() {
            return 0;
        }

        let negative = (self.data[0] & 0x80) != 0;
        let mut value: i128 = 0;

        if negative {
            for &b in &self.data {
                value = (value << 8) | (b as i128);
            }
            // Two's complement: subtract 2^(8*len)
            let bit_length = (self.data.len() * 8) as u32;
            value -= 1i128 << bit_length;
        } else {
            for &b in &self.data {
                value = (value << 8) | (b as i128);
            }
        }

        value
    }

    /// Create from a safe integer.
    pub fn from_i64(value: i64) -> Self {
        Self::from_i128(value as i128)
    }

    /// Convert to i64 (panics if out of range).
    pub fn to_i64(&self) -> Result<i64, &'static str> {
        let v = self.to_i128();
        if v > i64::MAX as i128 || v < i64::MIN as i128 {
            Err("Value is outside safe integer range")
        } else {
            Ok(v as i64)
        }
    }
}
