use json_joy_json_pack::codecs::{
    CborJsonValueCodec, Codecs, JsonJsonValueCodec, JsonValueCodec, MsgPackJsonValueCodec,
};
use json_joy_json_pack::{EncodingFormat, PackValue};

fn sample_value() -> PackValue {
    PackValue::Object(vec![
        ("a".to_owned(), PackValue::Integer(123)),
        ("b".to_owned(), PackValue::Bool(true)),
        ("c".to_owned(), PackValue::Str("hello".to_owned())),
        ("d".to_owned(), PackValue::Bytes(vec![1, 2, 3, 4])),
        (
            "e".to_owned(),
            PackValue::Array(vec![PackValue::Null, PackValue::Float(1.5)]),
        ),
    ])
}

fn roundtrip_codec<C: JsonValueCodec>(
    codec: &mut C,
    expected_id: &str,
    expected_format: EncodingFormat,
    value: &PackValue,
) {
    assert_eq!(codec.id(), expected_id);
    assert_eq!(codec.format(), expected_format);
    let bytes = codec.encode(value).unwrap();
    let decoded = codec.decode(&bytes).unwrap();
    assert_eq!(decoded, *value);
}

#[test]
fn codecs_individual_matrix() {
    let value = sample_value();

    let mut cbor = CborJsonValueCodec::new();
    roundtrip_codec(&mut cbor, "cbor", EncodingFormat::Cbor, &value);

    let mut msgpack = MsgPackJsonValueCodec::new();
    roundtrip_codec(&mut msgpack, "msgpack", EncodingFormat::MsgPack, &value);

    let mut json = JsonJsonValueCodec::new();
    roundtrip_codec(&mut json, "json", EncodingFormat::Json, &value);
}

#[test]
fn codecs_aggregate_matrix() {
    let value = sample_value();
    let mut codecs = Codecs::new();

    let cbor_bytes = codecs.cbor.encode(&value).unwrap();
    let msgpack_bytes = codecs.msgpack.encode(&value).unwrap();
    let json_bytes = codecs.json.encode(&value).unwrap();

    assert_eq!(codecs.cbor.decode(&cbor_bytes).unwrap(), value);
    assert_eq!(codecs.msgpack.decode(&msgpack_bytes).unwrap(), value);
    assert_eq!(codecs.json.decode(&json_bytes).unwrap(), value);
}
