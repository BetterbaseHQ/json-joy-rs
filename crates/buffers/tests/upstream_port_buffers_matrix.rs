//! Upstream: json-joy/packages/buffers/src/
//!
//! Writer/Reader roundtrip matrix and f16 edge-case tests for the buffers crate.

use json_joy_buffers::{
    cmp_uint8_array, cmp_uint8_array2, cmp_uint8_array3, concat, concat_list, decode_f16,
    is_float32, Reader, Writer,
};

// ---------------------------------------------------------------------------
// Writer/Reader roundtrip matrix
// ---------------------------------------------------------------------------

#[test]
fn roundtrip_u8() {
    let mut w = Writer::new();
    w.u8(0x00);
    w.u8(0x7F);
    w.u8(0xFF);
    let data = w.flush();
    let mut r = Reader::new(&data);
    assert_eq!(r.u8(), 0x00);
    assert_eq!(r.u8(), 0x7F);
    assert_eq!(r.u8(), 0xFF);
}

#[test]
fn roundtrip_i8() {
    let mut w = Writer::new();
    w.i8(i8::MIN);
    w.i8(-1);
    w.i8(0);
    w.i8(i8::MAX);
    let data = w.flush();
    let mut r = Reader::new(&data);
    assert_eq!(r.i8(), i8::MIN);
    assert_eq!(r.i8(), -1);
    assert_eq!(r.i8(), 0);
    assert_eq!(r.i8(), i8::MAX);
}

#[test]
fn roundtrip_u16() {
    let mut w = Writer::new();
    w.u16(0);
    w.u16(0x0102);
    w.u16(u16::MAX);
    let data = w.flush();
    let mut r = Reader::new(&data);
    assert_eq!(r.u16(), 0);
    assert_eq!(r.u16(), 0x0102);
    assert_eq!(r.u16(), u16::MAX);
}

#[test]
fn roundtrip_i16() {
    let mut w = Writer::new();
    w.i16(i16::MIN);
    w.i16(-1000);
    w.i16(0);
    w.i16(1000);
    w.i16(i16::MAX);
    let data = w.flush();
    let mut r = Reader::new(&data);
    assert_eq!(r.i16(), i16::MIN);
    assert_eq!(r.i16(), -1000);
    assert_eq!(r.i16(), 0);
    assert_eq!(r.i16(), 1000);
    assert_eq!(r.i16(), i16::MAX);
}

#[test]
fn roundtrip_u32() {
    let mut w = Writer::new();
    w.u32(0);
    w.u32(0x01020304);
    w.u32(u32::MAX);
    let data = w.flush();
    let mut r = Reader::new(&data);
    assert_eq!(r.u32(), 0);
    assert_eq!(r.u32(), 0x01020304);
    assert_eq!(r.u32(), u32::MAX);
}

#[test]
fn roundtrip_i32() {
    let mut w = Writer::new();
    w.i32(i32::MIN);
    w.i32(-123456);
    w.i32(0);
    w.i32(123456);
    w.i32(i32::MAX);
    let data = w.flush();
    let mut r = Reader::new(&data);
    assert_eq!(r.i32(), i32::MIN);
    assert_eq!(r.i32(), -123456);
    assert_eq!(r.i32(), 0);
    assert_eq!(r.i32(), 123456);
    assert_eq!(r.i32(), i32::MAX);
}

#[test]
fn roundtrip_u64() {
    let mut w = Writer::new();
    w.u64(0);
    w.u64(0x0102030405060708);
    w.u64(u64::MAX);
    let data = w.flush();
    let mut r = Reader::new(&data);
    assert_eq!(r.u64(), 0);
    assert_eq!(r.u64(), 0x0102030405060708);
    assert_eq!(r.u64(), u64::MAX);
}

#[test]
fn roundtrip_i64() {
    let mut w = Writer::new();
    w.i64(i64::MIN);
    w.i64(-9_999_999_999);
    w.i64(0);
    w.i64(9_999_999_999);
    w.i64(i64::MAX);
    let data = w.flush();
    let mut r = Reader::new(&data);
    assert_eq!(r.i64(), i64::MIN);
    assert_eq!(r.i64(), -9_999_999_999);
    assert_eq!(r.i64(), 0);
    assert_eq!(r.i64(), 9_999_999_999);
    assert_eq!(r.i64(), i64::MAX);
}

#[test]
fn roundtrip_f32() {
    let mut w = Writer::new();
    w.f32(0.0);
    w.f32(1.5);
    w.f32(-1.5);
    w.f32(f32::INFINITY);
    w.f32(f32::NEG_INFINITY);
    let data = w.flush();
    let mut r = Reader::new(&data);
    assert_eq!(r.f32(), 0.0);
    assert_eq!(r.f32(), 1.5);
    assert_eq!(r.f32(), -1.5);
    assert_eq!(r.f32(), f32::INFINITY);
    assert_eq!(r.f32(), f32::NEG_INFINITY);
}

