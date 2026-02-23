use json_joy::util_inner::diff::{bin as bin_diff, line as line_diff, str as str_diff};
use serde_json::{json, Map, Value};

pub(super) fn eval_util(
    scenario: &str,
    input: &Map<String, Value>,
    _fixture: &Value,
) -> Result<Value, String> {
    match scenario {
        "util_diff_parity" => {
            let kind = input
                .get("kind")
                .and_then(Value::as_str)
                .ok_or_else(|| "input.kind missing".to_string())?;
            match kind {
                "str" => {
                    let src = input
                        .get("src")
                        .and_then(Value::as_str)
                        .ok_or_else(|| "src".to_string())?;
                    let dst = input
                        .get("dst")
                        .and_then(Value::as_str)
                        .ok_or_else(|| "dst".to_string())?;
                    let patch = str_diff::diff(src, dst);
                    let patch_json = Value::Array(
                        patch
                            .iter()
                            .map(|(op, txt)| {
                                Value::Array(vec![
                                    Value::from(*op as i64),
                                    Value::String(txt.clone()),
                                ])
                            })
                            .collect(),
                    );
                    Ok(json!({
                        "patch": patch_json,
                        "src_from_patch": str_diff::patch_src(&patch),
                        "dst_from_patch": str_diff::patch_dst(&patch),
                    }))
                }
                "bin" => {
                    let src: Vec<u8> = input
                        .get("src")
                        .and_then(Value::as_array)
                        .ok_or_else(|| "src".to_string())?
                        .iter()
                        .map(|v| v.as_u64().unwrap_or(0) as u8)
                        .collect();
                    let dst: Vec<u8> = input
                        .get("dst")
                        .and_then(Value::as_array)
                        .ok_or_else(|| "dst".to_string())?
                        .iter()
                        .map(|v| v.as_u64().unwrap_or(0) as u8)
                        .collect();
                    let patch = bin_diff::diff(&src, &dst);
                    let patch_json = Value::Array(
                        patch
                            .iter()
                            .map(|(op, txt)| {
                                Value::Array(vec![
                                    Value::from(*op as i64),
                                    Value::String(txt.clone()),
                                ])
                            })
                            .collect(),
                    );
                    Ok(json!({
                        "patch": patch_json,
                        "src_from_patch": Value::Array(bin_diff::patch_src(&patch).into_iter().map(Value::from).collect()),
                        "dst_from_patch": Value::Array(bin_diff::patch_dst(&patch).into_iter().map(Value::from).collect()),
                    }))
                }
                "line" => {
                    let src: Vec<&str> = input
                        .get("src")
                        .and_then(Value::as_array)
                        .ok_or_else(|| "src".to_string())?
                        .iter()
                        .map(|v| v.as_str().unwrap_or(""))
                        .collect();
                    let dst: Vec<&str> = input
                        .get("dst")
                        .and_then(Value::as_array)
                        .ok_or_else(|| "dst".to_string())?
                        .iter()
                        .map(|v| v.as_str().unwrap_or(""))
                        .collect();
                    let patch = line_diff::diff(&src, &dst);
                    let patch_json = Value::Array(
                        patch
                            .iter()
                            .map(|(op, s, d)| {
                                Value::Array(vec![
                                    Value::from(*op as i64),
                                    Value::from(*s),
                                    Value::from(*d),
                                ])
                            })
                            .collect(),
                    );
                    Ok(json!({ "patch": patch_json }))
                }
                other => Err(format!("unsupported util_diff kind {other}")),
            }
        }
        _ => Err(format!("unknown util scenario: {scenario}")),
    }
}
