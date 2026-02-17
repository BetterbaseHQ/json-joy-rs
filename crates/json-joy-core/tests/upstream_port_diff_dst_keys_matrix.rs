use json_joy_core::diff_runtime::diff_model_dst_keys_to_patch_bytes;
use json_joy_core::less_db_compat::{create_model, model_to_binary};
use json_joy_core::model_runtime::RuntimeModel;
use json_joy_core::patch::{DecodedOp, Patch};

// Upstream references:
// - /Users/nchapman/Code/json-joy/packages/json-joy/src/json-crdt-diff/JsonCrdtDiff.ts (diffDstKeys)
// - /Users/nchapman/Code/json-joy/packages/json-joy/src/json-crdt/model/api/nodes.ts

#[test]
fn upstream_port_diff_dst_keys_matrix_updates_only_destination_keys() {
    let sid = 88971;
    let initial = serde_json::json!({
        "a": 1,
        "b": "keep",
        "c": {"x": 1}
    });
    let dst = serde_json::json!({
        "a": 2,
        "d": true
    });
    let model = create_model(&initial, sid).expect("create_model must succeed");
    let base_model = model_to_binary(&model);

    let patch = diff_model_dst_keys_to_patch_bytes(&base_model, &dst, sid)
        .expect("diffDstKeys should succeed")
        .expect("non-noop diff expected");
    let decoded = Patch::from_binary(&patch).expect("generated patch must decode");

    // diffDstKeys should not remove or rewrite unspecified keys.
    assert!(
        !decoded
            .decoded_ops()
            .iter()
            .any(|op| matches!(op, DecodedOp::InsObj { data, .. } if data.iter().any(|(k, _)| k == "b" || k == "c"))),
        "unspecified keys must not be touched"
    );

    let mut applied =
        RuntimeModel::from_model_binary(&base_model).expect("runtime decode must succeed");
    applied
        .apply_patch(&decoded)
        .expect("runtime apply must succeed");
    assert_eq!(
        applied.view_json(),
        serde_json::json!({
            "a": 2,
            "b": "keep",
            "c": {"x": 1},
            "d": true
        })
    );
}

#[test]
fn upstream_port_diff_dst_keys_matrix_noop_when_all_dst_keys_equal() {
    let sid = 88972;
    let initial = serde_json::json!({"a": 1, "b": 2});
    let dst = serde_json::json!({"a": 1});
    let model = create_model(&initial, sid).expect("create_model must succeed");
    let base_model = model_to_binary(&model);

    let patch = diff_model_dst_keys_to_patch_bytes(&base_model, &dst, sid)
        .expect("diffDstKeys should succeed");
    assert!(
        patch.is_none(),
        "equal destination keys should produce no-op patch"
    );
}

