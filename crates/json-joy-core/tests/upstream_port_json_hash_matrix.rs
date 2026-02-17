use json_joy_core::json_hash::{hash_bin, hash_json, hash_str, struct_hash_crdt, struct_hash_json, struct_hash_schema};
use json_joy_core::model_runtime::RuntimeModel;
use json_joy_core::patch::Timestamp;
use json_joy_core::schema::{
    arr_node, bin_node, con_json, obj_node, str_node, val_node, vec_node, SchemaNode,
};
use serde_json::json;

fn runtime_from_schema(schema: &SchemaNode, sid: u64) -> RuntimeModel {
    let mut runtime = RuntimeModel::new_logical_empty(sid);
    let patch = schema.to_patch(sid, 1).expect("schema patch");
    runtime.apply_patch(&patch).expect("apply patch");
    runtime
}

#[test]
fn upstream_port_json_hash_matrix_object_key_order_is_stable() {
    // Upstream reference:
    // /Users/nchapman/Code/json-joy/packages/json-joy/src/json-hash/__tests__/hash.spec.ts
    let a = json!({"a": 1, "b": 2});
    let b = json!({"b": 2, "a": 1});
    assert_eq!(hash_json(&a), hash_json(&b));
}

#[test]
fn upstream_port_json_hash_matrix_struct_hash_json_order_and_ascii() {
    // Upstream reference:
    // /Users/nchapman/Code/json-joy/packages/json-joy/src/json-hash/__tests__/structHash.spec.ts
    let a = json!({"b": [1, 2, 3], "a": {"x": true}});
    let b = json!({"a": {"x": true}, "b": [1, 2, 3]});
    let ha = struct_hash_json(&a);
    let hb = struct_hash_json(&b);
    assert_eq!(ha, hb);
    assert!(ha.is_ascii());
}

#[test]
fn upstream_port_json_hash_matrix_hash_str_and_bin_change_on_content() {
    let s1 = hash_str("hello");
    let s2 = hash_str("hello!");
    assert_ne!(s1, s2);

    let b1 = hash_bin(&[1, 2, 3]);
    let b2 = hash_bin(&[1, 2, 4]);
    assert_ne!(b1, b2);
}

#[test]
fn upstream_port_json_hash_matrix_struct_hash_schema_obj_vec_arr() {
    // Upstream reference:
    // /Users/nchapman/Code/json-joy/packages/json-joy/src/json-hash/structHashSchema.ts
    let left = obj_node(
        vec![
            ("k2".to_string(), arr_node(vec![val_node(con_json(json!(1))), str_node("ab")])),
            (
                "k1".to_string(),
                vec_node(vec![Some(val_node(con_json(json!(true)))), None]),
            ),
        ],
        vec![],
    );
    let right = obj_node(
        vec![
            (
                "k1".to_string(),
                vec_node(vec![Some(val_node(con_json(json!(true)))), None]),
            ),
            ("k2".to_string(), arr_node(vec![val_node(con_json(json!(1))), str_node("ab")])),
        ],
        vec![],
    );
    assert_eq!(struct_hash_schema(Some(&left)), struct_hash_schema(Some(&right)));
}

#[test]
fn upstream_port_json_hash_matrix_struct_hash_crdt_on_runtime_nodes() {
    // Upstream reference:
    // /Users/nchapman/Code/json-joy/packages/json-joy/src/json-hash/structHashCrdt.ts
    let schema = obj_node(
        vec![
            ("name".to_string(), str_node("doc")),
            (
                "items".to_string(),
                arr_node(vec![val_node(con_json(json!(1))), val_node(con_json(json!(2)))]),
            ),
            ("bytes".to_string(), bin_node(vec![1, 2, 3])),
        ],
        vec![],
    );
    let runtime = runtime_from_schema(&schema, 91000);
    let root = Timestamp {
        sid: 91000,
        time: 1,
    };
    let h = struct_hash_crdt(&runtime, Some(root));
    assert!(!h.is_empty());
    assert!(h.is_ascii());
}
