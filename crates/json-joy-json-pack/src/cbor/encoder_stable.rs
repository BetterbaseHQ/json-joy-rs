//! `CborEncoderStable` — stable CBOR encoder (sorts object keys).
//!
//! Direct port of `cbor/CborEncoderStable.ts` from upstream.
//! Extends `CborEncoder` by sorting object keys before encoding.

use json_joy_buffers::{is_float32, Writer};

use super::constants::*;

/// Stable CBOR encoder.
///
/// Same as [`super::encoder::CborEncoder`] but sorts object keys
/// lexicographically (consistent, deterministic output).
/// Also uses the optimized `write_str` with pre-computed header.
pub struct CborEncoderStable {
    pub writer: Writer,
}

impl Default for CborEncoderStable {
    fn default() -> Self {
        Self::new()
    }
}

impl CborEncoderStable {
    pub fn new() -> Self {
        Self {
            writer: Writer::new(),
        }
    }

    pub fn encode(&mut self, value: &crate::PackValue) -> Vec<u8> {
        self.writer.reset();
        self.write_any(value);
        self.writer.flush()
    }

    pub fn encode_json(&mut self, value: &serde_json::Value) -> Vec<u8> {
        self.writer.reset();
        self.write_any(&crate::PackValue::from(value.clone()));
        self.writer.flush()
    }

    pub fn write_any(&mut self, value: &crate::PackValue) {
        use crate::PackValue::*;
        match value {
            Null | Undefined => self.write_null(), // stable maps undefined → null
            Bool(b) => self.write_boolean(*b),
            Integer(i) => self.write_integer(*i),
            UInteger(u) => self.write_u_integer(*u),
            Float(f) => self.write_float(*f),
            BigInt(i) => self.write_big_int(*i),
            Bytes(b) => self.write_bin(b),
            Str(s) => self.write_str(s),
            Array(arr) => self.write_arr_values(arr),
            Object(obj) => {
                // Sort keys before encoding
                let mut sorted: Vec<&(String, crate::PackValue)> = obj.iter().collect();
                sorted.sort_by(|a, b| cmp_obj_key(&a.0, &b.0));
                self.write_obj_hdr(sorted.len());
                for (key, val) in sorted {
                    self.write_str(key);
                    self.write_any(val);
                }
            }
            Extension(ext) => self.write_tag(ext.tag, &ext.val),
            Blob(blob) => self.writer.buf(&blob.val),
        }
    }

    pub fn write_null(&mut self) {
        self.writer.u8(0xf6);
    }

    pub fn write_boolean(&mut self, b: bool) {
        self.writer.u8(if b { 0xf5 } else { 0xf4 });
    }

    pub fn write_integer(&mut self, int: i64) {
        if int >= 0 {
            self.write_u_integer(int as u64);
        } else {
            self.encode_nint(int);
        }
    }

    pub fn write_u_integer(&mut self, uint: u64) {
        let w = &mut self.writer;
        w.ensure_capacity(9);
        let x = w.x;
        if uint <= 23 {
            w.uint8[x] = OVERLAY_UIN | uint as u8;
            w.x = x + 1;
        } else if uint <= 0xff {
            w.uint8[x] = 0x18;
            w.uint8[x + 1] = uint as u8;
            w.x = x + 2;
        } else if uint <= 0xffff {
            w.uint8[x] = 0x19;
            let b = (uint as u16).to_be_bytes();
            w.uint8[x + 1] = b[0];
            w.uint8[x + 2] = b[1];
            w.x = x + 3;
        } else if uint <= 0xffffffff {
            w.uint8[x] = 0x1a;
            let b = (uint as u32).to_be_bytes();
            w.uint8[x + 1..x + 5].copy_from_slice(&b);
            w.x = x + 5;
        } else {
            w.uint8[x] = 0x1b;
            let b = uint.to_be_bytes();
            w.uint8[x + 1..x + 9].copy_from_slice(&b);
            w.x = x + 9;
        }
    }