#[test]
fn roundtrip_f32_nan() {
    let mut w = Writer::new();
    w.f32(f32::NAN);
    let data = w.flush();
    let mut r = Reader::new(&data);
    assert!(r.f32().is_nan());
}

#[test]
fn roundtrip_f64() {
    let mut w = Writer::new();
    w.f64(0.0);
    w.f64(std::f64::consts::PI);
    w.f64(-std::f64::consts::E);
    w.f64(f64::INFINITY);
    w.f64(f64::NEG_INFINITY);
    let data = w.flush();
    let mut r = Reader::new(&data);
    assert_eq!(r.f64(), 0.0);
    assert_eq!(r.f64(), std::f64::consts::PI);
    assert_eq!(r.f64(), -std::f64::consts::E);
    assert_eq!(r.f64(), f64::INFINITY);
    assert_eq!(r.f64(), f64::NEG_INFINITY);
}

#[test]
fn roundtrip_f64_nan() {
    let mut w = Writer::new();
    w.f64(f64::NAN);
    let data = w.flush();
    let mut r = Reader::new(&data);
    assert!(r.f64().is_nan());
}

#[test]
fn roundtrip_buf() {
    let mut w = Writer::new();
    w.buf(&[]);
    w.buf(&[0xDE, 0xAD, 0xBE, 0xEF]);
    let data = w.flush();
    let mut r = Reader::new(&data);
    assert_eq!(r.buf(0), &[]);
    assert_eq!(r.buf(4), &[0xDE, 0xAD, 0xBE, 0xEF]);
}

#[test]
fn roundtrip_utf8() {
    let mut w = Writer::new();
    w.utf8("hello");
    w.utf8("");
    w.utf8("cafe\u{0301}"); // e + combining accent
    w.utf8("\u{1F600}"); // emoji
    let data = w.flush();
    let mut r = Reader::new(&data);
    assert_eq!(r.utf8(5), "hello");
    assert_eq!(r.utf8(0), "");
    assert_eq!(r.utf8("cafe\u{0301}".len()), "cafe\u{0301}");
    assert_eq!(r.utf8("\u{1F600}".len()), "\u{1F600}");
}

#[test]
fn roundtrip_ascii() {
    let mut w = Writer::new();
    w.ascii("abc");
    let data = w.flush();
    let mut r = Reader::new(&data);
    assert_eq!(r.ascii(3), "abc");
}

// ---------------------------------------------------------------------------
// Combo write methods
// ---------------------------------------------------------------------------

#[test]
fn roundtrip_u8u16() {
    let mut w = Writer::new();
    w.u8u16(0xAB, 0x1234);
    let data = w.flush();
    let mut r = Reader::new(&data);
    assert_eq!(r.u8(), 0xAB);
    assert_eq!(r.u16(), 0x1234);
}

#[test]
fn roundtrip_u8u32() {
    let mut w = Writer::new();
    w.u8u32(0xCD, 0xDEADBEEF);
    let data = w.flush();
    let mut r = Reader::new(&data);
    assert_eq!(r.u8(), 0xCD);
    assert_eq!(r.u32(), 0xDEADBEEF);
}

#[test]
fn roundtrip_u8u64() {
    let mut w = Writer::new();
    w.u8u64(0xEF, 0x0102030405060708);
    let data = w.flush();
    let mut r = Reader::new(&data);
    assert_eq!(r.u8(), 0xEF);
    assert_eq!(r.u64(), 0x0102030405060708);
}

#[test]
fn roundtrip_u8f32() {
    let mut w = Writer::new();
    w.u8f32(0x01, 1.5f32);
    let data = w.flush();
    let mut r = Reader::new(&data);
    assert_eq!(r.u8(), 0x01);
    assert_eq!(r.f32(), 1.5f32);
}

#[test]
fn roundtrip_u8f64() {
    let mut w = Writer::new();
    w.u8f64(0x02, std::f64::consts::PI);
    let data = w.flush();
    let mut r = Reader::new(&data);
    assert_eq!(r.u8(), 0x02);
    assert_eq!(r.f64(), std::f64::consts::PI);
}

// ---------------------------------------------------------------------------
// Multiple flush cycles
// ---------------------------------------------------------------------------

#[test]
fn writer_flush_resets_window() {
    let mut w = Writer::new();
    w.u8(0x01);
    w.u8(0x02);
    let first = w.flush();
    assert_eq!(first, [0x01, 0x02]);

    w.u8(0x03);
    let second = w.flush();
    assert_eq!(second, [0x03]);
}

// ---------------------------------------------------------------------------
// f16 decode edge cases
// ---------------------------------------------------------------------------

#[test]
fn f16_positive_zero() {
    assert_eq!(decode_f16(0x0000), 0.0);
    assert!(decode_f16(0x0000).is_sign_positive());
}

#[test]
fn f16_negative_zero() {
    let val = decode_f16(0x8000);
    assert_eq!(val, 0.0);
    assert!(val.is_sign_negative());
}

#[test]
fn f16_one() {
    assert_eq!(decode_f16(0x3C00), 1.0);
}

