//! json-size — approximate MessagePack encoding size estimation.
//!
//! Mirrors `packages/json-joy/src/json-size/msgpackSizeFast.ts`.

use json_joy_json_pack::PackValue;

/// Approximate the byte size of a [`PackValue`] when encoded as MessagePack.
///
/// Same heuristic as the upstream `msgpackSizeFast`:
/// - null / undefined → 1 byte
/// - bool → 1 byte
/// - number → 9 bytes (worst-case: 1 header + 8-byte float64)
/// - string → 4 + byte length (1..4 header bytes)
/// - bytes (`Uint8Array`) → 5 + byte length
/// - array → 2 + sum of element sizes
/// - object → 2 + sum of (2 + key bytes + value size) per entry
/// - pre-encoded blob → raw byte length as-is
/// - extension with byte payload → 6 + payload length
pub fn msgpack_size_fast(value: &PackValue) -> usize {
    match value {
        PackValue::Null | PackValue::Undefined => 1,
        PackValue::Bool(_) => 1,
        PackValue::Integer(_)
        | PackValue::UInteger(_)
        | PackValue::Float(_)
        | PackValue::BigInt(_) => 9,
        PackValue::Str(s) => 4 + s.len(),
        PackValue::Bytes(b) => 5 + b.len(),
        PackValue::Array(arr) => {
            let mut size: usize = 2;
            for item in arr {
                size += msgpack_size_fast(item);
            }
            size
        }
        PackValue::Object(obj) => {
            let mut size: usize = 2;
            for (key, val) in obj {
                size += 2 + key.len() + msgpack_size_fast(val);
            }
            size
        }
        PackValue::Blob(blob) => blob.val.len(),
        // Upstream extensions always wrap raw bytes and use `6 + payload.length`.
        // Rust allows non-byte extension payloads; for those we mirror the local
        // msgpack encoder fallback and size the inner value directly.
        PackValue::Extension(ext) => match ext.val.as_ref() {
            PackValue::Bytes(bytes) => 6 + bytes.len(),
            other => msgpack_size_fast(other),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use json_joy_json_pack::{JsonPackExtension, JsonPackValue};

    const TEST_F64_3_14: f64 = 314.0 / 100.0;

    #[test]
    fn null_is_one_byte() {
        assert_eq!(msgpack_size_fast(&PackValue::Null), 1);
        assert_eq!(msgpack_size_fast(&PackValue::Undefined), 1);
    }

    #[test]
    fn bool_is_one_byte() {
        assert_eq!(msgpack_size_fast(&PackValue::Bool(true)), 1);
        assert_eq!(msgpack_size_fast(&PackValue::Bool(false)), 1);
    }

    #[test]
    fn numbers_are_nine_bytes() {
        assert_eq!(msgpack_size_fast(&PackValue::Integer(-42)), 9);
        assert_eq!(msgpack_size_fast(&PackValue::UInteger(u64::MAX)), 9);
        assert_eq!(msgpack_size_fast(&PackValue::Float(TEST_F64_3_14)), 9);
        assert_eq!(msgpack_size_fast(&PackValue::BigInt(i128::MAX)), 9);
    }

    #[test]
    fn string_size() {
        assert_eq!(msgpack_size_fast(&PackValue::Str("".to_owned())), 4);
        assert_eq!(msgpack_size_fast(&PackValue::Str("hello".to_owned())), 9); // 4 + 5
    }

    #[test]
    fn bytes_size() {
        assert_eq!(msgpack_size_fast(&PackValue::Bytes(vec![1, 2, 3])), 8); // 5 + 3
    }

    #[test]
    fn empty_array() {
        assert_eq!(msgpack_size_fast(&PackValue::Array(vec![])), 2);
    }

    #[test]
    fn array_with_items() {
        let arr = PackValue::Array(vec![
            PackValue::Null,
            PackValue::Bool(true),
            PackValue::Integer(42),
        ]);
        // 2 + 1 + 1 + 9 = 13
        assert_eq!(msgpack_size_fast(&arr), 13);
    }

    #[test]
    fn object_size() {
        let obj = PackValue::Object(vec![("key".to_owned(), PackValue::Integer(1))]);
        // 2 + (2 + 3 + 9) = 16
        assert_eq!(msgpack_size_fast(&obj), 16);
    }

    #[test]
    fn blob_size() {
        let blob = PackValue::Blob(JsonPackValue::new(vec![0xAA, 0xBB, 0xCC]));
        assert_eq!(msgpack_size_fast(&blob), 3);
    }

    #[test]
    fn extension_size() {
        let ext = PackValue::Extension(Box::new(JsonPackExtension::new(
            1,
            PackValue::Bytes(vec![1, 2, 3, 4, 5]),
        )));
        assert_eq!(msgpack_size_fast(&ext), 11);
    }

    #[test]
    fn extension_non_bytes_uses_fallback_value_size() {
        let ext = PackValue::Extension(Box::new(JsonPackExtension::new(
            1,
            PackValue::Str("hi".to_owned()),
        )));
        assert_eq!(msgpack_size_fast(&ext), 6);
    }

    #[test]
    fn complex_object_matches_upstream_math() {
        let embedded = PackValue::Blob(JsonPackValue::new(vec![1, 2]));
        let extension = PackValue::Extension(Box::new(JsonPackExtension::new(
            445,
            PackValue::Bytes(vec![1, 2, 3]),
        )));

        let json = PackValue::Object(vec![
            ("a".to_owned(), PackValue::Integer(1)),
            ("b".to_owned(), PackValue::Bool(true)),
            ("c".to_owned(), PackValue::Bool(false)),
            ("d".to_owned(), PackValue::Null),
            ("e.e".to_owned(), PackValue::Float(2.2)),
            ("f".to_owned(), PackValue::Str("".to_owned())),
            ("g".to_owned(), PackValue::Str("asdf".to_owned())),
            (
                "h".to_owned(),
                PackValue::Object(vec![
                    ("foo".to_owned(), PackValue::Bytes(vec![123])),
                    ("s".to_owned(), embedded.clone()),
                    ("ext".to_owned(), extension.clone()),
                ]),
            ),
            (
                "i".to_owned(),
                PackValue::Array(vec![
                    PackValue::Integer(1),
                    PackValue::Bool(true),
                    PackValue::Bool(false),
                    PackValue::Null,
                    PackValue::Float(2.2),
                    PackValue::Str("".to_owned()),
                    PackValue::Str("asdf".to_owned()),
                    PackValue::Object(vec![]),
                    PackValue::Bytes(vec![123]),
                    extension.clone(),
                    embedded,
                ]),
            ),
            ("j".to_owned(), PackValue::Bytes(vec![1, 2, 3])),
        ]);

        assert_eq!(msgpack_size_fast(&json), 161);
    }
}