    pub fn encode_nint(&mut self, int: i64) {
        let uint = (-1i64).wrapping_sub(int) as u64;
        let w = &mut self.writer;
        w.ensure_capacity(9);
        let x = w.x;
        if uint < 24 {
            w.uint8[x] = OVERLAY_NIN | uint as u8;
            w.x = x + 1;
        } else if uint <= 0xff {
            w.uint8[x] = 0x38;
            w.uint8[x + 1] = uint as u8;
            w.x = x + 2;
        } else if uint <= 0xffff {
            w.uint8[x] = 0x39;
            let b = (uint as u16).to_be_bytes();
            w.uint8[x + 1] = b[0];
            w.uint8[x + 2] = b[1];
            w.x = x + 3;
        } else if uint <= 0xffffffff {
            w.uint8[x] = 0x3a;
            let b = (uint as u32).to_be_bytes();
            w.uint8[x + 1..x + 5].copy_from_slice(&b);
            w.x = x + 5;
        } else {
            w.uint8[x] = 0x3b;
            let b = uint.to_be_bytes();
            w.uint8[x + 1..x + 9].copy_from_slice(&b);
            w.x = x + 9;
        }
    }

    pub fn write_big_int(&mut self, int: i128) {
        if int >= 0 {
            if int as u128 <= u64::MAX as u128 {
                self.write_u_integer(int as u64);
            } else {
                self.writer.u8u64(0x1b, u64::MAX);
            }
        } else if int >= i64::MIN as i128 {
            self.encode_nint(int as i64);
        } else {
            let uint = (-1i128 - int) as u64;
            self.writer.u8u64(0x3b, uint);
        }
    }

    pub fn write_float(&mut self, float: f64) {
        if is_float32(float) {
            self.writer.u8f32(0xfa, float as f32);
        } else {
            self.writer.u8f64(0xfb, float);
        }
    }

    pub fn write_bin(&mut self, buf: &[u8]) {
        let length = buf.len();
        self.write_bin_hdr(length);
        self.writer.buf(buf);
    }

    pub fn write_bin_hdr(&mut self, length: usize) {
        let w = &mut self.writer;
        if length <= 23 {
            w.u8(OVERLAY_BIN | length as u8);
        } else if length <= 0xff {
            w.u8(0x58);
            w.u8(length as u8);
        } else if length <= 0xffff {
            w.u8(0x59);
            w.u16(length as u16);
        } else {
            w.u8(0x5a);
            w.u32(length as u32);
        }
    }

    /// Optimized `write_str` — pre-computes exact header based on byte length.
    /// Mirrors `CborEncoderStable.writeStr` from upstream.
    pub fn write_str(&mut self, s: &str) {
        let byte_len = s.len();

        // Header length: bytes needed for the CBOR text header
        let header_len = str_header_length(byte_len);

        self.writer.ensure_capacity(header_len + byte_len);
        let x0 = self.writer.x;
        let x1 = x0 + header_len;
        self.writer.x = x1;

        // Write the string bytes
        let x = self.writer.x;
        self.writer.uint8[x..x + byte_len].copy_from_slice(s.as_bytes());
        self.writer.x = x + byte_len;

        // Write the header at x0
        match header_len {
            1 => self.writer.uint8[x0] = OVERLAY_STR | byte_len as u8,
            2 => {
                self.writer.uint8[x0] = 0x78;
                self.writer.uint8[x0 + 1] = byte_len as u8;
            }
            3 => {
                self.writer.uint8[x0] = 0x79;
                let b = (byte_len as u16).to_be_bytes();
                self.writer.uint8[x0 + 1] = b[0];
                self.writer.uint8[x0 + 2] = b[1];
            }
            5 => {
                self.writer.uint8[x0] = 0x7a;
                let b = (byte_len as u32).to_be_bytes();
                self.writer.uint8[x0 + 1..x0 + 5].copy_from_slice(&b);
            }
            _ => unreachable!(),
        }
    }

    pub fn write_str_hdr(&mut self, length: usize) {
        let w = &mut self.writer;
        if length <= 23 {
            w.u8(OVERLAY_STR | length as u8);
        } else if length <= 0xff {
            w.u8(0x78);
            w.u8(length as u8);
        } else if length <= 0xffff {
            w.u8(0x79);
            w.u16(length as u16);
        } else {
            w.u8(0x7a);
            w.u32(length as u32);
        }
    }

    pub fn write_ascii_str(&mut self, s: &str) {
        self.write_str_hdr(s.len());
        self.writer.ascii(s);
    }