#[test]
fn f16_negative_one() {
    assert_eq!(decode_f16(0xBC00), -1.0);
}

#[test]
fn f16_two() {
    assert_eq!(decode_f16(0x4000), 2.0);
}

#[test]
fn f16_positive_infinity() {
    let val = decode_f16(0x7C00);
    assert!(val.is_infinite());
    assert!(val.is_sign_positive());
}

#[test]
fn f16_negative_infinity() {
    let val = decode_f16(0xFC00);
    assert!(val.is_infinite());
    assert!(val.is_sign_negative());
}

#[test]
fn f16_nan() {
    assert!(decode_f16(0x7C01).is_nan());
    assert!(decode_f16(0xFC01).is_nan());
    // Different NaN payload
    assert!(decode_f16(0x7E00).is_nan());
}

#[test]
fn f16_subnormal_smallest() {
    // Smallest positive subnormal: 0x0001 = 2^-24 ~= 5.96e-8
    let val = decode_f16(0x0001);
    assert!(val > 0.0);
    assert!(val < 1e-4);
}

#[test]
fn f16_subnormal_largest() {
    // Largest positive subnormal: 0x03FF
    let val = decode_f16(0x03FF);
    assert!(val > 0.0);
    assert!(val < 1.0);
}

#[test]
fn f16_half() {
    // 0.5 in f16 = 0x3800
    assert_eq!(decode_f16(0x3800), 0.5);
}

// ---------------------------------------------------------------------------
// is_float32
// ---------------------------------------------------------------------------

#[test]
fn is_float32_exact_values() {
    assert!(is_float32(0.0));
    assert!(is_float32(1.0));
    assert!(is_float32(0.5));
    assert!(is_float32(0.25));
    assert!(is_float32(-1.0));
}

#[test]
fn is_float32_non_representable() {
    assert!(!is_float32(0.1));
    assert!(!is_float32(0.3));
}

// ---------------------------------------------------------------------------
// Byte comparison utilities
// ---------------------------------------------------------------------------

#[test]
fn cmp_uint8_array_equality() {
    assert!(cmp_uint8_array(&[], &[]));
    assert!(cmp_uint8_array(&[1, 2, 3], &[1, 2, 3]));
    assert!(!cmp_uint8_array(&[1, 2, 3], &[1, 2, 4]));
    assert!(!cmp_uint8_array(&[1], &[1, 2]));
}

#[test]
fn cmp_uint8_array2_ordering() {
    assert_eq!(cmp_uint8_array2(&[1, 2, 3], &[1, 2, 3]), 0);
    assert!(cmp_uint8_array2(&[1, 2], &[1, 2, 3]) < 0);
    assert!(cmp_uint8_array2(&[1, 2, 3], &[1, 2]) > 0);
    assert!(cmp_uint8_array2(&[1, 2, 3], &[1, 3, 2]) < 0);
    assert!(cmp_uint8_array2(&[1, 3, 2], &[1, 2, 3]) > 0);
    assert_eq!(cmp_uint8_array2(&[], &[]), 0);
}

#[test]
fn cmp_uint8_array3_length_first_ordering() {
    assert_eq!(cmp_uint8_array3(&[1, 2, 3], &[1, 2, 3]), 0);
    assert!(cmp_uint8_array3(&[1, 2], &[1, 2, 3]) < 0);
    assert!(cmp_uint8_array3(&[1, 2, 3], &[1, 2]) > 0);
    assert_eq!(cmp_uint8_array3(&[], &[]), 0);
}

// ---------------------------------------------------------------------------
// Concat utilities
// ---------------------------------------------------------------------------

#[test]
fn concat_two_slices() {
    let result = concat(&[1, 2], &[3, 4]);
    assert_eq!(result, vec![1, 2, 3, 4]);
}

#[test]
fn concat_list_slices() {
    let result = concat_list(&[&[1u8, 2][..], &[3, 4][..], &[5][..]]);
    assert_eq!(result, vec![1, 2, 3, 4, 5]);
}

#[test]
fn concat_empty() {
    let result = concat(&[], &[]);
    assert_eq!(result, Vec::<u8>::new());
}

// ---------------------------------------------------------------------------
// Mixed-type roundtrip: interleaved writes
// ---------------------------------------------------------------------------

#[test]
fn roundtrip_mixed_types() {
    let mut w = Writer::new();
    w.u8(0x42);
    w.u16(0xCAFE);
    w.u32(0xDEADBEEF);
    w.f64(std::f64::consts::PI);
    w.utf8("hello");
    w.i64(-12345678);
    let data = w.flush();

    let mut r = Reader::new(&data);
    assert_eq!(r.u8(), 0x42);
    assert_eq!(r.u16(), 0xCAFE);
    assert_eq!(r.u32(), 0xDEADBEEF);
    assert_eq!(r.f64(), std::f64::consts::PI);
    assert_eq!(r.utf8(5), "hello");
    assert_eq!(r.i64(), -12345678);
    assert_eq!(r.size(), 0);
}
