use json_joy_json_pack::msgpack::{decode, encode, encode_full, MsgPack};
use json_joy_json_pack::PackValue;

fn sample_value() -> PackValue {
    PackValue::Object(vec![
        ("foo".to_owned(), PackValue::Str("bar".to_owned())),
        (
            "arr".to_owned(),
            PackValue::Array(vec![PackValue::Integer(1), PackValue::Bool(true)]),
        ),
    ])
}

#[test]
fn msgpack_util_matrix() {
    let value = sample_value();

    let fast: MsgPack = encode(&value);
    let full: MsgPack = encode_full(&value);

    let fast_decoded = decode(&fast).unwrap();
    let full_decoded = decode(&full).unwrap();

    assert_eq!(fast_decoded, value);
    assert_eq!(full_decoded, value);
}
