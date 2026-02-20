#![allow(clippy::approx_constant)]

use json_joy_json_pack::bson::{
    BsonBinary, BsonDbPointer, BsonDecimal128, BsonFloat, BsonInt32, BsonInt64, BsonJavascriptCode,
    BsonJavascriptCodeWithScope, BsonMaxKey, BsonMinKey, BsonObjectId, BsonSymbol, BsonTimestamp,
    BsonValue,
};
use json_joy_json_pack::ejson::{
    EjsonDecodeError, EjsonDecoder, EjsonEncoder, EjsonEncoderOptions, EjsonValue,
};

fn obj(fields: &[(&str, EjsonValue)]) -> EjsonValue {
    EjsonValue::Object(
        fields
            .iter()
            .map(|(k, v)| ((*k).to_owned(), v.clone()))
            .collect(),
    )
}

fn strip_date_iso(value: EjsonValue) -> EjsonValue {
    match value {
        EjsonValue::Date { timestamp_ms, .. } => EjsonValue::Date {
            timestamp_ms,
            iso: None,
        },
        EjsonValue::Array(items) => {
            EjsonValue::Array(items.into_iter().map(strip_date_iso).collect())
        }
        EjsonValue::Object(fields) => EjsonValue::Object(
            fields
                .into_iter()
                .map(|(k, v)| (k, strip_date_iso(v)))
                .collect(),
        ),
        other => other,
    }
}

#[test]
fn ejson_encoder_canonical_matrix() {
    let mut encoder = EjsonEncoder::with_options(EjsonEncoderOptions { canonical: true });

    assert_eq!(encoder.encode_to_string(&EjsonValue::Null).unwrap(), "null");
    assert_eq!(
        encoder.encode_to_string(&EjsonValue::Bool(true)).unwrap(),
        "true"
    );
    assert_eq!(
        encoder.encode_to_string(&EjsonValue::Undefined).unwrap(),
        "{\"$undefined\":true}"
    );

    assert_eq!(
        encoder.encode_to_string(&EjsonValue::Number(42.0)).unwrap(),
        "{\"$numberInt\":\"42\"}"
    );
    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::Number(2_147_483_648.0))
            .unwrap(),
        "{\"$numberLong\":\"2147483648\"}"
    );
    assert_eq!(
        encoder.encode_to_string(&EjsonValue::Number(3.14)).unwrap(),
        "{\"$numberDouble\":\"3.14\"}"
    );
    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::Number(f64::INFINITY))
            .unwrap(),
        "{\"$numberDouble\":\"Infinity\"}"
    );
    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::Number(f64::NEG_INFINITY))
            .unwrap(),
        "{\"$numberDouble\":\"-Infinity\"}"
    );
    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::Number(f64::NAN))
            .unwrap(),
        "{\"$numberDouble\":\"NaN\"}"
    );

    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::Array(vec![
                EjsonValue::Number(1.0),
                EjsonValue::Number(2.0),
                EjsonValue::Number(3.0),
            ]))
            .unwrap(),
        "[{\"$numberInt\":\"1\"},{\"$numberInt\":\"2\"},{\"$numberInt\":\"3\"}]"
    );

    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::Date {
                timestamp_ms: 1_672_531_200_000,
                iso: Some("2023-01-01T00:00:00.000Z".into()),
            })
            .unwrap(),
        "{\"$date\":{\"$numberLong\":\"1672531200000\"}}"
    );

    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::RegExp("pattern".into(), "gi".into()))
            .unwrap(),
        "{\"$regularExpression\":{\"pattern\":\"pattern\",\"options\":\"gi\"}}"
    );

    let object_id = BsonObjectId {
        timestamp: 0x507f1f77,
        process: 0xbcf86cd799,
        counter: 0x439011,
    };
    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::ObjectId(object_id.clone()))
            .unwrap(),
        "{\"$oid\":\"507f1f77bcf86cd799439011\"}"
    );
    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::Int32(BsonInt32 { value: 42 }))
            .unwrap(),
        "{\"$numberInt\":\"42\"}"
    );
    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::Int64(BsonInt64 {
                value: 1_234_567_890_123,
            }))
            .unwrap(),
        "{\"$numberLong\":\"1234567890123\"}"
    );
    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::BsonFloat(BsonFloat { value: 3.14 }))
            .unwrap(),
        "{\"$numberDouble\":\"3.14\"}"
    );
    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::Decimal128(BsonDecimal128 {
                data: vec![0; 16]
            }))
            .unwrap(),
        "{\"$numberDecimal\":\"0\"}"
    );
    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::Binary(BsonBinary {
                subtype: 0,
                data: vec![1, 2, 3, 4],
            }))
            .unwrap(),
        "{\"$binary\":{\"base64\":\"AQIDBA==\",\"subType\":\"00\"}}"
    );
    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::Code(BsonJavascriptCode {
                code: "function() { return 42; }".into(),
            }))
            .unwrap(),
        "{\"$code\":\"function() { return 42; }\"}"
    );
    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::CodeWithScope(BsonJavascriptCodeWithScope {
                code: "function() { return x; }".into(),
                scope: vec![("x".into(), BsonValue::Int32(42))],
            }))
            .unwrap(),
        "{\"$code\":\"function() { return x; }\",\"$scope\":{\"x\":{\"$numberInt\":\"42\"}}}"
    );
    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::Symbol(BsonSymbol {
                symbol: "mySymbol".into(),
            }))
            .unwrap(),
        "{\"$symbol\":\"mySymbol\"}"
    );
    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::Timestamp(BsonTimestamp {
                increment: 12_345,
                timestamp: 1_234_567_890,
            }))
            .unwrap(),
        "{\"$timestamp\":{\"t\":1234567890,\"i\":12345}}"
    );
    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::DbPointer(BsonDbPointer {
                name: "collection".into(),
                id: object_id,
            }))
            .unwrap(),
        "{\"$dbPointer\":{\"$ref\":\"collection\",\"$id\":{\"$oid\":\"507f1f77bcf86cd799439011\"}}}"
    );
    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::MinKey(BsonMinKey))
            .unwrap(),
        "{\"$minKey\":1}"
    );
    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::MaxKey(BsonMaxKey))
            .unwrap(),
        "{\"$maxKey\":1}"
    );
}

