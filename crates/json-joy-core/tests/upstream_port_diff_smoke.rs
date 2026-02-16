use json_joy_core::diff_runtime::diff_model_to_patch_bytes;
use json_joy_core::less_db_compat::{create_model, model_to_binary};
use json_joy_core::model_runtime::RuntimeModel;
use json_joy_core::patch::Patch;

// Upstream references:
// - /Users/nchapman/Code/json-joy/packages/json-joy/src/json-crdt-diff/JsonCrdtDiff.ts
// - /Users/nchapman/Code/json-joy/packages/json-joy/src/json-crdt/model/Model.ts (api.diff/applyPatch)

#[test]
fn upstream_port_diff_noop_on_equal_object_returns_none() {
    // model_roundtrip_empty_object_v1 fixture payload (sid=73012).
    let base_model = decode_hex("00000002114001b4ba0402");

    let patch = diff_model_to_patch_bytes(&base_model, &serde_json::json!({}), 73012)
        .expect("diff should succeed");
    assert!(patch.is_none(), "equal object diff must be None");
}

#[test]
fn upstream_port_diff_apply_reaches_target_view() {
    // model_roundtrip_empty_object_v1 fixture payload (sid=73012).
    let base_model = decode_hex("00000002114001b4ba0402");
    let next = serde_json::json!({"a": 1, "b": "x"});

    let patch = diff_model_to_patch_bytes(&base_model, &next, 73012)
        .expect("diff should succeed")
        .expect("non-noop diff expected");

    let mut runtime = RuntimeModel::from_model_binary(&base_model).expect("runtime decode must succeed");
    let decoded = Patch::from_binary(&patch).expect("generated patch must decode");
    runtime.apply_patch(&decoded).expect("runtime apply must succeed");

    assert_eq!(runtime.view_json(), next);
}

#[test]
fn upstream_port_diff_nested_object_delta_reaches_target_view() {
    // Ports upstream JsonCrdtDiff.diffObj behavior for nested object edits:
    // source-key delete writes first, then destination-key inserts/updates.
    let sid = 88001;
    let initial = serde_json::json!({"doc": {"a": 1, "b": "x", "c": true}});
    let next = serde_json::json!({"doc": {"a": 2, "b": "x", "d": null}});
    let model = create_model(&initial, sid).expect("create_model must succeed");
    let base_model = model_to_binary(&model);

    let patch = diff_model_to_patch_bytes(&base_model, &next, sid)
        .expect("diff should succeed")
        .expect("non-noop diff expected");

    let mut runtime = RuntimeModel::from_model_binary(&base_model).expect("runtime decode must succeed");
    let decoded = Patch::from_binary(&patch).expect("generated patch must decode");
    runtime.apply_patch(&decoded).expect("runtime apply must succeed");
    assert_eq!(runtime.view_json(), next);
}

#[test]
fn upstream_port_diff_multi_array_key_delta_reaches_target_view() {
    // Ports multiple ArrNode edits in one root object diff pass.
    let sid = 88002;
    let initial = serde_json::json!({
        "a": [1, "x", 2],
        "b": ["q", 9]
    });
    let next = serde_json::json!({
        "a": [1, "x", "y", 2],
        "b": [9]
    });
    let model = create_model(&initial, sid).expect("create_model must succeed");
    let base_model = model_to_binary(&model);

    let patch = diff_model_to_patch_bytes(&base_model, &next, sid)
        .expect("diff should succeed")
        .expect("non-noop diff expected");

    let mut runtime = RuntimeModel::from_model_binary(&base_model).expect("runtime decode must succeed");
    let decoded = Patch::from_binary(&patch).expect("generated patch must decode");
    runtime.apply_patch(&decoded).expect("runtime apply must succeed");
    assert_eq!(runtime.view_json(), next);
}

fn decode_hex(s: &str) -> Vec<u8> {
    assert!(s.len() % 2 == 0, "hex string must have even length");
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    for i in (0..bytes.len()).step_by(2) {
        let hi = (bytes[i] as char).to_digit(16).expect("invalid hex") as u8;
        let lo = (bytes[i + 1] as char).to_digit(16).expect("invalid hex") as u8;
        out.push((hi << 4) | lo);
    }
    out
}
