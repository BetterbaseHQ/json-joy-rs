use json_joy_json_pack::json::{
    JsonDecoder, JsonDecoderDag, JsonDecoderPartial, JsonEncoder, JsonEncoderDag,
    JsonEncoderStable, JsonError,
};
use json_joy_json_pack::PackValue;

fn obj(fields: &[(&str, PackValue)]) -> PackValue {
    PackValue::Object(
        fields
            .iter()
            .map(|(k, v)| ((*k).to_owned(), v.clone()))
            .collect(),
    )
}

fn assert_json_eq(actual: &PackValue, expected: &PackValue) {
    match (actual, expected) {
        (PackValue::Float(a), PackValue::Float(b)) if a.is_nan() && b.is_nan() => {}
        (PackValue::Array(a), PackValue::Array(b)) => {
            assert_eq!(a.len(), b.len(), "array length mismatch");
            for (left, right) in a.iter().zip(b.iter()) {
                assert_json_eq(left, right);
            }
        }
        (PackValue::Object(a), PackValue::Object(b)) => {
            assert_eq!(a.len(), b.len(), "object field length mismatch");
            let mut left: Vec<_> = a.iter().collect();
            let mut right: Vec<_> = b.iter().collect();
            left.sort_by(|(ka, _), (kb, _)| ka.cmp(kb));
            right.sort_by(|(ka, _), (kb, _)| ka.cmp(kb));
            for ((ak, av), (bk, bv)) in left.into_iter().zip(right.into_iter()) {
                assert_eq!(ak, bk, "object key mismatch");
                assert_json_eq(av, bv);
            }
        }
        _ => assert_eq!(actual, expected),
    }
}

#[test]
fn json_encoder_decoder_matrix() {
    let mut encoder = JsonEncoder::new();
    let mut stable = JsonEncoderStable::new();
    let mut decoder = JsonDecoder::new();

    let values = vec![
        PackValue::Null,
        PackValue::Undefined,
        PackValue::Bool(true),
        PackValue::Bool(false),
        PackValue::Integer(0),
        PackValue::Integer(-123),
        PackValue::Float(1.1),
        PackValue::Float(-12321.321123),
        PackValue::Str("".into()),
        PackValue::Str("abc123".into()),
        PackValue::Str("...................🎉.....................".into()),
        PackValue::Bytes(vec![]),
        PackValue::Bytes(vec![4, 5, 6]),
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

    for value in values {
        let encoded = encoder.encode(&value);
        let stable_encoded = stable.encode(&value);

        let decoded = decoder
            .decode(&encoded)
            .unwrap_or_else(|e| panic!("decode failed for {value:?}: {e}"));
        let stable_decoded = decoder
            .decode(&stable_encoded)
            .unwrap_or_else(|e| panic!("stable decode failed for {value:?}: {e}"));

        assert_json_eq(&decoded, &value);
        assert_json_eq(&stable_decoded, &value);
    }
}

#[test]
fn json_decoder_edge_and_error_matrix() {
    let mut decoder = JsonDecoder::new();

    assert_eq!(decoder.decode(b"null").unwrap(), PackValue::Null);
    assert_eq!(
        decoder.decode(b" \n\r\t true \n").unwrap(),
        PackValue::Bool(true)
    );
    assert_eq!(decoder.decode(b"3n").unwrap(), PackValue::Integer(3));
    assert_eq!(
        decoder
            .decode(b"\"data:application/octet-stream;base64,BAUG\"")
            .unwrap(),
        PackValue::Bytes(vec![4, 5, 6])
    );
    assert_eq!(
        decoder
            .decode(b"\"data:application/cbor,base64;9w==\"")
            .unwrap(),
        PackValue::Undefined
    );

    assert!(matches!(decoder.decode(b"{"), Err(JsonError::Invalid(_))));
    assert!(matches!(
        decoder.decode(b"{\"__proto__\":1}"),
        Err(JsonError::InvalidKey)
    ));
}

#[test]
fn json_dag_encoder_decoder_matrix() {
    let mut encoder = JsonEncoderDag::new();
    let mut decoder = JsonDecoderDag::new();

    let sorted = obj(&[
        ("aaaaaa", PackValue::Integer(6)),
        ("aaaaab", PackValue::Integer(7)),
        ("aaaaac", PackValue::Integer(8)),
        ("aaaabb", PackValue::Integer(9)),
        ("bbbbb", PackValue::Integer(5)),
        ("cccc", PackValue::Integer(4)),
        ("ddd", PackValue::Integer(3)),
        ("ee", PackValue::Integer(2)),
        ("f", PackValue::Integer(1)),
    ]);
    let encoded_sorted = encoder.encode(&sorted);
    assert_eq!(
        String::from_utf8(encoded_sorted).unwrap(),
        "{\"f\":1,\"ee\":2,\"ddd\":3,\"cccc\":4,\"bbbbb\":5,\"aaaaaa\":6,\"aaaaab\":7,\"aaaaac\":8,\"aaaabb\":9}"
    );

    let bytes_doc = obj(&[("foo", PackValue::Bytes(b"hello world".to_vec()))]);
    let encoded_bytes = encoder.encode(&bytes_doc);
    assert_eq!(
        String::from_utf8(encoded_bytes.clone()).unwrap(),
        "{\"foo\":{\"/\":{\"bytes\":\"aGVsbG8gd29ybGQ\"}}}"
    );
    assert_eq!(decoder.decode(&encoded_bytes).unwrap(), bytes_doc);

    let cid_json = b"{\"/\":\"QmXn5v3z\"}";
    assert_eq!(
        decoder.decode(cid_json).unwrap(),
        PackValue::Str("QmXn5v3z".into())
    );

    let ws_bytes =
        b"  { \"foo\"  : {  \"/\"    :  {   \"bytes\" :    \"aGVsbG8gd29ybGQ\" }  }    }  ";
    assert_eq!(
        decoder.decode(ws_bytes).unwrap(),
        obj(&[("foo", PackValue::Bytes(b"hello world".to_vec()))])
    );
}

#[test]
fn json_decoder_partial_matrix() {
    let mut partial = JsonDecoderPartial::new();

    assert_eq!(
        partial.decode(b"[1, 2, 3 ").unwrap(),
        PackValue::Array(vec![
            PackValue::Integer(1),
            PackValue::Integer(2),
            PackValue::Integer(3),
        ])
    );
    assert_eq!(
        partial.decode(b"[true, \"asdf\",,").unwrap(),
        PackValue::Array(vec![PackValue::Bool(true), PackValue::Str("asdf".into())])
    );

    assert_eq!(
        partial.decode(b"{\"foo\": 1, \"bar\": ").unwrap(),
        obj(&[("foo", PackValue::Integer(1))])
    );

    assert_eq!(
        partial
            .decode(b"{ \"name\": { \"first\": \"ind\", \"last\": \"go")
            .unwrap(),
        obj(&[("name", obj(&[("first", PackValue::Str("ind".into()))]))])
    );

    let llm_output = br#"
{
    "name": "Alice",
    "age": 25,
    "hobbies": ["eat", "drink"
    "is_student": false
Some extra text after the JSON with missing closing brace."#;
    assert_eq!(
        partial.decode(llm_output).unwrap(),
        obj(&[
            ("name", PackValue::Str("Alice".into())),
            ("age", PackValue::Integer(25)),
            (
                "hobbies",
                PackValue::Array(vec![
                    PackValue::Str("eat".into()),
                    PackValue::Str("drink".into()),
                ]),
            ),
            ("is_student", PackValue::Bool(false)),
        ])
    );
}
