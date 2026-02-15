use std::fs;
use std::path::{Path, PathBuf};

use json_joy_core::model_runtime::RuntimeModel;
use json_joy_core::patch::Patch;
use serde_json::Value;

#[test]
fn property_replay_applying_same_sequence_twice_is_stable() {
    let fixtures = load_apply_replay_fixtures();
    assert!(fixtures.len() >= 30, "expected >=30 apply replay fixtures");

    for fixture in fixtures {
        let label = fixture["input"]["label"].as_str().unwrap_or_default();
        if !label.contains("dup") {
            continue;
        }
        let base = decode_hex(
            fixture["input"]["base_model_binary_hex"]
                .as_str()
                .expect("base_model_binary_hex must be string"),
        );
        let patches: Vec<Patch> = fixture["input"]["patches_binary_hex"]
            .as_array()
            .expect("patches_binary_hex must be array")
            .iter()
            .map(|v| {
                let bytes = decode_hex(v.as_str().expect("patch hex must be string"));
                Patch::from_binary(&bytes).expect("patch must decode")
            })
            .collect();
        let replay: Vec<usize> = fixture["input"]["replay_pattern"]
            .as_array()
            .expect("replay_pattern must be array")
            .iter()
            .map(|v| usize::try_from(v.as_u64().expect("index must be u64")).expect("index out of range"))
            .collect();

        let mut model_once = RuntimeModel::from_model_binary(&base).expect("base decode must succeed");
        for idx in replay.iter().copied() {
            model_once.apply_patch(&patches[idx]).expect("apply must succeed");
        }
        let once_view = model_once.view_json();

        for idx in replay.iter().copied() {
            model_once.apply_patch(&patches[idx]).expect("second pass apply must succeed");
        }
        let twice_view = model_once.view_json();

        assert_eq!(
            once_view, twice_view,
            "re-applying same replay pattern must be idempotent for {}",
            fixture["name"]
        );
    }
}

#[test]
fn property_duplicate_compression_preserves_view_for_duplicate_heavy_fixtures() {
    let fixtures = load_apply_replay_fixtures();

    for fixture in fixtures {
        let label = fixture["input"]["label"].as_str().unwrap_or_default();
        if !label.contains("dup") {
            continue;
        }

        let base = decode_hex(
            fixture["input"]["base_model_binary_hex"]
                .as_str()
                .expect("base_model_binary_hex must be string"),
        );
        let patches: Vec<Patch> = fixture["input"]["patches_binary_hex"]
            .as_array()
            .expect("patches_binary_hex must be array")
            .iter()
            .map(|v| {
                let bytes = decode_hex(v.as_str().expect("patch hex must be string"));
                Patch::from_binary(&bytes).expect("patch must decode")
            })
            .collect();
        let replay: Vec<usize> = fixture["input"]["replay_pattern"]
            .as_array()
            .expect("replay_pattern must be array")
            .iter()
            .map(|v| usize::try_from(v.as_u64().expect("index must be u64")).expect("index out of range"))
            .collect();

        let compressed = compress_adjacent_duplicates(&replay);

        let mut full = RuntimeModel::from_model_binary(&base).expect("base decode must succeed");
        for idx in replay.iter().copied() {
            full.apply_patch(&patches[idx]).expect("full apply must succeed");
        }

        let mut dedup = RuntimeModel::from_model_binary(&base).expect("base decode must succeed");
        for idx in compressed.iter().copied() {
            dedup.apply_patch(&patches[idx]).expect("dedup apply must succeed");
        }

        assert_eq!(
            full.view_json(),
            dedup.view_json(),
            "adjacent duplicate compression changed view for {}",
            fixture["name"]
        );
    }
}

fn load_apply_replay_fixtures() -> Vec<Value> {
    let dir = fixtures_dir();
    let manifest = read_json(&dir.join("manifest.json"));
    let entries = manifest["fixtures"].as_array().expect("manifest.fixtures must be array");

    let mut out = Vec::new();
    for entry in entries {
        if entry["scenario"].as_str() != Some("model_apply_replay") {
            continue;
        }
        let file = entry["file"].as_str().expect("fixture file must be string");
        out.push(read_json(&dir.join(file)));
    }
    out
}

fn compress_adjacent_duplicates(replay: &[usize]) -> Vec<usize> {
    if replay.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(replay.len());
    out.push(replay[0]);
    for idx in replay.iter().copied().skip(1) {
        if out.last().copied() != Some(idx) {
            out.push(idx);
        }
    }
    out
}

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("compat")
        .join("fixtures")
}

fn read_json(path: &Path) -> Value {
    let data = fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {:?}: {e}", path));
    serde_json::from_str(&data).unwrap_or_else(|e| panic!("failed to parse {:?}: {e}", path))
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