#[test]
fn ejson_encoder_relaxed_matrix() {
    let mut encoder = EjsonEncoder::with_options(EjsonEncoderOptions { canonical: false });

    assert_eq!(
        encoder.encode_to_string(&EjsonValue::Number(42.0)).unwrap(),
        "42"
    );
    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::Number(-42.0))
            .unwrap(),
        "-42"
    );
    assert_eq!(
        encoder.encode_to_string(&EjsonValue::Number(3.14)).unwrap(),
        "3.14"
    );
    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::Number(f64::INFINITY))
            .unwrap(),
        "{\"$numberDouble\":\"Infinity\"}"
    );

    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::Date {
                timestamp_ms: 1_672_531_200_000,
                iso: Some("2023-01-01T00:00:00.000Z".into()),
            })
            .unwrap(),
        "{\"$date\":\"2023-01-01T00:00:00.000Z\"}"
    );
    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::Date {
                timestamp_ms: 1_672_531_200_000,
                iso: None,
            })
            .unwrap(),
        "{\"$date\":\"2023-01-01T00:00:00.000Z\"}"
    );
    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::Date {
                timestamp_ms: -2_208_988_800_000,
                iso: Some("1900-01-01T00:00:00.000Z".into()),
            })
            .unwrap(),
        "{\"$date\":{\"$numberLong\":\"-2208988800000\"}}"
    );
    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::Date {
                timestamp_ms: 32_503_680_000_000,
                iso: Some("3000-01-01T00:00:00.000Z".into()),
            })
            .unwrap(),
        "{\"$date\":\"3000-01-01T00:00:00.000Z\"}"
    );

    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::Int32(BsonInt32 { value: 42 }))
            .unwrap(),
        "42"
    );
    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::Int64(BsonInt64 { value: 123 }))
            .unwrap(),
        "123"
    );
    assert_eq!(
        encoder
            .encode_to_string(&EjsonValue::BsonFloat(BsonFloat { value: 3.14 }))
            .unwrap(),
        "3.14"
    );
}

