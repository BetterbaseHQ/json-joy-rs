//! RESP2-compatible legacy encoder.
//!
//! Upstream reference: `json-pack/src/resp/RespEncoderLegacy.ts`

use super::encoder::RespEncoder;
use super::{RESP_EXTENSION_ATTRIBUTES, RESP_EXTENSION_PUSH, RESP_EXTENSION_VERBATIM_STRING};
use crate::PackValue;

/// Implements RESP2-style encoding semantics used by upstream legacy encoder.
pub struct RespEncoderLegacy {
    encoder: RespEncoder,
}

impl Default for RespEncoderLegacy {
    fn default() -> Self {
        Self::new()
    }
}

impl RespEncoderLegacy {
    pub fn new() -> Self {
        Self {
            encoder: RespEncoder::new(),
        }
    }

    pub fn encode(&mut self, value: &PackValue) -> Vec<u8> {
        self.write_any(value);
        self.encoder.writer.flush()
    }

    pub fn flush(&mut self) -> Vec<u8> {
        self.encoder.writer.flush()
    }

    pub fn write_any(&mut self, value: &PackValue) {
        match value {
            PackValue::Null | PackValue::Undefined => self.write_null(),
            PackValue::Bool(true) => self.encoder.write_simple_str("TRUE"),
            PackValue::Bool(false) => self.encoder.write_simple_str("FALSE"),
            PackValue::Integer(n) => self.write_number(*n as f64),
            PackValue::UInteger(n) => self.write_unsigned(*n),
            PackValue::Float(f) => self.write_number(*f),
            PackValue::BigInt(n) => self.encoder.write_simple_str(&n.to_string()),
            PackValue::Str(s) => self.write_str(s),
            PackValue::Bytes(buf) => self.encoder.write_bin(buf),
            PackValue::Array(arr) => self.write_arr(arr),
            PackValue::Object(obj) => self.write_obj(obj),
            PackValue::Extension(ext) => match ext.tag {
                RESP_EXTENSION_PUSH => {
                    if let PackValue::Array(arr) = ext.val.as_ref() {
                        self.write_arr(arr);
                    } else {
                        self.write_unknown();
                    }
                }
                RESP_EXTENSION_VERBATIM_STRING => {
                    if let PackValue::Str(s) = ext.val.as_ref() {
                        self.write_str(s);
                    } else {
                        self.write_unknown();
                    }
                }
                RESP_EXTENSION_ATTRIBUTES => {
                    if let PackValue::Object(obj) = ext.val.as_ref() {
                        self.write_obj(obj);
                    } else {
                        self.write_unknown();
                    }
                }
                _ => self.write_unknown(),
            },
            PackValue::Blob(_) => self.write_unknown(),
        }
    }

    pub fn write_null(&mut self) {
        self.encoder.write_null_arr();
    }

    pub fn write_str(&mut self, s: &str) {
        if s.len() < 64 && !s.contains('\r') && !s.contains('\n') {
            self.encoder.write_simple_str(s);
        } else {
            self.encoder.write_bulk_str(s);
        }
    }

    pub fn write_err(&mut self, s: &str) {
        if s.len() < 64 && !s.contains('\r') && !s.contains('\n') {
            self.encoder.write_simple_err(s);
        } else {
            self.encoder.write_bulk_str(s);
        }
    }

    pub fn write_arr(&mut self, arr: &[PackValue]) {
        self.encoder.write_arr_hdr(arr.len());
        for item in arr {
            if matches!(item, PackValue::Null) {
                self.encoder.write_null_str();
            } else {
                self.write_any(item);
            }
        }
    }

    pub fn write_obj(&mut self, obj: &[(String, PackValue)]) {
        self.encoder.write_arr_hdr(obj.len() * 2);
        for (key, value) in obj {
            self.write_str(key);
            if matches!(value, PackValue::Null) {
                self.encoder.write_null_str();
            } else {
                self.write_any(value);
            }
        }
    }

    fn write_unsigned(&mut self, n: u64) {
        if i64::try_from(n).is_ok() {
            self.encoder.write_integer(n as i64);
        } else {
            self.encoder.write_simple_str(&n.to_string());
        }
    }

    fn write_number(&mut self, n: f64) {
        let is_safe_integer = n.fract() == 0.0 && n.abs() <= 9_007_199_254_740_991.0;
        if is_safe_integer && n >= i64::MIN as f64 && n <= i64::MAX as f64 {
            self.encoder.write_integer(n as i64);
        } else {
            self.encoder.write_simple_str(&n.to_string());
        }
    }

    fn write_unknown(&mut self) {
        self.write_null();
    }
}
