//! Native compact-binary patch codec port (`codec/compact-binary/*`).

use crate::patch::Patch;
use crate::patch_compact_codec::{decode_patch_compact, encode_patch_compact, CompactCodecError};
use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum CompactBinaryCodecError {
    #[error("compact codec failed: {0}")]
    Compact(#[from] CompactCodecError),
    #[error("invalid compact-binary cbor payload")]
    InvalidCbor,
}

pub fn encode_patch_compact_binary(patch: &Patch) -> Result<Vec<u8>, CompactBinaryCodecError> {
    let compact = encode_patch_compact(patch)?;
    let mut out = Vec::with_capacity(256);
    write_json_like_json_pack(&compact, &mut out);
    Ok(out)
}

pub fn decode_patch_compact_binary(data: &[u8]) -> Result<Patch, CompactBinaryCodecError> {
    let compact: serde_json::Value =
        ciborium::de::from_reader(data).map_err(|_| CompactBinaryCodecError::InvalidCbor)?;
    Ok(decode_patch_compact(&compact)?)
}

fn write_json_like_json_pack(value: &Value, out: &mut Vec<u8>) {
    match value {
        Value::Null => out.push(0xf6),
        Value::Bool(b) => out.push(if *b { 0xf5 } else { 0xf4 }),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                write_cbor_signed(out, i);
            } else if let Some(u) = n.as_u64() {
                write_cbor_uint_major(out, 0, u);
            } else {
                let f = n.as_f64().unwrap_or(0.0);
                if is_f32_roundtrip(f) {
                    out.push(0xfa);
                    out.extend_from_slice(&(f as f32).to_bits().to_be_bytes());
                } else {
                    out.push(0xfb);
                    out.extend_from_slice(&f.to_bits().to_be_bytes());
                }
            }
        }
        Value::String(s) => write_cbor_text_like_json_pack(out, s),
        Value::Array(arr) => {
            write_cbor_uint_major(out, 4, arr.len() as u64);
            for item in arr {
                write_json_like_json_pack(item, out);
            }
        }
        Value::Object(map) => {
            write_cbor_uint_major(out, 5, map.len() as u64);
            for (k, v) in map {
                write_cbor_text_like_json_pack(out, k);
                write_json_like_json_pack(v, out);
            }
        }
    }
}

fn write_cbor_text_like_json_pack(out: &mut Vec<u8>, value: &str) {
    let utf8 = value.as_bytes();
    let bytes_len = utf8.len();
    let max_size = value.chars().count().saturating_mul(4);

    if max_size <= 23 {
        out.push(0x60u8.saturating_add(bytes_len as u8));
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

fn write_cbor_uint_major(out: &mut Vec<u8>, major: u8, n: u64) {
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

fn write_cbor_signed(out: &mut Vec<u8>, n: i64) {
    if n >= 0 {
        write_cbor_uint_major(out, 0, n as u64);
    } else {
        let encoded = (-1i128 - n as i128) as u64;
        write_cbor_uint_major(out, 1, encoded);
    }
}

fn is_f32_roundtrip(value: f64) -> bool {
    if !value.is_finite() {
        return false;
    }
    (value as f32) as f64 == value
}