#[test]
fn ejson_decoder_wrappers_matrix() {
    let mut decoder = EjsonDecoder::new();

    assert_eq!(decoder.decode_str("null").unwrap(), EjsonValue::Null);
    assert_eq!(decoder.decode_str("true").unwrap(), EjsonValue::Bool(true));
    assert_eq!(decoder.decode_str("42").unwrap(), EjsonValue::Integer(42));
    assert_eq!(decoder.decode_str("3.14").unwrap(), EjsonValue::Float(3.14));

    match decoder
        .decode_str("{\"$oid\":\"507f1f77bcf86cd799439011\"}")
        .unwrap()
    {
        EjsonValue::ObjectId(id) => {
            assert_eq!(id.timestamp, 0x507f1f77);
            assert_eq!(id.process, 0xbcf86cd799);
            assert_eq!(id.counter, 0x439011);
        }
        other => panic!("expected ObjectId, got {other:?}"),
    }

    assert_eq!(
        decoder.decode_str("{\"$numberInt\":\"42\"}").unwrap(),
        EjsonValue::Int32(BsonInt32 { value: 42 })
    );
    assert_eq!(
        decoder
            .decode_str("{\"$numberLong\":\"9223372036854775807\"}")
            .unwrap(),
        EjsonValue::Int64(BsonInt64 {
            value: 9_223_372_036_854_775_807
        })
    );
    assert_eq!(
        decoder
            .decode_str("{\"$numberDouble\":\"Infinity\"}")
            .unwrap(),
        EjsonValue::BsonFloat(BsonFloat {
            value: f64::INFINITY
        })
    );
    assert_eq!(
        decoder
            .decode_str("{\"$numberDouble\":\"-Infinity\"}")
            .unwrap(),
        EjsonValue::BsonFloat(BsonFloat {
            value: f64::NEG_INFINITY
        })
    );
    match decoder.decode_str("{\"$numberDouble\":\"NaN\"}").unwrap() {
        EjsonValue::BsonFloat(v) => assert!(v.value.is_nan()),
        other => panic!("expected NaN BsonFloat, got {other:?}"),
    }

    assert_eq!(
        decoder
            .decode_str("{\"$binary\":{\"base64\":\"AQIDBA==\",\"subType\":\"00\"}}")
            .unwrap(),
        EjsonValue::Binary(BsonBinary {
            subtype: 0,
            data: vec![1, 2, 3, 4]
        })
    );
    match decoder
        .decode_str("{\"$uuid\":\"c8edabc3-f738-4ca3-b68d-ab92a91478a3\"}")
        .unwrap()
    {
        EjsonValue::Binary(bin) => {
            assert_eq!(bin.subtype, 4);
            assert_eq!(bin.data.len(), 16);
        }
        other => panic!("expected UUID binary, got {other:?}"),
    }

    assert_eq!(
        decoder
            .decode_str(
                "{\"$dbPointer\":{\"$ref\":\"collection\",\"$id\":{\"$oid\":\"507f1f77bcf86cd799439011\"}}}"
            )
            .unwrap(),
        EjsonValue::DbPointer(BsonDbPointer {
            name: "collection".into(),
            id: BsonObjectId {
                timestamp: 0x507f1f77,
                process: 0xbcf86cd799,
                counter: 0x439011
            }
        })
    );
    assert_eq!(
        decoder
            .decode_str("{\"$date\":\"2023-01-01T00:00:00.000Z\"}")
            .unwrap(),
        EjsonValue::Date {
            timestamp_ms: 1_672_531_200_000,
            iso: None
        }
    );
    assert_eq!(
        decoder
            .decode_str("{\"$date\":{\"$numberLong\":\"1672531200000\"}}")
            .unwrap(),
        EjsonValue::Date {
            timestamp_ms: 1_672_531_200_000,
            iso: None
        }
    );
    assert_eq!(
        decoder.decode_str("{\"$minKey\":1}").unwrap(),
        EjsonValue::MinKey(BsonMinKey)
    );
    assert_eq!(
        decoder.decode_str("{\"$maxKey\":1}").unwrap(),
        EjsonValue::MaxKey(BsonMaxKey)
    );
    assert_eq!(
        decoder.decode_str("{\"$undefined\":true}").unwrap(),
        EjsonValue::Undefined
    );
}

