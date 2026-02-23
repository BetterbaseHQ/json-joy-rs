//! Binary serialization formats for json-joy (CBOR, MessagePack, JSON, and more).
//!
//! Upstream reference: `@jsonjoy.com/json-pack` v18.0.0
//! Source: `json-joy/packages/json-pack/src/`

mod constants;
mod json_pack_extension;
mod json_pack_mpint;
mod json_pack_value;
mod pack_value;

pub mod avro;
pub mod bencode;
pub mod bson;
pub mod cbor;
pub mod codecs;
pub mod ejson;
pub mod ion;
pub mod json;
pub mod json_binary;
pub mod msgpack;
pub mod resp;
pub mod rm;
pub mod rpc;
pub mod ssh;
pub mod ubjson;
pub mod util;
pub mod ws;
pub mod xdr;

pub use constants::EncodingFormat;
pub use json_pack_extension::JsonPackExtension;
pub use json_pack_mpint::JsonPackMpint;
pub use json_pack_value::JsonPackValue;
pub use pack_value::PackValue;

pub use cbor::{
    cbor_to_json, cbor_to_json_owned, decode_cbor_value, decode_cbor_value_with_consumed,
    decode_json_from_cbor_bytes, encode_cbor_value, encode_json_to_cbor_bytes, json_to_cbor,
    validate_cbor_exact_size, write_cbor_signed, write_cbor_text_like_json_pack,
    write_cbor_uint_major, write_json_like_json_pack, CborEncoder, CborError, CborJsonValueCodec,
};

#[cfg(test)]
mod tests {
    use super::bencode::{BencodeDecoder, BencodeEncoder};
    use super::cbor::*;
    use super::json_binary;
    use super::ubjson::{UbjsonDecoder, UbjsonEncoder};
    use super::PackValue;
    use serde_json::json;

    const TEST_F64_3_14: f64 = 314.0 / 100.0;
    const TEST_F64_3_14159: f64 = 314_159.0 / 100_000.0;
    const TEST_F64_2_71828: f64 = 271_828.0 / 100_000.0;

    #[test]
    fn json_cbor_roundtrip_matrix() {
        let cases = vec![
            json!(null),
            json!(true),
            json!(123),
            json!("hello"),
            json!([1, 2, 3]),
            json!({"a": 1, "b": [true, null, "x"]}),
        ];
        for case in cases {
            let bin = encode_json_to_cbor_bytes(&case).expect("encode cbor");
            let back = decode_json_from_cbor_bytes(&bin).expect("decode cbor");
            assert_eq!(back, case);
        }
    }

    #[test]
    fn json_pack_text_header_behavior_uses_max_utf8_size_guess() {
        // Six 3-byte codepoints: actual bytes=18 (fits short header), but
        // json-pack-style encoding uses max-size guess (6*4=24) => 0x78 length.
        let s = "€€€€€€";
        let mut out = Vec::new();
        write_cbor_text_like_json_pack(&mut out, s);
        assert_eq!(out[0], 0x78);
        assert_eq!(out[1], 18);
    }

    #[test]
    fn json_bytes_roundtrip_via_json_pack_encoding() {
        let value = json!({
            "k": ["x", 1, -2, true, null, {"nested": "v"}]
        });
        let bytes = encode_json_to_cbor_bytes(&value).expect("json-pack encode");
        let decoded = decode_json_from_cbor_bytes(&bytes).expect("decode to json");
        assert_eq!(decoded, value);
    }

    #[test]
    fn cbor_json_value_codec_roundtrip() {
        let mut codec = CborJsonValueCodec::new();
        let value = json!({"a":[1,2,3],"b":"x"});
        let bytes = codec.encode(&value).expect("encode");
        assert!(validate_cbor_exact_size(&bytes, bytes.len()).is_ok());
        let out = codec.decode(&bytes).expect("decode");
        assert_eq!(out, value);
    }

    #[test]
    fn bencode_roundtrip_json_values() {
        let cases = vec![
            (PackValue::Null, b"n".as_slice()),
            (PackValue::Bool(true), b"t".as_slice()),
            (PackValue::Bool(false), b"f".as_slice()),
            (PackValue::Integer(42), b"i42e".as_slice()),
            (PackValue::Integer(-7), b"i-7e".as_slice()),
        ];
        let mut enc = BencodeEncoder::new();
        for (val, expected) in cases {
            let bytes = enc.encode(&val);
            assert_eq!(&bytes, expected);
        }
        // String round-trip
        let mut enc = BencodeEncoder::new();
        let bytes = enc.encode(&PackValue::Str("hello".into()));
        assert_eq!(&bytes, b"5:hello");
        // Decode back
        let dec = BencodeDecoder::new();
        let result = dec.decode(b"5:hello").unwrap();
        // bencode strings decode as Bytes (raw binary)
        assert!(matches!(result, PackValue::Bytes(b) if b == b"hello"));
    }

    #[test]
    fn bencode_dict_sorted_keys() {
        let mut enc = BencodeEncoder::new();
        let value = PackValue::Object(vec![
            ("z".into(), PackValue::Integer(1)),
            ("a".into(), PackValue::Integer(2)),
        ]);
        let bytes = enc.encode(&value);
        // Keys must be sorted: 'a' before 'z'
        assert_eq!(&bytes, b"d1:ai2e1:zi1ee");
    }

    #[test]
    fn ubjson_roundtrip_null_and_bool() {
        let mut enc = UbjsonEncoder::new();
        assert_eq!(enc.encode(&PackValue::Null), &[0x5a]);
        assert_eq!(enc.encode(&PackValue::Bool(true)), &[0x54]);
        assert_eq!(enc.encode(&PackValue::Bool(false)), &[0x46]);
    }

    #[test]
    fn ubjson_integer_encoding() {
        let mut enc = UbjsonEncoder::new();
        // uint8
        assert_eq!(enc.encode(&PackValue::Integer(42)), &[0x55, 42]);
        // int8 negative
        let bytes = enc.encode(&PackValue::Integer(-5));
        assert_eq!(bytes[0], 0x69);
        assert_eq!(bytes[1] as i8, -5i8);
        // int32
        let bytes = enc.encode(&PackValue::Integer(100000));
        assert_eq!(bytes[0], 0x6c);
    }

    #[test]
    fn ubjson_string_roundtrip() {
        let mut enc = UbjsonEncoder::new();
        let dec = UbjsonDecoder::new();
        let bytes = enc.encode(&PackValue::Str("hello".into()));
        let result = dec.decode(&bytes).unwrap();
        assert!(matches!(result, PackValue::Str(s) if s == "hello"));
    }

    #[test]
    fn json_binary_wrap_unwrap_roundtrip() {
        let original = PackValue::Bytes(vec![1, 2, 3, 4]);
        let wrapped = json_binary::wrap_binary(original.clone());
        // Should be a string with the data URI prefix
        if let serde_json::Value::String(s) = &wrapped {
            assert!(s.starts_with("data:application/octet-stream;base64,"));
        } else {
            panic!("expected string");
        }
        let unwrapped = json_binary::unwrap_binary(wrapped);
        assert_eq!(unwrapped, original);
    }

    #[test]
    fn json_binary_parse_stringify_roundtrip() {
        let value = PackValue::Object(vec![
            ("key".into(), PackValue::Str("val".into())),
            ("bin".into(), PackValue::Bytes(vec![0xde, 0xad, 0xbe, 0xef])),
        ]);
        let json_str = json_binary::stringify(value.clone()).unwrap();
        let parsed = json_binary::parse(&json_str).unwrap();
        assert_eq!(parsed, value);
    }

    // --- Slice 2: JSON format ---

    #[test]
    fn json_encoder_primitives() {
        use super::json::JsonEncoder;
        let mut enc = JsonEncoder::new();
        assert_eq!(enc.encode(&PackValue::Null), b"null");
        assert_eq!(enc.encode(&PackValue::Bool(true)), b"true");
        assert_eq!(enc.encode(&PackValue::Bool(false)), b"false");
        assert_eq!(enc.encode(&PackValue::Integer(42)), b"42");
        assert_eq!(enc.encode(&PackValue::Integer(-7)), b"-7");
        assert_eq!(enc.encode(&PackValue::Float(1.5)), b"1.5");
    }

    #[test]
    fn json_encoder_string_and_binary() {
        use super::json::JsonEncoder;
        let mut enc = JsonEncoder::new();
        assert_eq!(enc.encode(&PackValue::Str("hello".into())), b"\"hello\"");
        let bin_out = enc.encode(&PackValue::Bytes(vec![1, 2, 3]));
        let s = std::str::from_utf8(&bin_out).unwrap();
        assert!(s.starts_with("\"data:application/octet-stream;base64,"));
        assert!(s.ends_with('"'));
    }

    #[test]
    fn json_encoder_array_and_object() {
        use super::json::JsonEncoder;
        let mut enc = JsonEncoder::new();
        let arr = PackValue::Array(vec![PackValue::Integer(1), PackValue::Integer(2)]);
        assert_eq!(enc.encode(&arr), b"[1,2]");
        let obj = PackValue::Object(vec![("a".into(), PackValue::Integer(1))]);
        assert_eq!(enc.encode(&obj), b"{\"a\":1}");
    }

    #[test]
    fn json_encoder_stable_sorts_keys() {
        use super::json::JsonEncoderStable;
        let mut enc = JsonEncoderStable::new();
        let obj = PackValue::Object(vec![
            ("bb".into(), PackValue::Integer(2)),
            ("a".into(), PackValue::Integer(1)),
            ("ccc".into(), PackValue::Integer(3)),
        ]);
        let out = enc.encode(&obj);
        let s = std::str::from_utf8(&out).unwrap();
        // "a" (len 1) before "bb" (len 2) before "ccc" (len 3)
        let a_pos = s.find("\"a\"").unwrap();
        let bb_pos = s.find("\"bb\"").unwrap();
        let ccc_pos = s.find("\"ccc\"").unwrap();
        assert!(a_pos < bb_pos);
        assert!(bb_pos < ccc_pos);
    }

    #[test]
    fn json_encoder_dag_binary() {
        use super::json::JsonEncoderDag;
        let mut enc = JsonEncoderDag::new();
        let out = enc.encode(&PackValue::Bytes(b"hello world".as_slice().to_vec()));
        let s = std::str::from_utf8(&out).unwrap();
        // DAG-JSON binary format: {"/":{"bytes":"<b64>"}}
        assert!(s.starts_with("{\"/\":{\"bytes\":\""), "got: {s}");
        assert!(s.ends_with("\"}}"), "got: {s}");
    }

    #[test]
    fn json_decoder_primitives() {
        use super::json::JsonDecoder;
        let mut dec = JsonDecoder::new();
        assert_eq!(dec.decode(b"null").unwrap(), PackValue::Null);
        assert_eq!(dec.decode(b"true").unwrap(), PackValue::Bool(true));
        assert_eq!(dec.decode(b"false").unwrap(), PackValue::Bool(false));
        assert_eq!(dec.decode(b"42").unwrap(), PackValue::Integer(42));
        assert_eq!(dec.decode(b"-7").unwrap(), PackValue::Integer(-7));
        assert_eq!(dec.decode(b"1.5").unwrap(), PackValue::Float(1.5));
    }

    #[test]
    fn json_decoder_string() {
        use super::json::JsonDecoder;
        let mut dec = JsonDecoder::new();
        assert_eq!(
            dec.decode(b"\"hello\"").unwrap(),
            PackValue::Str("hello".into())
        );
        assert_eq!(
            dec.decode(b"\"a\\nb\"").unwrap(),
            PackValue::Str("a\nb".into())
        );
    }

    #[test]
    fn json_decoder_undefined_sentinel() {
        use super::json::{JsonDecoder, JsonEncoder};
        let mut enc = JsonEncoder::new();
        let mut dec = JsonDecoder::new();
        // Encode undefined, decode it back
        let encoded = enc.encode(&PackValue::Undefined);
        assert_eq!(dec.decode(&encoded).unwrap(), PackValue::Undefined);
        // Also check undefined in an object context (regression for off-by-one cursor bug)
        let obj = PackValue::Object(vec![
            ("u".into(), PackValue::Undefined),
            ("n".into(), PackValue::Integer(1)),
        ]);
        let encoded = enc.encode(&obj);
        let decoded = dec.decode(&encoded).unwrap();
        assert_eq!(decoded, obj);
    }

    #[test]
    fn json_decoder_binary_data_uri() {
        use super::json::JsonDecoder;
        let mut dec = JsonDecoder::new();
        // Encode some bytes and decode back
        let mut enc = super::json::JsonEncoder::new();
        let original = vec![1u8, 2, 3, 4, 5];
        let encoded = enc.encode(&PackValue::Bytes(original.clone()));
        let decoded = dec.decode(&encoded).unwrap();
        assert_eq!(decoded, PackValue::Bytes(original));
    }

    #[test]
    fn json_decoder_array_and_object() {
        use super::json::JsonDecoder;
        let mut dec = JsonDecoder::new();
        let arr = dec.decode(b"[1,2,3]").unwrap();
        assert_eq!(
            arr,
            PackValue::Array(vec![
                PackValue::Integer(1),
                PackValue::Integer(2),
                PackValue::Integer(3),
            ])
        );
        let obj = dec.decode(b"{\"a\":1}").unwrap();
        assert_eq!(
            obj,
            PackValue::Object(vec![("a".into(), PackValue::Integer(1))])
        );
    }

    #[test]
    fn json_encoder_decoder_roundtrip() {
        use super::json::{JsonDecoder, JsonEncoder};
        let mut enc = JsonEncoder::new();
        let mut dec = JsonDecoder::new();
        let values = vec![
            PackValue::Null,
            PackValue::Bool(true),
            PackValue::Integer(12345),
            PackValue::Float(TEST_F64_3_14),
            PackValue::Str("hello, world!".into()),
            PackValue::Array(vec![PackValue::Integer(1), PackValue::Null]),
            PackValue::Object(vec![
                ("x".into(), PackValue::Bool(false)),
                ("y".into(), PackValue::Str("z".into())),
            ]),
        ];
        for v in values {
            let encoded = enc.encode(&v);
            let decoded = dec.decode(&encoded).unwrap();
            assert_eq!(decoded, v, "roundtrip failed for {v:?}");
        }
    }

    #[test]
    fn json_decoder_partial_incomplete_array() {
        use super::json::JsonDecoderPartial;
        let mut dec = JsonDecoderPartial::new();
        // Missing closing bracket
        let v = dec.decode(b"[1, 2, 3").unwrap();
        assert_eq!(
            v,
            PackValue::Array(vec![
                PackValue::Integer(1),
                PackValue::Integer(2),
                PackValue::Integer(3),
            ])
        );
        // Trailing comma
        let v = dec.decode(b"[1, 2, ").unwrap();
        assert_eq!(
            v,
            PackValue::Array(vec![PackValue::Integer(1), PackValue::Integer(2),])
        );
        // Corrupt element — upstream drops it, returns prior elements
        let v = dec.decode(b"[1, 2, x").unwrap();
        assert_eq!(
            v,
            PackValue::Array(vec![PackValue::Integer(1), PackValue::Integer(2),])
        );
    }

    #[test]
    fn json_decoder_partial_incomplete_object() {
        use super::json::JsonDecoderPartial;
        let mut dec = JsonDecoderPartial::new();
        // Missing value for last key — key-value pair is dropped
        let v = dec.decode(b"{\"foo\": 1, \"bar\":").unwrap();
        assert_eq!(
            v,
            PackValue::Object(vec![("foo".into(), PackValue::Integer(1))])
        );
        // Complete pairs
        let v = dec.decode(b"{\"a\":1,\"b\":2").unwrap();
        assert_eq!(
            v,
            PackValue::Object(vec![
                ("a".into(), PackValue::Integer(1)),
                ("b".into(), PackValue::Integer(2)),
            ])
        );
    }

    // --- Slice 4: MessagePack format ---

    #[test]
    fn msgpack_encoder_primitives() {
        use super::msgpack::MsgPackEncoderFast;
        let mut enc = MsgPackEncoderFast::new();
        // null = 0xc0
        assert_eq!(enc.encode(&PackValue::Null), &[0xc0]);
        // true = 0xc3, false = 0xc2
        assert_eq!(enc.encode(&PackValue::Bool(true)), &[0xc3]);
        assert_eq!(enc.encode(&PackValue::Bool(false)), &[0xc2]);
        // positive fixint
        assert_eq!(enc.encode(&PackValue::Integer(0)), &[0x00]);
        assert_eq!(enc.encode(&PackValue::Integer(127)), &[0x7f]);
        // uint16
        let out = enc.encode(&PackValue::Integer(1000));
        assert_eq!(out[0], 0xcd);
        // negative fixint
        let out = enc.encode(&PackValue::Integer(-1));
        assert_eq!(out[0], 0xff); // -1 as negative fixint
    }

    #[test]
    fn msgpack_encoder_string() {
        use super::msgpack::MsgPackEncoderFast;
        let mut enc = MsgPackEncoderFast::new();
        let out = enc.encode(&PackValue::Str("hello".into()));
        // fixstr: 0xa0 | 5 = 0xa5, then 5 bytes
        assert_eq!(out[0], 0xa5);
        assert_eq!(&out[1..], b"hello");
    }

    #[test]
    fn msgpack_encoder_binary() {
        use super::msgpack::MsgPackEncoderFast;
        let mut enc = MsgPackEncoderFast::new();
        let data = vec![1u8, 2, 3];
        let out = enc.encode(&PackValue::Bytes(data.clone()));
        // bin8: 0xc4, length, data
        assert_eq!(out[0], 0xc4);
        assert_eq!(out[1], 3);
        assert_eq!(&out[2..], &data);
    }

    #[test]
    fn msgpack_encoder_array() {
        use super::msgpack::MsgPackEncoderFast;
        let mut enc = MsgPackEncoderFast::new();
        let arr = PackValue::Array(vec![PackValue::Null, PackValue::Integer(1)]);
        let out = enc.encode(&arr);
        // fixarray: 0x92 (2 items)
        assert_eq!(out[0], 0x92);
        assert_eq!(out[1], 0xc0); // null
        assert_eq!(out[2], 0x01); // 1
    }

    #[test]
    fn msgpack_encoder_object() {
        use super::msgpack::MsgPackEncoderFast;
        let mut enc = MsgPackEncoderFast::new();
        let obj = PackValue::Object(vec![("a".into(), PackValue::Integer(1))]);
        let out = enc.encode(&obj);
        // fixmap: 0x81 (1 pair)
        assert_eq!(out[0], 0x81);
    }

    #[test]
    fn msgpack_encoder_stable_sorts_keys() {
        use super::msgpack::MsgPackEncoderStable;
        let mut enc = MsgPackEncoderStable::new();
        let obj = PackValue::Object(vec![
            ("z".into(), PackValue::Integer(1)),
            ("a".into(), PackValue::Integer(2)),
        ]);
        let out = enc.encode(&obj);
        // fixmap: 0x82 (2 pairs) — first key should be "a"
        assert_eq!(out[0], 0x82);
        // Second byte is fixstr header for "a" (0xa1)
        assert_eq!(out[1], 0xa1);
        assert_eq!(out[2], b'a');
    }

    #[test]
    fn msgpack_decoder_primitives() {
        use super::msgpack::MsgPackDecoderFast;
        let mut dec = MsgPackDecoderFast::new();
        assert_eq!(dec.decode(&[0xc0]).unwrap(), PackValue::Null);
        assert_eq!(dec.decode(&[0xc3]).unwrap(), PackValue::Bool(true));
        assert_eq!(dec.decode(&[0xc2]).unwrap(), PackValue::Bool(false));
        assert_eq!(dec.decode(&[0x7f]).unwrap(), PackValue::Integer(127));
        assert_eq!(dec.decode(&[0xff]).unwrap(), PackValue::Integer(-1));
    }

    #[test]
    fn msgpack_encoder_decoder_roundtrip() {
        use super::msgpack::{MsgPackDecoderFast, MsgPackEncoderFast};
        let mut enc = MsgPackEncoderFast::new();
        let mut dec = MsgPackDecoderFast::new();
        let values = vec![
            PackValue::Null,
            PackValue::Bool(true),
            PackValue::Bool(false),
            PackValue::Integer(0),
            PackValue::Integer(127),
            PackValue::Integer(-1),
            PackValue::Integer(1000),
            PackValue::Integer(-1000),
            PackValue::Float(TEST_F64_3_14),
            PackValue::Str("hello".into()),
            PackValue::Bytes(vec![1, 2, 3]),
            PackValue::Array(vec![PackValue::Integer(1), PackValue::Null]),
            PackValue::Object(vec![("key".into(), PackValue::Integer(42))]),
        ];
        for v in values {
            let encoded = enc.encode(&v);
            let decoded = dec.decode(&encoded).unwrap();
            assert_eq!(decoded, v, "roundtrip failed for {v:?}");
        }
    }

    #[test]
    fn msgpack_to_json_converter() {
        use super::msgpack::{MsgPackEncoderFast, MsgPackToJsonConverter};
        let mut enc = MsgPackEncoderFast::new();
        let mut conv = MsgPackToJsonConverter::new();
        let obj = PackValue::Object(vec![
            ("n".into(), PackValue::Null),
            ("b".into(), PackValue::Bool(true)),
            ("i".into(), PackValue::Integer(42)),
            ("s".into(), PackValue::Str("hi".into())),
        ]);
        let msgpack = enc.encode(&obj);
        let json_str = conv.convert(&msgpack);
        let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
        assert_eq!(parsed["n"], serde_json::Value::Null);
        assert_eq!(parsed["b"], serde_json::Value::Bool(true));
        assert_eq!(parsed["i"], serde_json::json!(42));
        assert_eq!(parsed["s"], serde_json::json!("hi"));
    }

    // --- Slice 5: RM (Record Marshalling) ---

    #[test]
    fn rm_encode_decode_simple_record() {
        use super::rm::{RmRecordDecoder, RmRecordEncoder};
        let mut enc = RmRecordEncoder::new();
        let mut dec = RmRecordDecoder::new();
        let payload = b"hello world";
        let frame = enc.encode_record(payload);
        // Header: 4 bytes (fin=1, length=11) + 11 bytes payload
        assert_eq!(frame.len(), 4 + payload.len());
        // fin bit should be set
        assert_eq!(frame[0] & 0x80, 0x80);
        // length = 11 in lower 31 bits
        let len = u32::from_be_bytes([frame[0] & 0x7f, frame[1], frame[2], frame[3]]);
        assert_eq!(len, payload.len() as u32);
        dec.push(&frame);
        let record = dec.read_record().expect("record should be available");
        assert_eq!(record, payload);
    }

    #[test]
    fn rm_encode_hdr() {
        use super::rm::RmRecordEncoder;
        let mut enc = RmRecordEncoder::new();
        // fin=true, length=42
        let hdr = enc.encode_hdr(true, 42);
        assert_eq!(hdr.len(), 4);
        let val = u32::from_be_bytes([hdr[0], hdr[1], hdr[2], hdr[3]]);
        assert_eq!(val, 0x8000_0000 | 42);
        // fin=false
        let hdr2 = enc.encode_hdr(false, 100);
        let val2 = u32::from_be_bytes([hdr2[0], hdr2[1], hdr2[2], hdr2[3]]);
        assert_eq!(val2, 100);
    }

    #[test]
    fn rm_decoder_needs_more_data() {
        use super::rm::RmRecordDecoder;
        let mut dec = RmRecordDecoder::new();
        // Push only the header, not the payload
        dec.push(&[0x80, 0x00, 0x00, 0x05]); // fin=1, len=5
                                             // No payload yet => should return None
        assert!(dec.read_record().is_none());
        // Push the payload
        dec.push(b"hello");
        let record = dec.read_record().expect("should now have record");
        assert_eq!(record, b"hello");
    }

    // --- Slice 5: SSH ---

    #[test]
    fn ssh_boolean_roundtrip() {
        use super::ssh::{SshDecoder, SshEncoder};
        let mut enc = SshEncoder::new();
        let mut dec = SshDecoder::new();
        for b in [true, false] {
            enc.write_boolean(b);
            let bytes = enc.writer.flush();
            dec.reset(&bytes);
            assert_eq!(dec.read_boolean().unwrap(), b);
        }
    }

    #[test]
    fn ssh_uint32_roundtrip() {
        use super::ssh::{SshDecoder, SshEncoder};
        let mut enc = SshEncoder::new();
        let mut dec = SshDecoder::new();
        for val in [0u32, 1, 255, 65535, 0xffff_ffff] {
            enc.write_uint32(val);
            let bytes = enc.writer.flush();
            dec.reset(&bytes);
            assert_eq!(dec.read_uint32().unwrap(), val);
        }
    }

