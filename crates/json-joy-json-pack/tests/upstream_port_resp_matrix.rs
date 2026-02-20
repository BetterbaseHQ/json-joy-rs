use json_joy_json_pack::resp::{RespDecoder, RespEncoder, RespEncoderLegacy, RespStreamingDecoder};
use json_joy_json_pack::{JsonPackExtension, PackValue};

fn as_utf8(bytes: &[u8]) -> String {
    std::str::from_utf8(bytes)
        .unwrap_or_else(|e| panic!("expected UTF-8 test bytes, got error: {e}"))
        .to_owned()
}

fn obj(fields: &[(&str, PackValue)]) -> PackValue {
    PackValue::Object(
        fields
            .iter()
            .map(|(k, v)| ((*k).to_owned(), v.clone()))
            .collect(),
    )
}

fn assert_pack_value_eq(actual: &PackValue, expected: &PackValue) {
    match (actual, expected) {
        (PackValue::Float(a), PackValue::Float(b)) if a.is_nan() && b.is_nan() => {}
        (PackValue::Array(a), PackValue::Array(b)) => {
            assert_eq!(a.len(), b.len(), "array length mismatch");
            for (left, right) in a.iter().zip(b.iter()) {
                assert_pack_value_eq(left, right);
            }
        }
        (PackValue::Object(a), PackValue::Object(b)) => {
            assert_eq!(a.len(), b.len(), "object field length mismatch");
            for ((ak, av), (bk, bv)) in a.iter().zip(b.iter()) {
                assert_eq!(ak, bk, "object key mismatch");
                assert_pack_value_eq(av, bv);
            }
        }
        _ => assert_eq!(actual, expected),
    }
}

#[test]
fn resp_encoder_wire_matrix() {
    let mut encoder = RespEncoder::new();

    encoder.write_simple_str("");
    assert_eq!(as_utf8(&encoder.writer.flush()), "+\r\n");

    encoder.write_simple_str("abc!");
    assert_eq!(as_utf8(&encoder.writer.flush()), "+abc!\r\n");

    encoder.write_bulk_str("");
    assert_eq!(as_utf8(&encoder.writer.flush()), "$0\r\n\r\n");

    encoder.write_bulk_str("abc!");
    assert_eq!(as_utf8(&encoder.writer.flush()), "$4\r\nabc!\r\n");

    encoder.write_verbatim_str("txt", "");
    assert_eq!(as_utf8(&encoder.writer.flush()), "=4\r\ntxt:\r\n");

    encoder.write_verbatim_str("txt", "asdf");
    assert_eq!(as_utf8(&encoder.writer.flush()), "=8\r\ntxt:asdf\r\n");

    let verbatim = PackValue::Extension(Box::new(JsonPackExtension::new(
        3,
        PackValue::Str("asdf".into()),
    )));
    assert_eq!(as_utf8(&encoder.encode(&verbatim)), "=8\r\ntxt:asdf\r\n");

    assert_eq!(
        as_utf8(&encoder.encode(&PackValue::Bytes(vec![]))),
        "$0\r\n\r\n"
    );
    assert_eq!(
        as_utf8(&encoder.encode(&PackValue::Bytes(vec![65, 66]))),
        "$2\r\nAB\r\n"
    );

    encoder.write_ascii_str("OK");
    assert_eq!(as_utf8(&encoder.writer.flush()), "+OK\r\n");

    encoder.write_simple_err("ERR");
    assert_eq!(as_utf8(&encoder.writer.flush()), "-ERR\r\n");

    encoder.write_bulk_err("a\nb");
    assert_eq!(as_utf8(&encoder.writer.flush()), "!3\r\na\nb\r\n");

    assert_eq!(as_utf8(&encoder.encode(&PackValue::Integer(0))), ":0\r\n");
    assert_eq!(
        as_utf8(&encoder.encode(&PackValue::Integer(23_423_432_543))),
        ":23423432543\r\n"
    );
    assert_eq!(
        as_utf8(&encoder.encode(&PackValue::Integer(-11_111_111))),
        ":-11111111\r\n"
    );

    assert_eq!(
        as_utf8(&encoder.encode(&PackValue::Array(vec![]))),
        "*0\r\n"
    );
    assert_eq!(
        as_utf8(&encoder.encode(&PackValue::Array(vec![
            PackValue::Integer(1),
            PackValue::Integer(2),
            PackValue::Integer(3),
        ]))),
        "*3\r\n:1\r\n:2\r\n:3\r\n"
    );
    assert_eq!(
        as_utf8(&encoder.encode(&PackValue::Array(vec![
            PackValue::Integer(1),
            PackValue::Str("abc".into()),
            PackValue::Integer(3),
        ]))),
        "*3\r\n:1\r\n+abc\r\n:3\r\n"
    );

    assert_eq!(as_utf8(&encoder.encode(&PackValue::Null)), "_\r\n");
    assert_eq!(
        as_utf8(&encoder.encode(&PackValue::Array(vec![
            PackValue::Integer(1),
            PackValue::Integer(2),
            PackValue::Null,
        ]))),
        "*3\r\n:1\r\n:2\r\n_\r\n"
    );

    encoder.write_null_str();
    assert_eq!(as_utf8(&encoder.writer.flush()), "$-1\r\n");

    encoder.write_null_arr();
    assert_eq!(as_utf8(&encoder.writer.flush()), "*-1\r\n");

    assert_eq!(as_utf8(&encoder.encode(&PackValue::Bool(true))), "#t\r\n");
    assert_eq!(as_utf8(&encoder.encode(&PackValue::Bool(false))), "#f\r\n");

    assert_eq!(as_utf8(&encoder.encode(&PackValue::Float(1.2))), ",1.2\r\n");
    assert_eq!(
        as_utf8(&encoder.encode(&PackValue::BigInt(12_345_678_901_234_567_890_i128))),
        "(12345678901234567890\r\n"
    );

    assert_eq!(
        as_utf8(&encoder.encode(&PackValue::Object(vec![]))),
        "%0\r\n"
    );
    assert_eq!(
        as_utf8(&encoder.encode(&obj(&[("foo", PackValue::Integer(123))]))),
        "%1\r\n+foo\r\n:123\r\n"
    );

    encoder.write_attr(&[]);
    assert_eq!(as_utf8(&encoder.writer.flush()), "|0\r\n");
    encoder.write_attr(&[("foo".into(), PackValue::Integer(1))]);
    assert_eq!(as_utf8(&encoder.writer.flush()), "|1\r\n+foo\r\n:1\r\n");
}

