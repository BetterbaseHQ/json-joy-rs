#![allow(dead_code)]

mod codec;
mod helpers;
mod lessdb;
mod model;
mod patch;
mod util;

use serde_json::Value;

pub fn evaluate_fixture(scenario: &str, fixture: &Value) -> Result<Value, String> {
    let input = fixture
        .get("input")
        .and_then(Value::as_object)
        .ok_or_else(|| "fixture.input missing".to_string())?;

    match scenario {
        s if s.starts_with("patch_") => patch::eval_patch(s, input, fixture),
        s if s.starts_with("model_") => model::eval_model(s, input, fixture),
        s if s.starts_with("codec_") => codec::eval_codec(s, input, fixture),
        s if s.starts_with("util_") => util::eval_util(s, input, fixture),
        s if s.starts_with("lessdb_") => lessdb::eval_lessdb(s, input, fixture),
        other => Err(format!("unknown scenario: {other}")),
    }
}
