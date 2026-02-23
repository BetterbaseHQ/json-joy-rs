//! `CborEncoderFast` — fast CBOR encoder for JSON-compatible values.
//!
//! Direct port of `cbor/CborEncoderFast.ts` from upstream.

use json_joy_buffers::Writer;

use super::constants::*;

/// Fast CBOR encoder supporting only JSON-compatible values.
///
/// For full CBOR support (including binary, extensions, Map, bigint, undefined),
/// use [`super::encoder::CborEncoder`].
pub struct CborEncoderFast {
    pub writer: Writer,
}

impl Default for CborEncoderFast {
    fn default() -> Self {
        Self::new()
    }
}

impl CborEncoderFast {
    pub fn new() -> Self {
        Self {
            writer: Writer::new(),
        }
    }

    pub fn with_writer(writer: Writer) -> Self {
        Self { writer }
    }

    /// Encode a value and return the CBOR bytes.
    pub fn encode(&mut self, value: &crate::PackValue) -> Vec<u8> {
        self.writer.reset();
        self.write_any(value);
        self.writer.flush()
    }

    pub fn write_any(&mut self, value: &crate::PackValue) {
        use crate::PackValue::*;
        match value {
            Null => self.write_null(),
            Undefined => self.write_null(), // fast encoder maps undefined to null
            Bool(b) => self.write_boolean(*b),
            Integer(i) => self.write_integer(*i),
            UInteger(u) => self.write_u_integer(*u),
            Float(f) => self.write_float(*f),
            BigInt(i) => self.write_big_int(*i),
            Bytes(b) => self.write_bin(b),
            Str(s) => self.write_str(s),
            Array(arr) => self.write_arr_values(arr),
            Object(obj) => self.write_obj_pairs(obj),
            Extension(ext) => self.write_tag(ext.tag, &ext.val),
            Blob(blob) => self.writer.buf(&blob.val),
        }
    }

    /// Write the CBOR self-describe tag (0xd9d9f7).
    pub fn write_cbor(&mut self) {
        self.writer.u8(0xd9);
        self.writer.u16(0xd9f7);
    }

    /// Write CBOR break code (0xff).
    pub fn write_end(&mut self) {
        self.writer.u8(CBOR_END);
    }

    pub fn write_null(&mut self) {
        self.writer.u8(0xf6);
    }

    pub fn write_boolean(&mut self, b: bool) {
        self.writer.u8(if b { 0xf5 } else { 0xf4 });
    }

    pub fn write_number(&mut self, num: f64) {
        // Upstream uses `isSafeInteger(num)` which checks:
        //   Number.isInteger(num) && Math.abs(num) <= Number.MAX_SAFE_INTEGER
        // MAX_SAFE_INTEGER = 2^53 - 1 = 9007199254740991
        const MAX_SAFE: f64 = 9_007_199_254_740_991.0; // 2^53 - 1
        if num.fract() == 0.0 && (-MAX_SAFE..=MAX_SAFE).contains(&num) {
            if num >= 0.0 {
                self.write_u_integer(num as u64);
            } else {
                self.encode_nint(num as i64);
            }
        } else {
            self.write_float(num);
        }
    }

    pub fn write_big_int(&mut self, int: i128) {
        if int >= 0 {
            self.write_big_uint(int as u128);
        } else {
            self.write_big_sint(int);
        }
    }

    pub fn write_big_uint(&mut self, uint: u128) {
        if uint <= u64::MAX as u128 {
            self.write_u_integer(uint as u64);
        } else {
            // Overflow: clamp to u64::MAX
            self.writer.u8u64(0x1b, u64::MAX);
        }
    }

    pub fn write_big_sint(&mut self, int: i128) {
        if int >= i64::MIN as i128 {
            self.encode_nint(int as i64);
        } else {
            let uint = (-1i128 - int) as u64;
            self.writer.u8u64(0x3b, uint);
        }
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

    pub fn write_float(&mut self, float: f64) {
        self.writer.u8f64(0xfb, float);
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
        } else if length <= 0xffffffff {
            w.u8(0x5a);
            w.u32(length as u32);
        } else {
            w.u8(0x5b);
            w.u64(length as u64);
        }
    }

