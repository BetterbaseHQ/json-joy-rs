pub fn diff_model_dst_keys_to_patch_bytes(
    base_model_binary: &[u8],
    dst_keys_view: &Value,
    sid: u64,
) -> Result<Option<Vec<u8>>, DiffError> {
    let model = Model::from_binary(base_model_binary).map_err(|_| DiffError::UnsupportedShape)?;
    let mut next_obj = model
        .view()
        .as_object()
        .cloned()
        .ok_or(DiffError::UnsupportedShape)?;
    let dst_obj = dst_keys_view.as_object().ok_or(DiffError::UnsupportedShape)?;

    // Upstream JsonCrdtDiff.diffDstKeys updates only destination keys.
    for (k, v) in dst_obj {
        next_obj.insert(k.clone(), v.clone());
    }
    diff_model_to_patch_bytes(base_model_binary, &Value::Object(next_obj), sid)
}
