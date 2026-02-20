use json_joy_json_pack::bencode::{BencodeDecoder, BencodeEncoder, BencodeError};
use json_joy_json_pack::PackValue;

fn obj(fields: &[(&str, PackValue)]) -> PackValue {
    PackValue::Object(
        fields
            .iter()
            .map(|(k, v)| ((*k).to_owned(), v.clone()))
            .collect(),
    )
}

#[test]
fn bencode_encoder_wire_matrix() {
    let mut encoder = BencodeEncoder::new();

    assert_eq!(encoder.encode(&PackValue::Null), b"n");
    assert_eq!(encoder.encode(&PackValue::Undefined), b"u");
    assert_eq!(encoder.encode(&PackValue::Bool(true)), b"t");
    assert_eq!(encoder.encode(&PackValue::Bool(false)), b"f");

    assert_eq!(encoder.encode(&PackValue::Integer(0)), b"i0e");
    assert_eq!(encoder.encode(&PackValue::Integer(1)), b"i1e");
    assert_eq!(encoder.encode(&PackValue::Integer(-1)), b"i-1e");
    assert_eq!(encoder.encode(&PackValue::BigInt(123_456)), b"i123456e");
    assert_eq!(encoder.encode(&PackValue::Float(1.9)), b"i2e");

    assert_eq!(encoder.encode(&PackValue::Str("".into())), b"0:");
    assert_eq!(encoder.encode(&PackValue::Str("abc".into())), b"3:abc");
    assert_eq!(
        encoder.encode(&PackValue::Str("✅".into())),
        b"3:\xE2\x9C\x85"
    );

    assert_eq!(encoder.encode(&PackValue::Bytes(vec![])), b"0:");
    assert_eq!(encoder.encode(&PackValue::Bytes(vec![65])), b"1:A");

    assert_eq!(
        encoder.encode(&PackValue::Array(vec![
            PackValue::Integer(1),
            PackValue::Integer(2)
        ])),
        b"li1ei2ee"
    );

    let sorted = PackValue::Object(vec![
        ("foo".into(), PackValue::Str("bar".into())),
        ("baz".into(), PackValue::Integer(123)),
    ]);
    assert_eq!(encoder.encode(&sorted), b"d3:bazi123e3:foo3:bare");
}

#[test]
fn bencode_decoder_matrix() {
    let decoder = BencodeDecoder::new();

    assert_eq!(decoder.decode(b"n").unwrap(), PackValue::Null);
    assert_eq!(decoder.decode(b"u").unwrap(), PackValue::Undefined);
    assert_eq!(decoder.decode(b"t").unwrap(), PackValue::Bool(true));
    assert_eq!(decoder.decode(b"f").unwrap(), PackValue::Bool(false));

    assert_eq!(decoder.decode(b"i123e").unwrap(), PackValue::Integer(123));
    assert_eq!(decoder.decode(b"i-123e").unwrap(), PackValue::Integer(-123));

    assert_eq!(
        decoder.decode(b"11:hello world").unwrap(),
        PackValue::Bytes(b"hello world".to_vec())
    );
    assert_eq!(
        decoder.decode("3:✅".as_bytes()).unwrap(),
        PackValue::Bytes("✅".as_bytes().to_vec())
    );

    assert_eq!(
        decoder.decode(b"li1ei2ee").unwrap(),
        PackValue::Array(vec![PackValue::Integer(1), PackValue::Integer(2)])
    );

    assert_eq!(
        decoder.decode(b"d3:foo3:bar3:bazi123ee").unwrap(),
        obj(&[
            ("foo", PackValue::Bytes(b"bar".to_vec())),
            ("baz", PackValue::Integer(123)),
        ])
    );

    assert_eq!(
        decoder.decode(b"d3:fooli1eui-1eee").unwrap(),
        obj(&[(
            "foo",
            PackValue::Array(vec![
                PackValue::Integer(1),
                PackValue::Undefined,
                PackValue::Integer(-1),
            ]),
        )])
    );
}

#[test]
fn bencode_automated_roundtrip_matrix() {
    let mut encoder = BencodeEncoder::new();
    let decoder = BencodeDecoder::new();

    let docs = vec![
        PackValue::Integer(0),
        PackValue::Integer(1),
        PackValue::Integer(12_345),
        PackValue::Integer(-12_345),
        PackValue::Integer(-4_444_444_444_444_444),
        PackValue::Bool(true),
        PackValue::Bool(false),
        PackValue::Null,
        PackValue::Undefined,
        PackValue::Bytes(vec![]),
        PackValue::Bytes(b"hello".to_vec()),
        PackValue::Object(vec![]),
        PackValue::Array(vec![]),
        PackValue::Array(vec![
            PackValue::Integer(1),
            PackValue::Integer(-2),
            PackValue::Null,
            PackValue::Bool(true),
            PackValue::Bytes(b"asdf".to_vec()),
            PackValue::Bool(false),
            PackValue::Bytes(vec![]),
            PackValue::Undefined,
        ]),
        PackValue::Array(vec![PackValue::Array(vec![PackValue::Array(vec![])])]),
        PackValue::Array(vec![
            PackValue::Integer(1),
            PackValue::Array(vec![
                PackValue::Integer(1),
                PackValue::Array(vec![PackValue::Integer(1)]),
                PackValue::Integer(1),
            ]),
            PackValue::Integer(1),
        ]),
        obj(&[(
            "a",
            obj(&[(
                "b",
                obj(&[(
                    "c",
                    obj(&[("d", obj(&[("foo", PackValue::Bytes(b"bar".to_vec()))]))]),
                )]),
            )]),
        )]),
    ];

    for doc in docs {
        let encoded = encoder.encode(&doc);
        let decoded = decoder
            .decode(&encoded)
            .unwrap_or_else(|e| panic!("decode failed for {doc:?}: {e}"));
        assert_eq!(decoded, doc);
    }
}

#[test]
fn bencode_decode_error_matrix() {
    let decoder = BencodeDecoder::new();

    assert!(matches!(
        decoder.decode(b""),
        Err(BencodeError::UnexpectedEof)
    ));
    assert!(matches!(
        decoder.decode(b"i"),
        Err(BencodeError::UnexpectedEof)
    ));
    assert!(matches!(
        decoder.decode(b"d9:__proto__1:ae"),
        Err(BencodeError::InvalidKey)
    ));
    assert!(matches!(
        decoder.decode(b"1"),
        Err(BencodeError::UnexpectedEof)
    ));
    assert!(matches!(
        decoder.decode(b"2:a"),
        Err(BencodeError::UnexpectedEof)
    ));
}
