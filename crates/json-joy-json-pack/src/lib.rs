//! CBOR-focused json-pack primitives for workspace-wide reuse.
//!
//! This crate intentionally starts with a compact API:
//! - decode/encode CBOR values,
//! - convert between CBOR values and `serde_json::Value`.
//!
//! It is intended to become the shared CBOR foundation for runtime crates.

use ciborium::value::Value as CborValue;
use serde_json::{Map, Number, Value};
use std::convert::TryFrom;
use std::io::Cursor;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CborError {
    #[error("invalid cbor payload")]
    InvalidPayload,
    #[error("unsupported cbor feature for json conversion")]
    Unsupported,
}

pub fn decode_cbor_value(bytes: &[u8]) -> Result<CborValue, CborError> {
    let mut cursor = Cursor::new(bytes);
    ciborium::de::from_reader::<CborValue, _>(&mut cursor).map_err(|_| CborError::InvalidPayload)
}

pub fn encode_cbor_value(value: &CborValue) -> Result<Vec<u8>, CborError> {
    let mut out = Vec::new();
    ciborium::ser::into_writer(value, &mut out).map_err(|_| CborError::InvalidPayload)?;
    Ok(out)
}

pub fn cbor_to_json(v: &CborValue) -> Result<Value, CborError> {
    Ok(match v {
        CborValue::Null => Value::Null,
        CborValue::Bool(b) => Value::Bool(*b),
        CborValue::Integer(i) => {
            let signed: i128 = (*i).into();
            if signed >= 0 {
                Value::Number(Number::from(
                    u64::try_from(signed).map_err(|_| CborError::Unsupported)?,
                ))
            } else {
                Value::Number(Number::from(
                    i64::try_from(signed).map_err(|_| CborError::Unsupported)?,
                ))
            }
        }
        CborValue::Float(f) => Number::from_f64(*f)
            .map(Value::Number)
            .ok_or(CborError::Unsupported)?,
        CborValue::Text(s) => Value::String(s.clone()),
        CborValue::Bytes(bytes) => Value::Array(
            bytes
                .iter()
                .copied()
                .map(|b| Value::Number(Number::from(b)))
                .collect(),
        ),
        CborValue::Array(items) => Value::Array(
            items
                .iter()
                .map(cbor_to_json)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        CborValue::Map(entries) => {
            let mut out = Map::new();
            for (k, v) in entries {
                let key = match k {
                    CborValue::Text(s) => s.clone(),
                    _ => return Err(CborError::Unsupported),
                };
                out.insert(key, cbor_to_json(v)?);
            }
            Value::Object(out)
        }
        CborValue::Tag(_, _) => return Err(CborError::Unsupported),
        _ => return Err(CborError::Unsupported),
    })
}

pub fn json_to_cbor(v: &Value) -> CborValue {
    match v {
        Value::Null => CborValue::Null,
        Value::Bool(b) => CborValue::Bool(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                CborValue::Integer(i.into())
            } else if let Some(u) = n.as_u64() {
                CborValue::Integer(u.into())
            } else {
                CborValue::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        Value::String(s) => CborValue::Text(s.clone()),
        Value::Array(arr) => CborValue::Array(arr.iter().map(json_to_cbor).collect()),
        Value::Object(map) => CborValue::Map(
            map.iter()
                .map(|(k, v)| (CborValue::Text(k.clone()), json_to_cbor(v)))
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
            let cbor = json_to_cbor(&case);
            let bin = encode_cbor_value(&cbor).expect("encode cbor");
            let decoded = decode_cbor_value(&bin).expect("decode cbor");
            let back = cbor_to_json(&decoded).expect("cbor to json");
            assert_eq!(back, case);
        }
    }
}