    #[test]
    fn ssh_str_roundtrip() {
        use super::ssh::{SshDecoder, SshEncoder};
        let mut enc = SshEncoder::new();
        let mut dec = SshDecoder::new();
        enc.write_str("hello, world!");
        let bytes = enc.writer.flush();
        dec.reset(&bytes);
        assert_eq!(dec.read_str().unwrap(), "hello, world!");
    }

    #[test]
    fn ssh_name_list_roundtrip() {
        use super::ssh::{SshDecoder, SshEncoder};
        use super::PackValue;
        let mut enc = SshEncoder::new();
        let mut dec = SshDecoder::new();
        let names = vec![
            PackValue::Str("aes128-ctr".into()),
            PackValue::Str("aes256-ctr".into()),
        ];
        enc.write_name_list(&names).unwrap();
        let bytes = enc.writer.flush();
        dec.reset(&bytes);
        let decoded = dec.read_name_list().unwrap();
        assert_eq!(decoded, vec!["aes128-ctr", "aes256-ctr"]);
    }

    // --- Slice 5: WebSocket ---

    #[test]
    fn ws_encode_ping_empty() {
        use super::ws::WsFrameEncoder;
        let mut enc = WsFrameEncoder::new();
        let frame = enc.encode_ping(None);
        // Minimum ping: 2-byte header (fin=1, opcode=9, no mask, length=0)
        assert_eq!(frame.len(), 2);
        assert_eq!(frame[0], 0b1000_1001); // fin=1, opcode=9
        assert_eq!(frame[1], 0x00); // no mask, length=0
    }

    #[test]
    fn ws_encode_ping_with_data() {
        use super::ws::WsFrameEncoder;
        let mut enc = WsFrameEncoder::new();
        let frame = enc.encode_ping(Some(b"test"));
        assert_eq!(frame.len(), 2 + 4);
        assert_eq!(frame[0], 0b1000_1001);
        assert_eq!(frame[1], 4); // length=4
        assert_eq!(&frame[2..], b"test");
    }

    #[test]
    fn ws_encode_hdr_short_length() {
        use super::ws::{WsFrameEncoder, WsFrameOpcode};
        let mut enc = WsFrameEncoder::new();
        let frame = enc.encode_hdr(true, WsFrameOpcode::Binary, 100, 0);
        assert_eq!(frame.len(), 2);
        assert_eq!(frame[0], 0b1000_0010); // fin=1, opcode=2
        assert_eq!(frame[1], 100);
    }

    #[test]
    fn ws_encode_data_msg_hdr_fast_small() {
        use super::ws::WsFrameEncoder;
        let mut enc = WsFrameEncoder::new();
        let frame = enc.encode_data_msg_hdr_fast(10);
        assert_eq!(frame.len(), 2);
        assert_eq!(frame[0], 0b1000_0010); // fin=1, binary
        assert_eq!(frame[1], 10);
    }

    #[test]
    fn ws_decode_simple_frame_header() {
        use super::ws::{WsFrame, WsFrameDecoder};
        let mut dec = WsFrameDecoder::new();
        // fin=1, opcode=2 (binary), no mask, length=5
        dec.push(vec![0b1000_0010, 5, b'h', b'e', b'l', b'l', b'o']);
        let frame = dec.read_frame_header().expect("ok").expect("frame");
        match frame {
            WsFrame::Data(h) => {
                assert!(h.fin);
                assert_eq!(h.opcode, 2);
                assert_eq!(h.length, 5);
                assert!(h.mask.is_none());
            }
            _ => panic!("expected Data frame"),
        }
    }

    // --- Slice 5: BSON ---

    #[test]
    fn bson_encode_decode_simple_document() {
        use super::bson::{BsonDecoder, BsonEncoder, BsonValue};
        let enc = BsonEncoder::new();
        let mut dec = BsonDecoder::new();
        let fields = vec![
            ("name".to_string(), BsonValue::Str("Alice".to_string())),
            ("age".to_string(), BsonValue::Int32(30)),
            ("active".to_string(), BsonValue::Boolean(true)),
        ];
        let bytes = enc.encode(&fields);
        let decoded = dec.decode(&bytes).unwrap();
        assert_eq!(decoded.len(), 3);
        assert_eq!(decoded[0].0, "name");
        assert!(matches!(decoded[0].1, BsonValue::Str(ref s) if s == "Alice"));
        assert_eq!(decoded[1].0, "age");
        assert!(matches!(decoded[1].1, BsonValue::Int32(30)));
        assert_eq!(decoded[2].0, "active");
        assert!(matches!(decoded[2].1, BsonValue::Boolean(true)));
    }

    #[test]
    fn bson_null_and_float() {
        use super::bson::{BsonDecoder, BsonEncoder, BsonValue};
        let enc = BsonEncoder::new();
        let mut dec = BsonDecoder::new();
        let fields = vec![
            ("n".to_string(), BsonValue::Null),
            ("f".to_string(), BsonValue::Float(TEST_F64_3_14)),
        ];
        let bytes = enc.encode(&fields);
        let decoded = dec.decode(&bytes).unwrap();
        assert!(matches!(decoded[0].1, BsonValue::Null));
        if let BsonValue::Float(f) = decoded[1].1 {
            assert!((f - TEST_F64_3_14).abs() < 1e-10);
        } else {
            panic!("expected float");
        }
    }

    #[test]
    fn bson_nested_document() {
        use super::bson::{BsonDecoder, BsonEncoder, BsonValue};
        let enc = BsonEncoder::new();
        let mut dec = BsonDecoder::new();
        let inner = vec![("x".to_string(), BsonValue::Int32(1))];
        let fields = vec![("obj".to_string(), BsonValue::Document(inner))];
        let bytes = enc.encode(&fields);
        let decoded = dec.decode(&bytes).unwrap();
        if let BsonValue::Document(inner_dec) = &decoded[0].1 {
            assert_eq!(inner_dec[0].0, "x");
            assert!(matches!(inner_dec[0].1, BsonValue::Int32(1)));
        } else {
            panic!("expected nested document");
        }
    }

    // --- Slice 5: RESP3 ---

    #[test]
    fn resp_encode_null() {
        use super::resp::RespEncoder;
        use super::PackValue;
        let mut enc = RespEncoder::new();
        let out = enc.encode(&PackValue::Null);
        assert_eq!(out, b"_\r\n");
    }

    #[test]
    fn resp_encode_bool() {
        use super::resp::RespEncoder;
        use super::PackValue;
        let mut enc = RespEncoder::new();
        assert_eq!(enc.encode(&PackValue::Bool(true)), b"#t\r\n");
        assert_eq!(enc.encode(&PackValue::Bool(false)), b"#f\r\n");
    }

    #[test]
    fn resp_encode_integer() {
        use super::resp::RespEncoder;
        use super::PackValue;
        let mut enc = RespEncoder::new();
        assert_eq!(enc.encode(&PackValue::Integer(42)), b":42\r\n");
        assert_eq!(enc.encode(&PackValue::Integer(-7)), b":-7\r\n");
        assert_eq!(enc.encode(&PackValue::Integer(0)), b":0\r\n");
    }

    #[test]
    fn resp_encode_simple_string() {
        use super::resp::RespEncoder;
        use super::PackValue;
        let mut enc = RespEncoder::new();
        let out = enc.encode(&PackValue::Str("hello".into()));
        assert_eq!(out, b"+hello\r\n");
    }

    #[test]
    fn resp_encode_binary() {
        use super::resp::RespEncoder;
        use super::PackValue;
        let mut enc = RespEncoder::new();
        let out = enc.encode(&PackValue::Bytes(b"bin".to_vec()));
        assert_eq!(out, b"$3\r\nbin\r\n");
    }

    #[test]
    fn resp_encode_array() {
        use super::resp::RespEncoder;
        use super::PackValue;
        let mut enc = RespEncoder::new();
        let arr = PackValue::Array(vec![PackValue::Integer(1), PackValue::Integer(2)]);
        let out = enc.encode(&arr);
        assert_eq!(out, b"*2\r\n:1\r\n:2\r\n");
    }

    #[test]
    fn resp_decode_null() {
        use super::resp::RespDecoder;
        use super::PackValue;
        let mut dec = RespDecoder::new();
        assert_eq!(dec.decode(b"_\r\n").unwrap(), PackValue::Null);
    }

    #[test]
    fn resp_decode_bool() {
        use super::resp::RespDecoder;
        use super::PackValue;
        let mut dec = RespDecoder::new();
        assert_eq!(dec.decode(b"#t\r\n").unwrap(), PackValue::Bool(true));
        assert_eq!(dec.decode(b"#f\r\n").unwrap(), PackValue::Bool(false));
    }

    #[test]
    fn resp_decode_integer() {
        use super::resp::RespDecoder;
        use super::PackValue;
        let mut dec = RespDecoder::new();
        assert_eq!(dec.decode(b":42\r\n").unwrap(), PackValue::Integer(42));
        assert_eq!(dec.decode(b":-7\r\n").unwrap(), PackValue::Integer(-7));
    }

    #[test]
    fn resp_decode_simple_string() {
        use super::resp::RespDecoder;
        use super::PackValue;
        let mut dec = RespDecoder::new();
        assert_eq!(
            dec.decode(b"+hello\r\n").unwrap(),
            PackValue::Str("hello".into())
        );
    }

    #[test]
    fn resp_encode_decode_roundtrip() {
        use super::resp::{RespDecoder, RespEncoder};
        use super::PackValue;
        let mut enc = RespEncoder::new();
        let mut dec = RespDecoder::new();
        let values = vec![
            PackValue::Null,
            PackValue::Bool(true),
            PackValue::Bool(false),
            PackValue::Integer(0),
            PackValue::Integer(42),
            PackValue::Integer(-100),
            PackValue::Float(TEST_F64_3_14),
            PackValue::Str("hello".into()),
            PackValue::Array(vec![PackValue::Integer(1), PackValue::Null]),
        ];
        for v in values {
            let bytes = enc.encode(&v);
            let decoded = dec
                .decode(&bytes)
                .unwrap_or_else(|e| panic!("decode failed for {v:?}: {e}"));
            // For arrays, check recursively
            match (&v, &decoded) {
                (PackValue::Array(a), PackValue::Array(b)) => assert_eq!(a.len(), b.len()),
                _ => assert_eq!(decoded, v, "roundtrip failed for {v:?}"),
            }
        }
    }

    // ---------------------------------------------------------------- Slice 6: XDR

    #[test]
    fn xdr_int_roundtrip() {
        use super::xdr::{XdrDecoder, XdrEncoder};
        let mut enc = XdrEncoder::new();
        let mut dec = XdrDecoder::new();
        for n in [-1i32, 0, 1, 42, -2147483648, 2147483647] {
            enc.write_int(n);
            let bytes = enc.writer.flush();
            dec.reset(&bytes);
            assert_eq!(dec.read_int().unwrap(), n, "int {n}");
        }
    }

    #[test]
    fn xdr_unsigned_int_roundtrip() {
        use super::xdr::{XdrDecoder, XdrEncoder};
        let mut enc = XdrEncoder::new();
        let mut dec = XdrDecoder::new();
        for n in [0u32, 1, 255, 65535, 4294967295] {
            enc.write_unsigned_int(n);
            let bytes = enc.writer.flush();
            dec.reset(&bytes);
            assert_eq!(dec.read_unsigned_int().unwrap(), n, "uint {n}");
        }
    }

    #[test]
    fn xdr_string_roundtrip() {
        use super::xdr::{XdrDecoder, XdrEncoder};
        let mut enc = XdrEncoder::new();
        let mut dec = XdrDecoder::new();
        let s = "hello world";
        enc.write_str(s);
        let bytes = enc.writer.flush();
        dec.reset(&bytes);
        assert_eq!(dec.read_string().unwrap(), s);
    }

    #[test]
    fn xdr_opaque_padding() {
        use super::xdr::{XdrDecoder, XdrEncoder};
        let mut enc = XdrEncoder::new();
        let data = b"abc"; // 3 bytes → padded to 4
        enc.write_unsigned_int(data.len() as u32);
        enc.write_opaque(data);
        let bytes = enc.writer.flush();
        // Should be 4 bytes (length) + 4 bytes (padded data) = 8
        assert_eq!(bytes.len(), 8);
        let mut dec = XdrDecoder::new();
        dec.reset(&bytes);
        let decoded = dec.read_varlen_opaque().unwrap();
        assert_eq!(decoded, data.to_vec());
    }

    #[test]
    fn xdr_double_roundtrip() {
        use super::xdr::{XdrDecoder, XdrEncoder};
        let mut enc = XdrEncoder::new();
        let mut dec = XdrDecoder::new();
        enc.write_double(TEST_F64_3_14159);
        let bytes = enc.writer.flush();
        dec.reset(&bytes);
        let decoded = dec.read_double().unwrap();
        assert!((decoded - TEST_F64_3_14159).abs() < 1e-10);
    }

    #[test]
    fn xdr_boolean_roundtrip() {
        use super::xdr::{XdrDecoder, XdrEncoder};
        let mut enc = XdrEncoder::new();
        let mut dec = XdrDecoder::new();
        enc.write_boolean(true);
        enc.write_boolean(false);
        let bytes = enc.writer.flush();
        dec.reset(&bytes);
        assert!(dec.read_boolean().unwrap());
        assert!(!dec.read_boolean().unwrap());
    }

    // ---------------------------------------------------------------- Slice 6: RPC

    #[test]
    fn rpc_call_message_roundtrip() {
        use super::rpc::{RpcMessage, RpcMessageDecoder, RpcMessageEncoder, RpcOpaqueAuth};
        let mut enc = RpcMessageEncoder::new();
        let cred = RpcOpaqueAuth::none();
        let verf = RpcOpaqueAuth::none();
        let bytes = enc
            .encode_call(42, 100003, 3, 1, &cred, &verf, &[])
            .unwrap();
        let dec = RpcMessageDecoder::new();
        let msg = dec.decode_message(&bytes).unwrap().unwrap();
        if let RpcMessage::Call(call) = msg {
            assert_eq!(call.xid, 42);
            assert_eq!(call.prog, 100003);
            assert_eq!(call.vers, 3);
            assert_eq!(call.proc_, 1);
        } else {
            panic!("expected Call message");
        }
    }

    #[test]
    fn rpc_accepted_reply_roundtrip() {
        use super::rpc::{
            RpcAcceptStat, RpcMessage, RpcMessageDecoder, RpcMessageEncoder, RpcOpaqueAuth,
        };
        let mut enc = RpcMessageEncoder::new();
        let verf = RpcOpaqueAuth::none();
        let results = b"\x00\x00\x00\x01";
        let bytes = enc
            .encode_accepted_reply(99, &verf, 0, None, results)
            .unwrap();
        let dec = RpcMessageDecoder::new();
        let msg = dec.decode_message(&bytes).unwrap().unwrap();
        if let RpcMessage::AcceptedReply(reply) = msg {
            assert_eq!(reply.xid, 99);
            assert_eq!(reply.stat, RpcAcceptStat::Success);
            assert_eq!(reply.results, Some(results.to_vec()));
        } else {
            panic!("expected AcceptedReply");
        }
    }

    #[test]
    fn rpc_rejected_reply_auth_error() {
        use super::rpc::{
            RpcAuthStat, RpcMessage, RpcMessageDecoder, RpcMessageEncoder, RpcRejectStat,
        };
        let mut enc = RpcMessageEncoder::new();
        let bytes = enc.encode_rejected_reply(7, 1, None, Some(1));
        let dec = RpcMessageDecoder::new();
        let msg = dec.decode_message(&bytes).unwrap().unwrap();
        if let RpcMessage::RejectedReply(reply) = msg {
            assert_eq!(reply.xid, 7);
            assert_eq!(reply.stat, RpcRejectStat::AuthError);
            assert_eq!(reply.auth_stat, Some(RpcAuthStat::AuthBadcred));
        } else {
            panic!("expected RejectedReply");
        }
    }

    #[test]
    fn rpc_opaque_auth_body() {
        use super::rpc::{
            RpcAuthFlavor, RpcMessage, RpcMessageDecoder, RpcMessageEncoder, RpcOpaqueAuth,
        };
        let mut enc = RpcMessageEncoder::new();
        let cred = RpcOpaqueAuth {
            flavor: RpcAuthFlavor::AuthSys,
            body: b"uid\x00".to_vec(),
        };
        let verf = RpcOpaqueAuth::none();
        let bytes = enc.encode_call(1, 1, 1, 1, &cred, &verf, &[]).unwrap();
        let dec = RpcMessageDecoder::new();
        let msg = dec.decode_message(&bytes).unwrap().unwrap();
        if let RpcMessage::Call(call) = msg {
            assert_eq!(call.cred.flavor, RpcAuthFlavor::AuthSys);
            assert_eq!(call.cred.body, b"uid\x00".to_vec());
        } else {
            panic!("expected Call");
        }
    }

    // ---------------------------------------------------------------- Slice 6: Avro

    #[test]
    fn avro_null_is_zero_bytes() {
        use super::avro::AvroEncoder;
        let mut enc = AvroEncoder::new();
        enc.write_null();
        assert!(enc.writer.flush().is_empty());
    }

    #[test]
    fn avro_boolean_roundtrip() {
        use super::avro::{AvroDecoder, AvroEncoder};
        let mut enc = AvroEncoder::new();
        let mut dec = AvroDecoder::new();
        enc.write_boolean(true);
        enc.write_boolean(false);
        let bytes = enc.writer.flush();
        assert_eq!(bytes, [1, 0]);
        dec.reset(&bytes);
        assert!(dec.read_boolean().unwrap());
        assert!(!dec.read_boolean().unwrap());
    }

    #[test]
    fn avro_int_zigzag_roundtrip() {
        use super::avro::{AvroDecoder, AvroEncoder};
        let mut enc = AvroEncoder::new();
        let mut dec = AvroDecoder::new();
        for n in [-64i32, -1, 0, 1, 63, 127, -2147483648, 2147483647] {
            enc.write_int(n);
            let bytes = enc.writer.flush();
            dec.reset(&bytes);
            assert_eq!(dec.read_int().unwrap(), n, "int {n}");
        }
    }

    #[test]
    fn avro_long_zigzag_roundtrip() {
        use super::avro::{AvroDecoder, AvroEncoder};
        let mut enc = AvroEncoder::new();
        let mut dec = AvroDecoder::new();
        for n in [-1i64, 0, 1, 1000, -9876543210, 9876543210] {
            enc.write_long(n);
            let bytes = enc.writer.flush();
            dec.reset(&bytes);
            assert_eq!(dec.read_long().unwrap(), n, "long {n}");
        }
    }

    #[test]
    fn avro_string_roundtrip() {
        use super::avro::{AvroDecoder, AvroEncoder};
        let mut enc = AvroEncoder::new();
        let mut dec = AvroDecoder::new();
        enc.write_str("hello");
        let bytes = enc.writer.flush();
        dec.reset(&bytes);
        assert_eq!(dec.read_str().unwrap(), "hello");
    }

    #[test]
    fn avro_bytes_roundtrip() {
        use super::avro::{AvroDecoder, AvroEncoder};
        let mut enc = AvroEncoder::new();
        let mut dec = AvroDecoder::new();
        let data = b"\x01\x02\x03\xff";
        enc.write_bytes(data);
        let bytes = enc.writer.flush();
        dec.reset(&bytes);
        assert_eq!(dec.read_bytes().unwrap(), data.to_vec());
    }

    #[test]
    fn avro_double_roundtrip() {
        use super::avro::{AvroDecoder, AvroEncoder};
        let mut enc = AvroEncoder::new();
        let mut dec = AvroDecoder::new();
        enc.write_double(TEST_F64_2_71828);
        let bytes = enc.writer.flush();
        dec.reset(&bytes);
        let v = dec.read_double().unwrap();
        assert!((v - TEST_F64_2_71828).abs() < 1e-10);
    }

    #[test]
    fn avro_str_encode_decode() {
        use super::avro::{AvroDecoder, AvroEncoder};
        let mut enc = AvroEncoder::new();
        let mut dec = AvroDecoder::new();
        enc.write_str("test");
        let bytes = enc.writer.flush();
        // String: unsigned varint(byteLen) + UTF-8 bytes.
        assert_eq!(bytes[0], 4);
        assert_eq!(&bytes[1..], b"test");
        dec.reset(&bytes);
        assert_eq!(dec.read_str().unwrap(), "test");
    }

    // ---------------------------------------------------------------- Slice 6: Ion

    #[test]
    fn ion_encode_null() {
        use super::ion::IonEncoder;
        let mut enc = IonEncoder::new();
        let bytes = enc.encode(&PackValue::Null);
        // IVM (4 bytes) + null typedesc (0x0f = 1 byte)
        assert_eq!(bytes, [0xe0, 0x01, 0x00, 0xea, 0x0f]);
    }

    #[test]
    fn ion_encode_bool() {
        use super::ion::IonEncoder;
        let mut enc = IonEncoder::new();
        let bytes = enc.encode(&PackValue::Bool(true));
        // IVM + BOOL|1 = 0x11
        assert_eq!(bytes, [0xe0, 0x01, 0x00, 0xea, 0x11]);
        let bytes2 = enc.encode(&PackValue::Bool(false));
        assert_eq!(bytes2, [0xe0, 0x01, 0x00, 0xea, 0x10]);
    }

    #[test]
    fn ion_encode_uint_zero() {
        use super::ion::IonEncoder;
        let mut enc = IonEncoder::new();
        let bytes = enc.encode(&PackValue::UInteger(0));
        // IVM + UINT|0 = 0x20
        assert_eq!(bytes, [0xe0, 0x01, 0x00, 0xea, 0x20]);
    }

    #[test]
    fn ion_roundtrip_primitives() {
        use super::ion::{IonDecoder, IonEncoder};
        let mut enc = IonEncoder::new();
        let mut dec = IonDecoder::new();
        let cases = vec![
            PackValue::Null,
            PackValue::Bool(true),
            PackValue::Bool(false),
            PackValue::UInteger(0),
            PackValue::UInteger(42),
            PackValue::Integer(-7),
            PackValue::Str("hello".to_string()),
        ];
        for val in cases {
            let bytes = enc.encode(&val);
            let decoded = dec.decode(&bytes).expect("ion decode");
            assert_eq!(decoded, val, "ion roundtrip for {val:?}");
        }
    }

