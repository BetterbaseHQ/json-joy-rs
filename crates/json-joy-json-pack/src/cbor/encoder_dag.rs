//! `CborEncoderDag` — DAG-JSON CBOR encoder.
//!
//! Direct port of `cbor/CborEncoderDag.ts` from upstream.
//! Extends `CborEncoderStable`:
//! - NaN and Infinity → null
//! - Only writes tag header for tag 42 (CID); other tags are passed through

use json_joy_buffers::Writer;

use super::encoder_stable::CborEncoderStable;

/// DAG-JSON CBOR encoder.
///
/// Same as [`CborEncoderStable`] but:
/// - Floats that are NaN or infinite are encoded as null
/// - Only tag 42 gets a tag header; all other tags are unwrapped
pub struct CborEncoderDag {
    stable: CborEncoderStable,
}

impl Default for CborEncoderDag {
    fn default() -> Self {
        Self::new()
    }
}

impl CborEncoderDag {
    pub fn new() -> Self {
        Self {
            stable: CborEncoderStable::new(),
        }
    }

    pub fn writer(&mut self) -> &mut Writer {
        &mut self.stable.writer
    }

    pub fn encode(&mut self, value: &crate::PackValue) -> Vec<u8> {
        self.stable.writer.reset();
        self.write_any(value);
        self.stable.writer.flush()
    }

    pub fn encode_json(&mut self, value: &serde_json::Value) -> Vec<u8> {
        self.stable.writer.reset();
        self.write_any(&crate::PackValue::from(value.clone()));
        self.stable.writer.flush()
    }

    pub fn write_any(&mut self, value: &crate::PackValue) {
        use crate::PackValue::*;
        match value {
            Null | Undefined => self.stable.write_null(),
            Bool(b) => self.stable.write_boolean(*b),
            Integer(i) => self.stable.write_integer(*i),
            UInteger(u) => self.stable.write_u_integer(*u),
            Float(f) => self.write_float(*f),
            BigInt(i) => self.stable.write_big_int(*i),
            Bytes(b) => self.stable.write_bin(b),
            Str(s) => self.stable.write_str(s),
            Array(arr) => {
                self.stable.write_arr_hdr(arr.len());
                for item in arr {
                    self.write_any(item);
                }
            }
            Object(obj) => {
                let mut sorted: Vec<&(String, crate::PackValue)> = obj.iter().collect();
                sorted.sort_by(|a, b| a.0.len().cmp(&b.0.len()).then_with(|| a.0.cmp(&b.0)));
                self.stable.write_obj_hdr(sorted.len());
                for (key, val) in sorted {
                    self.stable.write_str(key);
                    self.write_any(val);
                }
            }
            Extension(ext) => self.write_tag(ext.tag, &ext.val),
            Blob(blob) => self.stable.writer.buf(&blob.val),
        }
    }

    /// DAG float: NaN and Infinity → null; otherwise write as f64.
    pub fn write_float(&mut self, float: f64) {
        if float.is_nan() || !float.is_finite() {
            self.stable.write_null();
        } else {
            self.stable.writer.u8f64(0xfb, float);
        }
    }

    /// DAG tag: only tag 42 gets a tag header; all others unwrap.
    pub fn write_tag(&mut self, tag: u64, value: &crate::PackValue) {
        if tag == 42 {
            self.stable.write_tag_hdr(tag);
        }
        self.write_any(value);
    }
}
