use json_joy_core::diff_runtime::diff_model_to_patch_bytes;
use json_joy_core::model_runtime::RuntimeModel;
use json_joy_core::patch::{ConValue, DecodedOp, Patch, Timestamp};
use json_joy_core::patch_builder::encode_patch_from_ops;

// Upstream references:
// - /Users/nchapman/Code/json-joy/packages/json-joy/src/json-crdt-diff/JsonCrdtDiff.ts (diffVec)
// - /Users/nchapman/Code/json-joy/packages/json-joy/src/json-crdt-diff/__tests__/JsonCrdtDiff.spec.ts ("replaces con member with another con node")

#[test]
fn upstream_port_diff_vec_matrix_con_string_replacement_uses_con_not_str_node() {
    let sid = 88941;
    let mut runtime = RuntimeModel::new_logical_empty(sid);
    let root = Timestamp { sid, time: 1 };
    let vec_id = Timestamp { sid, time: 3 };
    let c1 = Timestamp { sid, time: 4 };
    let ops = vec![
        DecodedOp::NewObj { id: root },
        DecodedOp::InsVal {
            id: Timestamp { sid, time: 2 },
            obj: Timestamp { sid: 0, time: 0 },
            val: root,
        },
        DecodedOp::NewVec { id: vec_id },
        DecodedOp::NewCon {
            id: c1,
            value: ConValue::Json(serde_json::json!("abc")),
        },
        DecodedOp::InsVec {
            id: Timestamp { sid, time: 5 },
            obj: vec_id,
            data: vec![(0, c1)],
        },
        DecodedOp::InsObj {
            id: Timestamp { sid, time: 6 },
            obj: root,
            data: vec![("v".to_string(), vec_id)],
        },
    ];
    let seed = encode_patch_from_ops(sid, 1, &ops).expect("seed patch encode must succeed");
    let seed_patch = Patch::from_binary(&seed).expect("seed patch decode must succeed");
    runtime
        .apply_patch(&seed_patch)
        .expect("seed apply must succeed");
    let base_model = runtime
        .to_model_binary_like()
        .expect("runtime model encode must succeed");
    let next = serde_json::json!({"v": ["xyz"]});

    let patch = diff_model_to_patch_bytes(&base_model, &next, sid)
        .expect("diff should succeed")
        .expect("non-noop diff expected");
    let decoded = Patch::from_binary(&patch).expect("generated patch must decode");
    assert!(
        decoded
            .decoded_ops()
            .iter()
            .any(|op| matches!(op, DecodedOp::InsVec { .. })),
        "vec replacement should update index via ins_vec"
    );
    assert!(
        decoded.decoded_ops().iter().any(
            |op| matches!(op, DecodedOp::NewCon { value, .. } if *value == ConValue::Json(serde_json::json!("xyz")))
        ),
        "con->string replacement in vec should allocate new_con(\"xyz\")"
    );
    assert!(
        !decoded
            .decoded_ops()
            .iter()
            .any(|op| matches!(op, DecodedOp::NewStr { .. } | DecodedOp::InsStr { .. })),
        "con->string replacement in vec should not allocate str nodes"
    );

    let mut applied =
        RuntimeModel::from_model_binary(&base_model).expect("runtime decode must succeed");
    applied
        .apply_patch(&decoded)
        .expect("runtime apply must succeed");
    assert_eq!(applied.view_json(), next);
}