#[test]
fn resp_decoder_matrix() {
    let mut encoder = RespEncoder::new();
    let mut decoder = RespDecoder::new();

    let values = vec![
        PackValue::Null,
        PackValue::Bool(true),
        PackValue::Bool(false),
        PackValue::Integer(123),
        PackValue::Integer(-2_348_934),
        PackValue::BigInt(0),
        PackValue::BigInt(123),
        PackValue::BigInt(-2_348_934),
        PackValue::Array(vec![PackValue::Float(1.123)]),
        PackValue::Array(vec![PackValue::Float(-43.234435)]),
        PackValue::Array(vec![PackValue::Float(-5_445e-10)]),
        PackValue::Array(vec![PackValue::Float(f64::INFINITY)]),
        PackValue::Array(vec![PackValue::Float(f64::NEG_INFINITY)]),
        PackValue::Array(vec![PackValue::Float(f64::NAN)]),
        PackValue::Str("".into()),
        PackValue::Str("foo bar".into()),
        PackValue::Str("foo bar🍼".into()),
        PackValue::Str("foo bar\n🍼".into()),
        PackValue::Array(vec![]),
        PackValue::Array(vec![
            PackValue::Str("foo".into()),
            PackValue::Str("bar".into()),
        ]),
        obj(&[("foo", PackValue::Str("bar".into()))]),
        obj(&[
            ("foo", PackValue::Str("bar".into())),
            ("baz", PackValue::Array(vec![PackValue::Integer(1)])),
        ]),
    ];

    for value in values {
        let encoded = encoder.encode(&value);
        let decoded = decoder
            .read(&encoded)
            .unwrap_or_else(|e| panic!("decode failed for {value:?}: {e}"));
        assert_pack_value_eq(&decoded, &value);

        let encoded2 = encoder.encode(&value);
        let decoded2 = decoder.read(&encoded2).expect("repeat decode");
        assert_pack_value_eq(&decoded2, &value);
    }

    assert_eq!(decoder.read(b":+123\r\n").unwrap(), PackValue::Integer(123));
    assert_eq!(decoder.read(b"(+123\r\n").unwrap(), PackValue::BigInt(123));
    assert_eq!(
        decoder.read(b",inf\r\n").unwrap(),
        PackValue::Float(f64::INFINITY)
    );
    assert_eq!(
        decoder.read(b",-inf\r\n").unwrap(),
        PackValue::Float(f64::NEG_INFINITY)
    );
    match decoder.read(b",nan\r\n").unwrap() {
        PackValue::Float(f) => assert!(f.is_nan()),
        other => panic!("expected NaN float, got {other:?}"),
    }

    assert_eq!(
        decoder.read(b"=15\r\ntxt:Some string\r\n").unwrap(),
        PackValue::Str("Some string".into())
    );

    decoder.try_utf8 = true;
    assert_eq!(
        decoder.read(b"%1\r\n$3\r\nfoo\r\n$3\r\nbar\r\n").unwrap(),
        obj(&[("foo", PackValue::Str("bar".into()))]),
    );

    let invalid_utf8_obj = obj(&[("foo", PackValue::Bytes(vec![0xc3, 0x28]))]);
    let encoded = encoder.encode(&invalid_utf8_obj);
    let decoded = decoder.read(&encoded).unwrap();
    assert_eq!(decoded, invalid_utf8_obj);

    assert_eq!(decoder.read(b"$-1\r\n").unwrap(), PackValue::Null);
    assert_eq!(decoder.read(b"*-1\r\n").unwrap(), PackValue::Null);

    let cmd = encoder.encode_cmd(&["SET", "foo", "bar"]);
    decoder.reset(&cmd);
    let parsed_cmd = decoder.read_cmd().expect("read cmd");
    assert_eq!(
        parsed_cmd,
        vec![b"SET".to_vec(), b"foo".to_vec(), b"bar".to_vec()]
    );
}

