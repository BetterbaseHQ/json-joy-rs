use json_joy_json_pack::bencode::BencodeUint8Array;
use json_joy_json_pack::cbor::{decode as cbor_decode, encode as cbor_encode, CborUint8Array};
use json_joy_json_pack::json::JsonUint8Array;
use json_joy_json_pack::json_binary::{Base64String, BinaryString, CborString, MsgpackString};
use json_joy_json_pack::ws::{WsFrameDecodingError, WsFrameEncodingError};
use json_joy_json_pack::PackValue;

#[test]
fn surface_type_alias_matrix() {
    let bencode: BencodeUint8Array = vec![0x64];
    let cbor: CborUint8Array = vec![0xa1];
    let json: JsonUint8Array = b"{\"a\":1}".to_vec();
    let base64: Base64String = "AQID".to_owned();
    let binary: BinaryString = "data:application/octet-stream;base64,AQID".to_owned();
    let cbor_uri: CborString = "data:application/cbor;base64,oWE=".to_owned();
    let msgpack_uri: MsgpackString = "data:application/msgpack;base64,gaFhAQ==".to_owned();

    assert_eq!(bencode.len(), 1);
    assert_eq!(cbor.len(), 1);
    assert!(!json.is_empty());
    assert!(base64.ends_with("QID"));
    assert!(binary.starts_with("data:application/octet-stream;base64,"));
    assert!(cbor_uri.starts_with("data:application/cbor;base64,"));
    assert!(msgpack_uri.starts_with("data:application/msgpack;base64,"));
}

#[test]
fn cbor_shared_encode_decode_matrix() {
    let value = PackValue::Object(vec![
        (
            "k".to_owned(),
            PackValue::Array(vec![PackValue::Integer(1)]),
        ),
        ("ok".to_owned(), PackValue::Bool(true)),
    ]);
    let encoded = cbor_encode(&value);
    let decoded = cbor_decode(&encoded).expect("decode encoded cbor");
    assert_eq!(decoded, value);
}

#[test]
fn ws_error_surface_matrix() {
    assert_eq!(
        WsFrameDecodingError::InvalidFrame.to_string(),
        "invalid WebSocket frame"
    );
    assert_eq!(
        WsFrameEncodingError::InvalidFrame.to_string(),
        "WS_FRAME_ENCODING"
    );
}
