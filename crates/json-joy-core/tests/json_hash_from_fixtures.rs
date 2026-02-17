use std::fs;
use std::path::{Path, PathBuf};

use json_joy_core::json_hash::{hash_json, struct_hash_crdt, struct_hash_json, struct_hash_schema};
use json_joy_core::model_runtime::RuntimeModel;
use json_joy_core::patch::Timestamp;
use json_joy_core::schema::json as schema_json;
use serde_json::Value;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("compat")
        .join("fixtures")
}

fn read_json(path: &Path) -> Value {
    let data =
        fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {:?}: {e}", path));
    serde_json::from_str(&data).unwrap_or_else(|e| panic!("failed to parse {:?}: {e}", path))
}

#[test]
fn json_hash_fixtures_match_oracle_outputs() {
    let dir = fixtures_dir();
    let mut files = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("failed to list fixtures dir {:?}: {e}", dir))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("json_hash_parity_") && n.ends_with("_v1.json"))
        })
        .collect::<Vec<_>>();
    files.sort();
    assert!(
        !files.is_empty(),
        "expected json_hash_parity fixtures in {:?}",
        dir
    );

    for path in files {
        let fixture = read_json(&path);
        let name = fixture["name"]
            .as_str()
            .unwrap_or_else(|| panic!("fixture name missing in {:?}", path));
        let sid = fixture["input"]["sid"]
            .as_u64()
            .unwrap_or_else(|| panic!("input.sid missing for {name}"));
        let value = fixture["input"]["value_json"].clone();
        let expected_hash = fixture["expected"]["hash_u32"]
            .as_u64()
            .unwrap_or_else(|| panic!("expected.hash_u32 missing for {name}")) as u32;
        let expected_struct = fixture["expected"]["struct_hash"]
            .as_str()
            .unwrap_or_else(|| panic!("expected.struct_hash missing for {name}"));
        let expected_struct_crdt = fixture["expected"]["struct_hash_crdt"]
            .as_str()
            .unwrap_or_else(|| panic!("expected.struct_hash_crdt missing for {name}"));
        let expected_struct_schema = fixture["expected"]["struct_hash_schema"]
            .as_str()
            .unwrap_or_else(|| panic!("expected.struct_hash_schema missing for {name}"));

        let schema = schema_json(&value);
        let mut runtime = RuntimeModel::new_logical_empty(sid);
        let patch = schema.to_patch(sid, 1).expect("schema patch");
        runtime.apply_patch(&patch).expect("apply patch");
        let root = Timestamp { sid, time: 1 };

        assert_eq!(hash_json(&value), expected_hash, "hash mismatch for {name}");
        assert_eq!(
            struct_hash_json(&value),
            expected_struct,
            "struct_hash mismatch for {name}"
        );
        assert_eq!(
            struct_hash_crdt(&runtime, Some(root)),
            expected_struct_crdt,
            "struct_hash_crdt mismatch for {name}"
        );
        assert_eq!(
            struct_hash_schema(Some(&schema)),
            expected_struct_schema,
            "struct_hash_schema mismatch for {name}"
        );
    }
}