    #[test]
    fn ion_roundtrip_object_with_string_key() {
        use super::ion::{IonDecoder, IonEncoder};
        let mut enc = IonEncoder::new();
        let mut dec = IonDecoder::new();
        let val = PackValue::Object(vec![("key".to_string(), PackValue::UInteger(42))]);
        let bytes = enc.encode(&val);
        let decoded = dec.decode(&bytes).expect("ion decode object");
        if let PackValue::Object(fields) = decoded {
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].0, "key");
            assert_eq!(fields[0].1, PackValue::UInteger(42));
        } else {
            panic!("expected Object, got {decoded:?}");
        }
    }

    #[test]
    fn ion_roundtrip_array() {
        use super::ion::{IonDecoder, IonEncoder};
        let mut enc = IonEncoder::new();
        let mut dec = IonDecoder::new();
        // Ion encodes non-negative integers as UINT, so Integer(n >= 0) decodes as UInteger.
        let val = PackValue::Array(vec![
            PackValue::UInteger(1),
            PackValue::UInteger(2),
            PackValue::UInteger(3),
        ]);
        let bytes = enc.encode(&val);
        let decoded = dec.decode(&bytes).expect("ion decode array");
        assert_eq!(decoded, val);
    }

    // ---------------------------------------------------------------- EJSON

    #[test]
    fn ejson_encoder_null_and_primitives() {
        use super::ejson::{EjsonEncoder, EjsonValue};
        let mut enc = EjsonEncoder::new();
        let s = enc.encode_to_string(&EjsonValue::Null).unwrap();
        assert_eq!(s, "null");
        let s = enc.encode_to_string(&EjsonValue::Bool(true)).unwrap();
        assert_eq!(s, "true");
        let s = enc.encode_to_string(&EjsonValue::Bool(false)).unwrap();
        assert_eq!(s, "false");
        let s = enc
            .encode_to_string(&EjsonValue::Str("hello".to_string()))
            .unwrap();
        assert_eq!(s, "\"hello\"");
    }

    #[test]
    fn ejson_encoder_undefined_wrapper() {
        use super::ejson::{EjsonEncoder, EjsonValue};
        let mut enc = EjsonEncoder::new();
        let s = enc.encode_to_string(&EjsonValue::Undefined).unwrap();
        assert_eq!(s, r#"{"$undefined":true}"#);
    }

    #[test]
    fn ejson_encoder_canonical_numbers() {
        use super::ejson::{EjsonEncoder, EjsonEncoderOptions, EjsonValue};
        let mut enc = EjsonEncoder::with_options(EjsonEncoderOptions { canonical: true });
        // Integer in Int32 range
        let s = enc.encode_to_string(&EjsonValue::Number(42.0)).unwrap();
        assert_eq!(s, r#"{"$numberInt":"42"}"#);
        // Integer outside Int32 range
        let s = enc
            .encode_to_string(&EjsonValue::Number(2147483648.0))
            .unwrap();
        assert_eq!(s, r#"{"$numberLong":"2147483648"}"#);
        // Float
        let s = enc
            .encode_to_string(&EjsonValue::Number(TEST_F64_3_14))
            .unwrap();
        assert_eq!(s, r#"{"$numberDouble":"3.14"}"#);
    }

    #[test]
    fn ejson_encoder_relaxed_numbers() {
        use super::ejson::{EjsonEncoder, EjsonValue};
        // Relaxed mode (default) — native JSON numbers for finite values
        let mut enc = EjsonEncoder::new();
        let s = enc.encode_to_string(&EjsonValue::Number(42.0)).unwrap();
        assert_eq!(s, "42");
        let s = enc
            .encode_to_string(&EjsonValue::Number(TEST_F64_3_14))
            .unwrap();
        assert_eq!(s, "3.14");
        // Non-finite still get wrapped
        let s = enc
            .encode_to_string(&EjsonValue::Number(f64::INFINITY))
            .unwrap();
        assert_eq!(s, r#"{"$numberDouble":"Infinity"}"#);
        let s = enc
            .encode_to_string(&EjsonValue::Number(f64::NEG_INFINITY))
            .unwrap();
        assert_eq!(s, r#"{"$numberDouble":"-Infinity"}"#);
        let s = enc.encode_to_string(&EjsonValue::Number(f64::NAN)).unwrap();
        assert_eq!(s, r#"{"$numberDouble":"NaN"}"#);
    }

    #[test]
    fn ejson_encoder_bson_int32_canonical() {
        use super::bson::BsonInt32;
        use super::ejson::{EjsonEncoder, EjsonEncoderOptions, EjsonValue};
        let mut enc = EjsonEncoder::with_options(EjsonEncoderOptions { canonical: true });
        let v = BsonInt32 { value: 42 };
        let s = enc.encode_to_string(&EjsonValue::Int32(v)).unwrap();
        assert_eq!(s, r#"{"$numberInt":"42"}"#);
    }

    #[test]
    fn ejson_encoder_bson_int32_relaxed() {
        use super::bson::BsonInt32;
        use super::ejson::{EjsonEncoder, EjsonValue};
        let mut enc = EjsonEncoder::new();
        let v = BsonInt32 { value: 42 };
        let s = enc.encode_to_string(&EjsonValue::Int32(v)).unwrap();
        assert_eq!(s, "42");
    }

    #[test]
    fn ejson_encoder_bson_int64_canonical() {
        use super::bson::BsonInt64;
        use super::ejson::{EjsonEncoder, EjsonEncoderOptions, EjsonValue};
        let mut enc = EjsonEncoder::with_options(EjsonEncoderOptions { canonical: true });
        let v = BsonInt64 {
            value: 1234567890123,
        };
        let s = enc.encode_to_string(&EjsonValue::Int64(v)).unwrap();
        assert_eq!(s, r#"{"$numberLong":"1234567890123"}"#);
    }

    #[test]
    fn ejson_encoder_bson_float_canonical() {
        use super::bson::BsonFloat;
        use super::ejson::{EjsonEncoder, EjsonEncoderOptions, EjsonValue};
        let mut enc = EjsonEncoder::with_options(EjsonEncoderOptions { canonical: true });
        let v = BsonFloat {
            value: TEST_F64_3_14,
        };
        let s = enc.encode_to_string(&EjsonValue::BsonFloat(v)).unwrap();
        assert_eq!(s, r#"{"$numberDouble":"3.14"}"#);
    }

    #[test]
    fn ejson_encoder_object_id() {
        use super::bson::BsonObjectId;
        use super::ejson::{EjsonEncoder, EjsonValue};
        let mut enc = EjsonEncoder::new();
        let id = BsonObjectId {
            timestamp: 0x507f1f77,
            process: 0xbcf86cd799,
            counter: 0x439011,
        };
        let s = enc.encode_to_string(&EjsonValue::ObjectId(id)).unwrap();
        assert_eq!(s, r#"{"$oid":"507f1f77bcf86cd799439011"}"#);
    }

    #[test]
    fn ejson_encoder_binary() {
        use super::bson::BsonBinary;
        use super::ejson::{EjsonEncoder, EjsonValue};
        let mut enc = EjsonEncoder::new();
        let bin = BsonBinary {
            subtype: 0,
            data: vec![1, 2, 3, 4],
        };
        let s = enc.encode_to_string(&EjsonValue::Binary(bin)).unwrap();
        assert_eq!(s, r#"{"$binary":{"base64":"AQIDBA==","subType":"00"}}"#);
    }

    #[test]
    fn ejson_encoder_code() {
        use super::bson::BsonJavascriptCode;
        use super::ejson::{EjsonEncoder, EjsonValue};
        let mut enc = EjsonEncoder::new();
        let code = BsonJavascriptCode {
            code: "function() { return 42; }".to_string(),
        };
        let s = enc.encode_to_string(&EjsonValue::Code(code)).unwrap();
        assert_eq!(s, r#"{"$code":"function() { return 42; }"}"#);
    }

    #[test]
    fn ejson_encoder_symbol() {
        use super::bson::BsonSymbol;
        use super::ejson::{EjsonEncoder, EjsonValue};
        let mut enc = EjsonEncoder::new();
        let sym = BsonSymbol {
            symbol: "mySymbol".to_string(),
        };
        let s = enc.encode_to_string(&EjsonValue::Symbol(sym)).unwrap();
        assert_eq!(s, r#"{"$symbol":"mySymbol"}"#);
    }

    #[test]
    fn ejson_encoder_timestamp() {
        use super::bson::BsonTimestamp;
        use super::ejson::{EjsonEncoder, EjsonValue};
        let mut enc = EjsonEncoder::new();
        let ts = BsonTimestamp {
            timestamp: 1234567890,
            increment: 12345,
        };
        let s = enc.encode_to_string(&EjsonValue::Timestamp(ts)).unwrap();
        assert_eq!(s, r#"{"$timestamp":{"t":1234567890,"i":12345}}"#);
    }

    #[test]
    fn ejson_encoder_minkey_maxkey() {
        use super::bson::{BsonMaxKey, BsonMinKey};
        use super::ejson::{EjsonEncoder, EjsonValue};
        let mut enc = EjsonEncoder::new();
        assert_eq!(
            enc.encode_to_string(&EjsonValue::MinKey(BsonMinKey))
                .unwrap(),
            r#"{"$minKey":1}"#
        );
        assert_eq!(
            enc.encode_to_string(&EjsonValue::MaxKey(BsonMaxKey))
                .unwrap(),
            r#"{"$maxKey":1}"#
        );
    }

    #[test]
    fn ejson_encoder_regexp() {
        use super::ejson::{EjsonEncoder, EjsonValue};
        let mut enc = EjsonEncoder::new();
        let s = enc
            .encode_to_string(&EjsonValue::RegExp("pattern".to_string(), "gi".to_string()))
            .unwrap();
        assert_eq!(
            s,
            r#"{"$regularExpression":{"pattern":"pattern","options":"gi"}}"#
        );
    }

    #[test]
    fn ejson_encoder_date_relaxed_iso() {
        use super::ejson::{EjsonEncoder, EjsonValue};
        // 2023-01-01T00:00:00.000Z = 1672531200000 ms
        let mut enc = EjsonEncoder::new();
        let s = enc
            .encode_to_string(&EjsonValue::Date {
                timestamp_ms: 1672531200000,
                iso: Some("2023-01-01T00:00:00.000Z".to_string()),
            })
            .unwrap();
        assert_eq!(s, r#"{"$date":"2023-01-01T00:00:00.000Z"}"#);
    }

    #[test]
    fn ejson_encoder_date_canonical() {
        use super::ejson::{EjsonEncoder, EjsonEncoderOptions, EjsonValue};
        let mut enc = EjsonEncoder::with_options(EjsonEncoderOptions { canonical: true });
        let s = enc
            .encode_to_string(&EjsonValue::Date {
                timestamp_ms: 1672531200000,
                iso: Some("2023-01-01T00:00:00.000Z".to_string()),
            })
            .unwrap();
        assert_eq!(s, r#"{"$date":{"$numberLong":"1672531200000"}}"#);
    }

    #[test]
    fn ejson_encoder_db_pointer() {
        use super::bson::{BsonDbPointer, BsonObjectId};
        use super::ejson::{EjsonEncoder, EjsonValue};
        let mut enc = EjsonEncoder::new();
        let id = BsonObjectId {
            timestamp: 0x507f1f77,
            process: 0xbcf86cd799,
            counter: 0x439011,
        };
        let ptr = BsonDbPointer {
            name: "collection".to_string(),
            id,
        };
        let s = enc.encode_to_string(&EjsonValue::DbPointer(ptr)).unwrap();
        assert_eq!(
            s,
            r#"{"$dbPointer":{"$ref":"collection","$id":{"$oid":"507f1f77bcf86cd799439011"}}}"#
        );
    }

    #[test]
    fn ejson_encoder_array_canonical() {
        use super::ejson::{EjsonEncoder, EjsonEncoderOptions, EjsonValue};
        let mut enc = EjsonEncoder::with_options(EjsonEncoderOptions { canonical: true });
        let arr = EjsonValue::Array(vec![
            EjsonValue::Number(1.0),
            EjsonValue::Number(2.0),
            EjsonValue::Number(3.0),
        ]);
        let s = enc.encode_to_string(&arr).unwrap();
        assert_eq!(
            s,
            r#"[{"$numberInt":"1"},{"$numberInt":"2"},{"$numberInt":"3"}]"#
        );
    }

    // ---------------------------------------------------------------- EJSON decoder

    #[test]
    fn ejson_decoder_primitives() {
        use super::ejson::{EjsonDecoder, EjsonValue};
        let mut dec = EjsonDecoder::new();
        assert_eq!(dec.decode_str("null").unwrap(), EjsonValue::Null);
        assert_eq!(dec.decode_str("true").unwrap(), EjsonValue::Bool(true));
        assert_eq!(dec.decode_str("false").unwrap(), EjsonValue::Bool(false));
        assert_eq!(dec.decode_str("42").unwrap(), EjsonValue::Integer(42));
        assert_eq!(
            dec.decode_str("3.14").unwrap(),
            EjsonValue::Float(TEST_F64_3_14)
        );
        assert_eq!(
            dec.decode_str("\"hello\"").unwrap(),
            EjsonValue::Str("hello".to_string())
        );
    }

    #[test]
    fn ejson_decoder_object_id() {
        use super::bson::BsonObjectId;
        use super::ejson::{EjsonDecoder, EjsonValue};
        let mut dec = EjsonDecoder::new();
        let v = dec
            .decode_str(r#"{"$oid":"507f1f77bcf86cd799439011"}"#)
            .unwrap();
        let expected = BsonObjectId {
            timestamp: 0x507f1f77,
            process: 0xbcf86cd799,
            counter: 0x439011,
        };
        assert_eq!(v, EjsonValue::ObjectId(expected));
    }

    #[test]
    fn ejson_decoder_invalid_object_id() {
        use super::ejson::{EjsonDecodeError, EjsonDecoder};
        let mut dec = EjsonDecoder::new();
        assert!(matches!(
            dec.decode_str(r#"{"$oid":"invalid"}"#),
            Err(EjsonDecodeError::InvalidObjectId)
        ));
    }

    #[test]
    fn ejson_decoder_int32() {
        use super::bson::BsonInt32;
        use super::ejson::{EjsonDecoder, EjsonValue};
        let mut dec = EjsonDecoder::new();
        let v = dec.decode_str(r#"{"$numberInt":"42"}"#).unwrap();
        assert_eq!(v, EjsonValue::Int32(BsonInt32 { value: 42 }));
        let v2 = dec.decode_str(r#"{"$numberInt":"-42"}"#).unwrap();
        assert_eq!(v2, EjsonValue::Int32(BsonInt32 { value: -42 }));
    }

    #[test]
    fn ejson_decoder_invalid_int32() {
        use super::ejson::{EjsonDecodeError, EjsonDecoder};
        let mut dec = EjsonDecoder::new();
        // Out of range
        assert!(matches!(
            dec.decode_str(r#"{"$numberInt":"2147483648"}"#),
            Err(EjsonDecodeError::InvalidInt32)
        ));
        // Not a string
        assert!(matches!(
            dec.decode_str(r#"{"$numberInt":42}"#),
            Err(EjsonDecodeError::InvalidInt32)
        ));
    }

    #[test]
    fn ejson_decoder_int64() {
        use super::bson::BsonInt64;
        use super::ejson::{EjsonDecoder, EjsonValue};
        let mut dec = EjsonDecoder::new();
        let v = dec
            .decode_str(r#"{"$numberLong":"1234567890123"}"#)
            .unwrap();
        assert_eq!(
            v,
            EjsonValue::Int64(BsonInt64 {
                value: 1234567890123
            })
        );
    }

    #[test]
    fn ejson_decoder_double() {
        use super::bson::BsonFloat;
        use super::ejson::{EjsonDecoder, EjsonValue};
        let mut dec = EjsonDecoder::new();
        let v = dec.decode_str(r#"{"$numberDouble":"3.14"}"#).unwrap();
        assert_eq!(
            v,
            EjsonValue::BsonFloat(BsonFloat {
                value: TEST_F64_3_14
            })
        );
        // Special values
        let v_inf = dec.decode_str(r#"{"$numberDouble":"Infinity"}"#).unwrap();
        assert_eq!(
            v_inf,
            EjsonValue::BsonFloat(BsonFloat {
                value: f64::INFINITY
            })
        );
        let v_neginf = dec.decode_str(r#"{"$numberDouble":"-Infinity"}"#).unwrap();
        assert_eq!(
            v_neginf,
            EjsonValue::BsonFloat(BsonFloat {
                value: f64::NEG_INFINITY
            })
        );
        let v_nan = dec.decode_str(r#"{"$numberDouble":"NaN"}"#).unwrap();
        if let EjsonValue::BsonFloat(bf) = v_nan {
            assert!(bf.value.is_nan());
        } else {
            panic!("expected BsonFloat");
        }
    }

    #[test]
    fn ejson_decoder_decimal128() {
        use super::bson::BsonDecimal128;
        use super::ejson::{EjsonDecoder, EjsonValue};
        let mut dec = EjsonDecoder::new();
        let v = dec.decode_str(r#"{"$numberDecimal":"123.456"}"#).unwrap();
        assert_eq!(
            v,
            EjsonValue::Decimal128(BsonDecimal128 {
                data: vec![0u8; 16]
            })
        );
    }

    #[test]
    fn ejson_decoder_binary() {
        use super::bson::BsonBinary;
        use super::ejson::{EjsonDecoder, EjsonValue};
        let mut dec = EjsonDecoder::new();
        let v = dec
            .decode_str(r#"{"$binary":{"base64":"AQIDBA==","subType":"00"}}"#)
            .unwrap();
        assert_eq!(
            v,
            EjsonValue::Binary(BsonBinary {
                subtype: 0,
                data: vec![1, 2, 3, 4]
            })
        );
    }

    #[test]
    fn ejson_decoder_uuid() {
        use super::ejson::{EjsonDecoder, EjsonValue};
        let mut dec = EjsonDecoder::new();
        let v = dec
            .decode_str(r#"{"$uuid":"c8edabc3-f738-4ca3-b68d-ab92a91478a3"}"#)
            .unwrap();
        if let EjsonValue::Binary(bin) = v {
            assert_eq!(bin.subtype, 4);
            assert_eq!(bin.data.len(), 16);
        } else {
            panic!("expected Binary");
        }
    }

    #[test]
    fn ejson_decoder_invalid_uuid() {
        use super::ejson::{EjsonDecodeError, EjsonDecoder};
        let mut dec = EjsonDecoder::new();
        assert!(matches!(
            dec.decode_str(r#"{"$uuid":"invalid-uuid"}"#),
            Err(EjsonDecodeError::InvalidUuid)
        ));
    }

    #[test]
    fn ejson_decoder_code() {
        use super::bson::BsonJavascriptCode;
        use super::ejson::{EjsonDecoder, EjsonValue};
        let mut dec = EjsonDecoder::new();
        let v = dec
            .decode_str(r#"{"$code":"function() { return 42; }"}"#)
            .unwrap();
        assert_eq!(
            v,
            EjsonValue::Code(BsonJavascriptCode {
                code: "function() { return 42; }".to_string()
            })
        );
    }

    #[test]
    fn ejson_decoder_symbol() {
        use super::bson::BsonSymbol;
        use super::ejson::{EjsonDecoder, EjsonValue};
        let mut dec = EjsonDecoder::new();
        let v = dec.decode_str(r#"{"$symbol":"mySymbol"}"#).unwrap();
        assert_eq!(
            v,
            EjsonValue::Symbol(BsonSymbol {
                symbol: "mySymbol".to_string()
            })
        );
    }

    #[test]
    fn ejson_decoder_timestamp() {
        use super::bson::BsonTimestamp;
        use super::ejson::{EjsonDecoder, EjsonValue};
        let mut dec = EjsonDecoder::new();
        let v = dec
            .decode_str(r#"{"$timestamp":{"t":1234567890,"i":12345}}"#)
            .unwrap();
        assert_eq!(
            v,
            EjsonValue::Timestamp(BsonTimestamp {
                timestamp: 1234567890,
                increment: 12345
            })
        );
    }

    #[test]
    fn ejson_decoder_invalid_timestamp() {
        use super::ejson::{EjsonDecodeError, EjsonDecoder};
        let mut dec = EjsonDecoder::new();
        // Negative t
        assert!(matches!(
            dec.decode_str(r#"{"$timestamp":{"t":-1,"i":12345}}"#),
            Err(EjsonDecodeError::InvalidTimestamp)
        ));
        // Negative i
        assert!(matches!(
            dec.decode_str(r#"{"$timestamp":{"t":123,"i":-1}}"#),
            Err(EjsonDecodeError::InvalidTimestamp)
        ));
    }

    #[test]
    fn ejson_decoder_regexp() {
        use super::ejson::{EjsonDecoder, EjsonValue};
        let mut dec = EjsonDecoder::new();
        let v = dec
            .decode_str(r#"{"$regularExpression":{"pattern":"test","options":"gi"}}"#)
            .unwrap();
        assert_eq!(v, EjsonValue::RegExp("test".to_string(), "gi".to_string()));
    }

    #[test]
    fn ejson_decoder_db_pointer() {
        use super::bson::{BsonDbPointer, BsonObjectId};
        use super::ejson::{EjsonDecoder, EjsonValue};
        let mut dec = EjsonDecoder::new();
        let v = dec
            .decode_str(
                r#"{"$dbPointer":{"$ref":"collection","$id":{"$oid":"507f1f77bcf86cd799439011"}}}"#,
            )
            .unwrap();
        let expected = BsonDbPointer {
            name: "collection".to_string(),
            id: BsonObjectId {
                timestamp: 0x507f1f77,
                process: 0xbcf86cd799,
                counter: 0x439011,
            },
        };
        assert_eq!(v, EjsonValue::DbPointer(expected));
    }

    #[test]
    fn ejson_decoder_date_iso() {
        use super::ejson::{EjsonDecoder, EjsonValue};
        let mut dec = EjsonDecoder::new();
        let v = dec
            .decode_str(r#"{"$date":"2023-01-01T00:00:00.000Z"}"#)
            .unwrap();
        assert_eq!(
            v,
            EjsonValue::Date {
                timestamp_ms: 1672531200000,
                iso: None
            }
        );
    }

    #[test]
    fn ejson_decoder_date_canonical() {
        use super::ejson::{EjsonDecoder, EjsonValue};
        let mut dec = EjsonDecoder::new();
        let v = dec
            .decode_str(r#"{"$date":{"$numberLong":"1672531200000"}}"#)
            .unwrap();
        assert_eq!(
            v,
            EjsonValue::Date {
                timestamp_ms: 1672531200000,
                iso: None
            }
        );
    }

    #[test]
    fn ejson_decoder_invalid_date() {
        use super::ejson::{EjsonDecodeError, EjsonDecoder};
        let mut dec = EjsonDecoder::new();
        assert!(matches!(
            dec.decode_str(r#"{"$date":"not-a-date"}"#),
            Err(EjsonDecodeError::InvalidDate)
        ));
    }

    #[test]
    fn ejson_decoder_minkey_maxkey() {
        use super::bson::{BsonMaxKey, BsonMinKey};
        use super::ejson::{EjsonDecoder, EjsonValue};
        let mut dec = EjsonDecoder::new();
        assert_eq!(
            dec.decode_str(r#"{"$minKey":1}"#).unwrap(),
            EjsonValue::MinKey(BsonMinKey)
        );
        assert_eq!(
            dec.decode_str(r#"{"$maxKey":1}"#).unwrap(),
            EjsonValue::MaxKey(BsonMaxKey)
        );
    }

    #[test]
    fn ejson_decoder_undefined() {
        use super::ejson::{EjsonDecoder, EjsonValue};
        let mut dec = EjsonDecoder::new();
        assert_eq!(
            dec.decode_str(r#"{"$undefined":true}"#).unwrap(),
            EjsonValue::Undefined
        );
    }

    #[test]
    fn ejson_decoder_plain_object() {
        use super::ejson::{EjsonDecoder, EjsonValue};
        let mut dec = EjsonDecoder::new();
        let v = dec.decode_str(r#"{"name":"John","age":30}"#).unwrap();
        if let EjsonValue::Object(pairs) = v {
            assert_eq!(pairs.len(), 2);
            assert_eq!(
                pairs[0],
                ("name".to_string(), EjsonValue::Str("John".to_string()))
            );
            assert_eq!(pairs[1], ("age".to_string(), EjsonValue::Integer(30)));
        } else {
            panic!("expected Object");
        }
    }

    #[test]
    fn ejson_decoder_nested_ejson_in_object() {
        use super::bson::BsonInt32;
        use super::ejson::{EjsonDecoder, EjsonValue};
        let mut dec = EjsonDecoder::new();
        let v = dec.decode_str(r#"{"count":{"$numberInt":"42"}}"#).unwrap();
        if let EjsonValue::Object(pairs) = v {
            assert_eq!(pairs.len(), 1);
            assert_eq!(pairs[0].0, "count");
            assert_eq!(pairs[0].1, EjsonValue::Int32(BsonInt32 { value: 42 }));
        } else {
            panic!("expected Object");
        }
    }

    #[test]
    fn ejson_decoder_extra_keys_error() {
        use super::ejson::{EjsonDecodeError, EjsonDecoder};
        let mut dec = EjsonDecoder::new();
        // Extra key alongside $numberInt should error
        let res = dec.decode_str(r#"{"$numberInt":"42","extra":"field"}"#);
        assert!(matches!(res, Err(EjsonDecodeError::ExtraKeys(_))));
    }

    #[test]
    fn ejson_decoder_array() {
        use super::ejson::{EjsonDecoder, EjsonValue};
        let mut dec = EjsonDecoder::new();
        let v = dec.decode_str(r#"[1,2,3]"#).unwrap();
        assert_eq!(
            v,
            EjsonValue::Array(vec![
                EjsonValue::Integer(1),
                EjsonValue::Integer(2),
                EjsonValue::Integer(3),
            ])
        );
    }

    #[test]
    fn ejson_roundtrip_object_id() {
        use super::bson::BsonObjectId;
        use super::ejson::{EjsonDecoder, EjsonEncoder, EjsonValue};
        let mut enc = EjsonEncoder::new();
        let mut dec = EjsonDecoder::new();
        let id = BsonObjectId {
            timestamp: 0x507f1f77,
            process: 0xbcf86cd799,
            counter: 0x439011,
        };
        let encoded = enc
            .encode_to_string(&EjsonValue::ObjectId(id.clone()))
            .unwrap();
        let decoded = dec.decode_str(&encoded).unwrap();
        assert_eq!(decoded, EjsonValue::ObjectId(id));
    }

    #[test]
    fn ejson_roundtrip_binary() {
        use super::bson::BsonBinary;
        use super::ejson::{EjsonDecoder, EjsonEncoder, EjsonValue};
        let mut enc = EjsonEncoder::new();
        let mut dec = EjsonDecoder::new();
        let bin = BsonBinary {
            subtype: 0,
            data: vec![1, 2, 3, 4],
        };
        let encoded = enc
            .encode_to_string(&EjsonValue::Binary(bin.clone()))
            .unwrap();
        let decoded = dec.decode_str(&encoded).unwrap();
        assert_eq!(decoded, EjsonValue::Binary(bin));
    }

    #[test]
    fn ejson_roundtrip_timestamp() {
        use super::bson::BsonTimestamp;
        use super::ejson::{EjsonDecoder, EjsonEncoder, EjsonValue};
        let mut enc = EjsonEncoder::new();
        let mut dec = EjsonDecoder::new();
        let ts = BsonTimestamp {
            timestamp: 1234567890,
            increment: 12345,
        };
        let encoded = enc
            .encode_to_string(&EjsonValue::Timestamp(ts.clone()))
            .unwrap();
        let decoded = dec.decode_str(&encoded).unwrap();
        assert_eq!(decoded, EjsonValue::Timestamp(ts));
    }

    // ---------------------------------------------------------------- Boundary / error-path tests

    // --- CBOR truncated input ---

    #[test]
    fn cbor_empty_input_returns_error() {
        let result = decode_json_from_cbor_bytes(&[]);
        assert!(result.is_err(), "empty CBOR must return Err");
    }

    #[test]
    fn cbor_truncated_uint16_returns_error() {
        // 0x19 = major 0, additional 25 → expects 2 more bytes, we give 1
        let result = decode_json_from_cbor_bytes(&[0x19, 0x00]);
        assert!(result.is_err(), "truncated uint16 must return Err");
    }

    #[test]
    fn cbor_truncated_uint32_returns_error() {
        // 0x1a = major 0, additional 26 → expects 4 bytes, we give 2
        let result = decode_json_from_cbor_bytes(&[0x1a, 0x00, 0x00]);
        assert!(result.is_err(), "truncated uint32 must return Err");
    }

    #[test]
    fn cbor_truncated_uint64_returns_error() {
        // 0x1b = major 0, additional 27 → expects 8 bytes, we give 4
        let result = decode_json_from_cbor_bytes(&[0x1b, 0x00, 0x00, 0x00, 0x00]);
        assert!(result.is_err(), "truncated uint64 must return Err");
    }

    #[test]
    fn cbor_truncated_text_string_returns_error() {
        // 0x63 = major 3 (text), length 3 → expects 3 bytes, we give 2
        let result = decode_json_from_cbor_bytes(&[0x63, b'h', b'i']);
        assert!(result.is_err(), "truncated text string must return Err");
    }

    #[test]
    fn cbor_truncated_byte_string_returns_error() {
        // 0x42 = major 2 (bytes), length 2 → expects 2 bytes, we give 1
        let result = decode_json_from_cbor_bytes(&[0x42, 0xDE]);
        assert!(result.is_err(), "truncated byte string must return Err");
    }

    #[test]
    fn cbor_truncated_array_returns_error() {
        // 0x82 = major 4 (array), length 2 → expects 2 items, we give header only
        let result = decode_json_from_cbor_bytes(&[0x82]);
        assert!(result.is_err(), "truncated array must return Err");
    }

    #[test]
    fn cbor_truncated_map_returns_error() {
        // 0xa1 = major 5 (map), length 1 → expects 1 pair, we give the key but not value
        let result = decode_json_from_cbor_bytes(&[0xa1, 0x61, b'k']);
        assert!(result.is_err(), "truncated map must return Err");
    }

    #[test]
    fn cbor_validate_size_rejects_wrong_size() {
        let bytes = encode_json_to_cbor_bytes(&serde_json::json!(42)).expect("encode");
        // validate_cbor_exact_size with wrong size must fail
        let result = validate_cbor_exact_size(&bytes, bytes.len() + 1);
        assert!(result.is_err(), "wrong size must return Err");
    }

    // --- MsgPack boundary / error-path tests ---

    #[test]
    fn msgpack_empty_input_returns_error() {
        use super::msgpack::MsgPackDecoderFast;
        let mut dec = MsgPackDecoderFast::new();
        assert!(dec.decode(&[]).is_err(), "empty MsgPack must return Err");
    }

    #[test]
    fn msgpack_truncated_str8_returns_error() {
        use super::msgpack::MsgPackDecoderFast;
        let mut dec = MsgPackDecoderFast::new();
        // 0xd9 = str 8, length byte = 5, then only 2 bytes of payload
        assert!(dec.decode(&[0xd9, 0x05, b'h', b'i']).is_err());
    }

    #[test]
    fn msgpack_truncated_bin8_returns_error() {
        use super::msgpack::MsgPackDecoderFast;
        let mut dec = MsgPackDecoderFast::new();
        // 0xc4 = bin8, length=3, only 1 byte given
        assert!(dec.decode(&[0xc4, 0x03, 0xDE]).is_err());
    }

    #[test]
    fn msgpack_fixarray_boundary_correct() {
        use super::msgpack::{MsgPackDecoderFast, MsgPackEncoderFast};
        let mut enc = MsgPackEncoderFast::new();
        let mut dec = MsgPackDecoderFast::new();
        // fixarray holds 0..=15 items; 15 items → 0x9f header
        let items: Vec<PackValue> = (0..15).map(PackValue::Integer).collect();
        let arr = PackValue::Array(items.clone());
        let bytes = enc.encode(&arr);
        assert_eq!(bytes[0], 0x9f, "fixarray(15) header");
        let decoded = dec.decode(&bytes).unwrap();
        assert_eq!(decoded, PackValue::Array(items));
    }

    #[test]
    fn msgpack_array16_boundary_correct() {
        use super::msgpack::{MsgPackDecoderFast, MsgPackEncoderFast};
        let mut enc = MsgPackEncoderFast::new();
        let mut dec = MsgPackDecoderFast::new();
        // 16 items → array16 (0xdc) header
        let items: Vec<PackValue> = (0..16).map(PackValue::Integer).collect();
        let arr = PackValue::Array(items.clone());
        let bytes = enc.encode(&arr);
        assert_eq!(bytes[0], 0xdc, "array16 header");
        // bytes[1..2] = length as u16 BE
        let len = u16::from_be_bytes([bytes[1], bytes[2]]) as usize;
        assert_eq!(len, 16);
        let decoded = dec.decode(&bytes).unwrap();
        assert_eq!(decoded, PackValue::Array(items));
    }

    #[test]
    fn msgpack_fixmap_boundary_correct() {
        use super::msgpack::{MsgPackDecoderFast, MsgPackEncoderFast};
        let mut enc = MsgPackEncoderFast::new();
        let mut dec = MsgPackDecoderFast::new();
        // fixmap holds 0..=15 pairs; 15 pairs → 0x8f header
        let pairs: Vec<(String, PackValue)> = (0..15)
            .map(|i| (format!("k{i}"), PackValue::Integer(i)))
            .collect();
        let obj = PackValue::Object(pairs.clone());
        let bytes = enc.encode(&obj);
        assert_eq!(bytes[0], 0x8f, "fixmap(15) header");
        // Decode and check we get 15 pairs back
        if let PackValue::Object(decoded_pairs) = dec.decode(&bytes).unwrap() {
            assert_eq!(decoded_pairs.len(), 15);
        } else {
            panic!("expected Object");
        }
    }

    #[test]
    fn msgpack_uint_128_to_255_uses_uint16_format() {
        use super::msgpack::{MsgPackDecoderFast, MsgPackEncoderFast};
        let mut enc = MsgPackEncoderFast::new();
        let mut dec = MsgPackDecoderFast::new();
        // Upstream encoder skips uint8 (0xcc); values 128..=65535 use uint16 (0xcd).
        // Decoder maps uint16 back to Integer (not UInteger).
        let bytes = enc.encode(&PackValue::UInteger(200));
        assert_eq!(bytes[0], 0xcd, "values 128-65535 use uint16 format");
        let v = u16::from_be_bytes([bytes[1], bytes[2]]);
        assert_eq!(v, 200);
        assert_eq!(dec.decode(&bytes).unwrap(), PackValue::Integer(200));
    }

    #[test]
    fn msgpack_uint16_range_roundtrips_as_integer() {
        use super::msgpack::{MsgPackDecoderFast, MsgPackEncoderFast};
        let mut enc = MsgPackEncoderFast::new();
        let mut dec = MsgPackDecoderFast::new();
        // uint16 (0xcd); decoder returns Integer (signed), not UInteger
        let bytes = enc.encode(&PackValue::UInteger(1000));
        assert_eq!(bytes[0], 0xcd, "uint16 format");
        let v = u16::from_be_bytes([bytes[1], bytes[2]]);
        assert_eq!(v, 1000);
        assert_eq!(dec.decode(&bytes).unwrap(), PackValue::Integer(1000));
    }

    #[test]
    fn msgpack_negative_mid_range_uses_int16_format() {
        use super::msgpack::{MsgPackDecoderFast, MsgPackEncoderFast};
        let mut enc = MsgPackEncoderFast::new();
        let mut dec = MsgPackDecoderFast::new();
        // Upstream encoder skips int8 (0xd0); values -33..-32768 use int16 (0xd1).
        let bytes = enc.encode(&PackValue::Integer(-100));
        assert_eq!(bytes[0], 0xd1, "values -33..-32768 use int16 format");
        let v = i16::from_be_bytes([bytes[1], bytes[2]]);
        assert_eq!(v, -100);
        assert_eq!(dec.decode(&bytes).unwrap(), PackValue::Integer(-100));
    }

    #[test]
    fn msgpack_truncated_array_returns_error() {
        use super::msgpack::MsgPackDecoderFast;
        let mut dec = MsgPackDecoderFast::new();
        // fixarray with 3 elements, but no element data follows
        assert!(dec.decode(&[0x93]).is_err());
    }

    // --- RESP3 boundary / error-path tests ---

    #[test]
    fn resp_empty_input_returns_error() {
        use super::resp::RespDecoder;
        let mut dec = RespDecoder::new();
        assert!(dec.decode(&[]).is_err(), "empty RESP must return Err");
    }

    #[test]
    fn resp_unknown_type_byte_returns_error() {
        use super::resp::RespDecoder;
        let mut dec = RespDecoder::new();
        // 0x00 is not a valid RESP3 type prefix
        assert!(dec.decode(&[0x00]).is_err(), "unknown type must return Err");
    }

    #[test]
    fn resp_decode_float() {
        use super::resp::RespDecoder;
        let mut dec = RespDecoder::new();
        assert_eq!(
            dec.decode(b",3.14\r\n").unwrap(),
            PackValue::Float(TEST_F64_3_14)
        );
    }

    #[test]
    fn resp_decode_float_inf_neginf_nan() {
        use super::resp::RespDecoder;
        let mut dec = RespDecoder::new();
        let v = dec.decode(b",inf\r\n").unwrap();
        assert!(matches!(v, PackValue::Float(f) if f.is_infinite() && f > 0.0));
        let v = dec.decode(b",-inf\r\n").unwrap();
        assert!(matches!(v, PackValue::Float(f) if f.is_infinite() && f < 0.0));
        let v = dec.decode(b",nan\r\n").unwrap();
        assert!(matches!(v, PackValue::Float(f) if f.is_nan()));
    }

    #[test]
    fn resp_decode_bigint() {
        use super::resp::RespDecoder;
        let mut dec = RespDecoder::new();
        let v = dec.decode(b"(1234567890123456789\r\n").unwrap();
        assert_eq!(v, PackValue::BigInt(1234567890123456789_i128));
        let neg = dec.decode(b"(-42\r\n").unwrap();
        assert_eq!(neg, PackValue::BigInt(-42));
    }

    #[test]
    fn resp_decode_set_as_array() {
        use super::resp::RespDecoder;
        let mut dec = RespDecoder::new();
        // ~2\r\n:1\r\n:2\r\n — set with 2 integer elements
        let v = dec.decode(b"~2\r\n:1\r\n:2\r\n").unwrap();
        assert_eq!(
            v,
            PackValue::Array(vec![PackValue::Integer(1), PackValue::Integer(2)])
        );
    }

    #[test]
    fn resp_decode_object() {
        use super::resp::RespDecoder;
        let mut dec = RespDecoder::new();
        // %1\r\n+key\r\n:42\r\n — map with 1 pair
        let v = dec.decode(b"%1\r\n+key\r\n:42\r\n").unwrap();
        assert_eq!(
            v,
            PackValue::Object(vec![("key".to_string(), PackValue::Integer(42))])
        );
    }

    #[test]
    fn resp_decode_push_as_extension() {
        use super::resp::RespDecoder;
        use super::PackValue;
        let mut dec = RespDecoder::new();
        // >2\r\n+foo\r\n:1\r\n — push with 2 elements
        let v = dec.decode(b">2\r\n+foo\r\n:1\r\n").unwrap();
        if let PackValue::Extension(ext) = v {
            assert_eq!(ext.tag, 1); // RESP_EXTENSION_PUSH
            assert!(matches!(*ext.val, PackValue::Array(_)));
        } else {
            panic!("expected Extension for push, got {v:?}");
        }
    }

    #[test]
    fn resp_decode_attributes_as_extension() {
        use super::resp::RespDecoder;
        let mut dec = RespDecoder::new();
        // |1\r\n+ttl\r\n:3600\r\n
        let v = dec.decode(b"|1\r\n+ttl\r\n:3600\r\n").unwrap();
        if let PackValue::Extension(ext) = v {
            assert_eq!(ext.tag, 2); // RESP_EXTENSION_ATTRIBUTES
            assert!(matches!(*ext.val, PackValue::Object(_)));
        } else {
            panic!("expected Extension for attributes, got {v:?}");
        }
    }

    #[test]
    fn resp_decode_verbatim_txt_string() {
        use super::resp::RespDecoder;
        let mut dec = RespDecoder::new();
        // =7\r\ntxt:abc\r\n — verbatim text string
        let v = dec.decode(b"=7\r\ntxt:abc\r\n").unwrap();
        assert_eq!(v, PackValue::Str("abc".to_string()));
    }

    #[test]
    fn resp_decode_verbatim_non_txt_as_bytes() {
        use super::resp::RespDecoder;
        let mut dec = RespDecoder::new();
        // =8\r\nraw:data\r\n — verbatim with non-txt prefix → Bytes
        let v = dec.decode(b"=8\r\nraw:data\r\n").unwrap();
        assert_eq!(v, PackValue::Bytes(b"data".to_vec()));
    }

    #[test]
    fn resp_decode_simple_error_as_str() {
        use super::resp::RespDecoder;
        let mut dec = RespDecoder::new();
        // Simple error (-) decoded as Str
        let v = dec.decode(b"-ERR some error\r\n").unwrap();
        assert_eq!(v, PackValue::Str("ERR some error".to_string()));
    }

    #[test]
    fn resp_decode_bulk_error_as_str() {
        use super::resp::RespDecoder;
        let mut dec = RespDecoder::new();
        // Bulk error (!) decoded as Str. "ERR bulk error" = 14 bytes.
        let v = dec.decode(b"!14\r\nERR bulk error\r\n").unwrap();
        assert_eq!(v, PackValue::Str("ERR bulk error".to_string()));
    }

    #[test]
    fn resp_decode_null_bulk_string() {
        use super::resp::RespDecoder;
        let mut dec = RespDecoder::new();
        // $-1\r\n → Null bulk string
        let v = dec.decode(b"$-1\r\n").unwrap();
        assert_eq!(v, PackValue::Null);
    }

    #[test]
    fn resp_decode_null_array() {
        use super::resp::RespDecoder;
        let mut dec = RespDecoder::new();
        // *-1\r\n → Null array
        let v = dec.decode(b"*-1\r\n").unwrap();
        assert_eq!(v, PackValue::Null);
    }

    #[test]
    fn resp_decode_nested_array() {
        use super::resp::RespDecoder;
        let mut dec = RespDecoder::new();
        // *2\r\n*2\r\n:1\r\n:2\r\n:3\r\n — nested arrays
        let v = dec.decode(b"*2\r\n*2\r\n:1\r\n:2\r\n:3\r\n").unwrap();
        assert_eq!(
            v,
            PackValue::Array(vec![
                PackValue::Array(vec![PackValue::Integer(1), PackValue::Integer(2)]),
                PackValue::Integer(3),
            ])
        );
    }

    // ── CBOR safe integer boundary tests (via CborEncoderFast) ─────────

    #[test]
    fn cbor_encoder_fast_safe_integer_boundary() {
        let max_safe: f64 = 9_007_199_254_740_991.0; // 2^53 - 1
        let mut enc = CborEncoderFast::new();
        enc.write_number(max_safe);
        let buf = enc.writer.flush();
        // CBOR major type 0 uint, 8-byte payload: 0x1b
        assert_eq!(
            buf[0], 0x1b,
            "MAX_SAFE_INTEGER should encode as CBOR uint64"
        );
    }

    #[test]
    fn cbor_encoder_fast_negative_safe_integer_boundary() {
        let neg_max_safe: f64 = -9_007_199_254_740_991.0; // -(2^53 - 1)
        let mut enc = CborEncoderFast::new();
        enc.write_number(neg_max_safe);
        let buf = enc.writer.flush();
        // CBOR major type 1 nint, 8-byte payload: 0x3b
        assert_eq!(
            buf[0], 0x3b,
            "negative MAX_SAFE should encode as CBOR nint64"
        );
    }

    #[test]
    fn cbor_encoder_fast_above_safe_integer_encodes_as_float() {
        let above_safe: f64 = 9_007_199_254_740_992.0; // 2^53
        let mut enc = CborEncoderFast::new();
        enc.write_number(above_safe);
        let buf = enc.writer.flush();
        // CBOR float64: 0xfb
        assert_eq!(buf[0], 0xfb, "above MAX_SAFE should encode as float64");
    }

    #[test]
    fn cbor_encoder_fast_fractional_encodes_as_float() {
        let mut enc = CborEncoderFast::new();
        enc.write_number(1.5);
        let buf = enc.writer.flush();
        assert_eq!(buf[0], 0xfb, "fractional numbers should encode as float64");
    }

    #[test]
    fn cbor_encoder_fast_zero_encodes_as_integer() {
        let mut enc = CborEncoderFast::new();
        enc.write_number(0.0);
        let buf = enc.writer.flush();
        // CBOR uint 0: 0x00
        assert_eq!(buf[0], 0x00, "zero should encode as CBOR uint 0");
    }

    // ── CborEncoderFast: write_str ──────────────────────────────────────

    #[test]
    fn cbor_encoder_fast_write_str_empty() {
        let mut enc = CborEncoderFast::new();
        enc.write_str("");
        let buf = enc.writer.flush();
        // Empty text string: 0x60 (major 3, length 0)
        assert_eq!(buf, &[0x60]);
    }

    #[test]
    fn cbor_encoder_fast_write_str_short() {
        let mut enc = CborEncoderFast::new();
        enc.write_str("hi");
        let buf = enc.writer.flush();
        // 0x62 = major 3, length 2
        assert_eq!(buf[0], 0x62);
        assert_eq!(&buf[1..], b"hi");
    }

    #[test]
    fn cbor_encoder_fast_write_str_23_chars() {
        let mut enc = CborEncoderFast::new();
        let s = "a".repeat(23);
        enc.write_str(&s);
        let buf = enc.writer.flush();
        // 23 ASCII chars → max_size = 23*4 = 92, which is > 23 so uses 0x78 header
        assert_eq!(buf[0], 0x78);
        assert_eq!(buf[1], 23);
        assert_eq!(&buf[2..], s.as_bytes());
    }

    #[test]
    fn cbor_encoder_fast_write_str_multibyte_utf8() {
        let mut enc = CborEncoderFast::new();
        // "€" is 3 bytes in UTF-8, 1 char → max_size = 4, actual = 3
        enc.write_str("€");
        let buf = enc.writer.flush();
        // max_size=4 <= 23 → short header
        assert_eq!(buf[0], 0x60 | 3); // 0x63
        assert_eq!(&buf[1..], "€".as_bytes());
    }

    #[test]
    fn cbor_encoder_fast_write_str_medium() {
        let mut enc = CborEncoderFast::new();
        let s = "x".repeat(100);
        enc.write_str(&s);
        let buf = enc.writer.flush();
        // 100 ASCII chars → max_size = 400 > 255 → 0x79 header (u16 length)
        assert_eq!(buf[0], 0x79);
        let len = u16::from_be_bytes([buf[1], buf[2]]);
        assert_eq!(len, 100);
        assert_eq!(&buf[3..], s.as_bytes());
    }

    #[test]
    fn cbor_encoder_fast_write_str_long() {
        let mut enc = CborEncoderFast::new();
        // Need char_count * 4 > 0xffff, so char_count > 16383
        let s = "a".repeat(16384);
        enc.write_str(&s);
        let buf = enc.writer.flush();
        // max_size = 16384 * 4 = 65536 > 0xffff → 0x7a header (u32 length)
        assert_eq!(buf[0], 0x7a);
        let len = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
        assert_eq!(len, 16384);
    }

    // ── CborEncoderFast: write_u_integer branches ───────────────────────

    #[test]
    fn cbor_encoder_fast_u_integer_tiny() {
        let mut enc = CborEncoderFast::new();
        enc.write_u_integer(0);
        let buf = enc.writer.flush();
        assert_eq!(buf, &[0x00]);

        enc.writer.reset();
        enc.write_u_integer(23);
        let buf = enc.writer.flush();
        assert_eq!(buf, &[0x17]);
    }

    #[test]
    fn cbor_encoder_fast_u_integer_u8() {
        let mut enc = CborEncoderFast::new();
        enc.write_u_integer(24);
        let buf = enc.writer.flush();
        assert_eq!(buf, &[0x18, 24]);

        enc.writer.reset();
        enc.write_u_integer(255);
        let buf = enc.writer.flush();
        assert_eq!(buf, &[0x18, 0xff]);
    }

    #[test]
    fn cbor_encoder_fast_u_integer_u16() {
        let mut enc = CborEncoderFast::new();
        enc.write_u_integer(256);
        let buf = enc.writer.flush();
        assert_eq!(buf[0], 0x19);
        assert_eq!(u16::from_be_bytes([buf[1], buf[2]]), 256);

        enc.writer.reset();
        enc.write_u_integer(0xffff);
        let buf = enc.writer.flush();
        assert_eq!(buf[0], 0x19);
        assert_eq!(u16::from_be_bytes([buf[1], buf[2]]), 0xffff);
    }

    #[test]
    fn cbor_encoder_fast_u_integer_u32() {
        let mut enc = CborEncoderFast::new();
        enc.write_u_integer(0x10000);
        let buf = enc.writer.flush();
        assert_eq!(buf[0], 0x1a);
        assert_eq!(
            u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]),
            0x10000
        );

        enc.writer.reset();
        enc.write_u_integer(0xffffffff);
        let buf = enc.writer.flush();
        assert_eq!(buf[0], 0x1a);
        assert_eq!(
            u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]),
            0xffffffff
        );
    }

    #[test]
    fn cbor_encoder_fast_u_integer_u64() {
        let mut enc = CborEncoderFast::new();
        enc.write_u_integer(0x100000000);
        let buf = enc.writer.flush();
        assert_eq!(buf[0], 0x1b);
        assert_eq!(
            u64::from_be_bytes(buf[1..9].try_into().unwrap()),
            0x100000000
        );
    }

    // ── CborEncoderFast: encode_nint branches ───────────────────────────

    #[test]
    fn cbor_encoder_fast_nint_tiny() {
        let mut enc = CborEncoderFast::new();
        enc.encode_nint(-1); // uint = 0 → single byte 0x20
        let buf = enc.writer.flush();
        assert_eq!(buf, &[0x20]);

        enc.writer.reset();
        enc.encode_nint(-24); // uint = 23 → 0x37
        let buf = enc.writer.flush();
        assert_eq!(buf, &[0x37]);
    }

    #[test]
    fn cbor_encoder_fast_nint_u8() {
        let mut enc = CborEncoderFast::new();
        enc.encode_nint(-25); // uint = 24 → 0x38, 24
        let buf = enc.writer.flush();
        assert_eq!(buf, &[0x38, 24]);

        enc.writer.reset();
        enc.encode_nint(-256); // uint = 255 → 0x38, 0xff
        let buf = enc.writer.flush();
        assert_eq!(buf, &[0x38, 0xff]);
    }

    #[test]
    fn cbor_encoder_fast_nint_u16() {
        let mut enc = CborEncoderFast::new();
        enc.encode_nint(-257); // uint = 256
        let buf = enc.writer.flush();
        assert_eq!(buf[0], 0x39);
        assert_eq!(u16::from_be_bytes([buf[1], buf[2]]), 256);
    }

    #[test]
    fn cbor_encoder_fast_nint_u32() {
        let mut enc = CborEncoderFast::new();
        enc.encode_nint(-65537); // uint = 65536
        let buf = enc.writer.flush();
        assert_eq!(buf[0], 0x3a);
        assert_eq!(u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]), 65536);
    }

    #[test]
    fn cbor_encoder_fast_nint_u64() {
        let mut enc = CborEncoderFast::new();
        enc.encode_nint(i64::MIN); // uint = i64::MAX as u64
        let buf = enc.writer.flush();
        assert_eq!(buf[0], 0x3b);
        assert_eq!(
            u64::from_be_bytes(buf[1..9].try_into().unwrap()),
            i64::MAX as u64
        );
    }

    // ── CborEncoderFast: write_number (integer vs float dispatch) ───────

    #[test]
    fn cbor_encoder_fast_write_number_positive_integer() {
        let mut enc = CborEncoderFast::new();
        enc.write_number(42.0);
        let buf = enc.writer.flush();
        // 42 > 23, so uses 0x18 (u8 uint) format
        assert_eq!(buf, &[0x18, 42]);
    }

    #[test]
    fn cbor_encoder_fast_write_number_negative_integer() {
        let mut enc = CborEncoderFast::new();
        enc.write_number(-10.0);
        let buf = enc.writer.flush();
        // nint: uint = 9, so 0x20 | 9 = 0x29
        assert_eq!(buf, &[0x29]);
    }

    // ── CborEncoderFast: write_arr / write_obj ──────────────────────────

    #[test]
    fn cbor_encoder_fast_write_empty_array() {
        let mut enc = CborEncoderFast::new();
        enc.write_arr(&[]);
        let buf = enc.writer.flush();
        // Empty array: 0x80
        assert_eq!(buf, &[0x80]);
    }

    #[test]
    fn cbor_encoder_fast_write_array_with_elements() {
        let mut enc = CborEncoderFast::new();
        let arr = vec![PackValue::Integer(1), PackValue::Bool(true)];
        enc.write_arr(&arr);
        let buf = enc.writer.flush();
        // 0x82 = array(2), 0x01 = uint(1), 0xf5 = true
        assert_eq!(buf, &[0x82, 0x01, 0xf5]);
    }

    #[test]
    fn cbor_encoder_fast_write_empty_object() {
        let mut enc = CborEncoderFast::new();
        enc.write_obj_pairs(&[]);
        let buf = enc.writer.flush();
        // Empty map: 0xa0
        assert_eq!(buf, &[0xa0]);
    }

    #[test]
    fn cbor_encoder_fast_write_object_with_pairs() {
        let mut enc = CborEncoderFast::new();
        let pairs = vec![("a".to_string(), PackValue::Integer(1))];
        enc.write_obj_pairs(&pairs);
        let buf = enc.writer.flush();
        // 0xa1 = map(1), 0x61 = text(1), 'a', 0x01 = uint(1)
        assert_eq!(buf, &[0xa1, 0x61, b'a', 0x01]);
    }

    // ── CborEncoderFast: write_bin ──────────────────────────────────────

    #[test]
    fn cbor_encoder_fast_write_bin_empty() {
        let mut enc = CborEncoderFast::new();
        enc.write_bin(&[]);
        let buf = enc.writer.flush();
        // Empty byte string: 0x40
        assert_eq!(buf, &[0x40]);
    }

    #[test]
    fn cbor_encoder_fast_write_bin_small() {
        let mut enc = CborEncoderFast::new();
        enc.write_bin(&[0xDE, 0xAD]);
        let buf = enc.writer.flush();
        // 0x42 = bytes(2), then payload
        assert_eq!(buf, &[0x42, 0xDE, 0xAD]);
    }

    #[test]
    fn cbor_encoder_fast_write_bin_medium() {
        let mut enc = CborEncoderFast::new();
        let data = vec![0xAB; 100];
        enc.write_bin(&data);
        let buf = enc.writer.flush();
        // length=100 > 23 → 0x58, length as u8
        assert_eq!(buf[0], 0x58);
        assert_eq!(buf[1], 100);
        assert_eq!(&buf[2..], &data[..]);
    }

    // ── CborEncoderFast: write_tag ──────────────────────────────────────

    #[test]
    fn cbor_encoder_fast_write_tag_small() {
        let mut enc = CborEncoderFast::new();
        enc.write_tag(1, &PackValue::Integer(42));
        let buf = enc.writer.flush();
        // tag(1) = 0xc1, then uint(42) = 0x18, 42
        assert_eq!(buf, &[0xc1, 0x18, 42]);
    }

    #[test]
    fn cbor_encoder_fast_write_tag_u8() {
        let mut enc = CborEncoderFast::new();
        enc.write_tag(100, &PackValue::Null);
        let buf = enc.writer.flush();
        // 0xd8 = tag, one-byte follow, 100, then 0xf6 = null
        assert_eq!(buf, &[0xd8, 100, 0xf6]);
    }

    #[test]
    fn cbor_encoder_fast_write_tag_u16() {
        let mut enc = CborEncoderFast::new();
        enc.write_tag(1000, &PackValue::Null);
        let buf = enc.writer.flush();
        assert_eq!(buf[0], 0xd9);
        assert_eq!(u16::from_be_bytes([buf[1], buf[2]]), 1000);
        assert_eq!(buf[3], 0xf6);
    }

    #[test]
    fn cbor_encoder_fast_write_tag_u32() {
        let mut enc = CborEncoderFast::new();
        enc.write_tag(100_000, &PackValue::Null);
        let buf = enc.writer.flush();
        assert_eq!(buf[0], 0xda);
        assert_eq!(
            u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]),
            100_000
        );
    }

    #[test]
    fn cbor_encoder_fast_write_tag_u64() {
        let mut enc = CborEncoderFast::new();
        enc.write_tag(0x100000000, &PackValue::Null);
        let buf = enc.writer.flush();
        assert_eq!(buf[0], 0xdb);
        assert_eq!(
            u64::from_be_bytes(buf[1..9].try_into().unwrap()),
            0x100000000
        );
    }

    // ── CborEncoderFast: bigint ─────────────────────────────────────────

    #[test]
    fn cbor_encoder_fast_big_int_positive_small() {
        let mut enc = CborEncoderFast::new();
        enc.write_big_int(42);
        let buf = enc.writer.flush();
        assert_eq!(buf, &[0x18, 42]);
    }

    #[test]
    fn cbor_encoder_fast_big_int_negative_small() {
        let mut enc = CborEncoderFast::new();
        enc.write_big_int(-1);
        let buf = enc.writer.flush();
        assert_eq!(buf, &[0x20]);
    }

    #[test]
    fn cbor_encoder_fast_big_uint_overflow_clamps() {
        let mut enc = CborEncoderFast::new();
        // Value larger than u64::MAX → clamped to u64::MAX
        enc.write_big_uint(u128::MAX);
        let buf = enc.writer.flush();
        assert_eq!(buf[0], 0x1b);
        assert_eq!(u64::from_be_bytes(buf[1..9].try_into().unwrap()), u64::MAX);
    }

    #[test]
    fn cbor_encoder_fast_big_sint_below_i64_min() {
        let mut enc = CborEncoderFast::new();
        let val = i64::MIN as i128 - 1;
        enc.write_big_sint(val);
        let buf = enc.writer.flush();
        assert_eq!(buf[0], 0x3b);
        let uint = (-1i128 - val) as u64;
        assert_eq!(u64::from_be_bytes(buf[1..9].try_into().unwrap()), uint);
    }

    // ── CborEncoderFast: write_any (PackValue dispatch) ─────────────────

    #[test]
    fn cbor_encoder_fast_write_any_null_and_undefined() {
        let mut enc = CborEncoderFast::new();
        enc.write_any(&PackValue::Null);
        let buf = enc.writer.flush();
        assert_eq!(buf, &[0xf6]);

        enc.writer.reset();
        enc.write_any(&PackValue::Undefined);
        let buf = enc.writer.flush();
        // Fast encoder maps Undefined → null
        assert_eq!(buf, &[0xf6]);
    }

    #[test]
    fn cbor_encoder_fast_write_any_bool() {
        let mut enc = CborEncoderFast::new();
        enc.write_any(&PackValue::Bool(true));
        let buf = enc.writer.flush();
        assert_eq!(buf, &[0xf5]);

        enc.writer.reset();
        enc.write_any(&PackValue::Bool(false));
        let buf = enc.writer.flush();
        assert_eq!(buf, &[0xf4]);
    }

    #[test]
    fn cbor_encoder_fast_encode_convenience() {
        let mut enc = CborEncoderFast::new();
        let buf = enc.encode(&PackValue::Integer(5));
        assert_eq!(buf, &[0x05]);
        // encode resets and produces fresh output
        let buf = enc.encode(&PackValue::Bool(true));
        assert_eq!(buf, &[0xf5]);
    }

    // ── CborEncoderFast: encode_json ────────────────────────────────────

    #[test]
    fn cbor_encoder_fast_encode_json_all_types() {
        let mut enc = CborEncoderFast::new();
        let val = json!({"key": [1, -2, true, null, "hi"]});
        let buf = enc.encode_json(&val);
        // Decode back as JSON and compare
        let dec = CborDecoder::new();
        let decoded = dec.decode_json(&buf).unwrap();
        assert_eq!(decoded, val);
    }

    // ── CborEncoderFast: streaming helpers ──────────────────────────────

    #[test]
    fn cbor_encoder_fast_streaming_markers() {
        let mut enc = CborEncoderFast::new();
        enc.write_start_arr();
        enc.write_u_integer(1);
        enc.write_u_integer(2);
        enc.write_end_arr();
        let buf = enc.writer.flush();
        // 0x9f = indefinite array start, 0x01, 0x02, 0xff = break
        assert_eq!(buf, &[0x9f, 0x01, 0x02, 0xff]);
    }

    #[test]
    fn cbor_encoder_fast_streaming_obj() {
        let mut enc = CborEncoderFast::new();
        enc.write_start_obj();
        enc.write_str("k");
        enc.write_u_integer(1);
        enc.write_end_obj();
        let buf = enc.writer.flush();
        // 0xbf = indefinite map, key "k", value 1, 0xff = break
        assert_eq!(buf, &[0xbf, 0x61, b'k', 0x01, 0xff]);
    }

    #[test]
    fn cbor_encoder_fast_write_cbor_self_describe_tag() {
        let mut enc = CborEncoderFast::new();
        enc.write_cbor();
        let buf = enc.writer.flush();
        assert_eq!(buf, &[0xd9, 0xd9, 0xf7]);
    }

    // ── CborEncoderStable: sorted keys ──────────────────────────────────

    #[test]
    fn cbor_encoder_stable_sorts_keys_by_length_then_lex() {
        let mut enc = CborEncoderStable::new();
        let obj = PackValue::Object(vec![
            ("bb".into(), PackValue::Integer(2)),
            ("a".into(), PackValue::Integer(1)),
            ("ccc".into(), PackValue::Integer(3)),
        ]);
        let buf = enc.encode(&obj);
        let dec = CborDecoder::new();
        let decoded = dec.decode(&buf).unwrap();
        // Keys should be sorted: "a" (len 1) < "bb" (len 2) < "ccc" (len 3)
        if let PackValue::Object(pairs) = decoded {
            let keys: Vec<&str> = pairs.iter().map(|(k, _)| k.as_str()).collect();
            assert_eq!(keys, vec!["a", "bb", "ccc"]);
        } else {
            panic!("expected Object");
        }
    }

    #[test]
    fn cbor_encoder_stable_sorts_same_length_keys_lex() {
        let mut enc = CborEncoderStable::new();
        let obj = PackValue::Object(vec![
            ("zz".into(), PackValue::Integer(1)),
            ("aa".into(), PackValue::Integer(2)),
            ("mm".into(), PackValue::Integer(3)),
        ]);
        let buf = enc.encode(&obj);
        let dec = CborDecoder::new();
        let decoded = dec.decode(&buf).unwrap();
        if let PackValue::Object(pairs) = decoded {
            let keys: Vec<&str> = pairs.iter().map(|(k, _)| k.as_str()).collect();
            assert_eq!(keys, vec!["aa", "mm", "zz"]);
        } else {
            panic!("expected Object");
        }
    }

    #[test]
    fn cbor_encoder_stable_deterministic_output() {
        let mut enc = CborEncoderStable::new();
        let obj = PackValue::Object(vec![
            ("z".into(), PackValue::Integer(1)),
            ("a".into(), PackValue::Integer(2)),
        ]);
        let buf1 = enc.encode(&obj);
        let buf2 = enc.encode(&obj);
        assert_eq!(buf1, buf2, "stable encoder must produce identical bytes");
    }

    // ── CborEncoderStable: float32 optimization ─────────────────────────

    #[test]
    fn cbor_encoder_stable_float32_when_lossless() {
        let mut enc_stable = CborEncoderStable::new();
        let mut enc_fast = CborEncoderFast::new();

        // 1.5 is exactly representable as f32
        let buf_stable = enc_stable.encode(&PackValue::Float(1.5));
        let buf_fast = enc_fast.encode(&PackValue::Float(1.5));

        // Stable uses f32 (0xfa), fast always uses f64 (0xfb)
        assert_eq!(buf_stable[0], 0xfa, "stable should use f32 for 1.5");
        assert_eq!(buf_fast[0], 0xfb, "fast should use f64 for 1.5");
    }

    #[test]
    fn cbor_encoder_stable_float64_when_needed() {
        let mut enc = CborEncoderStable::new();
        let buf = enc.encode(&PackValue::Float(TEST_F64_3_14159));
        // pi-ish value needs f64
        assert_eq!(buf[0], 0xfb);
    }

    // ── CborEncoderStable: write_str uses exact header ──────────────────

    #[test]
    fn cbor_encoder_stable_write_str_exact_header() {
        let mut enc = CborEncoderStable::new();
        // "€€€€€€" (6 chars, 18 bytes) — Stable uses exact byte length,
        // so 18 <= 23 → short header 0x60|18 = 0x72
        let s = "€€€€€€";
        let buf = enc.encode(&PackValue::Str(s.into()));
        assert_eq!(
            buf[0],
            0x60 | 18,
            "stable should use exact byte length header"
        );
    }

    // ── CborEncoder (full): basic types ─────────────────────────────────

    #[test]
    fn cbor_encoder_full_undefined() {
        let mut enc = CborEncoder::new();
        let buf = enc.encode(&PackValue::Undefined);
        // Full encoder writes 0xf7 for undefined (not null)
        assert_eq!(buf, &[0xf7]);
    }

    #[test]
    fn cbor_encoder_full_float32_optimization() {
        let mut enc = CborEncoder::new();
        let buf = enc.encode(&PackValue::Float(1.0));
        // 1.0 fits in f32 → 0xfa prefix
        assert_eq!(buf[0], 0xfa);
    }

    #[test]
    fn cbor_encoder_full_bin() {
        let mut enc = CborEncoder::new();
        let data = vec![1u8, 2, 3, 4, 5];
        let buf = enc.encode(&PackValue::Bytes(data.clone()));
        // 0x45 = bytes(5)
        assert_eq!(buf[0], 0x45);
        assert_eq!(&buf[1..], &data[..]);
    }

    #[test]
    fn cbor_encoder_full_bigint_positive() {
        let mut enc = CborEncoder::new();
        let buf = enc.encode(&PackValue::BigInt(1000));
        // 1000 fits in u64 → regular uint encoding
        assert_eq!(buf[0], 0x19); // u16 header
        assert_eq!(u16::from_be_bytes([buf[1], buf[2]]), 1000);
    }

    #[test]
    fn cbor_encoder_full_bigint_negative() {
        let mut enc = CborEncoder::new();
        let buf = enc.encode(&PackValue::BigInt(-100));
        // nint: uint = 99
        assert_eq!(buf[0], 0x38); // u8 nint
        assert_eq!(buf[1], 99);
    }

    #[test]
    fn cbor_encoder_full_bigint_overflow() {
        let mut enc = CborEncoder::new();
        let buf = enc.encode(&PackValue::BigInt(u64::MAX as i128 + 1));
        // Overflows u64 → clamps to u64::MAX
        assert_eq!(buf[0], 0x1b);
        assert_eq!(u64::from_be_bytes(buf[1..9].try_into().unwrap()), u64::MAX);
    }

    #[test]
    fn cbor_encoder_full_bigint_below_i64_min() {
        let mut enc = CborEncoder::new();
        let val = i64::MIN as i128 - 1;
        let buf = enc.encode(&PackValue::BigInt(val));
        assert_eq!(buf[0], 0x3b);
        let uint = (-1i128 - val) as u64;
        assert_eq!(u64::from_be_bytes(buf[1..9].try_into().unwrap()), uint);
    }

    #[test]
    fn cbor_encoder_full_extension() {
        use super::JsonPackExtension;
        let mut enc = CborEncoder::new();
        let ext = PackValue::Extension(Box::new(JsonPackExtension::new(
            42,
            PackValue::Str("cid".into()),
        )));
        let buf = enc.encode(&ext);
        let dec = CborDecoder::new();
        let decoded = dec.decode(&buf).unwrap();
        if let PackValue::Extension(e) = decoded {
            assert_eq!(e.tag, 42);
            assert_eq!(*e.val, PackValue::Str("cid".into()));
        } else {
            panic!("expected Extension, got {decoded:?}");
        }
    }

    // ── CborEncoderDag ──────────────────────────────────────────────────

    #[test]
    fn cbor_encoder_dag_nan_becomes_null() {
        let mut enc = CborEncoderDag::new();
        let buf = enc.encode(&PackValue::Float(f64::NAN));
        assert_eq!(buf, &[0xf6], "DAG encoder should write null for NaN");
    }

    #[test]
    fn cbor_encoder_dag_infinity_becomes_null() {
        let mut enc = CborEncoderDag::new();
        let buf = enc.encode(&PackValue::Float(f64::INFINITY));
        assert_eq!(buf, &[0xf6], "DAG encoder should write null for Infinity");

        let buf = enc.encode(&PackValue::Float(f64::NEG_INFINITY));
        assert_eq!(buf, &[0xf6], "DAG encoder should write null for -Infinity");
    }

    #[test]
    fn cbor_encoder_dag_finite_float_is_f64() {
        let mut enc = CborEncoderDag::new();
        let buf = enc.encode(&PackValue::Float(TEST_F64_3_14));
        // DAG encoder always uses f64 for finite floats
        assert_eq!(buf[0], 0xfb);
    }

    #[test]
    fn cbor_encoder_dag_tag_42_preserved() {
        use super::JsonPackExtension;
        let mut enc = CborEncoderDag::new();
        let ext = PackValue::Extension(Box::new(JsonPackExtension::new(
            42,
            PackValue::Bytes(vec![1, 2, 3]),
        )));
        let buf = enc.encode(&ext);
        let dec = CborDecoderDag::new();
        let decoded = dec.decode(&buf).unwrap();
        if let PackValue::Extension(e) = decoded {
            assert_eq!(e.tag, 42);
        } else {
            panic!("expected Extension for tag 42, got {decoded:?}");
        }
    }

    #[test]
    fn cbor_encoder_dag_non_42_tag_unwrapped() {
        use super::JsonPackExtension;
        let mut enc = CborEncoderDag::new();
        let ext = PackValue::Extension(Box::new(JsonPackExtension::new(99, PackValue::Integer(7))));
        let buf = enc.encode(&ext);
        let dec = CborDecoderDag::new();
        let decoded = dec.decode(&buf).unwrap();
        // Tag 99 is unwrapped, so we just get the value
        assert_eq!(decoded, PackValue::Integer(7));
    }

    #[test]
    fn cbor_encoder_dag_sorts_object_keys() {
        let mut enc = CborEncoderDag::new();
        let obj = PackValue::Object(vec![
            ("zz".into(), PackValue::Integer(1)),
            ("a".into(), PackValue::Integer(2)),
        ]);
        let buf = enc.encode(&obj);
        let dec = CborDecoderDag::new();
        let decoded = dec.decode(&buf).unwrap();
        if let PackValue::Object(pairs) = decoded {
            let keys: Vec<&str> = pairs.iter().map(|(k, _)| k.as_str()).collect();
            assert_eq!(keys, vec!["a", "zz"]);
        } else {
            panic!("expected Object");
        }
    }

    // ── Roundtrip tests: encode → decode ────────────────────────────────

    #[test]
    fn cbor_encoder_fast_roundtrip_all_types() {
        let mut enc = CborEncoderFast::new();
        let dec = CborDecoder::new();
        let values = vec![
            PackValue::Null,
            PackValue::Bool(true),
            PackValue::Bool(false),
            PackValue::Integer(0),
            PackValue::Integer(-1),
            PackValue::Integer(1000),
            PackValue::Integer(-1000),
            // Note: UInteger values decode as UInteger only when > i64::MAX.
            // Small UInteger values decode as Integer, so we test encoding separately.
            PackValue::UInteger(u64::MAX),
            PackValue::Float(TEST_F64_2_71828),
            PackValue::Str("".into()),
            PackValue::Str("hello".into()),
            PackValue::Str("a".repeat(300)),
            PackValue::Bytes(vec![]),
            PackValue::Bytes(vec![0xDE, 0xAD, 0xBE, 0xEF]),
            PackValue::Array(vec![]),
            PackValue::Array(vec![PackValue::Integer(1), PackValue::Str("x".into())]),
            PackValue::Object(vec![]),
            PackValue::Object(vec![("k".into(), PackValue::Bool(true))]),
        ];
        for v in &values {
            let buf = enc.encode(v);
            let decoded = dec.decode(&buf).unwrap();
            assert_eq!(&decoded, v, "roundtrip failed for {v:?}");
        }
    }

    #[test]
    fn cbor_encoder_stable_roundtrip() {
        let mut enc = CborEncoderStable::new();
        let dec = CborDecoder::new();
        let values = vec![
            PackValue::Null,
            PackValue::Bool(true),
            PackValue::Integer(42),
            PackValue::Integer(-100),
            PackValue::Float(1.5),
            PackValue::Str("test".into()),
            PackValue::Bytes(vec![1, 2, 3]),
            PackValue::Array(vec![PackValue::Integer(1)]),
        ];
        for v in &values {
            let buf = enc.encode(v);
            let decoded = dec.decode(&buf).unwrap();
            assert_eq!(&decoded, v, "stable roundtrip failed for {v:?}");
        }
    }

    #[test]
    fn cbor_encoder_full_roundtrip() {
        let mut enc = CborEncoder::new();
        let dec = CborDecoder::new();
        let values = vec![
            PackValue::Null,
            PackValue::Undefined,
            PackValue::Bool(true),
            PackValue::Integer(0),
            PackValue::Integer(-1),
            PackValue::UInteger(u64::MAX),
            PackValue::Float(TEST_F64_2_71828),
            PackValue::Str("".into()),
            PackValue::Str("x".repeat(1000)),
            PackValue::Bytes(vec![0; 50]),
            PackValue::Array(vec![PackValue::Null, PackValue::Bool(false)]),
            PackValue::Object(vec![("key".into(), PackValue::Str("val".into()))]),
            // BigInt values that fit in i64 decode as Integer, not BigInt.
            // Test large BigInt separately; here we test values that roundtrip exactly.
        ];
        for v in &values {
            let buf = enc.encode(v);
            let decoded = dec.decode(&buf).unwrap();
            assert_eq!(&decoded, v, "full encoder roundtrip failed for {v:?}");
        }
    }

    #[test]
    fn cbor_encoder_dag_roundtrip() {
        let mut enc = CborEncoderDag::new();
        let dec = CborDecoderDag::new();
        let values = vec![
            PackValue::Null,
            PackValue::Bool(true),
            PackValue::Integer(99),
            PackValue::Str("dag".into()),
            PackValue::Bytes(vec![1, 2]),
            PackValue::Array(vec![PackValue::Integer(1)]),
            PackValue::Object(vec![("k".into(), PackValue::Integer(1))]),
        ];
        for v in &values {
            let buf = enc.encode(v);
            let decoded = dec.decode(&buf).unwrap();
            assert_eq!(decoded, *v, "dag roundtrip failed for {v:?}");
        }
    }

    // ── CborEncoderFast: arr_hdr size branches ──────────────────────────

    #[test]
    fn cbor_encoder_fast_arr_hdr_u8() {
        let mut enc = CborEncoderFast::new();
        let items: Vec<PackValue> = (0..24).map(PackValue::Integer).collect();
        let buf = enc.encode(&PackValue::Array(items));
        // 24 items → 0x98 header (array, 1-byte length)
        assert_eq!(buf[0], 0x98);
        assert_eq!(buf[1], 24);
    }

    #[test]
    fn cbor_encoder_fast_obj_hdr_u8() {
        let mut enc = CborEncoderFast::new();
        let pairs: Vec<(String, PackValue)> = (0..24)
            .map(|i| (format!("{i:02}"), PackValue::Integer(i)))
            .collect();
        let buf = enc.encode(&PackValue::Object(pairs));
        // 24 pairs → 0xb8 header (map, 1-byte length)
        assert_eq!(buf[0], 0xb8);
        assert_eq!(buf[1], 24);
    }

    // ── Standalone functions ────────────────────────────────────────────

    #[test]
    fn write_cbor_uint_major_all_sizes() {
        // Tiny
        let mut out = Vec::new();
        write_cbor_uint_major(&mut out, 0, 5);
        assert_eq!(out, &[0x05]);

        // u8
        let mut out = Vec::new();
        write_cbor_uint_major(&mut out, 0, 200);
        assert_eq!(out, &[0x18, 200]);

        // u16
        let mut out = Vec::new();
        write_cbor_uint_major(&mut out, 0, 1000);
        assert_eq!(out[0], 0x19);

        // u32
        let mut out = Vec::new();
        write_cbor_uint_major(&mut out, 0, 100_000);
        assert_eq!(out[0], 0x1a);

        // u64
        let mut out = Vec::new();
        write_cbor_uint_major(&mut out, 0, 0x100000000);
        assert_eq!(out[0], 0x1b);
    }

    #[test]
    fn write_cbor_signed_positive_and_negative() {
        let mut out = Vec::new();
        write_cbor_signed(&mut out, 10);
        assert_eq!(out, &[0x0a]); // major 0, value 10

        let mut out = Vec::new();
        write_cbor_signed(&mut out, -10);
        // major 1, encoded value 9
        assert_eq!(out, &[0x29]);
    }

    #[test]
    fn encode_json_to_cbor_bytes_and_decode_roundtrip() {
        let value = json!({"nested": [1, null, true, "str", -5]});
        let bytes = encode_json_to_cbor_bytes(&value).unwrap();
        let decoded = decode_json_from_cbor_bytes(&bytes).unwrap();
        assert_eq!(decoded, value);
    }

    // ================================================================
    // Coverage-fill: MsgPack to_json converter — all type branches
    // ================================================================

    #[test]
    fn msgpack_to_json_negative_fixint() {
        use super::msgpack::{MsgPackEncoderFast, MsgPackToJsonConverter};
        let mut enc = MsgPackEncoderFast::new();
        let mut conv = MsgPackToJsonConverter::new();
        let bytes = enc.encode(&PackValue::Integer(-5));
        let json = conv.convert(&bytes);
        assert_eq!(json, "-5");
    }

    #[test]
    fn msgpack_to_json_positive_fixint() {
        use super::msgpack::{MsgPackEncoderFast, MsgPackToJsonConverter};
        let mut enc = MsgPackEncoderFast::new();
        let mut conv = MsgPackToJsonConverter::new();
        let bytes = enc.encode(&PackValue::Integer(42));
        let json = conv.convert(&bytes);
        assert_eq!(json, "42");
    }

    #[test]
    fn msgpack_to_json_bool_null() {
        use super::msgpack::{MsgPackEncoderFast, MsgPackToJsonConverter};
        let mut enc = MsgPackEncoderFast::new();
        let mut conv = MsgPackToJsonConverter::new();
        assert_eq!(conv.convert(&enc.encode(&PackValue::Null)), "null");
        assert_eq!(conv.convert(&enc.encode(&PackValue::Bool(true))), "true");
        assert_eq!(conv.convert(&enc.encode(&PackValue::Bool(false))), "false");
    }

    #[test]
    fn msgpack_to_json_float() {
        use super::msgpack::{MsgPackEncoderFast, MsgPackToJsonConverter};
        let mut enc = MsgPackEncoderFast::new();
        let mut conv = MsgPackToJsonConverter::new();
        let bytes = enc.encode(&PackValue::Float(1.5));
        let json = conv.convert(&bytes);
        assert_eq!(json, "1.5");
    }

    #[test]
    fn msgpack_to_json_string() {
        use super::msgpack::{MsgPackEncoderFast, MsgPackToJsonConverter};
        let mut enc = MsgPackEncoderFast::new();
        let mut conv = MsgPackToJsonConverter::new();
        let bytes = enc.encode(&PackValue::Str("hello \"world\"".into()));
        let json = conv.convert(&bytes);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, serde_json::json!("hello \"world\""));
    }

    #[test]
    fn msgpack_to_json_binary() {
        use super::msgpack::{MsgPackEncoderFast, MsgPackToJsonConverter};
        let mut enc = MsgPackEncoderFast::new();
        let mut conv = MsgPackToJsonConverter::new();
        let bytes = enc.encode(&PackValue::Bytes(vec![1, 2, 3]));
        let json = conv.convert(&bytes);
        assert!(json.contains("data:application/octet-stream;base64,"));
    }

    #[test]
    fn msgpack_to_json_array() {
        use super::msgpack::{MsgPackEncoderFast, MsgPackToJsonConverter};
        let mut enc = MsgPackEncoderFast::new();
        let mut conv = MsgPackToJsonConverter::new();
        let arr = PackValue::Array(vec![
            PackValue::Integer(1),
            PackValue::Bool(true),
            PackValue::Null,
        ]);
        let bytes = enc.encode(&arr);
        let json = conv.convert(&bytes);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, serde_json::json!([1, true, null]));
    }

    #[test]
    fn msgpack_to_json_nested_object() {
        use super::msgpack::{MsgPackEncoderFast, MsgPackToJsonConverter};
        let mut enc = MsgPackEncoderFast::new();
        let mut conv = MsgPackToJsonConverter::new();
        let obj = PackValue::Object(vec![
            ("arr".into(), PackValue::Array(vec![PackValue::Integer(1)])),
            (
                "obj".into(),
                PackValue::Object(vec![("k".into(), PackValue::Str("v".into()))]),
            ),
        ]);
        let bytes = enc.encode(&obj);
        let json = conv.convert(&bytes);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["arr"], serde_json::json!([1]));
        assert_eq!(parsed["obj"]["k"], serde_json::json!("v"));
    }

    #[test]
    fn msgpack_to_json_empty_input() {
        use super::msgpack::MsgPackToJsonConverter;
        let mut conv = MsgPackToJsonConverter::new();
        assert_eq!(conv.convert(&[]), "null");
    }

    #[test]
    fn msgpack_to_json_undefined() {
        use super::msgpack::MsgPackToJsonConverter;
        let mut conv = MsgPackToJsonConverter::new();
        // 0xc1 = undefined/never-used -> null in JSON
        assert_eq!(conv.convert(&[0xc1]), "null");
    }

    #[test]
    fn msgpack_to_json_uint8() {
        use super::msgpack::MsgPackToJsonConverter;
        let mut conv = MsgPackToJsonConverter::new();
        // 0xcc = uint8, followed by 200
        assert_eq!(conv.convert(&[0xcc, 200]), "200");
    }

    #[test]
    fn msgpack_to_json_uint16() {
        use super::msgpack::MsgPackToJsonConverter;
        let mut conv = MsgPackToJsonConverter::new();
        // 0xcd = uint16
        let bytes = [0xcd, 0x01, 0x00]; // 256
        assert_eq!(conv.convert(&bytes), "256");
    }

    #[test]
    fn msgpack_to_json_uint32() {
        use super::msgpack::MsgPackToJsonConverter;
        let mut conv = MsgPackToJsonConverter::new();
        // 0xce = uint32
        let bytes = [0xce, 0x00, 0x01, 0x00, 0x00]; // 65536
        assert_eq!(conv.convert(&bytes), "65536");
    }

    #[test]
    fn msgpack_to_json_uint64() {
        use super::msgpack::MsgPackToJsonConverter;
        let mut conv = MsgPackToJsonConverter::new();
        // 0xcf = uint64 (hi32:lo32)
        let bytes = [0xcf, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00]; // 4294967296
        assert_eq!(conv.convert(&bytes), "4294967296");
    }

    #[test]
    fn msgpack_to_json_int8() {
        use super::msgpack::MsgPackToJsonConverter;
        let mut conv = MsgPackToJsonConverter::new();
        // 0xd0 = int8, -100
        let bytes = [0xd0, (-100i8) as u8];
        assert_eq!(conv.convert(&bytes), "-100");
    }

    #[test]
    fn msgpack_to_json_int16() {
        use super::msgpack::MsgPackToJsonConverter;
        let mut conv = MsgPackToJsonConverter::new();
        // 0xd1 = int16
        let val: i16 = -1000;
        let be = val.to_be_bytes();
        let bytes = [0xd1, be[0], be[1]];
        assert_eq!(conv.convert(&bytes), "-1000");
    }

    #[test]
    fn msgpack_to_json_int32() {
        use super::msgpack::MsgPackToJsonConverter;
        let mut conv = MsgPackToJsonConverter::new();
        // 0xd2 = int32
        let val: i32 = -100000;
        let be = val.to_be_bytes();
        let bytes = [0xd2, be[0], be[1], be[2], be[3]];
        assert_eq!(conv.convert(&bytes), "-100000");
    }

    #[test]
    fn msgpack_to_json_int64() {
        use super::msgpack::MsgPackToJsonConverter;
        let mut conv = MsgPackToJsonConverter::new();
        // 0xd3 = int64 (hi_i32:lo_u32), value = -1
        let bytes = [0xd3, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        assert_eq!(conv.convert(&bytes), "-1");
    }

    #[test]
    fn msgpack_to_json_float32() {
        use super::msgpack::MsgPackToJsonConverter;
        let mut conv = MsgPackToJsonConverter::new();
        // 0xca = float32
        let val: f32 = 1.5;
        let be = val.to_be_bytes();
        let bytes = [0xca, be[0], be[1], be[2], be[3]];
        let json = conv.convert(&bytes);
        assert_eq!(json, "1.5");
    }

    #[test]
    fn msgpack_to_json_str8() {
        use super::msgpack::MsgPackToJsonConverter;
        let mut conv = MsgPackToJsonConverter::new();
        // 0xd9 = str8, length=5, "hello"
        let bytes = [0xd9, 5, b'h', b'e', b'l', b'l', b'o'];
        assert_eq!(conv.convert(&bytes), "\"hello\"");
    }

    #[test]
    fn msgpack_to_json_bin16() {
        use super::msgpack::MsgPackToJsonConverter;
        let mut conv = MsgPackToJsonConverter::new();
        // 0xc5 = bin16
        let bytes = [0xc5, 0x00, 0x02, 0xAA, 0xBB];
        let json = conv.convert(&bytes);
        assert!(json.contains("data:application/octet-stream;base64,"));
    }

    #[test]
    fn msgpack_to_json_ext_fixext1() {
        use super::msgpack::MsgPackToJsonConverter;
        let mut conv = MsgPackToJsonConverter::new();
        // 0xd4 = fixext1 (1 byte of ext data + 1 byte type)
        let bytes = [0xd4, 0x01, 0xFF]; // type=1, data=[0xFF]
        let json = conv.convert(&bytes);
        assert!(json.contains("data:application/octet-stream;base64,"));
    }

    #[test]
    fn msgpack_to_json_ext8() {
        use super::msgpack::MsgPackToJsonConverter;
        let mut conv = MsgPackToJsonConverter::new();
        // 0xc7 = ext8, length=2, type=5, data=[0x01, 0x02]
        let bytes = [0xc7, 0x02, 0x05, 0x01, 0x02];
        let json = conv.convert(&bytes);
        assert!(json.contains("data:application/octet-stream;base64,"));
    }

    // ================================================================
    // Coverage-fill: MsgPack decoder — skip, validate, find, read_level
    // ================================================================

    #[test]
    fn msgpack_decoder_skip_any_primitives() {
        use super::msgpack::{MsgPackDecoder, MsgPackEncoderFast};
        let mut enc = MsgPackEncoderFast::new();
        let mut dec = MsgPackDecoder::new();
        // Skip a positive fixint
        let bytes = enc.encode(&PackValue::Integer(42));
        dec.reset(&bytes);
        assert_eq!(dec.skip_any().unwrap(), 1);
        // Skip a null
        let bytes = enc.encode(&PackValue::Null);
        dec.reset(&bytes);
        assert_eq!(dec.skip_any().unwrap(), 1);
        // Skip a float64
        let bytes = enc.encode(&PackValue::Float(1.5));
        dec.reset(&bytes);
        assert_eq!(dec.skip_any().unwrap(), 9); // 1 byte header + 8 bytes
                                                // Skip a string
        let bytes = enc.encode(&PackValue::Str("abc".into()));
        dec.reset(&bytes);
        assert_eq!(dec.skip_any().unwrap(), 4); // 1 byte fixstr header + 3 bytes
    }

    #[test]
    fn msgpack_decoder_skip_any_array_and_map() {
        use super::msgpack::{MsgPackDecoder, MsgPackEncoderFast};
        let mut enc = MsgPackEncoderFast::new();
        let mut dec = MsgPackDecoder::new();
        let arr = PackValue::Array(vec![PackValue::Integer(1), PackValue::Integer(2)]);
        let bytes = enc.encode(&arr);
        dec.reset(&bytes);
        let skipped = dec.skip_any().unwrap();
        assert_eq!(skipped, bytes.len());

        let obj = PackValue::Object(vec![("a".into(), PackValue::Integer(1))]);
        let bytes = enc.encode(&obj);
        dec.reset(&bytes);
        let skipped = dec.skip_any().unwrap();
        assert_eq!(skipped, bytes.len());
    }

    #[test]
    fn msgpack_decoder_skip_any_eof_error() {
        use super::msgpack::MsgPackDecoder;
        let mut dec = MsgPackDecoder::new();
        dec.reset(&[]);
        assert!(dec.skip_any().is_err());
    }

    #[test]
    fn msgpack_decoder_validate_correct() {
        use super::msgpack::{MsgPackDecoder, MsgPackEncoderFast};
        let mut enc = MsgPackEncoderFast::new();
        let mut dec = MsgPackDecoder::new();
        let bytes = enc.encode(&PackValue::Integer(42));
        assert!(dec.validate(&bytes, 0, bytes.len()).is_ok());
    }

    #[test]
    fn msgpack_decoder_validate_wrong_size() {
        use super::msgpack::{MsgPackDecoder, MsgPackEncoderFast};
        let mut enc = MsgPackEncoderFast::new();
        let mut dec = MsgPackDecoder::new();
        let bytes = enc.encode(&PackValue::Integer(42));
        assert!(dec.validate(&bytes, 0, bytes.len() + 1).is_err());
    }

    #[test]
    fn msgpack_decoder_find_key_found() {
        use super::msgpack::{MsgPackDecoder, MsgPackEncoderFast};
        let mut enc = MsgPackEncoderFast::new();
        let mut dec = MsgPackDecoder::new();
        let obj = PackValue::Object(vec![
            ("alpha".into(), PackValue::Integer(1)),
            ("beta".into(), PackValue::Integer(2)),
        ]);
        let bytes = enc.encode(&obj);
        dec.reset(&bytes);
        dec.find_key("beta").unwrap();
        let val = dec.read_any().unwrap();
        assert_eq!(val, PackValue::Integer(2));
    }

    #[test]
    fn msgpack_decoder_find_key_not_found() {
        use super::msgpack::{MsgPackDecoder, MsgPackEncoderFast};
        let mut enc = MsgPackEncoderFast::new();
        let mut dec = MsgPackDecoder::new();
        let obj = PackValue::Object(vec![("alpha".into(), PackValue::Integer(1))]);
        let bytes = enc.encode(&obj);
        dec.reset(&bytes);
        assert!(dec.find_key("gamma").is_err());
    }

    #[test]
    fn msgpack_decoder_find_index() {
        use super::msgpack::{MsgPackDecoder, MsgPackEncoderFast};
        let mut enc = MsgPackEncoderFast::new();
        let mut dec = MsgPackDecoder::new();
        let arr = PackValue::Array(vec![
            PackValue::Integer(10),
            PackValue::Integer(20),
            PackValue::Integer(30),
        ]);
        let bytes = enc.encode(&arr);
        dec.reset(&bytes);
        dec.find_index(2).unwrap();
        let val = dec.read_any().unwrap();
        assert_eq!(val, PackValue::Integer(30));
    }

    #[test]
    fn msgpack_decoder_find_index_out_of_bounds() {
        use super::msgpack::{MsgPackDecoder, MsgPackEncoderFast};
        let mut enc = MsgPackEncoderFast::new();
        let mut dec = MsgPackDecoder::new();
        let arr = PackValue::Array(vec![PackValue::Integer(10)]);
        let bytes = enc.encode(&arr);
        dec.reset(&bytes);
        assert!(dec.find_index(5).is_err());
    }

    #[test]
    fn msgpack_decoder_find_path() {
        use super::msgpack::{MsgPackDecoder, MsgPackEncoderFast, MsgPackPathSegment};
        let mut enc = MsgPackEncoderFast::new();
        let mut dec = MsgPackDecoder::new();
        let nested = PackValue::Object(vec![(
            "items".into(),
            PackValue::Array(vec![PackValue::Str("a".into()), PackValue::Str("b".into())]),
        )]);
        let bytes = enc.encode(&nested);
        dec.reset(&bytes);
        dec.find_path(&[
            MsgPackPathSegment::Key("items"),
            MsgPackPathSegment::Index(1),
        ])
        .unwrap();
        let val = dec.read_any().unwrap();
        assert_eq!(val, PackValue::Str("b".into()));
    }

    #[test]
    fn msgpack_decoder_read_level() {
        use super::msgpack::{MsgPackDecoder, MsgPackEncoderFast};
        let mut enc = MsgPackEncoderFast::new();
        let mut dec = MsgPackDecoder::new();
        let inner_arr = PackValue::Array(vec![PackValue::Integer(1)]);
        let inner_bytes = enc.encode(&inner_arr);
        let outer = PackValue::Object(vec![
            ("x".into(), PackValue::Integer(42)),
            ("nested".into(), inner_arr),
        ]);
        let bytes = enc.encode(&outer);
        let result = dec.read_level(&bytes).unwrap();
        if let PackValue::Object(pairs) = result {
            assert_eq!(pairs[0].0, "x");
            assert_eq!(pairs[0].1, PackValue::Integer(42));
            assert_eq!(pairs[1].0, "nested");
            // Nested array should be a Blob
            assert!(
                matches!(&pairs[1].1, PackValue::Blob(b) if b.val == inner_bytes),
                "nested should be Blob"
            );
        } else {
            panic!("expected Object");
        }
    }

    #[test]
    fn msgpack_decoder_read_obj_hdr() {
        use super::msgpack::MsgPackDecoder;
        let mut dec = MsgPackDecoder::new();
        // fixmap with 3 entries: 0x83
        dec.reset(&[0x83]);
        assert_eq!(dec.read_obj_hdr().unwrap(), 3);
        // not a map
        dec.reset(&[0x92]); // fixarray
        assert!(dec.read_obj_hdr().is_err());
    }

    #[test]
    fn msgpack_decoder_read_arr_hdr() {
        use super::msgpack::MsgPackDecoder;
        let mut dec = MsgPackDecoder::new();
        // fixarray with 5 entries: 0x95
        dec.reset(&[0x95]);
        assert_eq!(dec.read_arr_hdr().unwrap(), 5);
        // not an array
        dec.reset(&[0x82]); // fixmap
        assert!(dec.read_arr_hdr().is_err());
    }

    #[test]
    fn msgpack_decoder_read_str_hdr() {
        use super::msgpack::MsgPackDecoder;
        let mut dec = MsgPackDecoder::new();
        // fixstr with length 5: 0xa5
        dec.reset(&[0xa5]);
        assert_eq!(dec.read_str_hdr().unwrap(), 5);
        // not a string
        dec.reset(&[0xc0]); // null
        assert!(dec.read_str_hdr().is_err());
    }

    // ================================================================
    // Coverage-fill: MsgPack decoder_fast — additional type branches
    // ================================================================

    #[test]
    fn msgpack_decoder_fast_undefined() {
        use super::msgpack::MsgPackDecoderFast;
        let mut dec = MsgPackDecoderFast::new();
        assert_eq!(dec.decode(&[0xc1]).unwrap(), PackValue::Undefined);
    }

    #[test]
    fn msgpack_decoder_fast_uint64() {
        use super::msgpack::MsgPackDecoderFast;
        let mut dec = MsgPackDecoderFast::new();
        // 0xcf = uint64, value = 2^32 = 4294967296
        let bytes = [0xcf, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00];
        let v = dec.decode(&bytes).unwrap();
        assert_eq!(v, PackValue::UInteger(4294967296));
    }

    #[test]
    fn msgpack_decoder_fast_int64() {
        use super::msgpack::MsgPackDecoderFast;
        let mut dec = MsgPackDecoderFast::new();
        // 0xd3 = int64, value = -1 (0xffffffffffffffff)
        let bytes = [0xd3, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        let v = dec.decode(&bytes).unwrap();
        assert_eq!(v, PackValue::Integer(-1));
    }

    #[test]
    fn msgpack_decoder_fast_int8() {
        use super::msgpack::MsgPackDecoderFast;
        let mut dec = MsgPackDecoderFast::new();
        // 0xd0 = int8, -100
        let bytes = [0xd0, (-100i8) as u8];
        assert_eq!(dec.decode(&bytes).unwrap(), PackValue::Integer(-100));
    }

    #[test]
    fn msgpack_decoder_fast_int16() {
        use super::msgpack::MsgPackDecoderFast;
        let mut dec = MsgPackDecoderFast::new();
        let val: i16 = -1000;
        let be = val.to_be_bytes();
        let bytes = [0xd1, be[0], be[1]];
        assert_eq!(dec.decode(&bytes).unwrap(), PackValue::Integer(-1000));
    }

    #[test]
    fn msgpack_decoder_fast_int32() {
        use super::msgpack::MsgPackDecoderFast;
        let mut dec = MsgPackDecoderFast::new();
        let val: i32 = -100000;
        let be = val.to_be_bytes();
        let bytes = [0xd2, be[0], be[1], be[2], be[3]];
        assert_eq!(dec.decode(&bytes).unwrap(), PackValue::Integer(-100000));
    }

    #[test]
    fn msgpack_decoder_fast_float32() {
        use super::msgpack::MsgPackDecoderFast;
        let mut dec = MsgPackDecoderFast::new();
        let val: f32 = 1.5;
        let be = val.to_be_bytes();
        let bytes = [0xca, be[0], be[1], be[2], be[3]];
        if let PackValue::Float(f) = dec.decode(&bytes).unwrap() {
            assert!((f - 1.5).abs() < 1e-6);
        } else {
            panic!("expected Float");
        }
    }

    #[test]
    fn msgpack_decoder_fast_ext_types() {
        use super::msgpack::{MsgPackDecoderFast, MsgPackEncoderFast};
        use crate::JsonPackExtension;
        let mut enc = MsgPackEncoderFast::new();
        let mut dec = MsgPackDecoderFast::new();
        let ext = PackValue::Extension(Box::new(JsonPackExtension::new(
            42,
            PackValue::Bytes(vec![1, 2, 3, 4]),
        )));
        let bytes = enc.encode(&ext);
        let decoded = dec.decode(&bytes).unwrap();
        if let PackValue::Extension(e) = decoded {
            assert_eq!(e.tag, 42);
            assert_eq!(*e.val, PackValue::Bytes(vec![1, 2, 3, 4]));
        } else {
            panic!("expected Extension");
        }
    }

    #[test]
    fn msgpack_decoder_fast_bin16() {
        use super::msgpack::MsgPackDecoderFast;
        let mut dec = MsgPackDecoderFast::new();
        // 0xc5 = bin16, length=3
        let bytes = [0xc5, 0x00, 0x03, 0xAA, 0xBB, 0xCC];
        assert_eq!(
            dec.decode(&bytes).unwrap(),
            PackValue::Bytes(vec![0xAA, 0xBB, 0xCC])
        );
    }

    #[test]
    fn msgpack_decoder_fast_proto_key_rejected() {
        use super::msgpack::MsgPackDecoderFast;
        let mut dec = MsgPackDecoderFast::new();
        // Build a map with __proto__ key manually
        let key = b"__proto__";
        let mut bytes = vec![0x81]; // fixmap 1 entry
        bytes.push(0xa0 | key.len() as u8); // fixstr header
        bytes.extend_from_slice(key);
        bytes.push(0x01); // value: positive fixint 1
        assert!(dec.decode(&bytes).is_err());
    }

    #[test]
    fn msgpack_decoder_fast_str8_str16() {
        use super::msgpack::MsgPackDecoderFast;
        let mut dec = MsgPackDecoderFast::new();
        // str8: 0xd9
        let mut bytes = vec![0xd9, 5];
        bytes.extend_from_slice(b"hello");
        assert_eq!(dec.decode(&bytes).unwrap(), PackValue::Str("hello".into()));
        // str16: 0xda
        let mut bytes = vec![0xda, 0x00, 0x03];
        bytes.extend_from_slice(b"abc");
        assert_eq!(dec.decode(&bytes).unwrap(), PackValue::Str("abc".into()));
    }

    #[test]
    fn msgpack_decoder_fast_read_key_str8() {
        use super::msgpack::MsgPackDecoderFast;
        let mut dec = MsgPackDecoderFast::new();
        // str8 key: 0xd9
        let mut bytes = vec![0xd9, 3];
        bytes.extend_from_slice(b"abc");
        dec.data = bytes;
        dec.x = 0;
        assert_eq!(dec.read_key().unwrap(), "abc");
    }

    // ================================================================
    // Coverage-fill: MsgPack decoder — skip various binary/ext/int types
    // ================================================================

    #[test]
    fn msgpack_decoder_skip_binary_types() {
        use super::msgpack::{MsgPackDecoder, MsgPackEncoderFast};
        let mut enc = MsgPackEncoderFast::new();
        let mut dec = MsgPackDecoder::new();
        // bin8 (0xc4)
        let bytes = enc.encode(&PackValue::Bytes(vec![1, 2, 3]));
        dec.reset(&bytes);
        let n = dec.skip_any().unwrap();
        assert_eq!(n, bytes.len());
    }

    #[test]
    fn msgpack_decoder_skip_ext_types() {
        use super::msgpack::{MsgPackDecoder, MsgPackEncoderFast};
        use crate::JsonPackExtension;
        let mut enc = MsgPackEncoderFast::new();
        let mut dec = MsgPackDecoder::new();
        // fixext4 (0xd6)
        let ext = PackValue::Extension(Box::new(JsonPackExtension::new(
            1,
            PackValue::Bytes(vec![1, 2, 3, 4]),
        )));
        let bytes = enc.encode(&ext);
        dec.reset(&bytes);
        let n = dec.skip_any().unwrap();
        assert_eq!(n, bytes.len());
    }

    #[test]
    fn msgpack_decoder_skip_int_types() {
        use super::msgpack::MsgPackDecoder;
        let mut dec = MsgPackDecoder::new();
        // int8 (0xd0)
        dec.reset(&[0xd0, 0x80]);
        assert_eq!(dec.skip_any().unwrap(), 2);
        // int16 (0xd1)
        dec.reset(&[0xd1, 0x80, 0x00]);
        assert_eq!(dec.skip_any().unwrap(), 3);
        // int32 (0xd2)
        dec.reset(&[0xd2, 0x80, 0x00, 0x00, 0x00]);
        assert_eq!(dec.skip_any().unwrap(), 5);
        // int64 (0xd3)
        dec.reset(&[0xd3, 0, 0, 0, 0, 0, 0, 0, 1]);
        assert_eq!(dec.skip_any().unwrap(), 9);
        // uint8 (0xcc)
        dec.reset(&[0xcc, 0xFF]);
        assert_eq!(dec.skip_any().unwrap(), 2);
        // uint64 (0xcf)
        dec.reset(&[0xcf, 0, 0, 0, 0, 0, 0, 0, 1]);
        assert_eq!(dec.skip_any().unwrap(), 9);
    }

    #[test]
    fn msgpack_decoder_skip_float_types() {
        use super::msgpack::MsgPackDecoder;
        let mut dec = MsgPackDecoder::new();
        // float32 (0xca)
        dec.reset(&[0xca, 0, 0, 0, 0]);
        assert_eq!(dec.skip_any().unwrap(), 5);
        // float64 (0xcb)
        dec.reset(&[0xcb, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(dec.skip_any().unwrap(), 9);
    }

    #[test]
    fn msgpack_decoder_skip_fixext_sizes() {
        use super::msgpack::MsgPackDecoder;
        let mut dec = MsgPackDecoder::new();
        // fixext1 (0xd4): 1 type byte + 1 data byte
        dec.reset(&[0xd4, 0x01, 0xFF]);
        assert_eq!(dec.skip_any().unwrap(), 3);
        // fixext2 (0xd5): 1 type byte + 2 data bytes
        dec.reset(&[0xd5, 0x01, 0xFF, 0xFF]);
        assert_eq!(dec.skip_any().unwrap(), 4);
        // fixext4 (0xd6)
        dec.reset(&[0xd6, 0x01, 0, 0, 0, 0]);
        assert_eq!(dec.skip_any().unwrap(), 6);
        // fixext8 (0xd7)
        dec.reset(&[0xd7, 0x01, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(dec.skip_any().unwrap(), 10);
        // fixext16 (0xd8)
        let mut data = vec![0xd8, 0x01];
        data.extend_from_slice(&[0u8; 16]);
        dec.reset(&data);
        assert_eq!(dec.skip_any().unwrap(), 18);
    }

    // ================================================================
    // Coverage-fill: JSON encoder — additional branches
    // ================================================================

    #[test]
    fn json_encoder_encode_json() {
        use super::json::JsonEncoder;
        let mut enc = JsonEncoder::new();
        let val = serde_json::json!({"a": [1, 2], "b": "x"});
        let bytes = enc.encode_json(&val);
        let s = std::str::from_utf8(&bytes).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(s).unwrap();
        assert_eq!(parsed, val);
    }

    #[test]
    fn json_encoder_encode_json_empty_object() {
        use super::json::JsonEncoder;
        let mut enc = JsonEncoder::new();
        let val = serde_json::json!({});
        let bytes = enc.encode_json(&val);
        assert_eq!(&bytes, b"{}");
    }

    #[test]
    fn json_encoder_nan_and_infinity() {
        use super::json::JsonEncoder;
        let mut enc = JsonEncoder::new();
        // NaN -> "null"
        let out = enc.encode(&PackValue::Float(f64::NAN));
        assert_eq!(&out, b"null");
        // +Infinity -> "1e308"
        let out = enc.encode(&PackValue::Float(f64::INFINITY));
        assert_eq!(&out, b"1e308");
        // -Infinity -> "-1e308"
        let out = enc.encode(&PackValue::Float(f64::NEG_INFINITY));
        assert_eq!(&out, b"-1e308");
    }

    #[test]
    fn json_encoder_uinteger() {
        use super::json::JsonEncoder;
        let mut enc = JsonEncoder::new();
        let out = enc.encode(&PackValue::UInteger(999));
        assert_eq!(&out, b"999");
    }

    #[test]
    fn json_encoder_big_int() {
        use super::json::JsonEncoder;
        let mut enc = JsonEncoder::new();
        let out = enc.encode(&PackValue::BigInt(123456789012345));
        assert_eq!(&out, b"123456789012345");
    }

    #[test]
    fn json_encoder_extension_and_blob_as_null() {
        use super::json::JsonEncoder;
        use crate::JsonPackExtension;
        let mut enc = JsonEncoder::new();
        let ext = PackValue::Extension(Box::new(JsonPackExtension::new(1, PackValue::Null)));
        assert_eq!(enc.encode(&ext), b"null");
        let blob = PackValue::Blob(crate::JsonPackValue::new(vec![1, 2, 3]));
        assert_eq!(enc.encode(&blob), b"null");
    }

    #[test]
    fn json_encoder_str_with_special_chars() {
        use super::json::JsonEncoder;
        let mut enc = JsonEncoder::new();
        // String with quotes and backslash
        let out = enc.encode(&PackValue::Str("he said \"hi\" \\ there".into()));
        let s = std::str::from_utf8(&out).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(s).unwrap();
        assert_eq!(parsed, serde_json::json!("he said \"hi\" \\ there"));
    }

    #[test]
    fn json_encoder_str_with_unicode() {
        use super::json::JsonEncoder;
        let mut enc = JsonEncoder::new();
        // Long string (>256 bytes) triggers fallback path
        let long_str = "a".repeat(300);
        let out = enc.encode(&PackValue::Str(long_str.clone()));
        let s = std::str::from_utf8(&out).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(s).unwrap();
        assert_eq!(parsed.as_str().unwrap(), long_str);
    }

    #[test]
    fn json_encoder_empty_array_and_object() {
        use super::json::JsonEncoder;
        let mut enc = JsonEncoder::new();
        assert_eq!(enc.encode(&PackValue::Array(vec![])), b"[]");
        assert_eq!(enc.encode(&PackValue::Object(vec![])), b"{}");
    }

    #[test]
    fn json_encoder_ascii_str() {
        use super::json::JsonEncoder;
        let mut enc = JsonEncoder::new();
        enc.writer.reset();
        enc.write_ascii_str("hello \"world\"");
        let out = enc.writer.flush();
        let s = std::str::from_utf8(&out).unwrap();
        assert_eq!(s, "\"hello \\\"world\\\"\"");
    }

    #[test]
    fn json_encoder_streaming_api() {
        use super::json::JsonEncoder;
        let mut enc = JsonEncoder::new();
        enc.writer.reset();
        enc.write_start_arr();
        enc.write_any(&PackValue::Integer(1));
        enc.write_arr_separator();
        enc.write_any(&PackValue::Integer(2));
        enc.write_end_arr();
        let out = enc.writer.flush();
        assert_eq!(&out, b"[1,2]");
    }

    #[test]
    fn json_encoder_streaming_obj() {
        use super::json::JsonEncoder;
        let mut enc = JsonEncoder::new();
        enc.writer.reset();
        enc.write_start_obj();
        enc.write_str("k");
        enc.write_obj_key_separator();
        enc.write_any(&PackValue::Integer(1));
        enc.write_end_obj();
        let out = enc.writer.flush();
        assert_eq!(&out, b"{\"k\":1}");
    }

    #[test]
    fn json_encoder_write_number() {
        use super::json::JsonEncoder;
        let mut enc = JsonEncoder::new();
        enc.writer.reset();
        enc.write_number(42.0);
        let out = enc.writer.flush();
        assert_eq!(&out, b"42");
        enc.writer.reset();
        enc.write_number(1.5);
        let out = enc.writer.flush();
        assert_eq!(&out, b"1.5");
    }

    // ================================================================
    // Coverage-fill: RESP encoder — extensions, errors, streaming, commands
    // ================================================================

    #[test]
    fn resp_encode_object() {
        use super::resp::RespEncoder;
        let mut enc = RespEncoder::new();
        let obj = PackValue::Object(vec![("key".into(), PackValue::Integer(1))]);
        let out = enc.encode(&obj);
        assert_eq!(out, b"%1\r\n+key\r\n:1\r\n");
    }

    #[test]
    fn resp_encode_float() {
        use super::resp::RespEncoder;
        let mut enc = RespEncoder::new();
        let out = enc.encode(&PackValue::Float(1.5));
        assert_eq!(out, b",1.5\r\n");
    }

    #[test]
    fn resp_encode_float_special() {
        use super::resp::RespEncoder;
        let mut enc = RespEncoder::new();
        assert_eq!(enc.encode(&PackValue::Float(f64::INFINITY)), b",inf\r\n");
        assert_eq!(
            enc.encode(&PackValue::Float(f64::NEG_INFINITY)),
            b",-inf\r\n"
        );
        assert_eq!(enc.encode(&PackValue::Float(f64::NAN)), b",nan\r\n");
    }

    #[test]
    fn resp_encode_big_int() {
        use super::resp::RespEncoder;
        let mut enc = RespEncoder::new();
        let out = enc.encode(&PackValue::BigInt(123456789));
        assert_eq!(out, b"(123456789\r\n");
    }

    #[test]
    fn resp_encode_undefined_as_null() {
        use super::resp::RespEncoder;
        let mut enc = RespEncoder::new();
        assert_eq!(enc.encode(&PackValue::Undefined), b"_\r\n");
    }

    #[test]
    fn resp_encode_long_string_as_verbatim() {
        use super::resp::RespEncoder;
        let mut enc = RespEncoder::new();
        let long = "a".repeat(100);
        let out = enc.encode(&PackValue::Str(long));
        // Long strings should use verbatim format (=)
        assert!(out.starts_with(b"="));
    }

    #[test]
    fn resp_encode_string_with_crlf_as_verbatim() {
        use super::resp::RespEncoder;
        let mut enc = RespEncoder::new();
        let out = enc.encode(&PackValue::Str("line1\r\nline2".into()));
        assert!(out.starts_with(b"="));
    }

    #[test]
    fn resp_encode_null_str_and_null_arr() {
        use super::resp::RespEncoder;
        let mut enc = RespEncoder::new();
        enc.write_null_str();
        let out = enc.writer.flush();
        assert_eq!(out, b"$-1\r\n");
        enc.write_null_arr();
        let out = enc.writer.flush();
        assert_eq!(out, b"*-1\r\n");
    }

    #[test]
    fn resp_encode_err_simple() {
        use super::resp::RespEncoder;
        let mut enc = RespEncoder::new();
        enc.write_simple_err("ERR bad");
        let out = enc.writer.flush();
        assert_eq!(out, b"-ERR bad\r\n");
    }

    #[test]
    fn resp_encode_err_bulk() {
        use super::resp::RespEncoder;
        let mut enc = RespEncoder::new();
        enc.write_bulk_err("ERR bulk error");
        let out = enc.writer.flush();
        assert_eq!(out, b"!14\r\nERR bulk error\r\n");
    }

    #[test]
    fn resp_encode_err_auto_selects_format() {
        use super::resp::RespEncoder;
        let mut enc = RespEncoder::new();
        // Short error -> simple
        enc.write_err("ERR short");
        let out = enc.writer.flush();
        assert_eq!(out, b"-ERR short\r\n");
        // Long error -> bulk
        let long_err = "E".repeat(100);
        enc.write_err(&long_err);
        let out = enc.writer.flush();
        assert!(out.starts_with(b"!"));
    }

    #[test]
    fn resp_encode_set() {
        use super::resp::RespEncoder;
        let mut enc = RespEncoder::new();
        enc.write_set(&[PackValue::Integer(1), PackValue::Integer(2)]);
        let out = enc.writer.flush();
        assert_eq!(out, b"~2\r\n:1\r\n:2\r\n");
    }

    #[test]
    fn resp_encode_push_extension() {
        use super::resp::RespEncoder;
        use crate::JsonPackExtension;
        let mut enc = RespEncoder::new();
        let ext = PackValue::Extension(Box::new(JsonPackExtension::new(
            1, // RESP_EXTENSION_PUSH
            PackValue::Array(vec![PackValue::Str("msg".into())]),
        )));
        let out = enc.encode(&ext);
        assert!(out.starts_with(b">"));
    }

    #[test]
    fn resp_encode_attr_extension() {
        use super::resp::RespEncoder;
        use crate::JsonPackExtension;
        let mut enc = RespEncoder::new();
        let ext = PackValue::Extension(Box::new(JsonPackExtension::new(
            2, // RESP_EXTENSION_ATTRIBUTES
            PackValue::Object(vec![("ttl".into(), PackValue::Integer(3600))]),
        )));
        let out = enc.encode(&ext);
        assert!(out.starts_with(b"|"));
    }

    #[test]
    fn resp_encode_verbatim_extension() {
        use super::resp::RespEncoder;
        use crate::JsonPackExtension;
        let mut enc = RespEncoder::new();
        let ext = PackValue::Extension(Box::new(JsonPackExtension::new(
            3, // RESP_EXTENSION_VERBATIM_STRING
            PackValue::Str("hello".into()),
        )));
        let out = enc.encode(&ext);
        assert!(out.starts_with(b"="));
    }

    #[test]
    fn resp_encode_extension_unknown_tag() {
        use super::resp::RespEncoder;
        use crate::JsonPackExtension;
        let mut enc = RespEncoder::new();
        let ext = PackValue::Extension(Box::new(JsonPackExtension::new(
            99, // unknown tag
            PackValue::Null,
        )));
        let out = enc.encode(&ext);
        assert_eq!(out, b"_\r\n");
    }

    #[test]
    fn resp_encode_extension_wrong_inner_type() {
        use super::resp::RespEncoder;
        use crate::JsonPackExtension;
        let mut enc = RespEncoder::new();
        // Push extension with non-array inner -> null
        let ext = PackValue::Extension(Box::new(JsonPackExtension::new(1, PackValue::Integer(42))));
        let out = enc.encode(&ext);
        assert_eq!(out, b"_\r\n");
    }

    #[test]
    fn resp_encode_cmd() {
        use super::resp::RespEncoder;
        let mut enc = RespEncoder::new();
        let out = enc.encode_cmd(&["SET", "key", "value"]);
        assert_eq!(out, b"*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n");
    }

    #[test]
    fn resp_encode_streaming_str() {
        use super::resp::RespEncoder;
        let mut enc = RespEncoder::new();
        enc.write_start_str();
        enc.write_str_chunk("hello");
        enc.write_str_chunk(" world");
        enc.write_end_str();
        let out = enc.writer.flush();
        assert_eq!(out, b"$?\r\n;5\r\nhello\r\n;6\r\n world\r\n;0\r\n");
    }

    #[test]
    fn resp_encode_streaming_arr() {
        use super::resp::RespEncoder;
        let mut enc = RespEncoder::new();
        enc.write_start_arr();
        enc.write_arr_chunk(&PackValue::Integer(1));
        enc.write_arr_chunk(&PackValue::Integer(2));
        enc.write_end_arr();
        let out = enc.writer.flush();
        assert_eq!(out, b"*?\r\n:1\r\n:2\r\n.\r\n");
    }

    #[test]
    fn resp_encode_streaming_obj() {
        use super::resp::RespEncoder;
        let mut enc = RespEncoder::new();
        enc.write_start_obj();
        enc.write_obj_chunk("key", &PackValue::Integer(1));
        enc.write_end_obj();
        let out = enc.writer.flush();
        assert_eq!(out, b"%?\r\n+key\r\n:1\r\n.\r\n");
    }

    #[test]
    fn resp_encode_streaming_bin() {
        use super::resp::RespEncoder;
        let mut enc = RespEncoder::new();
        enc.write_start_bin();
        enc.write_bin_chunk(b"data");
        enc.write_end_bin();
        let out = enc.writer.flush();
        assert_eq!(out, b"$?\r\n;4\r\ndata\r\n;0\r\n");
    }

    #[test]
    fn resp_encode_write_length_multi_digit() {
        use super::resp::RespEncoder;
        let mut enc = RespEncoder::new();
        // Two-digit length (10-99)
        enc.write_arr_hdr(42);
        let out = enc.writer.flush();
        assert_eq!(out, b"*42\r\n");
        // Three-digit length (100+)
        enc.write_arr_hdr(123);
        let out = enc.writer.flush();
        assert_eq!(out, b"*123\r\n");
    }

    #[test]
    fn resp_encode_ascii_str_with_crlf() {
        use super::resp::RespEncoder;
        let mut enc = RespEncoder::new();
        enc.write_ascii_str("no newlines");
        let out = enc.writer.flush();
        assert_eq!(out, b"+no newlines\r\n");
        // With CRLF -> bulk string
        enc.write_ascii_str("line1\r\nline2");
        let out = enc.writer.flush();
        assert_eq!(out, b"$12\r\nline1\r\nline2\r\n");
    }

    // ================================================================
    // Coverage-fill: RESP legacy encoder
    // ================================================================

    #[test]
    fn resp_legacy_encode_null() {
        use super::resp::RespEncoderLegacy;
        let mut enc = RespEncoderLegacy::new();
        let out = enc.encode(&PackValue::Null);
        assert_eq!(out, b"*-1\r\n");
    }

    #[test]
    fn resp_legacy_encode_bool() {
        use super::resp::RespEncoderLegacy;
        let mut enc = RespEncoderLegacy::new();
        assert_eq!(enc.encode(&PackValue::Bool(true)), b"+TRUE\r\n");
        assert_eq!(enc.encode(&PackValue::Bool(false)), b"+FALSE\r\n");
    }

    #[test]
    fn resp_legacy_encode_integer() {
        use super::resp::RespEncoderLegacy;
        let mut enc = RespEncoderLegacy::new();
        assert_eq!(enc.encode(&PackValue::Integer(42)), b":42\r\n");
    }

    #[test]
    fn resp_legacy_encode_float() {
        use super::resp::RespEncoderLegacy;
        let mut enc = RespEncoderLegacy::new();
        // Non-integer float -> simple string
        let out = enc.encode(&PackValue::Float(1.5));
        assert_eq!(out, b"+1.5\r\n");
        // Integer float -> integer encoding
        let out = enc.encode(&PackValue::Float(42.0));
        assert_eq!(out, b":42\r\n");
    }

    #[test]
    fn resp_legacy_encode_string() {
        use super::resp::RespEncoderLegacy;
        let mut enc = RespEncoderLegacy::new();
        let out = enc.encode(&PackValue::Str("hello".into()));
        assert_eq!(out, b"+hello\r\n");
        // Long string -> bulk
        let long = "x".repeat(100);
        let out = enc.encode(&PackValue::Str(long));
        assert!(out.starts_with(b"$100\r\n"));
    }

    #[test]
    fn resp_legacy_encode_binary() {
        use super::resp::RespEncoderLegacy;
        let mut enc = RespEncoderLegacy::new();
        let out = enc.encode(&PackValue::Bytes(b"bin".to_vec()));
        assert_eq!(out, b"$3\r\nbin\r\n");
    }

    #[test]
    fn resp_legacy_encode_array_with_null() {
        use super::resp::RespEncoderLegacy;
        let mut enc = RespEncoderLegacy::new();
        let arr = PackValue::Array(vec![PackValue::Integer(1), PackValue::Null]);
        let out = enc.encode(&arr);
        assert_eq!(out, b"*2\r\n:1\r\n$-1\r\n");
    }

    #[test]
    fn resp_legacy_encode_object() {
        use super::resp::RespEncoderLegacy;
        let mut enc = RespEncoderLegacy::new();
        let obj = PackValue::Object(vec![("key".into(), PackValue::Integer(1))]);
        let out = enc.encode(&obj);
        // Object as flat array of key-value pairs
        assert_eq!(out, b"*2\r\n+key\r\n:1\r\n");
    }

    #[test]
    fn resp_legacy_encode_object_with_null_value() {
        use super::resp::RespEncoderLegacy;
        let mut enc = RespEncoderLegacy::new();
        let obj = PackValue::Object(vec![("key".into(), PackValue::Null)]);
        let out = enc.encode(&obj);
        assert_eq!(out, b"*2\r\n+key\r\n$-1\r\n");
    }

    #[test]
    fn resp_legacy_encode_big_int() {
        use super::resp::RespEncoderLegacy;
        let mut enc = RespEncoderLegacy::new();
        let out = enc.encode(&PackValue::BigInt(42));
        assert_eq!(out, b"+42\r\n");
    }

    #[test]
    fn resp_legacy_encode_u_integer() {
        use super::resp::RespEncoderLegacy;
        let mut enc = RespEncoderLegacy::new();
        let out = enc.encode(&PackValue::UInteger(42));
        assert_eq!(out, b":42\r\n");
        // Very large u64 that overflows i64
        let out = enc.encode(&PackValue::UInteger(u64::MAX));
        let s = std::str::from_utf8(&out).unwrap();
        assert!(s.starts_with('+'));
    }

    #[test]
    fn resp_legacy_encode_undefined() {
        use super::resp::RespEncoderLegacy;
        let mut enc = RespEncoderLegacy::new();
        assert_eq!(enc.encode(&PackValue::Undefined), b"*-1\r\n");
    }

    #[test]
    fn resp_legacy_encode_push_extension() {
        use super::resp::RespEncoderLegacy;
        use crate::JsonPackExtension;
        let mut enc = RespEncoderLegacy::new();
        let ext = PackValue::Extension(Box::new(JsonPackExtension::new(
            1, // RESP_EXTENSION_PUSH
            PackValue::Array(vec![PackValue::Integer(1)]),
        )));
        let out = enc.encode(&ext);
        assert_eq!(out, b"*1\r\n:1\r\n");
    }

    #[test]
    fn resp_legacy_encode_verbatim_extension() {
        use super::resp::RespEncoderLegacy;
        use crate::JsonPackExtension;
        let mut enc = RespEncoderLegacy::new();
        let ext = PackValue::Extension(Box::new(JsonPackExtension::new(
            3, // RESP_EXTENSION_VERBATIM_STRING
            PackValue::Str("msg".into()),
        )));
        let out = enc.encode(&ext);
        assert_eq!(out, b"+msg\r\n");
    }

    #[test]
    fn resp_legacy_encode_attr_extension() {
        use super::resp::RespEncoderLegacy;
        use crate::JsonPackExtension;
        let mut enc = RespEncoderLegacy::new();
        let ext = PackValue::Extension(Box::new(JsonPackExtension::new(
            2, // RESP_EXTENSION_ATTRIBUTES
            PackValue::Object(vec![("k".into(), PackValue::Integer(1))]),
        )));
        let out = enc.encode(&ext);
        assert_eq!(out, b"*2\r\n+k\r\n:1\r\n");
    }

    #[test]
    fn resp_legacy_encode_unknown_extension() {
        use super::resp::RespEncoderLegacy;
        use crate::JsonPackExtension;
        let mut enc = RespEncoderLegacy::new();
        let ext = PackValue::Extension(Box::new(JsonPackExtension::new(99, PackValue::Null)));
        let out = enc.encode(&ext);
        assert_eq!(out, b"*-1\r\n");
    }

    #[test]
    fn resp_legacy_encode_err() {
        use super::resp::RespEncoderLegacy;
        let mut enc = RespEncoderLegacy::new();
        enc.write_err("ERR short");
        let out = enc.flush();
        assert_eq!(out, b"-ERR short\r\n");
        // Long error with CRLF -> bulk str
        enc.write_err("line1\r\nline2");
        let out = enc.flush();
        assert_eq!(out, b"$12\r\nline1\r\nline2\r\n");
    }

    // ================================================================
    // Coverage-fill: Avro encoder — write_any for all PackValue types
    // ================================================================

    #[test]
    fn avro_encoder_write_any_primitives() {
        use super::avro::{AvroDecoder, AvroEncoder};
        let mut enc = AvroEncoder::new();
        let mut dec = AvroDecoder::new();
        // Null produces zero bytes
        enc.write_any(&PackValue::Null);
        assert!(enc.writer.flush().is_empty());
        // Bool
        enc.write_any(&PackValue::Bool(true));
        let bytes = enc.writer.flush();
        dec.reset(&bytes);
        assert!(dec.read_boolean().unwrap());
        // Integer (i32 range)
        enc.write_any(&PackValue::Integer(42));
        let bytes = enc.writer.flush();
        dec.reset(&bytes);
        assert_eq!(dec.read_int().unwrap(), 42);
        // Integer (outside i32 range -> long)
        enc.write_any(&PackValue::Integer(i64::MAX));
        let bytes = enc.writer.flush();
        dec.reset(&bytes);
        assert_eq!(dec.read_long().unwrap(), i64::MAX);
    }

    #[test]
    fn avro_encoder_write_any_uinteger() {
        use super::avro::{AvroDecoder, AvroEncoder};
        let mut enc = AvroEncoder::new();
        let mut dec = AvroDecoder::new();
        // UInteger in i32 range -> int
        enc.write_any(&PackValue::UInteger(100));
        let bytes = enc.writer.flush();
        dec.reset(&bytes);
        assert_eq!(dec.read_int().unwrap(), 100);
        // UInteger outside i32 range -> long
        enc.write_any(&PackValue::UInteger(u64::MAX / 2));
        let bytes = enc.writer.flush();
        dec.reset(&bytes);
        let val = dec.read_long().unwrap();
        assert_eq!(val, (u64::MAX / 2) as i64);
    }

    #[test]
    fn avro_encoder_write_any_float() {
        use super::avro::{AvroDecoder, AvroEncoder};
        let mut enc = AvroEncoder::new();
        let mut dec = AvroDecoder::new();
        enc.write_any(&PackValue::Float(TEST_F64_3_14));
        let bytes = enc.writer.flush();
        dec.reset(&bytes);
        let v = dec.read_double().unwrap();
        assert!((v - TEST_F64_3_14).abs() < 1e-10);
    }

    #[test]
    fn avro_encoder_write_any_string_and_bytes() {
        use super::avro::{AvroDecoder, AvroEncoder};
        let mut enc = AvroEncoder::new();
        let mut dec = AvroDecoder::new();
        enc.write_any(&PackValue::Str("test".into()));
        let bytes = enc.writer.flush();
        dec.reset(&bytes);
        assert_eq!(dec.read_str().unwrap(), "test");

        enc.write_any(&PackValue::Bytes(vec![0xDE, 0xAD]));
        let bytes = enc.writer.flush();
        dec.reset(&bytes);
        assert_eq!(dec.read_bytes().unwrap(), vec![0xDE, 0xAD]);
    }

    #[test]
    fn avro_encoder_write_any_array_and_object() {
        use super::avro::AvroEncoder;
        let mut enc = AvroEncoder::new();
        // Array: varint(count) + items + varint(0)
        enc.write_any(&PackValue::Array(vec![
            PackValue::Integer(1),
            PackValue::Integer(2),
        ]));
        let bytes = enc.writer.flush();
        assert!(!bytes.is_empty());
        // Last byte should be 0 (array end marker)
        assert_eq!(bytes[bytes.len() - 1], 0);

        // Object
        enc.write_any(&PackValue::Object(vec![(
            "k".into(),
            PackValue::Integer(1),
        )]));
        let bytes = enc.writer.flush();
        assert!(!bytes.is_empty());
        assert_eq!(bytes[bytes.len() - 1], 0);
    }

    #[test]
    fn avro_encoder_write_any_big_int() {
        use super::avro::{AvroDecoder, AvroEncoder};
        let mut enc = AvroEncoder::new();
        let mut dec = AvroDecoder::new();
        enc.write_any(&PackValue::BigInt(42));
        let bytes = enc.writer.flush();
        dec.reset(&bytes);
        assert_eq!(dec.read_long().unwrap(), 42);
    }

    #[test]
    fn avro_encoder_encode_helpers() {
        use super::avro::AvroEncoder;
        let mut enc = AvroEncoder::new();
        // encode_null
        assert!(enc.encode_null().is_empty());
        // encode_boolean
        assert_eq!(enc.encode_boolean(true), vec![1]);
        assert_eq!(enc.encode_boolean(false), vec![0]);
        // encode_int
        let bytes = enc.encode_int(42);
        assert!(!bytes.is_empty());
        // encode_long
        let bytes = enc.encode_long(-1);
        assert!(!bytes.is_empty());
        // encode_float
        let bytes = enc.encode_float(1.5);
        assert_eq!(bytes.len(), 4);
        // encode_double
        let bytes = enc.encode_double(1.5);
        assert_eq!(bytes.len(), 8);
        // encode_bytes
        let bytes = enc.encode_bytes(&[1, 2, 3]);
        assert!(!bytes.is_empty());
        // encode_str
        let bytes = enc.encode_str("hello");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn avro_encoder_float_roundtrip() {
        use super::avro::{AvroDecoder, AvroEncoder};
        let mut enc = AvroEncoder::new();
        let mut dec = AvroDecoder::new();
        let val: f32 = 1.5;
        enc.write_float(val);
        let bytes = enc.writer.flush();
        // Float is 4 bytes LE
        assert_eq!(bytes.len(), 4);
        dec.reset(&bytes);
        let decoded = dec.read_float().unwrap();
        assert!((decoded - 1.5).abs() < 1e-6);
    }

    // ================================================================
    // Coverage-fill: Avro schema encoder
    // ================================================================

    #[test]
    fn avro_schema_encoder_primitives() {
        use super::avro::schema_encoder::AvroSchemaEncoder;
        use super::avro::types::{AvroSchema, AvroValue};
        let mut enc = AvroSchemaEncoder::new();
        // Null
        let bytes = enc.encode(&AvroValue::Null, &AvroSchema::Null).unwrap();
        assert!(bytes.is_empty());
        // Boolean
        let bytes = enc
            .encode(&AvroValue::Bool(true), &AvroSchema::Boolean)
            .unwrap();
        assert_eq!(bytes, vec![1]);
        // Int
        let bytes = enc.encode(&AvroValue::Int(42), &AvroSchema::Int).unwrap();
        assert!(!bytes.is_empty());
        // Long
        let bytes = enc
            .encode(&AvroValue::Long(100), &AvroSchema::Long)
            .unwrap();
        assert!(!bytes.is_empty());
        // Float
        let bytes = enc
            .encode(&AvroValue::Float(1.5), &AvroSchema::Float)
            .unwrap();
        assert_eq!(bytes.len(), 4);
        // Double
        let bytes = enc
            .encode(&AvroValue::Double(1.5), &AvroSchema::Double)
            .unwrap();
        assert_eq!(bytes.len(), 8);
        // Bytes
        let bytes = enc
            .encode(&AvroValue::Bytes(vec![1, 2]), &AvroSchema::Bytes)
            .unwrap();
        assert!(!bytes.is_empty());
        // String
        let bytes = enc
            .encode(&AvroValue::Str("hi".into()), &AvroSchema::String)
            .unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn avro_schema_encoder_record() {
        use super::avro::schema_encoder::AvroSchemaEncoder;
        use super::avro::types::{AvroField, AvroSchema, AvroValue};
        let mut enc = AvroSchemaEncoder::new();
        let schema = AvroSchema::Record {
            name: "Person".into(),
            namespace: None,
            fields: vec![
                AvroField {
                    name: "name".into(),
                    type_: AvroSchema::String,
                    default: None,
                    doc: None,
                    aliases: vec![],
                },
                AvroField {
                    name: "age".into(),
                    type_: AvroSchema::Int,
                    default: None,
                    doc: None,
                    aliases: vec![],
                },
            ],
            aliases: vec![],
            doc: None,
        };
        let value = AvroValue::Record(vec![
            ("name".into(), AvroValue::Str("Alice".into())),
            ("age".into(), AvroValue::Int(30)),
        ]);
        let bytes = enc.encode(&value, &schema).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn avro_schema_encoder_enum() {
        use super::avro::schema_encoder::AvroSchemaEncoder;
        use super::avro::types::{AvroSchema, AvroValue};
        let mut enc = AvroSchemaEncoder::new();
        let schema = AvroSchema::Enum {
            name: "Color".into(),
            namespace: None,
            symbols: vec!["RED".into(), "GREEN".into(), "BLUE".into()],
            default: None,
            aliases: vec![],
        };
        let bytes = enc
            .encode(&AvroValue::Enum("GREEN".into()), &schema)
            .unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn avro_schema_encoder_array() {
        use super::avro::schema_encoder::AvroSchemaEncoder;
        use super::avro::types::{AvroSchema, AvroValue};
        let mut enc = AvroSchemaEncoder::new();
        let schema = AvroSchema::Array {
            items: Box::new(AvroSchema::Int),
        };
        let value = AvroValue::Array(vec![AvroValue::Int(1), AvroValue::Int(2)]);
        let bytes = enc.encode(&value, &schema).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn avro_schema_encoder_map() {
        use super::avro::schema_encoder::AvroSchemaEncoder;
        use super::avro::types::{AvroSchema, AvroValue};
        let mut enc = AvroSchemaEncoder::new();
        let schema = AvroSchema::Map {
            values: Box::new(AvroSchema::Int),
        };
        let value = AvroValue::Map(vec![("key".into(), AvroValue::Int(42))]);
        let bytes = enc.encode(&value, &schema).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn avro_schema_encoder_fixed() {
        use super::avro::schema_encoder::AvroSchemaEncoder;
        use super::avro::types::{AvroSchema, AvroValue};
        let mut enc = AvroSchemaEncoder::new();
        let schema = AvroSchema::Fixed {
            name: "Hash".into(),
            namespace: None,
            size: 4,
            aliases: vec![],
        };
        let bytes = enc
            .encode(&AvroValue::Fixed(vec![1, 2, 3, 4]), &schema)
            .unwrap();
        assert_eq!(bytes, vec![1, 2, 3, 4]);
    }

    #[test]
    fn avro_schema_encoder_fixed_size_mismatch() {
        use super::avro::schema_encoder::AvroSchemaEncoder;
        use super::avro::types::{AvroSchema, AvroValue};
        let mut enc = AvroSchemaEncoder::new();
        let schema = AvroSchema::Fixed {
            name: "Hash".into(),
            namespace: None,
            size: 4,
            aliases: vec![],
        };
        let result = enc.encode(&AvroValue::Fixed(vec![1, 2]), &schema);
        // Validator catches the mismatch before the encoder reaches FixedSizeMismatch
        assert!(result.is_err());
    }

    #[test]
    fn avro_schema_encoder_union() {
        use super::avro::schema_encoder::AvroSchemaEncoder;
        use super::avro::types::{AvroSchema, AvroValue};
        let mut enc = AvroSchemaEncoder::new();
        let schema = AvroSchema::Union(vec![AvroSchema::Null, AvroSchema::Int]);
        // Explicit union index
        let bytes = enc
            .encode(
                &AvroValue::Union {
                    index: 1,
                    value: Box::new(AvroValue::Int(42)),
                },
                &schema,
            )
            .unwrap();
        assert!(!bytes.is_empty());
        // Auto-matched union
        let bytes = enc.encode(&AvroValue::Null, &schema).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn avro_schema_encoder_type_promotions() {
        use super::avro::schema_encoder::AvroSchemaEncoder;
        use super::avro::types::{AvroSchema, AvroValue};
        let mut enc = AvroSchemaEncoder::new();
        // Int schema, Long value (fits in i32)
        let bytes = enc.encode(&AvroValue::Long(42), &AvroSchema::Int).unwrap();
        assert!(!bytes.is_empty());
        // Long schema, Int value
        let bytes = enc.encode(&AvroValue::Int(42), &AvroSchema::Long).unwrap();
        assert!(!bytes.is_empty());
        // Float schema, Int value
        let bytes = enc.encode(&AvroValue::Int(42), &AvroSchema::Float).unwrap();
        assert_eq!(bytes.len(), 4);
        // Float schema, Long value
        let bytes = enc
            .encode(&AvroValue::Long(42), &AvroSchema::Float)
            .unwrap();
        assert_eq!(bytes.len(), 4);
        // Float schema, Double value
        let bytes = enc
            .encode(&AvroValue::Double(1.5), &AvroSchema::Float)
            .unwrap();
        assert_eq!(bytes.len(), 4);
        // Double schema, Float value
        let bytes = enc
            .encode(&AvroValue::Float(1.5), &AvroSchema::Double)
            .unwrap();
        assert_eq!(bytes.len(), 8);
        // Double schema, Int value
        let bytes = enc
            .encode(&AvroValue::Int(42), &AvroSchema::Double)
            .unwrap();
        assert_eq!(bytes.len(), 8);
        // Double schema, Long value
        let bytes = enc
            .encode(&AvroValue::Long(42), &AvroSchema::Double)
            .unwrap();
        assert_eq!(bytes.len(), 8);
    }

    #[test]
    fn avro_schema_encoder_type_mismatch() {
        use super::avro::schema_encoder::{AvroEncodeError, AvroSchemaEncoder};
        use super::avro::types::{AvroSchema, AvroValue};
        let mut enc = AvroSchemaEncoder::new();
        let result = enc.encode(&AvroValue::Str("x".into()), &AvroSchema::Int);
        assert!(matches!(result, Err(AvroEncodeError::ValueDoesNotConform)));
    }

    #[test]
    fn avro_schema_encoder_enum_symbol_not_found() {
        use super::avro::schema_encoder::AvroSchemaEncoder;
        use super::avro::types::{AvroSchema, AvroValue};
        let mut enc = AvroSchemaEncoder::new();
        let schema = AvroSchema::Enum {
            name: "Color".into(),
            namespace: None,
            symbols: vec!["RED".into()],
            default: None,
            aliases: vec![],
        };
        let result = enc.encode(&AvroValue::Enum("PURPLE".into()), &schema);
        // Validator catches the invalid symbol before the encoder
        assert!(result.is_err());
    }

    #[test]
    fn avro_schema_encoder_record_with_default() {
        use super::avro::schema_encoder::AvroSchemaEncoder;
        use super::avro::types::{AvroField, AvroSchema, AvroValue};
        let mut enc = AvroSchemaEncoder::new();
        let schema = AvroSchema::Record {
            name: "Test".into(),
            namespace: None,
            fields: vec![AvroField {
                name: "x".into(),
                type_: AvroSchema::Int,
                default: Some(AvroValue::Int(99)),
                doc: None,
                aliases: vec![],
            }],
            aliases: vec![],
            doc: None,
        };
        // Empty record -> default applies
        let bytes = enc.encode(&AvroValue::Record(vec![]), &schema).unwrap();
        assert!(!bytes.is_empty());
    }

    // ================================================================
    // Coverage-fill: Bencode encoder — additional branches
    // ================================================================

    #[test]
    fn bencode_encode_undefined() {
        let mut enc = BencodeEncoder::new();
        let out = enc.encode(&PackValue::Undefined);
        assert_eq!(&out, b"u");
    }

    #[test]
    fn bencode_encode_float_rounds() {
        let mut enc = BencodeEncoder::new();
        let out = enc.encode(&PackValue::Float(3.7));
        assert_eq!(&out, b"i4e"); // rounds to 4
    }

    #[test]
    fn bencode_encode_u_integer() {
        let mut enc = BencodeEncoder::new();
        let out = enc.encode(&PackValue::UInteger(999));
        assert_eq!(&out, b"i999e");
    }

    #[test]
    fn bencode_encode_big_int() {
        let mut enc = BencodeEncoder::new();
        let out = enc.encode(&PackValue::BigInt(123456789012345));
        assert_eq!(&out, b"i123456789012345e");
    }

    #[test]
    fn bencode_encode_binary() {
        let mut enc = BencodeEncoder::new();
        let out = enc.encode(&PackValue::Bytes(vec![0xDE, 0xAD]));
        assert_eq!(&out[0..2], b"2:");
        assert_eq!(&out[2..], &[0xDE, 0xAD]);
    }

    #[test]
    fn bencode_encode_list() {
        let mut enc = BencodeEncoder::new();
        let arr = PackValue::Array(vec![PackValue::Integer(1), PackValue::Integer(2)]);
        let out = enc.encode(&arr);
        assert_eq!(&out, b"li1ei2ee");
    }

    #[test]
    fn bencode_encode_nested_structures() {
        let mut enc = BencodeEncoder::new();
        let val = PackValue::Object(vec![(
            "list".into(),
            PackValue::Array(vec![PackValue::Str("x".into())]),
        )]);
        let out = enc.encode(&val);
        assert_eq!(&out, b"d4:listl1:xee");
    }

    #[test]
    fn bencode_encode_extension_as_null() {
        use crate::JsonPackExtension;
        let mut enc = BencodeEncoder::new();
        let ext = PackValue::Extension(Box::new(JsonPackExtension::new(1, PackValue::Null)));
        assert_eq!(enc.encode(&ext), b"n");
    }

    #[test]
    fn bencode_encode_json() {
        let mut enc = BencodeEncoder::new();
        let val = serde_json::json!({"a": [1, 2], "b": "hello"});
        let out = enc.encode_json(&val);
        // Dict keys should be sorted ("a" before "b")
        let s = std::str::from_utf8(&out).unwrap();
        assert!(s.starts_with('d'));
        assert!(s.ends_with('e'));
        assert!(s.contains("1:a"));
        assert!(s.contains("1:b"));
    }

    #[test]
    fn bencode_encode_json_null_and_bool() {
        let mut enc = BencodeEncoder::new();
        assert_eq!(&enc.encode_json(&serde_json::json!(null)), b"n");
        assert_eq!(&enc.encode_json(&serde_json::json!(true)), b"t");
        assert_eq!(&enc.encode_json(&serde_json::json!(false)), b"f");
    }

    #[test]
    fn bencode_decode_integer() {
        let dec = BencodeDecoder::new();
        assert_eq!(dec.decode(b"i42e").unwrap(), PackValue::Integer(42));
        assert_eq!(dec.decode(b"i-7e").unwrap(), PackValue::Integer(-7));
        assert_eq!(dec.decode(b"i0e").unwrap(), PackValue::Integer(0));
    }

    #[test]
    fn bencode_decode_list() {
        let dec = BencodeDecoder::new();
        let result = dec.decode(b"li1ei2ee").unwrap();
        assert_eq!(
            result,
            PackValue::Array(vec![PackValue::Integer(1), PackValue::Integer(2)])
        );
    }

    #[test]
    fn bencode_decode_dict() {
        let dec = BencodeDecoder::new();
        let result = dec.decode(b"d1:ai1e1:bi2ee").unwrap();
        if let PackValue::Object(pairs) = result {
            assert_eq!(pairs.len(), 2);
        } else {
            panic!("expected Object");
        }
    }

    #[test]
    fn bencode_decode_nested() {
        let dec = BencodeDecoder::new();
        let result = dec.decode(b"d4:listli1eee").unwrap();
        if let PackValue::Object(pairs) = result {
            assert_eq!(pairs[0].0, "list");
            assert!(matches!(pairs[0].1, PackValue::Array(_)));
        } else {
            panic!("expected Object");
        }
    }

    #[test]
    fn bencode_roundtrip_complex() {
        let mut enc = BencodeEncoder::new();
        let dec = BencodeDecoder::new();
        let val = PackValue::Object(vec![
            ("a".into(), PackValue::Integer(1)),
            ("b".into(), PackValue::Str("hello".into())),
        ]);
        let encoded = enc.encode(&val);
        let decoded = dec.decode(&encoded).unwrap();
        if let PackValue::Object(pairs) = decoded {
            assert_eq!(pairs.len(), 2);
            // Bencode sorts keys
            assert_eq!(pairs[0].0, "a");
            assert_eq!(pairs[0].1, PackValue::Integer(1));
            assert_eq!(pairs[1].0, "b");
            // Strings come back as Bytes in bencode
            assert!(matches!(&pairs[1].1, PackValue::Bytes(b) if b == b"hello"));
        } else {
            panic!("expected Object");
        }
    }

    // ================================================================
    // Coverage-fill: RM encoder — start_record/end_record, fragment
    // ================================================================

    #[test]
    fn rm_encoder_start_end_record() {
        use super::rm::RmRecordEncoder;
        let mut enc = RmRecordEncoder::new();
        let pos = enc.start_record();
        enc.writer.buf(b"payload");
        enc.end_record(pos);
        let out = enc.writer.flush();
        // Should have a 4-byte header + "payload" (7 bytes)
        assert_eq!(out.len(), 4 + 7);
        // fin bit should be set
        assert_eq!(out[0] & 0x80, 0x80);
        let len = u32::from_be_bytes([out[0] & 0x7f, out[1], out[2], out[3]]);
        assert_eq!(len, 7);
        assert_eq!(&out[4..], b"payload");
    }

    #[test]
    fn rm_encoder_write_fragment() {
        use super::rm::RmRecordEncoder;
        let mut enc = RmRecordEncoder::new();
        let record = b"hello world";
        enc.write_fragment(record, 0, 5, false);
        enc.write_fragment(record, 5, 6, true);
        let out = enc.writer.flush();
        // Two headers (4 bytes each) + 11 bytes data
        assert_eq!(out.len(), 8 + 11);
        // First fragment: fin=0
        assert_eq!(out[0] & 0x80, 0x00);
        // Second fragment: fin=1
        assert_eq!(out[4 + 5] & 0x80, 0x80);
    }

    #[test]
    fn rm_encoder_empty_record() {
        use super::rm::RmRecordEncoder;
        let mut enc = RmRecordEncoder::new();
        let out = enc.encode_record(b"");
        assert_eq!(out.len(), 4);
        let val = u32::from_be_bytes([out[0], out[1], out[2], out[3]]);
        assert_eq!(val, 0x8000_0000); // fin=1, length=0
    }

    #[test]
    fn rm_decoder_multiple_records() {
        use super::rm::{RmRecordDecoder, RmRecordEncoder};
        let mut enc = RmRecordEncoder::new();
        let mut dec = RmRecordDecoder::new();
        let frame1 = enc.encode_record(b"one");
        let frame2 = enc.encode_record(b"two");
        dec.push(&frame1);
        dec.push(&frame2);
        assert_eq!(dec.read_record().unwrap(), b"one");
        assert_eq!(dec.read_record().unwrap(), b"two");
    }
}
