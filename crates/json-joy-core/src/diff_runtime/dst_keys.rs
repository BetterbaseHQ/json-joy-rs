pub fn diff_model_dst_keys_to_patch_bytes(
    base_model_binary: &[u8],
    dst_keys_view: &Value,
    sid: u64,
) -> Result<Option<Vec<u8>>, DiffError> {
    let dst_obj = match dst_keys_view.as_object() {
        Some(v) => v,
        None => return diff_model_to_patch_bytes(base_model_binary, dst_keys_view, sid),
    };

    let model = match Model::from_binary(base_model_binary) {
        Ok(v) => v,
        Err(_) => return diff_model_to_patch_bytes(base_model_binary, dst_keys_view, sid),
    };
    let mut next_obj = match model.view().as_object() {
        Some(v) => v.clone(),
        None => serde_json::Map::new(),
    };

    // Upstream JsonCrdtDiff.diffDstKeys updates only destination keys.
    for (k, v) in dst_obj {
        next_obj.insert(k.clone(), v.clone());
    }
    diff_model_to_patch_bytes(base_model_binary, &Value::Object(next_obj), sid)
}
