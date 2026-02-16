use std::sync::{Arc, Mutex};

use json_joy_core::model_api::{ChangeEventOrigin, NativeModelApi, PathStep};
use serde_json::json;

#[test]
fn upstream_port_model_api_events_local_change_and_unsubscribe_matrix() {
    let mut api = NativeModelApi::from_model_binary(
        &json_joy_core::less_db_compat::model_to_binary(
            &json_joy_core::less_db_compat::create_model(&json!({"k":1}), 700_001).unwrap(),
        ),
        Some(700_001),
    )
    .unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let seen2 = Arc::clone(&seen);
    let id = api.on_change(move |ev| {
        seen2.lock().unwrap().push((ev.origin, ev.before, ev.after));
    });

    api.set(&[PathStep::Key("k".to_string())], json!(2)).unwrap();
    {
        let v = seen.lock().unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].0, ChangeEventOrigin::Local);
        assert_eq!(v[0].1, json!({"k":1}));
        assert_eq!(v[0].2, json!({"k":2}));
    }

    assert!(api.off_change(id));
    api.set(&[PathStep::Key("k".to_string())], json!(3)).unwrap();
    assert_eq!(seen.lock().unwrap().len(), 1);
}

#[test]
fn upstream_port_model_api_events_remote_origin_matrix() {
    let base = json_joy_core::less_db_compat::model_to_binary(
        &json_joy_core::less_db_compat::create_model(&json!({"k":1}), 701_001).unwrap(),
    );
    let mut api = NativeModelApi::from_model_binary(&base, Some(701_001)).unwrap();
    let next = json!({"k": 9});
    let patch = json_joy_core::diff_runtime::diff_model_to_patch_bytes(&base, &next, 900_123)
        .unwrap()
        .unwrap();
    let patch = json_joy_core::patch::Patch::from_binary(&patch).unwrap();

    let seen = Arc::new(Mutex::new(Vec::new()));
    let seen2 = Arc::clone(&seen);
    api.on_change(move |ev| {
        seen2.lock().unwrap().push(ev.origin);
    });

    api.apply_patch(&patch).unwrap();
    assert_eq!(seen.lock().unwrap().as_slice(), &[ChangeEventOrigin::Remote]);
}
