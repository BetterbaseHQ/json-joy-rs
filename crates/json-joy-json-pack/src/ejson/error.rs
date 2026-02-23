//! Error types for EJSON encoding and decoding.

use std::fmt;

/// Errors that can occur during EJSON encoding.
#[derive(Debug, Clone, PartialEq)]
pub enum EjsonEncodeError {
    /// Attempted to encode an invalid Date (NaN timestamp).
    InvalidDate,
}

impl fmt::Display for EjsonEncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EjsonEncodeError::InvalidDate => write!(f, "Invalid Date"),
        }
    }
}

impl std::error::Error for EjsonEncodeError {}

/// Errors that can occur during EJSON decoding.
#[derive(Debug, Clone, PartialEq)]
pub enum EjsonDecodeError {
    /// Generic JSON parse error at the given byte offset.
    InvalidJson(usize),
    /// Invalid UTF-8 in input.
    InvalidUtf8,
    /// Invalid `{"$oid": "..."}` ObjectId format.
    InvalidObjectId,
    /// Invalid `{"$numberInt": "..."}` format.
    InvalidInt32,
    /// Invalid `{"$numberLong": "..."}` format.
    InvalidInt64,
    /// Invalid `{"$numberDouble": "..."}` format.
    InvalidDouble,
    /// Invalid `{"$numberDecimal": "..."}` format.
    InvalidDecimal128,
    /// Invalid `{"$binary": {...}}` format.
    InvalidBinary,
    /// Invalid `{"$uuid": "..."}` format.
    InvalidUuid,
    /// Invalid `{"$code": "..."}` format.
    InvalidCode,
    /// Invalid `{"$code": "...", "$scope": {...}}` format.
    InvalidCodeWithScope,
    /// Invalid `{"$symbol": "..."}` format.
    InvalidSymbol,
    /// Invalid `{"$timestamp": {"t": ..., "i": ...}}` format.
    InvalidTimestamp,
    /// Invalid `{"$regularExpression": {"pattern": ..., "options": ...}}` format.
    InvalidRegularExpression,
    /// Invalid `{"$dbPointer": {"$ref": ..., "$id": {...}}}` format.
    InvalidDbPointer,
    /// Invalid `{"$date": ...}` format.
    InvalidDate,
    /// Invalid `{"$minKey": 1}` format.
    InvalidMinKey,
    /// Invalid `{"$maxKey": 1}` format.
    InvalidMaxKey,
    /// Invalid `{"$undefined": true}` format.
    InvalidUndefined,
    /// Extra keys found where not allowed (strict single-key wrapper).
    ExtraKeys(&'static str),
}

impl fmt::Display for EjsonDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EjsonDecodeError::InvalidJson(pos) => write!(f, "Invalid JSON at position {pos}"),
            EjsonDecodeError::InvalidUtf8 => write!(f, "Invalid UTF-8"),
            EjsonDecodeError::InvalidObjectId => write!(f, "Invalid ObjectId format"),
            EjsonDecodeError::InvalidInt32 => write!(f, "Invalid Int32 format"),
            EjsonDecodeError::InvalidInt64 => write!(f, "Invalid Int64 format"),
            EjsonDecodeError::InvalidDouble => write!(f, "Invalid Double format"),
            EjsonDecodeError::InvalidDecimal128 => write!(f, "Invalid Decimal128 format"),
            EjsonDecodeError::InvalidBinary => write!(f, "Invalid Binary format"),
            EjsonDecodeError::InvalidUuid => write!(f, "Invalid UUID format"),
            EjsonDecodeError::InvalidCode => write!(f, "Invalid Code format"),
            EjsonDecodeError::InvalidCodeWithScope => write!(f, "Invalid CodeWScope format"),
            EjsonDecodeError::InvalidSymbol => write!(f, "Invalid Symbol format"),
            EjsonDecodeError::InvalidTimestamp => write!(f, "Invalid Timestamp format"),
            EjsonDecodeError::InvalidRegularExpression => {
                write!(f, "Invalid RegularExpression format")
            }
            EjsonDecodeError::InvalidDbPointer => write!(f, "Invalid DBPointer format"),
            EjsonDecodeError::InvalidDate => write!(f, "Invalid Date format"),
            EjsonDecodeError::InvalidMinKey => write!(f, "Invalid MinKey format"),
            EjsonDecodeError::InvalidMaxKey => write!(f, "Invalid MaxKey format"),
            EjsonDecodeError::InvalidUndefined => write!(f, "Invalid Undefined format"),
            EjsonDecodeError::ExtraKeys(kind) => {
                write!(f, "Invalid {kind} format: extra keys not allowed")
            }
        }
    }
}

