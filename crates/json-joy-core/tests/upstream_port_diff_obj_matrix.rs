use json_joy_core::diff_runtime::diff_model_to_patch_bytes;
use json_joy_core::less_db_compat::{create_model, model_to_binary};
use json_joy_core::model_runtime::RuntimeModel;
use json_joy_core::patch::{ConValue, DecodedOp, Patch, Timestamp};

// Upstream references:
// - /Users/nchapman/Code/json-joy/packages/json-joy/src/json-crdt-diff/JsonCrdtDiff.ts (diffObj)
// - /Users/nchapman/Code/json-joy/packages/json-joy/src/json-crdt-diff/__tests__/JsonCrdtDiff.spec.ts (obj cases)

#[test]
fn upstream_port_diff_obj_matrix_ins_obj_tuples_follow_delete_then_dst_order() {
    let sid = 88921;
    let initial = serde_json::json!({
        "a": 1,
        "b": "x",
        "c": true
    });
    let next = serde_json::json!({
        "b": "x",
        "d": 2,
        "a": 3
    });
    let model = create_model(&initial, sid).expect("create_model must succeed");
    let base_model = model_to_binary(&model);

    let patch = diff_model_to_patch_bytes(&base_model, &next, sid)
        .expect("diff should succeed")
        .expect("non-noop diff expected");
    let decoded = Patch::from_binary(&patch).expect("generated patch must decode");

    let ins_obj = decoded
        .decoded_ops()
        .iter()
        .find_map(|op| match op {
            DecodedOp::InsObj { data, .. } => Some(data),
            _ => None,
        })
        .expect("expected at least one ins_obj");
    let keys: Vec<&str> = ins_obj.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(
        keys,
        vec!["c", "d", "a"],
        "diffObj should emit delete tuples first, then destination traversal order"
    );

    let mut undef_id: Option<Timestamp> = None;
    for op in decoded.decoded_ops() {
        if let DecodedOp::NewCon {
            id,
            value: ConValue::Undef,
        } = op
        {
            undef_id = Some(*id);
            break;
        }
    }
    let undef_id = undef_id.expect("expected new_con undef for deleted key");
    assert_eq!(
        ins_obj[0].1, undef_id,
        "deleted key tuple should point to undef constant"
    );

    let mut applied =
        RuntimeModel::from_model_binary(&base_model).expect("runtime decode must succeed");
    applied
        .apply_patch(&decoded)
        .expect("runtime apply must succeed");
    assert_eq!(applied.view_json(), next);
}

#[test]
fn upstream_port_diff_obj_matrix_nested_object_delta_prefers_child_recursion_over_root_replace() {
    let sid = 88922;
    let initial = serde_json::json!({
        "doc": {
            "x": 1,
            "y": 2
        }
    });
    let next = serde_json::json!({
        "doc": {
            "x": 1,
            "y": 3
        }
    });
    let model = create_model(&initial, sid).expect("create_model must succeed");
    let base_model = model_to_binary(&model);

    let patch = diff_model_to_patch_bytes(&base_model, &next, sid)
        .expect("diff should succeed")
        .expect("non-noop diff expected");
    let decoded = Patch::from_binary(&patch).expect("generated patch must decode");

    assert!(
        !decoded.decoded_ops().iter().any(
            |op| matches!(op, DecodedOp::InsObj { data, .. } if data.iter().any(|(k, _)| k == "doc"))
        ),
        "nested object scalar delta should recurse into child object, not replace root field"
    );

    let mut applied =
        RuntimeModel::from_model_binary(&base_model).expect("runtime decode must succeed");
    applied
        .apply_patch(&decoded)
        .expect("runtime apply must succeed");
    assert_eq!(applied.view_json(), next);
}