#[test]
fn resp_decoder_skip_matrix() {
    let mut encoder = RespEncoder::new();
    let mut decoder = RespDecoder::new();
    let docs = vec![
        PackValue::Null,
        PackValue::Bool(true),
        PackValue::Integer(1),
        PackValue::Float(1.25),
        PackValue::Str("abc".into()),
        PackValue::Bytes(vec![1, 2, 3]),
        PackValue::Array(vec![PackValue::Integer(1), PackValue::Null]),
        obj(&[("foo", PackValue::Str("bar".into()))]),
    ];

    for doc in docs {
        encoder.write_any(&doc);
        encoder.write_any(&obj(&[("foo", PackValue::Str("bar".into()))]));
        let encoded = encoder.writer.flush();

        decoder.reset(&encoded);
        decoder.skip_any().expect("skip first");
        let decoded = decoder.read_any().expect("decode second");
        assert_eq!(decoded, obj(&[("foo", PackValue::Str("bar".into()))]));
    }
}

#[test]
fn resp_streaming_decoder_matrix() {
    let mut encoder = RespEncoder::new();
    let mut decoder = RespStreamingDecoder::new();

    let encoded = encoder.encode(&PackValue::Str("abc".into()));
    assert!(decoder.read().unwrap().is_none());
    decoder.push(&encoded);
    assert_eq!(decoder.read().unwrap(), Some(PackValue::Str("abc".into())));
    assert!(decoder.read().unwrap().is_none());

    let docs = vec![
        PackValue::Integer(1),
        PackValue::Float(123.1234),
        PackValue::Integer(-3),
        PackValue::Bool(true),
        PackValue::Null,
        PackValue::Bool(false),
        PackValue::Float(f64::INFINITY),
        PackValue::Float(f64::NEG_INFINITY),
        PackValue::Str("".into()),
        PackValue::Str("abc".into()),
        PackValue::Str("a\nb".into()),
        PackValue::Str("emoji: 🐶".into()),
        obj(&[]),
        PackValue::Array(vec![obj(&[
            ("foo", PackValue::Integer(-43)),
            ("bar", PackValue::Str("a\nb".into())),
        ])]),
    ];
    let mut stream = Vec::new();
    for doc in &docs {
        stream.extend_from_slice(&encoder.encode(doc));
    }
    let mut decoded = Vec::new();
    for byte in stream {
        decoder.push(&[byte]);
        if let Some(value) = decoder.read().expect("streaming read") {
            decoded.push(value);
        }
    }
    assert_eq!(decoded.len(), docs.len());
    for (actual, expected) in decoded.iter().zip(docs.iter()) {
        assert_pack_value_eq(actual, expected);
    }

    let docs = vec![
        obj(&[("a", PackValue::Integer(1))]),
        obj(&[("b", PackValue::Array(vec![PackValue::Str("x".into())]))]),
        PackValue::Array(vec![PackValue::Integer(1), PackValue::Integer(2)]),
    ];
    let mut stream = Vec::new();
    for doc in &docs {
        stream.extend_from_slice(&encoder.encode(doc));
    }
    let mut decoded = Vec::new();
    for chunk in stream.chunks(49) {
        decoder.push(chunk);
        while let Some(value) = decoder.read().expect("streaming read chunk") {
            decoded.push(value);
        }
    }
    assert_eq!(decoded, docs);

    let cmd = encoder.encode_cmd(&["SET", "foo", "bar"]);
    decoder.push(&cmd);
    let parsed_cmd = decoder.read_cmd().expect("stream read cmd");
    assert_eq!(
        parsed_cmd,
        Some(vec![b"SET".to_vec(), b"foo".to_vec(), b"bar".to_vec()])
    );

    encoder.write_any(&PackValue::Integer(1));
    encoder.write_any(&obj(&[("foo", PackValue::Str("bar".into()))]));
    let two_values = encoder.writer.flush();
    decoder.push(&two_values);
    assert_eq!(decoder.skip().unwrap(), Some(()));
    assert_eq!(
        decoder.read().unwrap(),
        Some(obj(&[("foo", PackValue::Str("bar".into()))]))
    );
}