    /// Write a CBOR text string using the max-size-guess header strategy
    /// (mirrors upstream `CborEncoderFast.writeStr`).
    ///
    /// The header slot is reserved based on `char_count * 4` (worst case),
    /// then patched with the actual UTF-8 byte count after writing.
    pub fn write_str(&mut self, s: &str) {
        let char_count = s.chars().count();
        let max_size = char_count * 4;
        let byte_len = s.len();

        self.writer.ensure_capacity(5 + byte_len);

        let length_offset: usize;
        if max_size <= 23 {
            length_offset = self.writer.x;
            self.writer.x += 1;
        } else if max_size <= 0xff {
            self.writer.uint8[self.writer.x] = 0x78;
            self.writer.x += 1;
            length_offset = self.writer.x;
            self.writer.x += 1;
        } else if max_size <= 0xffff {
            self.writer.uint8[self.writer.x] = 0x79;
            self.writer.x += 1;
            length_offset = self.writer.x;
            self.writer.x += 2;
        } else {
            self.writer.uint8[self.writer.x] = 0x7a;
            self.writer.x += 1;
            length_offset = self.writer.x;
            self.writer.x += 4;
        }

        // Write UTF-8 bytes
        let x = self.writer.x;
        self.writer.uint8[x..x + byte_len].copy_from_slice(s.as_bytes());
        self.writer.x = x + byte_len;

        // Patch the header with the actual byte count
        if max_size <= 23 {
            self.writer.uint8[length_offset] = OVERLAY_STR | byte_len as u8;
        } else if max_size <= 0xff {
            self.writer.uint8[length_offset] = byte_len as u8;
        } else if max_size <= 0xffff {
            let b = (byte_len as u16).to_be_bytes();
            self.writer.uint8[length_offset] = b[0];
            self.writer.uint8[length_offset + 1] = b[1];
        } else {
            let b = (byte_len as u32).to_be_bytes();
            self.writer.uint8[length_offset..length_offset + 4].copy_from_slice(&b);
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
        } else if length <= 0xffffffff {
            w.u8(0x7a);
            w.u32(length as u32);
        } else {
            w.u8(0x7b);
            w.u64(length as u64);
        }
    }

    pub fn write_ascii_str(&mut self, s: &str) {
        self.write_str_hdr(s.len());
        self.writer.ascii(s);
    }

    pub fn write_arr(&mut self, arr: &[crate::PackValue]) {
        self.write_arr_hdr(arr.len());
        for item in arr {
            self.write_any(item);
        }
    }

    pub fn write_arr_values(&mut self, arr: &[crate::PackValue]) {
        self.write_arr(arr);
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
        } else if length <= 0xffffffff {
            w.u8(0x9a);
            w.u32(length as u32);
        } else {
            w.u8(0x9b);
            w.u64(length as u64);
        }
    }

    pub fn write_obj(&mut self, obj: &serde_json::Map<String, serde_json::Value>) {
        self.write_obj_hdr(obj.len());
        for (key, value) in obj {
            self.write_str(key);
            self.write_any(&crate::PackValue::from(value.clone()));
        }
    }

    pub fn write_obj_pairs(&mut self, pairs: &[(String, crate::PackValue)]) {
        self.write_obj_hdr(pairs.len());
        for (key, value) in pairs {
            self.write_str(key);
            self.write_any(value);
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
        } else if length <= 0xffffffff {
            w.u8(0xba);
            w.u32(length as u32);
        } else {
            w.u8(0xbb);
            w.u64(length as u64);
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

    // ---- Streaming ----

    pub fn write_start_str(&mut self) {
        self.writer.u8(0x7f);
    }

    pub fn write_start_bin(&mut self) {
        self.writer.u8(0x5f);
    }

    pub fn write_start_arr(&mut self) {
        self.writer.u8(0x9f);
    }

    pub fn write_end_arr(&mut self) {
        self.writer.u8(CBOR_END);
    }

    pub fn write_start_obj(&mut self) {
        self.writer.u8(0xbf);
    }

    pub fn write_end_obj(&mut self) {
        self.writer.u8(CBOR_END);
    }
}

// ---- JSON convenience methods (operate on serde_json::Value) ----

impl CborEncoderFast {
    /// Encode a `serde_json::Value` to CBOR bytes.
    pub fn encode_json(&mut self, value: &serde_json::Value) -> Vec<u8> {
        self.writer.reset();
        self.write_json(value);
        self.writer.flush()
    }

    pub fn write_json(&mut self, value: &serde_json::Value) {
        match value {
            serde_json::Value::Null => self.write_null(),
            serde_json::Value::Bool(b) => self.write_boolean(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    self.write_integer(i);
                } else if let Some(u) = n.as_u64() {
                    self.write_u_integer(u);
                } else if let Some(f) = n.as_f64() {
                    self.write_float(f);
                }
            }
            serde_json::Value::String(s) => self.write_str(s),
            serde_json::Value::Array(arr) => {
                self.write_arr_hdr(arr.len());
                for item in arr {
                    self.write_json(item);
                }
            }
            serde_json::Value::Object(obj) => {
                self.write_obj_hdr(obj.len());
                for (key, value) in obj {
                    self.write_str(key);
                    self.write_json(value);
                }
            }
        }
    }
}