    pub fn write_arr_values(&mut self, arr: &[crate::PackValue]) {
        self.write_arr_hdr(arr.len());
        for item in arr {
            self.write_any(item);
        }
    }

    pub fn write_arr_hdr(&mut self, length: usize) {
        let w = &mut self.writer;
        if length <= 23 {
            w.u8(OVERLAY_ARR | length as u8);
        } else if length <= 0xff {
            w.u8(0x98);
            w.u8(length as u8);
        } else if length <= 0xffff {
            w.u8(0x99);
            w.u16(length as u16);
        } else {
            w.u8(0x9a);
            w.u32(length as u32);
        }
    }

    pub fn write_obj_hdr(&mut self, length: usize) {
        let w = &mut self.writer;
        if length <= 23 {
            w.u8(OVERLAY_MAP | length as u8);
        } else if length <= 0xff {
            w.u8(0xb8);
            w.u8(length as u8);
        } else if length <= 0xffff {
            w.u8(0xb9);
            w.u16(length as u16);
        } else {
            w.u8(0xba);
            w.u32(length as u32);
        }
    }

    pub fn write_tag(&mut self, tag: u64, value: &crate::PackValue) {
        self.write_tag_hdr(tag);
        self.write_any(value);
    }

    pub fn write_tag_hdr(&mut self, tag: u64) {
        let w = &mut self.writer;
        if tag <= 23 {
            w.u8(OVERLAY_TAG | tag as u8);
        } else if tag <= 0xff {
            w.u8(0xd8);
            w.u8(tag as u8);
        } else if tag <= 0xffff {
            w.u8(0xd9);
            w.u16(tag as u16);
        } else if tag <= 0xffffffff {
            w.u8(0xda);
            w.u32(tag as u32);
        } else {
            w.u8(0xdb);
            w.u64(tag);
        }
    }
}

/// Number of bytes needed for the CBOR text string header given the byte length.
fn str_header_length(size: usize) -> usize {
    if size <= 23 {
        1
    } else if size <= 0xff {
        2
    } else if size <= 0xffff {
        3
    } else {
        5
    }
}

