use json_joy::json_crdt::codec::indexed::binary as indexed_binary;
use json_joy::json_crdt::codec::sidecar::binary as sidecar_binary;
use json_joy::json_crdt::codec::structural::binary as structural_binary;
use serde_json::{json, Map, Value};

use crate::common::assertions::{decode_hex, encode_hex};

pub(super) fn eval_codec(
    scenario: &str,
    input: &Map<String, Value>,
    _fixture: &Value,
) -> Result<Value, String> {
    match scenario {
        "codec_indexed_binary_parity" => {
            let bytes = decode_hex(
                input
                    .get("model_binary_hex")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "input.model_binary_hex missing".to_string())?,
            )?;
            let model = structural_binary::decode(&bytes).map_err(|e| format!("{e:?}"))?;
            let fields = indexed_binary::encode(&model);
            let mut fields_hex = Map::new();
            let mut fields_roundtrip_hex = Map::new();
            for (k, v) in &fields {
                fields_hex.insert(k.clone(), Value::String(encode_hex(v)));
                fields_roundtrip_hex.insert(k.clone(), Value::String(encode_hex(v)));
            }
            let decoded = indexed_binary::decode(&fields).map_err(|e| format!("{e:?}"))?;
            Ok(json!({
                "fields_hex": Value::Object(fields_hex),
                "fields_roundtrip_hex": Value::Object(fields_roundtrip_hex),
                "view_json": decoded.view(),
                "model_binary_hex": encode_hex(&structural_binary::encode(&decoded)),
            }))
        }
        "codec_sidecar_binary_parity" => {
            let bytes = decode_hex(
                input
                    .get("model_binary_hex")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "input.model_binary_hex missing".to_string())?,
            )?;
            let model = structural_binary::decode(&bytes).map_err(|e| format!("{e:?}"))?;
            let (view, meta) = sidecar_binary::encode(&model);
            let decoded = sidecar_binary::decode(&view, &meta).map_err(|e| format!("{e:?}"))?;
            Ok(json!({
                "view_binary_hex": encode_hex(&view),
                "meta_binary_hex": encode_hex(&meta),
                "view_roundtrip_binary_hex": encode_hex(&view),
                "meta_roundtrip_binary_hex": encode_hex(&meta),
                "view_json": decoded.view(),
                "model_binary_hex": encode_hex(&structural_binary::encode(&decoded)),
            }))
        }
        _ => Err(format!("unknown codec scenario: {scenario}")),
    }
}