#[test]
fn resp_legacy_encoder_matrix() {
    let mut encoder = RespEncoderLegacy::new();

    assert_eq!(
        as_utf8(&encoder.encode(&PackValue::Str("".into()))),
        "+\r\n"
    );
    assert_eq!(
        as_utf8(&encoder.encode(&PackValue::Str("asdf".into()))),
        "+asdf\r\n"
    );

    encoder.write_err("asdf");
    assert_eq!(as_utf8(&encoder.flush()), "-asdf\r\n");

    assert_eq!(as_utf8(&encoder.encode(&PackValue::Integer(0))), ":0\r\n");
    assert_eq!(
        as_utf8(&encoder.encode(&PackValue::Integer(123))),
        ":123\r\n"
    );
    assert_eq!(
        as_utf8(&encoder.encode(&PackValue::Integer(-422_469_777))),
        ":-422469777\r\n"
    );

    assert_eq!(
        as_utf8(&encoder.encode(&PackValue::Str("ab\nc".into()))),
        "$4\r\nab\nc\r\n"
    );
    assert_eq!(
        as_utf8(&encoder.encode(&PackValue::Bytes(vec![65]))),
        "$1\r\nA\r\n"
    );

    assert_eq!(
        as_utf8(&encoder.encode(&PackValue::Array(vec![
            PackValue::Str("a".into()),
            PackValue::Integer(1),
        ]))),
        "*2\r\n+a\r\n:1\r\n"
    );

    assert_eq!(as_utf8(&encoder.encode(&PackValue::Null)), "*-1\r\n");

    assert_eq!(
        as_utf8(&encoder.encode(&PackValue::Array(vec![
            PackValue::Str("a".into()),
            PackValue::Str("b".into()),
            PackValue::Null,
        ]))),
        "*3\r\n+a\r\n+b\r\n$-1\r\n"
    );

    assert_eq!(
        as_utf8(&encoder.encode(&PackValue::Bool(true))),
        "+TRUE\r\n"
    );
    assert_eq!(
        as_utf8(&encoder.encode(&PackValue::Bool(false))),
        "+FALSE\r\n"
    );
    assert_eq!(
        as_utf8(&encoder.encode(&PackValue::Float(1.23))),
        "+1.23\r\n"
    );
    assert_eq!(
        as_utf8(&encoder.encode(&obj(&[("foo", PackValue::Str("bar".into()))]))),
        "*2\r\n+foo\r\n+bar\r\n"
    );
}