impl std::error::Error for EjsonDecodeError {}

#[cfg(test)]
mod tests {
    use super::*;

    // --- EjsonEncodeError ---

    #[test]
    fn test_encode_error_display_invalid_date() {
        let err = EjsonEncodeError::InvalidDate;
        assert_eq!(err.to_string(), "Invalid Date");
    }

    #[test]
    fn test_encode_error_debug() {
        let err = EjsonEncodeError::InvalidDate;
        assert_eq!(format!("{err:?}"), "InvalidDate");
    }

    #[test]
    fn test_encode_error_clone_eq() {
        let err1 = EjsonEncodeError::InvalidDate;
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_encode_error_is_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(EjsonEncodeError::InvalidDate);
        assert_eq!(err.to_string(), "Invalid Date");
    }

    // --- EjsonDecodeError ---

    #[test]
    fn test_decode_error_display_invalid_json() {
        let err = EjsonDecodeError::InvalidJson(42);
        assert_eq!(err.to_string(), "Invalid JSON at position 42");
    }

    #[test]
    fn test_decode_error_display_invalid_utf8() {
        assert_eq!(EjsonDecodeError::InvalidUtf8.to_string(), "Invalid UTF-8");
    }

    #[test]
    fn test_decode_error_display_all_variants() {
        let cases: Vec<(EjsonDecodeError, &str)> = vec![
            (
                EjsonDecodeError::InvalidJson(0),
                "Invalid JSON at position 0",
            ),
            (EjsonDecodeError::InvalidUtf8, "Invalid UTF-8"),
            (EjsonDecodeError::InvalidObjectId, "Invalid ObjectId format"),
            (EjsonDecodeError::InvalidInt32, "Invalid Int32 format"),
            (EjsonDecodeError::InvalidInt64, "Invalid Int64 format"),
            (EjsonDecodeError::InvalidDouble, "Invalid Double format"),
            (
                EjsonDecodeError::InvalidDecimal128,
                "Invalid Decimal128 format",
            ),
            (EjsonDecodeError::InvalidBinary, "Invalid Binary format"),
            (EjsonDecodeError::InvalidUuid, "Invalid UUID format"),
            (EjsonDecodeError::InvalidCode, "Invalid Code format"),
            (
                EjsonDecodeError::InvalidCodeWithScope,
                "Invalid CodeWScope format",
            ),
            (EjsonDecodeError::InvalidSymbol, "Invalid Symbol format"),
            (
                EjsonDecodeError::InvalidTimestamp,
                "Invalid Timestamp format",
            ),
            (
                EjsonDecodeError::InvalidRegularExpression,
                "Invalid RegularExpression format",
            ),
            (
                EjsonDecodeError::InvalidDbPointer,
                "Invalid DBPointer format",
            ),
            (EjsonDecodeError::InvalidDate, "Invalid Date format"),
            (EjsonDecodeError::InvalidMinKey, "Invalid MinKey format"),
            (EjsonDecodeError::InvalidMaxKey, "Invalid MaxKey format"),
            (
                EjsonDecodeError::InvalidUndefined,
                "Invalid Undefined format",
            ),
            (
                EjsonDecodeError::ExtraKeys("$oid"),
                "Invalid $oid format: extra keys not allowed",
            ),
        ];
        for (err, expected) in cases {
            assert_eq!(err.to_string(), expected, "mismatch for {err:?}");
        }
    }

    #[test]
    fn test_decode_error_clone_eq() {
        let err1 = EjsonDecodeError::InvalidBinary;
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_decode_error_ne() {
        assert_ne!(
            EjsonDecodeError::InvalidBinary,
            EjsonDecodeError::InvalidUuid
        );
    }

    #[test]
    fn test_decode_error_is_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(EjsonDecodeError::InvalidJson(10));
        assert_eq!(err.to_string(), "Invalid JSON at position 10");
    }

    #[test]
    fn test_decode_error_extra_keys_display() {
        let err = EjsonDecodeError::ExtraKeys("$date");
        assert!(err.to_string().contains("extra keys not allowed"));
        assert!(err.to_string().contains("$date"));
    }
}
