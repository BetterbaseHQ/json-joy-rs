use json_joy_json_pack::json_binary::{
    parse, stringify, stringify_binary, unwrap_binary, wrap_binary,
};
use json_joy_json_pack::{JsonPackExtension, JsonPackValue, PackValue};

#[test]
fn json_binary_stringify_wire_matrix() {
    assert_eq!(
        stringify(PackValue::Bytes(vec![])).unwrap(),
        "\"data:application/octet-stream;base64,\""
    );

    assert_eq!(
        stringify(PackValue::Bytes(vec![0, 1, 2, 3])).unwrap(),
        "\"data:application/octet-stream;base64,AAECAw==\""
    );

    assert_eq!(
        stringify(PackValue::Object(vec![(
            "foo".into(),
            PackValue::Bytes(vec![0, 1, 2, 3]),
        )]))
        .unwrap(),
        "{\"foo\":\"data:application/octet-stream;base64,AAECAw==\"}"
    );

    assert_eq!(
        stringify(PackValue::Array(vec![
            PackValue::Null,
            PackValue::Integer(1),
            PackValue::Bytes(vec![0, 1, 2, 3]),
            PackValue::Str("a".into()),
        ]))
        .unwrap(),
        "[null,1,\"data:application/octet-stream;base64,AAECAw==\",\"a\"]"
    );
}

#[test]
fn json_binary_roundtrip_matrix() {
    let docs = vec![
        PackValue::Null,
        PackValue::Bool(true),
        PackValue::Integer(42),
        PackValue::Str("hello".into()),
        PackValue::Bytes(vec![1, 2, 3]),
        PackValue::Blob(JsonPackValue::new(vec![0x91, 0xa3, b'f', b'o', b'o'])),
        PackValue::Extension(Box::new(JsonPackExtension::new(
            33,
            PackValue::Bytes(vec![0xaa, 0xbb]),
        ))),
        PackValue::Array(vec![PackValue::Integer(1), PackValue::Bytes(vec![9, 8, 7])]),
        PackValue::Object(vec![
            ("a".into(), PackValue::Bytes(vec![0, 1, 2, 3])),
            ("b".into(), PackValue::Str("c".into())),
        ]),
    ];

    for doc in docs {
        let json = stringify(doc.clone()).unwrap();
        let decoded = parse(&json).unwrap();
        assert_eq!(decoded, doc);
    }
}

#[test]
fn json_binary_wrap_unwrap_matrix() {
    let wrapped = wrap_binary(PackValue::Bytes(vec![0xde, 0xad, 0xbe, 0xef]));
    assert_eq!(
        wrapped,
        serde_json::Value::String("data:application/octet-stream;base64,3q2+7w==".into())
    );

    let blob_wrapped = wrap_binary(PackValue::Blob(JsonPackValue::new(vec![1, 2, 3])));
    assert_eq!(
        blob_wrapped,
        serde_json::Value::String("data:application/msgpack;base64,AQID".into())
    );

    let ext_wrapped = wrap_binary(PackValue::Extension(Box::new(JsonPackExtension::new(
        33,
        PackValue::Bytes(vec![1, 2, 3, 4]),
    ))));
    assert_eq!(
        ext_wrapped,
        serde_json::Value::String("data:application/msgpack;base64;ext=33,AQIDBA==".into())
    );

    assert_eq!(
        unwrap_binary(serde_json::Value::String(
            "data:application/msgpack;base64;ext=33,AQIDBA==".into()
        )),
        PackValue::Extension(Box::new(JsonPackExtension::new(
            33,
            PackValue::Bytes(vec![1, 2, 3, 4]),
        )))
    );

    // Invalid base64 should remain an ordinary string.
    assert_eq!(
        unwrap_binary(serde_json::Value::String(
            "data:application/octet-stream;base64,%%%".into()
        )),
        PackValue::Str("data:application/octet-stream;base64,%%%".into())
    );
}

#[test]
fn json_binary_stringify_binary_helper_matrix() {
    assert_eq!(
        stringify_binary(&[0, 1, 2, 3]),
        "data:application/octet-stream;base64,AAECAw=="
    );
}