#[test]
fn ejson_roundtrip_integration_matrix() {
    let mut canonical = EjsonEncoder::with_options(EjsonEncoderOptions { canonical: true });
    let mut relaxed = EjsonEncoder::with_options(EjsonEncoderOptions { canonical: false });
    let mut decoder = EjsonDecoder::new();

    let values = vec![
        EjsonValue::Null,
        EjsonValue::Bool(true),
        EjsonValue::Bool(false),
        EjsonValue::Str("hello".into()),
        EjsonValue::Undefined,
        EjsonValue::Array(vec![
            EjsonValue::Str("a".into()),
            EjsonValue::Bool(true),
            EjsonValue::Null,
            obj(&[("nested", EjsonValue::Str("x".into()))]),
        ]),
        EjsonValue::Object(vec![
            ("name".into(), EjsonValue::Str("test".into())),
            (
                "timestamp".into(),
                EjsonValue::Date {
                    timestamp_ms: 1_672_531_200_000,
                    iso: Some("2023-01-01T00:00:00.000Z".into()),
                },
            ),
            (
                "oid".into(),
                EjsonValue::ObjectId(BsonObjectId {
                    timestamp: 0x507f1f77,
                    process: 0xbcf86cd799,
                    counter: 0x439011,
                }),
            ),
        ]),
        EjsonValue::RegExp("test.*pattern".into(), "gim".into()),
    ];

    for value in values {
        let canonical_json = canonical.encode_to_string(&value).unwrap();
        let relaxed_json = relaxed.encode_to_string(&value).unwrap();
        let canonical_decoded = decoder.decode_str(&canonical_json).unwrap();
        let relaxed_decoded = decoder.decode_str(&relaxed_json).unwrap();
        let expected = strip_date_iso(value.clone());
        assert_eq!(strip_date_iso(canonical_decoded), expected);
        assert_eq!(strip_date_iso(relaxed_decoded), expected);
    }

    let canonical_number = canonical
        .encode_to_string(&EjsonValue::Number(42.0))
        .unwrap();
    let relaxed_number = relaxed.encode_to_string(&EjsonValue::Number(42.0)).unwrap();
    assert_eq!(
        decoder.decode_str(&canonical_number).unwrap(),
        EjsonValue::Int32(BsonInt32 { value: 42 })
    );
    assert_eq!(
        decoder.decode_str(&relaxed_number).unwrap(),
        EjsonValue::Integer(42)
    );
}

#[test]
fn ejson_decoder_error_matrix() {
    let mut decoder = EjsonDecoder::new();

    assert!(matches!(
        decoder.decode_str("{"),
        Err(EjsonDecodeError::InvalidJson(_))
    ));
    assert!(matches!(
        decoder.decode_str("invalid json"),
        Err(EjsonDecodeError::InvalidJson(_))
    ));

    assert_eq!(
        decoder
            .decode_str("{\"$oid\":123}")
            .expect_err("invalid oid should fail"),
        EjsonDecodeError::InvalidObjectId
    );
    assert_eq!(
        decoder
            .decode_str("{\"$numberInt\":\"invalid\"}")
            .expect_err("invalid int32 should fail"),
        EjsonDecodeError::InvalidInt32
    );
    assert_eq!(
        decoder
            .decode_str("{\"$binary\":\"not an object\"}")
            .expect_err("invalid binary should fail"),
        EjsonDecodeError::InvalidBinary
    );
    assert_eq!(
        decoder
            .decode_str("{\"$binary\":{\"base64\":\"AQIDBA==\"}}")
            .expect_err("incomplete binary wrapper should fail"),
        EjsonDecodeError::InvalidBinary
    );
    assert_eq!(
        decoder
            .decode_str("{\"$timestamp\":{\"t\":123}}")
            .expect_err("incomplete timestamp wrapper should fail"),
        EjsonDecodeError::InvalidTimestamp
    );
    assert_eq!(
        decoder
            .decode_str("{\"$oid\":\"507f1f77bcf86cd799439011\",\"extra\":\"field\"}")
            .expect_err("extra keys on strict wrapper should fail"),
        EjsonDecodeError::ExtraKeys("ObjectId")
    );
}
