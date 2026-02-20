use json_joy_json_pack::bson::{
    BsonBinary, BsonDbPointer, BsonDecimal128, BsonDecoder, BsonEncoder, BsonError,
    BsonJavascriptCode, BsonJavascriptCodeWithScope, BsonObjectId, BsonSymbol, BsonTimestamp,
    BsonValue,
};

fn doc(fields: &[(&str, BsonValue)]) -> Vec<(String, BsonValue)> {
    fields
        .iter()
        .map(|(k, v)| ((*k).to_owned(), v.clone()))
        .collect()
}

#[test]
fn bson_encoder_decoder_matrix() {
    let encoder = BsonEncoder::new();
    let mut decoder = BsonDecoder::new();

    let object_id = BsonObjectId {
        timestamp: 0x1234_5678,
        process: 0x0001_0203_0405,
        counter: 0x010203,
    };

    let docs = vec![
        doc(&[]),
        doc(&[("null", BsonValue::Null)]),
        doc(&[("bool", BsonValue::Boolean(true))]),
        doc(&[
            ("i32", BsonValue::Int32(123)),
            ("i64", BsonValue::Int64(12_321_321_123)),
            ("f64", BsonValue::Float(123.456)),
        ]),
        doc(&[
            ("str", BsonValue::Str("hello".into())),
            ("unicode", BsonValue::Str("yes! - 👍🏻👍🏼👍🏽👍🏾👍🏿".into())),
        ]),
        doc(&[(
            "arr",
            BsonValue::Array(vec![
                BsonValue::Int32(1),
                BsonValue::Int32(2),
                BsonValue::Str("x".into()),
            ]),
        )]),
        doc(&[(
            "obj",
            BsonValue::Document(doc(&[
                ("foo", BsonValue::Str("bar".into())),
                ("baz", BsonValue::Int32(42)),
            ])),
        )]),
        doc(&[(
            "bin",
            BsonValue::Binary(BsonBinary {
                subtype: 0x80,
                data: vec![1, 2, 3],
            }),
        )]),
        doc(&[("id", BsonValue::ObjectId(object_id.clone()))]),
        doc(&[(
            "ptr",
            BsonValue::DbPointer(BsonDbPointer {
                name: "users".into(),
                id: object_id.clone(),
            }),
        )]),
        doc(&[(
            "code",
            BsonValue::JavaScriptCode(BsonJavascriptCode {
                code: "function() { return 42; }".into(),
            }),
        )]),
        doc(&[(
            "scope",
            BsonValue::JavaScriptCodeWithScope(BsonJavascriptCodeWithScope {
                code: "function() { return x; }".into(),
                scope: doc(&[("x", BsonValue::Int32(42))]),
            }),
        )]),
        doc(&[(
            "sym",
            BsonValue::Symbol(BsonSymbol {
                symbol: "sym".into(),
            }),
        )]),
        doc(&[(
            "ts",
            BsonValue::Timestamp(BsonTimestamp {
                increment: 1,
                timestamp: 1_689_235_200,
            }),
        )]),
        doc(&[(
            "dec",
            BsonValue::Decimal128(BsonDecimal128 {
                data: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            }),
        )]),
        doc(&[("min", BsonValue::MinKey), ("max", BsonValue::MaxKey)]),
    ];

    for input in docs {
        let encoded = encoder.encode(&input);
        let decoded = decoder
            .decode(&encoded)
            .unwrap_or_else(|e| panic!("decode failed for {input:?}: {e}"));
        assert_eq!(decoded, input);
    }
}

#[test]
fn bson_special_value_wire_matrix() {
    let encoder = BsonEncoder::new();
    let mut decoder = BsonDecoder::new();

    let value = doc(&[
        (
            "bin1",
            BsonValue::Binary(BsonBinary {
                subtype: 0x00,
                data: vec![1, 2, 3],
            }),
        ),
        (
            "bin2",
            BsonValue::Binary(BsonBinary {
                subtype: 0x01,
                data: vec![1, 2, 3],
            }),
        ),
        (
            "bin3",
            BsonValue::Binary(BsonBinary {
                subtype: 0x80,
                data: vec![1, 2, 3],
            }),
        ),
    ]);

    let encoded = encoder.encode(&value);
    let decoded = decoder.decode(&encoded).unwrap();
    assert_eq!(decoded, value);

    // BSON document starts with LE size and ends with null terminator.
    assert_eq!(encoded[encoded.len() - 1], 0x00);
    let declared_len = i32::from_le_bytes([encoded[0], encoded[1], encoded[2], encoded[3]]);
    assert_eq!(declared_len as usize, encoded.len());
}

#[test]
fn bson_decoder_error_matrix() {
    let mut decoder = BsonDecoder::new();

    assert!(matches!(decoder.decode(&[]), Err(BsonError::UnexpectedEof)));

    // Valid-sized document with unsupported element type 0x14.
    let unsupported = vec![8, 0, 0, 0, 0x14, b'a', 0x00, 0x00];
    assert!(matches!(
        decoder.decode(&unsupported),
        Err(BsonError::UnsupportedType(0x14))
    ));

    // String with invalid UTF-8 payload.
    let invalid_utf8 = vec![
        14, 0, 0, 0, // doc len
        0x02, b'a', 0x00, // type + key cstring
        2, 0, 0, 0, // string length including null
        0xff, 0x00, // invalid utf8 + null
        0x00, // doc terminator
    ];
    assert!(matches!(
        decoder.decode(&invalid_utf8),
        Err(BsonError::InvalidUtf8)
    ));
}
