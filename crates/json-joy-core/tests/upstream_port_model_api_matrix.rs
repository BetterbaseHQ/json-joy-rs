use json_joy_core::model_api::{NativeModelApi, PathStep};
use json_joy_core::patch::{DecodedOp, Patch, Timestamp};
use json_joy_core::patch_builder::encode_patch_from_ops;
use serde_json::json;

fn patch_from_ops(sid: u64, time: u64, ops: &[DecodedOp]) -> Patch {
    let bytes = encode_patch_from_ops(sid, time, ops).expect("patch encode must succeed");
    Patch::from_binary(&bytes).expect("patch decode must succeed")
}

#[test]
fn upstream_port_model_api_from_patches_and_apply_batch() {
    // Upstream mapping:
    // - json-crdt/model/Model.ts `fromPatches`, `applyBatch`
    let sid = 97001;
    let root = Timestamp { sid, time: 1 };
    let hello = Timestamp { sid, time: 3 };
    let world = Timestamp { sid, time: 5 };

    let p1_ops = vec![
        DecodedOp::NewObj { id: root },
        DecodedOp::InsVal {
            id: Timestamp { sid, time: 2 },
            obj: Timestamp { sid: 0, time: 0 },
            val: root,
        },
        DecodedOp::NewCon {
            id: hello,
            value: json_joy_core::patch::ConValue::Json(json!("hello")),
        },
        DecodedOp::InsObj {
            id: Timestamp { sid, time: 4 },
            obj: root,
            data: vec![("msg".into(), hello)],
        },
    ];
    let p2_ops = vec![
        DecodedOp::NewCon {
            id: world,
            value: json_joy_core::patch::ConValue::Json(json!("world")),
        },
        DecodedOp::InsObj {
            id: Timestamp { sid, time: 6 },
            obj: root,
            data: vec![("next".into(), world)],
        },
    ];
    let p1 = patch_from_ops(sid, 1, &p1_ops);
    let p2 = patch_from_ops(sid, 5, &p2_ops);

    let mut api = NativeModelApi::from_patches(std::slice::from_ref(&p1)).expect("from_patches must succeed");
    api.apply_batch(std::slice::from_ref(&p2))
        .expect("apply_batch must succeed");

    assert_eq!(api.view(), json!({"msg":"hello","next":"world"}));
}

#[test]
fn upstream_port_model_api_find_path_matrix() {
    // Upstream mapping:
    // - json-crdt/model/api/find.ts
    let sid = 97002;
    let mut api = NativeModelApi::from_patches(&[patch_from_ops(
        sid,
        1,
        &[
            DecodedOp::NewObj {
                id: Timestamp { sid, time: 1 },
            },
            DecodedOp::InsVal {
                id: Timestamp { sid, time: 2 },
                obj: Timestamp { sid: 0, time: 0 },
                val: Timestamp { sid, time: 1 },
            },
            DecodedOp::NewObj {
                id: Timestamp { sid, time: 3 },
            },
            DecodedOp::NewArr {
                id: Timestamp { sid, time: 4 },
            },
            DecodedOp::NewCon {
                id: Timestamp { sid, time: 5 },
                value: json_joy_core::patch::ConValue::Json(json!(1)),
            },
            DecodedOp::InsArr {
                id: Timestamp { sid, time: 6 },
                obj: Timestamp { sid, time: 4 },
                reference: Timestamp { sid, time: 4 },
                data: vec![Timestamp { sid, time: 5 }],
            },
            DecodedOp::InsObj {
                id: Timestamp { sid, time: 7 },
                obj: Timestamp { sid, time: 3 },
                data: vec![("items".into(), Timestamp { sid, time: 4 })],
            },
            DecodedOp::InsObj {
                id: Timestamp { sid, time: 8 },
                obj: Timestamp { sid, time: 1 },
                data: vec![("doc".into(), Timestamp { sid, time: 3 })],
            },
        ],
    )])
    .expect("from_patches must succeed");

    assert_eq!(
        api.find(&[
            PathStep::Key("doc".into()),
            PathStep::Key("items".into()),
            PathStep::Index(0),
        ]),
        Some(json!(1))
    );
    api.arr_push(
        &[PathStep::Key("doc".into()), PathStep::Key("items".into())],
        json!(2),
    )
    .expect("arr_push must succeed");
    assert_eq!(
        api.find(&[
            PathStep::Key("doc".into()),
            PathStep::Key("items".into()),
            PathStep::Index(1),
        ]),
        Some(json!(2))
    );
}

#[test]
fn upstream_port_model_api_mutator_matrix() {
    // Upstream mapping:
    // - json-crdt/model/api/nodes.ts (set/object/array/string style edits)
    let sid = 97003;
    let mut api = NativeModelApi::from_patches(&[patch_from_ops(
        sid,
        1,
        &[
            DecodedOp::NewObj {
                id: Timestamp { sid, time: 1 },
            },
            DecodedOp::InsVal {
                id: Timestamp { sid, time: 2 },
                obj: Timestamp { sid: 0, time: 0 },
                val: Timestamp { sid, time: 1 },
            },
        ],
    )])
    .expect("from_patches must succeed");

    api.obj_put(&[], "title", json!("hello"))
        .expect("obj_put must succeed");
    api.obj_put(&[], "list", json!([1]))
        .expect("obj_put must succeed");
    api.arr_push(&[PathStep::Key("list".into())], json!(2))
        .expect("arr_push must succeed");
    api.obj_put(&[], "name", json!("ab"))
        .expect("obj_put must succeed");
    api.str_ins(&[PathStep::Key("name".into())], 1, "Z")
        .expect("str_ins must succeed");
    api.set(&[PathStep::Key("title".into())], json!("world"))
        .expect("set must succeed");

    assert_eq!(api.view(), json!({"title":"world","list":[1,2],"name":"aZb"}));
}
