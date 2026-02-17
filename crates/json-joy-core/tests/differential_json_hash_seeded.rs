use std::path::{Path, PathBuf};
use std::process::Command;

use json_joy_core::json_hash::{hash_json, struct_hash_crdt, struct_hash_json};
use json_joy_core::model_runtime::RuntimeModel;
use json_joy_core::patch::Timestamp;
use json_joy_core::schema::json as schema_json;
use serde_json::Value;

fn oracle_cwd() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tools")
        .join("oracle-node")
}

#[test]
fn differential_json_hash_seeded_matches_oracle() {
    let mut cases: Vec<Value> = vec![
        serde_json::json!(null),
        serde_json::json!(true),
        serde_json::json!(123),
        serde_json::json!("abc"),
        serde_json::json!([1, 2, 3]),
        serde_json::json!({"a": 1, "b": [true, null]}),
        serde_json::json!({"nested": {"x": "y"}, "arr": [1, {"k": 2}]}),
    ];

    let mut rng = Lcg::new(0x9876_5432_10ab_cdef);
    while cases.len() < 40 {
        cases.push(random_json(&mut rng, 0));
    }

    for (idx, value) in cases.iter().enumerate() {
        let oracle = oracle_hash_triplet(value);
        let rust_hash = hash_json(value);
        let rust_struct_hash = struct_hash_json(value);
        let rust_struct_crdt = struct_hash_crdt_for_json(value, 95000 + idx as u64);

        assert_eq!(rust_hash, oracle.hash, "hash mismatch at case {idx}");
        assert_eq!(
            rust_struct_hash, oracle.struct_hash,
            "struct_hash mismatch at case {idx}"
        );
        assert_eq!(
            rust_struct_crdt, oracle.struct_hash_crdt,
            "struct_hash_crdt mismatch at case {idx}"
        );
    }
}

fn struct_hash_crdt_for_json(value: &Value, sid: u64) -> String {
    let schema = schema_json(value);
    let mut runtime = RuntimeModel::new_logical_empty(sid);
    let patch = schema.to_patch(sid, 1).expect("schema patch");
    runtime.apply_patch(&patch).expect("apply patch");
    let root = Timestamp { sid, time: 1 };
    struct_hash_crdt(&runtime, Some(root))
}

struct OracleHashTriplet {
    hash: u32,
    struct_hash: String,
    struct_hash_crdt: String,
}

fn oracle_hash_triplet(value: &Value) -> OracleHashTriplet {
    let script = r#"
const {hash, structHash} = require('json-joy/lib/json-hash');
const {structHashCrdt} = require('json-joy/lib/json-hash/structHashCrdt');
const {Model} = require('json-joy/lib/json-crdt');
const input = JSON.parse(process.argv[1]);
const model = Model.create();
model.api.set(input.value);
model.api.flush();
const root = model.root.node();
process.stdout.write(JSON.stringify({
  hash: hash(input.value),
  structHash: structHash(input.value),
  structHashCrdt: structHashCrdt(root),
}));
"#;

    let payload = serde_json::json!({ "value": value });
    let out = Command::new("node")
        .current_dir(oracle_cwd())
        .arg("-e")
        .arg(script)
        .arg(payload.to_string())
        .output()
        .expect("failed to run json-hash oracle");
    assert!(
        out.status.success(),
        "json-hash oracle failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let parsed: Value = serde_json::from_slice(&out.stdout).expect("oracle output must be json");
    OracleHashTriplet {
        hash: parsed["hash"].as_u64().expect("hash must be u64") as u32,
        struct_hash: parsed["structHash"]
            .as_str()
            .expect("structHash must be str")
            .to_string(),
        struct_hash_crdt: parsed["structHashCrdt"]
            .as_str()
            .expect("structHashCrdt must be str")
            .to_string(),
    }
}

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        self.state
    }
    fn range(&mut self, max: u64) -> u64 {
        if max == 0 {
            0
        } else {
            self.next_u64() % max
        }
    }
}

fn random_json(rng: &mut Lcg, depth: usize) -> Value {
    if depth > 3 {
        return random_primitive(rng);
    }
    match rng.range(6) {
        0 => Value::Null,
        1 => Value::Bool(rng.range(2) == 1),
        2 => Value::Number(serde_json::Number::from((rng.range(1000) as i64) - 500)),
        3 => Value::String(random_string(rng, 0, 8)),
        4 => {
            let len = rng.range(5) as usize;
            let mut arr = Vec::with_capacity(len);
            for _ in 0..len {
                arr.push(random_json(rng, depth + 1));
            }
            Value::Array(arr)
        }
        _ => {
            let len = rng.range(5) as usize;
            let mut map = serde_json::Map::new();
            for _ in 0..len {
                map.insert(random_string(rng, 1, 6), random_json(rng, depth + 1));
            }
            Value::Object(map)
        }
    }
}

fn random_primitive(rng: &mut Lcg) -> Value {
    match rng.range(4) {
        0 => Value::Null,
        1 => Value::Bool(rng.range(2) == 1),
        2 => Value::Number(serde_json::Number::from((rng.range(1000) as i64) - 500)),
        _ => Value::String(random_string(rng, 0, 8)),
    }
}

fn random_string(rng: &mut Lcg, min_len: usize, max_len: usize) -> String {
    let span = if max_len > min_len {
        max_len - min_len + 1
    } else {
        1
    };
    let len = min_len + rng.range(span as u64) as usize;
    let mut s = String::with_capacity(len);
    for _ in 0..len {
        let c = (b'a' + (rng.range(26) as u8)) as char;
        s.push(c);
    }
    s
}