/// Compare object keys for stable sort (mirrors `objKeyCmp` from upstream).
/// Keys are compared by byte length first, then lexicographically.
fn cmp_obj_key(a: &str, b: &str) -> std::cmp::Ordering {
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PackValue;

    fn enc(value: &PackValue) -> Vec<u8> {
        let mut encoder = CborEncoderStable::new();
        encoder.encode(value)
    }

    // --- str_header_length ---

    #[test]
    fn test_str_header_length_tiny() {
        assert_eq!(str_header_length(0), 1);
        assert_eq!(str_header_length(23), 1);
    }

    #[test]
    fn test_str_header_length_u8() {
        assert_eq!(str_header_length(24), 2);
        assert_eq!(str_header_length(255), 2);
    }

    #[test]
    fn test_str_header_length_u16() {
        assert_eq!(str_header_length(256), 3);
        assert_eq!(str_header_length(0xffff), 3);
    }

    #[test]
    fn test_str_header_length_u32() {
        assert_eq!(str_header_length(0x10000), 5);
    }

    // --- cmp_obj_key ---

    #[test]
    fn test_cmp_obj_key_shorter_first() {
        assert_eq!(cmp_obj_key("a", "bb"), std::cmp::Ordering::Less);
    }

    #[test]
    fn test_cmp_obj_key_same_length_lexicographic() {
        assert_eq!(cmp_obj_key("ab", "ba"), std::cmp::Ordering::Less);
        assert_eq!(cmp_obj_key("ba", "ab"), std::cmp::Ordering::Greater);
    }

    #[test]
    fn test_cmp_obj_key_equal() {
        assert_eq!(cmp_obj_key("abc", "abc"), std::cmp::Ordering::Equal);
    }

    // --- write_null ---

    #[test]
    fn test_encode_null() {
        assert_eq!(enc(&PackValue::Null), vec![0xf6]);
    }

    #[test]
    fn test_encode_undefined_maps_to_null() {
        assert_eq!(enc(&PackValue::Undefined), vec![0xf6]);
    }

    // --- write_boolean ---

    #[test]
    fn test_encode_true() {
        assert_eq!(enc(&PackValue::Bool(true)), vec![0xf5]);
    }

    #[test]
    fn test_encode_false() {
        assert_eq!(enc(&PackValue::Bool(false)), vec![0xf4]);
    }

    // --- write_u_integer (all size classes) ---

    #[test]
    fn test_encode_uint_tiny() {
        assert_eq!(enc(&PackValue::UInteger(0)), vec![0x00]);
        assert_eq!(enc(&PackValue::UInteger(23)), vec![0x17]);
    }

    #[test]
    fn test_encode_uint_u8() {
        assert_eq!(enc(&PackValue::UInteger(24)), vec![0x18, 24]);
        assert_eq!(enc(&PackValue::UInteger(255)), vec![0x18, 0xff]);
    }

    #[test]
    fn test_encode_uint_u16() {
        assert_eq!(enc(&PackValue::UInteger(256)), vec![0x19, 0x01, 0x00]);
        assert_eq!(enc(&PackValue::UInteger(0xffff)), vec![0x19, 0xff, 0xff]);
    }

    #[test]
    fn test_encode_uint_u32() {
        assert_eq!(
            enc(&PackValue::UInteger(0x10000)),
            vec![0x1a, 0x00, 0x01, 0x00, 0x00]
        );
        assert_eq!(
            enc(&PackValue::UInteger(0xffffffff)),
            vec![0x1a, 0xff, 0xff, 0xff, 0xff]
        );
    }

    #[test]
    fn test_encode_uint_u64() {
        assert_eq!(
            enc(&PackValue::UInteger(0x100000000)),
            vec![0x1b, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00]
        );
    }

    // --- encode_nint (negative integers, all size classes) ---

    #[test]
    fn test_encode_nint_tiny() {
        // -1 => uint=0 => 0x20
        assert_eq!(enc(&PackValue::Integer(-1)), vec![0x20]);
        // -24 => uint=23 => 0x37
        assert_eq!(enc(&PackValue::Integer(-24)), vec![0x37]);
    }

    #[test]
    fn test_encode_nint_u8() {
        // -25 => uint=24
        assert_eq!(enc(&PackValue::Integer(-25)), vec![0x38, 24]);
        // -256 => uint=255
        assert_eq!(enc(&PackValue::Integer(-256)), vec![0x38, 0xff]);
    }

    #[test]
    fn test_encode_nint_u16() {
        // -257 => uint=256
        assert_eq!(enc(&PackValue::Integer(-257)), vec![0x39, 0x01, 0x00]);
    }

    #[test]
    fn test_encode_nint_u32() {
        // -(0x10000 + 1) => uint=0x10000
        assert_eq!(
            enc(&PackValue::Integer(-0x10001)),
            vec![0x3a, 0x00, 0x01, 0x00, 0x00]
        );
    }

    #[test]
    fn test_encode_nint_u64() {
        // -(0x100000000 + 1) => uint=0x100000000
        assert_eq!(
            enc(&PackValue::Integer(-0x100000001)),
            vec![0x3b, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00]
        );
    }

    // --- write_integer positive path ---

    #[test]
    fn test_encode_positive_integer_uses_uint_path() {
        assert_eq!(enc(&PackValue::Integer(42)), vec![0x18, 42]);
    }

    // --- write_big_int ---

    #[test]
    fn test_encode_big_int_small_positive() {
        assert_eq!(enc(&PackValue::BigInt(10)), vec![0x0a]);
    }

    #[test]
    fn test_encode_big_int_large_positive_clamps() {
        // value > u64::MAX => clamped to u64::MAX
        let large = u64::MAX as i128 + 1;
        let result = enc(&PackValue::BigInt(large));
        assert_eq!(result[0], 0x1b);
    }

    #[test]
    fn test_encode_big_int_negative_fits_i64() {
        let result = enc(&PackValue::BigInt(-100));
        // -100 => uint=99 => 0x38, 99
        assert_eq!(result, vec![0x38, 99]);
    }

    #[test]
    fn test_encode_big_int_very_negative() {
        // smaller than i64::MIN
        let val = i64::MIN as i128 - 1;
        let result = enc(&PackValue::BigInt(val));
        assert_eq!(result[0], 0x3b);
    }

    // --- write_float ---

    #[test]
    fn test_encode_float32_representable() {
        // 1.5 can be losslessly represented as f32
        let result = enc(&PackValue::Float(1.5));
        assert_eq!(result[0], 0xfa);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_encode_float64_needed() {
        // 1.1 cannot be losslessly f32
        let result = enc(&PackValue::Float(1.1));
        assert_eq!(result[0], 0xfb);
        assert_eq!(result.len(), 9);
    }

    // --- write_bin ---

    #[test]
    fn test_encode_bin_tiny() {
        let result = enc(&PackValue::Bytes(vec![0xaa, 0xbb]));
        // header: OVERLAY_BIN | 2 = 0x42, then data
        assert_eq!(result, vec![0x42, 0xaa, 0xbb]);
    }

    #[test]
    fn test_encode_bin_empty() {
        let result = enc(&PackValue::Bytes(vec![]));
        assert_eq!(result, vec![0x40]);
    }

    #[test]
    fn test_encode_bin_u8_length() {
        let data = vec![0u8; 24];
        let result = enc(&PackValue::Bytes(data.clone()));
        assert_eq!(result[0], 0x58);
        assert_eq!(result[1], 24);
        assert_eq!(result.len(), 2 + 24);
    }

    #[test]
    fn test_encode_bin_u16_length() {
        let data = vec![0u8; 256];
        let result = enc(&PackValue::Bytes(data));
        assert_eq!(result[0], 0x59);
        assert_eq!(result.len(), 3 + 256);
    }

    // --- write_str ---

    #[test]
    fn test_encode_str_tiny() {
        let result = enc(&PackValue::Str("hi".into()));
        assert_eq!(result, vec![0x62, b'h', b'i']);
    }

    #[test]
    fn test_encode_str_empty() {
        let result = enc(&PackValue::Str(String::new()));
        assert_eq!(result, vec![0x60]);
    }

    #[test]
    fn test_encode_str_u8_length() {
        let s = "a".repeat(24);
        let result = enc(&PackValue::Str(s));
        assert_eq!(result[0], 0x78);
        assert_eq!(result[1], 24);
    }

    #[test]
    fn test_encode_str_u16_length() {
        let s = "a".repeat(256);
        let result = enc(&PackValue::Str(s));
        assert_eq!(result[0], 0x79);
        assert_eq!(result[1..3], [0x01, 0x00]);
    }

    // --- write_arr ---

    #[test]
    fn test_encode_array_empty() {
        let result = enc(&PackValue::Array(vec![]));
        assert_eq!(result, vec![0x80]);
    }

    #[test]
    fn test_encode_array_with_items() {
        let result = enc(&PackValue::Array(vec![
            PackValue::Null,
            PackValue::Bool(true),
        ]));
        assert_eq!(result, vec![0x82, 0xf6, 0xf5]);
    }

    #[test]
    fn test_encode_arr_hdr_u8() {
        let arr: Vec<PackValue> = (0..24).map(|_| PackValue::Null).collect();
        let result = enc(&PackValue::Array(arr));
        assert_eq!(result[0], 0x98);
        assert_eq!(result[1], 24);
    }

    // --- write_obj (sorted keys) ---

    #[test]
    fn test_encode_object_empty() {
        let result = enc(&PackValue::Object(vec![]));
        assert_eq!(result, vec![0xa0]);
    }

    #[test]
    fn test_encode_object_sorts_keys() {
        let obj = PackValue::Object(vec![
            ("bb".into(), PackValue::Integer(2)),
            ("a".into(), PackValue::Integer(1)),
        ]);
        let result = enc(&obj);
        // Sorted: "a" (len 1) before "bb" (len 2)
        // map(2) = 0xa2
        // str "a" = 0x61 0x61
        // int 1 = 0x01
        // str "bb" = 0x62 0x62 0x62
        // int 2 = 0x02
        assert_eq!(result[0], 0xa2);
        // first key is "a"
        assert_eq!(result[1], 0x61);
        assert_eq!(result[2], b'a');
    }

    #[test]
    fn test_encode_object_hdr_u8() {
        let obj: Vec<(String, PackValue)> = (0..24)
            .map(|i| (format!("k{i:02}"), PackValue::Null))
            .collect();
        let result = enc(&PackValue::Object(obj));
        assert_eq!(result[0], 0xb8);
        assert_eq!(result[1], 24);
    }

    // --- write_tag ---

    #[test]
    fn test_encode_tag_tiny() {
        let ext = crate::JsonPackExtension::new(1, PackValue::Str("test".into()));
        let result = enc(&PackValue::Extension(Box::new(ext)));
        assert_eq!(result[0], 0xc1); // OVERLAY_TAG | 1
    }

    #[test]
    fn test_encode_tag_u8() {
        let ext = crate::JsonPackExtension::new(24, PackValue::Null);
        let result = enc(&PackValue::Extension(Box::new(ext)));
        assert_eq!(result[0], 0xd8);
        assert_eq!(result[1], 24);
    }

    #[test]
    fn test_encode_tag_u16() {
        let ext = crate::JsonPackExtension::new(256, PackValue::Null);
        let result = enc(&PackValue::Extension(Box::new(ext)));
        assert_eq!(result[0], 0xd9);
    }

    #[test]
    fn test_encode_tag_u32() {
        let ext = crate::JsonPackExtension::new(0x10000, PackValue::Null);
        let result = enc(&PackValue::Extension(Box::new(ext)));
        assert_eq!(result[0], 0xda);
    }

    #[test]
    fn test_encode_tag_u64() {
        let ext = crate::JsonPackExtension::new(0x100000000, PackValue::Null);
        let result = enc(&PackValue::Extension(Box::new(ext)));
        assert_eq!(result[0], 0xdb);
    }

    // --- encode_json ---

    #[test]
    fn test_encode_json_null() {
        let mut encoder = CborEncoderStable::new();
        let json = serde_json::Value::Null;
        let result = encoder.encode_json(&json);
        assert_eq!(result, vec![0xf6]);
    }

    // --- Default ---

    #[test]
    fn test_default() {
        let encoder = CborEncoderStable::default();
        assert_eq!(encoder.writer.x, 0);
    }

    // --- Blob passthrough ---

    #[test]
    fn test_encode_blob_passes_through() {
        let blob = crate::JsonPackValue::new(vec![0xde, 0xad]);
        let result = enc(&PackValue::Blob(blob));
        assert_eq!(result, vec![0xde, 0xad]);
    }

    // --- write_str_hdr ---

    #[test]
    fn test_write_str_hdr_all_sizes() {
        let mut e = CborEncoderStable::new();

        e.writer.reset();
        e.write_str_hdr(5);
        let r = e.writer.flush();
        assert_eq!(r, vec![OVERLAY_STR | 5]);

        e.writer.reset();
        e.write_str_hdr(100);
        let r = e.writer.flush();
        assert_eq!(r[0], 0x78);

        e.writer.reset();
        e.write_str_hdr(300);
        let r = e.writer.flush();
        assert_eq!(r[0], 0x79);

        e.writer.reset();
        e.write_str_hdr(70000);
        let r = e.writer.flush();
        assert_eq!(r[0], 0x7a);
    }

    // --- write_arr_hdr ---

    #[test]
    fn test_write_arr_hdr_u16() {
        let mut e = CborEncoderStable::new();
        e.write_arr_hdr(300);
        let r = e.writer.flush();
        assert_eq!(r[0], 0x99);
    }

    #[test]
    fn test_write_arr_hdr_u32() {
        let mut e = CborEncoderStable::new();
        e.write_arr_hdr(70000);
        let r = e.writer.flush();
        assert_eq!(r[0], 0x9a);
    }

    // --- write_obj_hdr ---

    #[test]
    fn test_write_obj_hdr_u16() {
        let mut e = CborEncoderStable::new();
        e.write_obj_hdr(300);
        let r = e.writer.flush();
        assert_eq!(r[0], 0xb9);
    }

    #[test]
    fn test_write_obj_hdr_u32() {
        let mut e = CborEncoderStable::new();
        e.write_obj_hdr(70000);
        let r = e.writer.flush();
        assert_eq!(r[0], 0xba);
    }

    // --- write_bin_hdr u32 branch ---

    #[test]
    fn test_write_bin_hdr_u32() {
        let mut e = CborEncoderStable::new();
        e.write_bin_hdr(70000);
        let r = e.writer.flush();
        assert_eq!(r[0], 0x5a);
    }
}
