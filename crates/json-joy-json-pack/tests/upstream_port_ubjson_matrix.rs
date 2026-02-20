use json_joy_json_pack::ubjson::{UbjsonDecoder, UbjsonEncoder, UbjsonError};
use json_joy_json_pack::{JsonPackExtension, PackValue};

fn obj(fields: &[(&str, PackValue)]) -> PackValue {
    PackValue::Object(
        fields
            .iter()
            .map(|(k, v)| ((*k).to_owned(), v.clone()))
            .collect(),
    )
}

#[test]
fn ubjson_encoder_decoder_matrix() {
    let mut encoder = UbjsonEncoder::new();
    let decoder = UbjsonDecoder::new();

    let docs = vec![
        PackValue::Undefined,
        PackValue::Null,
        PackValue::Bool(true),
        PackValue::Bool(false),
        PackValue::Integer(0),
        PackValue::Integer(1),
        PackValue::Integer(-1),
        PackValue::Integer(32_767),
        PackValue::Integer(-32_768),
        PackValue::Integer(2_147_483_647),
        PackValue::Integer(-1_147_483_647),
        PackValue::Integer(12_321_321_123),
        PackValue::Integer(-12_321_321_123),
        PackValue::Float(0.0),
        PackValue::Float(1.1),
        PackValue::Float(-12_321.321_123),
        PackValue::Str("".into()),
        PackValue::Str("abc123".into()),
        PackValue::Str("...................🎉.....................".into()),
        PackValue::Bytes(vec![]),
        PackValue::Bytes(vec![1, 2, 3]),
        PackValue::Array(vec![]),
        PackValue::Array(vec![PackValue::Integer(1)]),
        PackValue::Array(vec![
            PackValue::Integer(0),
            PackValue::Float(1.32),
            PackValue::Str("str".into()),
            PackValue::Bool(true),
            PackValue::Bool(false),
            PackValue::Null,
            PackValue::Array(vec![
                PackValue::Integer(1),
                PackValue::Integer(2),
                PackValue::Integer(3),
            ]),
        ]),
        obj(&[]),
        obj(&[("foo", PackValue::Str("bar".into()))]),
        obj(&[
            ("foo", PackValue::Str("bar".into())),
            ("baz", PackValue::Integer(123)),
        ]),
        obj(&[
            ("", PackValue::Null),
            ("null", PackValue::Bool(false)),
            ("true", PackValue::Bool(true)),
            (
                "str",
                PackValue::Str(
                    "asdfasdf ,asdf asdf asdf asdf asdf, asdflkasjdflakjsdflajskdlfkasdf".into(),
                ),
            ),
            ("num", PackValue::Integer(123)),
            (
                "arr",
                PackValue::Array(vec![
                    PackValue::Integer(1),
                    PackValue::Integer(2),
                    PackValue::Integer(3),
                ]),
            ),
            ("obj", obj(&[("foo", PackValue::Str("bar".into()))])),
            (
                "obj2",
                obj(&[("1", PackValue::Integer(2)), ("3", PackValue::Integer(4))]),
            ),
        ]),
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
fn ubjson_wire_and_extension_matrix() {
    let mut encoder = UbjsonEncoder::new();
    let decoder = UbjsonDecoder::new();

    assert_eq!(encoder.encode(&PackValue::Null), vec![0x5a]);
    assert_eq!(encoder.encode(&PackValue::Undefined), vec![0x4e]);
    assert_eq!(encoder.encode(&PackValue::Bool(true)), vec![0x54]);
    assert_eq!(encoder.encode(&PackValue::Bool(false)), vec![0x46]);

    assert_eq!(encoder.encode(&PackValue::Integer(42)), vec![0x55, 42]);
    assert_eq!(encoder.encode(&PackValue::Integer(-5)), vec![0x69, 251]);
    assert_eq!(encoder.encode(&PackValue::Integer(100_000))[0], 0x6c);

    let bytes = encoder.encode(&PackValue::Bytes(vec![1, 2, 3]));
    assert_eq!(&bytes[0..4], &[0x5b, 0x24, 0x55, 0x23]);
    assert_eq!(
        decoder.decode(&bytes).unwrap(),
        PackValue::Bytes(vec![1, 2, 3])
    );

    // Optimized typed array syntax decodes into JsonPackExtension in json-joy upstream.
    let typed = vec![0x5b, 0x24, 0x49, 0x23, 0x55, 0x02, 0x00, 0x2a, 0x00, 0x2b];
    assert_eq!(
        decoder.decode(&typed).unwrap(),
        PackValue::Extension(Box::new(JsonPackExtension::new(
            0x49,
            PackValue::Bytes(vec![0x00, 0x2a, 0x00, 0x2b])
        )))
    );
}

#[test]
fn ubjson_streaming_and_error_matrix() {
    let mut encoder = UbjsonEncoder::new();
    let decoder = UbjsonDecoder::new();

    encoder.writer.reset();
    encoder.write_start_arr();
    encoder.write_any(&PackValue::Integer(1));
    encoder.write_any(&PackValue::Integer(2));
    encoder.write_end_arr();
    let arr = encoder.writer.flush();
    assert_eq!(
        decoder.decode(&arr).unwrap(),
        PackValue::Array(vec![PackValue::Integer(1), PackValue::Integer(2)])
    );

    encoder.writer.reset();
    encoder.write_start_obj();
    encoder.write_key("foo");
    encoder.write_any(&PackValue::Str("bar".into()));
    encoder.write_end_obj();
    let obj_bytes = encoder.writer.flush();
    assert_eq!(
        decoder.decode(&obj_bytes).unwrap(),
        obj(&[("foo", PackValue::Str("bar".into()))])
    );

    assert!(matches!(
        decoder.decode(&[
            0x7b, 0x55, 0x09, b'_', b'_', b'p', b'r', b'o', b't', b'o', b'_', b'_', 0x55, 0x01,
            0x7d
        ]),
        Err(UbjsonError::InvalidKey)
    ));
    assert!(matches!(
        decoder.decode(&[0x53, 0x55, 0x02, b'a']),
        Err(UbjsonError::UnexpectedEof)
    ));
}
