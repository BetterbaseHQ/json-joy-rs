use json_joy_json_pack::ssh::{SshDecoder, SshEncoder, SshError};
use json_joy_json_pack::{JsonPackMpint, PackValue};

#[test]
fn ssh_encoder_primitive_matrix() {
    let mut encoder = SshEncoder::new();

    encoder.writer.reset();
    encoder.write_boolean(true);
    assert_eq!(encoder.writer.flush(), vec![1]);

    encoder.writer.reset();
    encoder.write_boolean(false);
    assert_eq!(encoder.writer.flush(), vec![0]);

    encoder.writer.reset();
    encoder.write_byte(0x42);
    assert_eq!(encoder.writer.flush(), vec![0x42]);

    encoder.writer.reset();
    encoder.write_uint32(0x1234_5678);
    assert_eq!(encoder.writer.flush(), vec![0x12, 0x34, 0x56, 0x78]);

    encoder.writer.reset();
    encoder.write_uint64(0x1234_5678_9abc_def0);
    assert_eq!(
        encoder.writer.flush(),
        vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0]
    );
}

#[test]
fn ssh_encoder_decoder_string_and_mpint_matrix() {
    let mut encoder = SshEncoder::new();
    let mut decoder = SshDecoder::new();

    encoder.writer.reset();
    encoder.write_str("testing");
    decoder.reset(&encoder.writer.flush());
    assert_eq!(decoder.read_str().unwrap(), "testing");

    encoder.writer.reset();
    encoder.write_ascii_str("test");
    decoder.reset(&encoder.writer.flush());
    assert_eq!(decoder.read_ascii_str().unwrap(), "test");

    encoder.writer.reset();
    encoder.write_bin_str(&[1, 2, 3]);
    decoder.reset(&encoder.writer.flush());
    assert_eq!(decoder.read_bin_str().unwrap(), vec![1, 2, 3]);

    encoder.writer.reset();
    encoder.write_name_list(&[PackValue::Str("zlib".into()), PackValue::Str("none".into())]);
    decoder.reset(&encoder.writer.flush());
    assert_eq!(
        decoder.read_name_list().unwrap(),
        vec!["zlib".to_owned(), "none".to_owned()]
    );

    encoder.writer.reset();
    let mpint = JsonPackMpint::from_i128(-1234);
    encoder.write_mpint(&mpint);
    decoder.reset(&encoder.writer.flush());
    assert_eq!(decoder.read_mpint().unwrap().to_i128(), -1234);
}

#[test]
fn ssh_codec_roundtrip_matrix() {
    let mut encoder = SshEncoder::new();
    let mut decoder = SshDecoder::new();

    encoder.writer.reset();
    encoder.write_byte(20);
    encoder.write_bin_str(&[0x42; 16]);
    encoder.write_name_list(&[PackValue::Str("diffie-hellman-group14-sha1".into())]);
    encoder.write_name_list(&[PackValue::Str("ssh-rsa".into())]);
    encoder.write_name_list(&[PackValue::Str("aes128-ctr".into())]);
    encoder.write_name_list(&[PackValue::Str("aes128-ctr".into())]);
    encoder.write_name_list(&[PackValue::Str("hmac-sha1".into())]);
    encoder.write_name_list(&[PackValue::Str("hmac-sha1".into())]);
    encoder.write_name_list(&[PackValue::Str("none".into())]);
    encoder.write_name_list(&[PackValue::Str("none".into())]);
    encoder.write_name_list(&[]);
    encoder.write_name_list(&[]);
    encoder.write_boolean(false);
    encoder.write_uint32(0);

    let packet = encoder.writer.flush();
    decoder.reset(&packet);

    assert_eq!(decoder.read_byte().unwrap(), 20);
    assert_eq!(decoder.read_bin_str().unwrap(), vec![0x42; 16]);
    assert_eq!(
        decoder.read_name_list().unwrap(),
        vec!["diffie-hellman-group14-sha1".to_owned()]
    );
    assert_eq!(
        decoder.read_name_list().unwrap(),
        vec!["ssh-rsa".to_owned()]
    );
    assert_eq!(
        decoder.read_name_list().unwrap(),
        vec!["aes128-ctr".to_owned()]
    );
    assert_eq!(
        decoder.read_name_list().unwrap(),
        vec!["aes128-ctr".to_owned()]
    );
    assert_eq!(
        decoder.read_name_list().unwrap(),
        vec!["hmac-sha1".to_owned()]
    );
    assert_eq!(
        decoder.read_name_list().unwrap(),
        vec!["hmac-sha1".to_owned()]
    );
    assert_eq!(decoder.read_name_list().unwrap(), vec!["none".to_owned()]);
    assert_eq!(decoder.read_name_list().unwrap(), vec!["none".to_owned()]);
    assert!(decoder.read_name_list().unwrap().is_empty());
    assert!(decoder.read_name_list().unwrap().is_empty());
    assert!(!decoder.read_boolean().unwrap());
    assert_eq!(decoder.read_uint32().unwrap(), 0);
}

#[test]
fn ssh_error_and_write_any_matrix() {
    let mut encoder = SshEncoder::new();
    let mut decoder = SshDecoder::new();

    let encoded_bool = encoder.encode(&PackValue::Bool(true));
    decoder.reset(&encoded_bool);
    assert!(decoder.read_boolean().unwrap());

    let encoded_int = encoder.encode(&PackValue::Integer(42));
    decoder.reset(&encoded_int);
    assert_eq!(decoder.read_uint32().unwrap(), 42);

    decoder.reset(&[0, 0, 0]);
    assert!(matches!(
        decoder.read_uint32(),
        Err(SshError::UnexpectedEof)
    ));

    decoder.reset(&[0, 0, 0, 2, 0xff, 0xff]);
    assert!(matches!(decoder.read_str(), Err(SshError::InvalidUtf8)));
}