// ---- Backward-compatible standalone functions ----

/// Write a CBOR integer header (major type + length) to a `Vec<u8>`.
/// Kept for backward compatibility with existing code.
pub fn write_cbor_uint_major(out: &mut Vec<u8>, major: u8, n: u64) {
    let major_bits = major << 5;
    if n <= 23 {
        out.push(major_bits | (n as u8));
    } else if n <= 0xff {
        out.push(major_bits | 24);
        out.push(n as u8);
    } else if n <= 0xffff {
        out.push(major_bits | 25);
        out.extend_from_slice(&(n as u16).to_be_bytes());
    } else if n <= 0xffff_ffff {
        out.push(major_bits | 26);
        out.extend_from_slice(&(n as u32).to_be_bytes());
    } else {
        out.push(major_bits | 27);
        out.extend_from_slice(&n.to_be_bytes());
    }
}

/// Write a signed integer to a CBOR `Vec<u8>`.
pub fn write_cbor_signed(out: &mut Vec<u8>, n: i64) {
    if n >= 0 {
        write_cbor_uint_major(out, 0, n as u64); // MAJOR_UNSIGNED
    } else {
        let encoded = (-1i128 - n as i128) as u64;
        write_cbor_uint_major(out, 1, encoded); // MAJOR_NEGATIVE
    }
}

/// Write a CBOR text string with the max-size-guess header strategy.
/// This is the `json-pack`-style behavior that reserves header space based on
/// `char_count * 4` (worst-case UTF-8 bytes).
pub fn write_cbor_text_like_json_pack(out: &mut Vec<u8>, value: &str) {
    let utf8 = value.as_bytes();
    let bytes_len = utf8.len();
    let char_count = value.chars().count();
    let max_size = char_count * 4;

    if max_size <= 23 {
        out.push(0x60u8 | bytes_len as u8);
    } else if max_size <= 0xff {
        out.push(0x78);
        out.push(bytes_len as u8);
    } else if max_size <= 0xffff {
        out.push(0x79);
        out.extend_from_slice(&(bytes_len as u16).to_be_bytes());
    } else {
        out.push(0x7a);
        out.extend_from_slice(&(bytes_len as u32).to_be_bytes());
    }
    out.extend_from_slice(utf8);
}

/// Write a `serde_json::Value` as CBOR to a `Vec<u8>`.
pub fn write_json_like_json_pack(
    out: &mut Vec<u8>,
    value: &serde_json::Value,
) -> Result<(), super::error::CborError> {
    match value {
        serde_json::Value::Null => out.push(0xf6),
        serde_json::Value::Bool(b) => out.push(if *b { 0xf5 } else { 0xf4 }),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                write_cbor_signed(out, i);
            } else if let Some(u) = n.as_u64() {
                write_cbor_uint_major(out, 0, u);
            } else if let Some(f) = n.as_f64() {
                if !f.is_finite() {
                    return Err(super::error::CborError::Unsupported);
                }
                use super::constants::is_f32_roundtrip;
                if is_f32_roundtrip(f) {
                    out.push(0xfa);
                    out.extend_from_slice(&(f as f32).to_bits().to_be_bytes());
                } else {
                    out.push(0xfb);
                    out.extend_from_slice(&f.to_bits().to_be_bytes());
                }
            } else {
                return Err(super::error::CborError::Unsupported);
            }
        }
        serde_json::Value::String(s) => write_cbor_text_like_json_pack(out, s),
        serde_json::Value::Array(items) => {
            write_cbor_uint_major(out, 4, items.len() as u64); // MAJOR_ARRAY
            for item in items {
                write_json_like_json_pack(out, item)?;
            }
        }
        serde_json::Value::Object(map) => {
            write_cbor_uint_major(out, 5, map.len() as u64); // MAJOR_MAP
            for (k, v) in map {
                write_cbor_text_like_json_pack(out, k);
                write_json_like_json_pack(out, v)?;
            }
        }
    }
    Ok(())
}

/// Encode a `serde_json::Value` to CBOR bytes.
pub fn encode_json_to_cbor_bytes(
    value: &serde_json::Value,
) -> Result<Vec<u8>, super::error::CborError> {
    let mut out = Vec::new();
    write_json_like_json_pack(&mut out, value)?;
    Ok(out)
}
