//! Core types for the JSON Patch module.
//!
//! Mirrors `packages/json-joy/src/json-patch/types.ts`,
//! `packages/json-joy/src/json-patch/constants.ts`, and
//! the Op class hierarchy in `packages/json-joy/src/json-patch/op/`.

use serde_json::{Map, Value};
use std::str::FromStr;
use thiserror::Error;

pub use json_joy_json_pointer::Path;

// ── Error ─────────────────────────────────────────────────────────────────

#[derive(Debug, Error, PartialEq)]
pub enum PatchError {
    #[error("NOT_FOUND")]
    NotFound,
    #[error("TEST")]
    Test,
    #[error("NOT_A_STRING")]
    NotAString,
    #[error("INVALID_INDEX")]
    InvalidIndex,
    #[error("INVALID_TARGET")]
    InvalidTarget,
    #[error("INVALID_OP: {0}")]
    InvalidOp(String),
}

// ── Type enum for test_type / type operations ─────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonPatchType {
    String,
    Number,
    Boolean,
    Object,
    Integer,
    Array,
    Null,
}

impl JsonPatchType {
    pub fn as_str(&self) -> &'static str {
        match self {
            JsonPatchType::String => "string",
            JsonPatchType::Number => "number",
            JsonPatchType::Boolean => "boolean",
            JsonPatchType::Object => "object",
            JsonPatchType::Integer => "integer",
            JsonPatchType::Array => "array",
            JsonPatchType::Null => "null",
        }
    }

    pub fn parse_str(s: &str) -> Result<Self, PatchError> {
        match s {
            "string" => Ok(JsonPatchType::String),
            "number" => Ok(JsonPatchType::Number),
            "boolean" => Ok(JsonPatchType::Boolean),
            "object" => Ok(JsonPatchType::Object),
            "integer" => Ok(JsonPatchType::Integer),
            "array" => Ok(JsonPatchType::Array),
            "null" => Ok(JsonPatchType::Null),
            other => Err(PatchError::InvalidOp(format!("unknown type: {other}"))),
        }
    }

    /// Returns true if the given JSON value matches this type.
    pub fn matches_value(&self, val: &Value) -> bool {
        match self {
            JsonPatchType::String => val.is_string(),
            JsonPatchType::Number => val.is_number(),
            JsonPatchType::Boolean => val.is_boolean(),
            JsonPatchType::Object => val.is_object(),
            JsonPatchType::Integer => val.as_f64().map(|f| f.fract() == 0.0).unwrap_or(false),
            JsonPatchType::Array => val.is_array(),
            JsonPatchType::Null => val.is_null(),
        }
    }
}

impl FromStr for JsonPatchType {
    type Err = PatchError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        JsonPatchType::parse_str(s)
    }
}

// ── Op enum ───────────────────────────────────────────────────────────────

/// A JSON Patch operation.
///
/// Mirrors the class hierarchy in `packages/json-joy/src/json-patch/op/`.
#[derive(Debug, Clone)]
pub enum Op {
    // ── RFC 6902 operations ───────────────────────────────────────────────
    Add {
        path: Path,
        value: Value,
    },
    Remove {
        path: Path,
        old_value: Option<Value>,
    },
    Replace {
        path: Path,
        value: Value,
        old_value: Option<Value>,
    },
    Copy {
        path: Path,
        from: Path,
    },
    Move {
        path: Path,
        from: Path,
    },
    Test {
        path: Path,
        value: Value,
        not: bool,
    },

    // ── Extended operations ───────────────────────────────────────────────
    StrIns {
        path: Path,
        pos: usize,
        str_val: String,
    },
    StrDel {
        path: Path,
        pos: usize,
        str_val: Option<String>,
        len: Option<usize>,
    },
    Flip {
        path: Path,
    },
    Inc {
        path: Path,
        inc: f64,
    },
    Split {
        path: Path,
        pos: usize,
        props: Option<Value>,
    },
    Merge {
        path: Path,
        pos: usize,
        props: Option<Value>,
    },
    Extend {
        path: Path,
        props: Map<String, Value>,
        delete_null: bool,
    },

    // ── First-order predicate operations ─────────────────────────────────
    Defined {
        path: Path,
    },
    Undefined {
        path: Path,
    },
    Contains {
        path: Path,
        value: String,
        ignore_case: bool,
    },
    Ends {
        path: Path,
        value: String,
        ignore_case: bool,
    },
    Starts {
        path: Path,
        value: String,
        ignore_case: bool,
    },
    In {
        path: Path,
        value: Vec<Value>,
    },
    Less {
        path: Path,
        value: f64,
    },
    More {
        path: Path,
        value: f64,
    },
    Matches {
        path: Path,
        value: String,
        ignore_case: bool,
    },
    TestType {
        path: Path,
        type_vals: Vec<JsonPatchType>,
    },
    TestString {
        path: Path,
        pos: usize,
        str_val: String,
        not: bool,
    },
    TestStringLen {
        path: Path,
        len: usize,
        not: bool,
    },
    Type {
        path: Path,
        value: JsonPatchType,
    },

    // ── Second-order predicate operations ─────────────────────────────────
    And {
        path: Path,
        ops: Vec<Op>,
    },
    Not {
        path: Path,
        ops: Vec<Op>,
    },
    Or {
        path: Path,
        ops: Vec<Op>,
    },
}

impl Op {
    /// Returns the operation name string (matching the TypeScript `op()` method).
    pub fn op_name(&self) -> &'static str {
        match self {
            Op::Add { .. } => "add",
            Op::Remove { .. } => "remove",
            Op::Replace { .. } => "replace",
            Op::Copy { .. } => "copy",
            Op::Move { .. } => "move",
            Op::Test { .. } => "test",
            Op::StrIns { .. } => "str_ins",
            Op::StrDel { .. } => "str_del",
            Op::Flip { .. } => "flip",
            Op::Inc { .. } => "inc",
            Op::Split { .. } => "split",
            Op::Merge { .. } => "merge",
            Op::Extend { .. } => "extend",
            Op::Defined { .. } => "defined",
            Op::Undefined { .. } => "undefined",
            Op::Contains { .. } => "contains",
            Op::Ends { .. } => "ends",
            Op::Starts { .. } => "starts",
            Op::In { .. } => "in",
            Op::Less { .. } => "less",
            Op::More { .. } => "more",
            Op::Matches { .. } => "matches",
            Op::TestType { .. } => "test_type",
            Op::TestString { .. } => "test_string",
            Op::TestStringLen { .. } => "test_string_len",
            Op::Type { .. } => "type",
            Op::And { .. } => "and",
            Op::Not { .. } => "not",
            Op::Or { .. } => "or",
        }
    }

    /// Returns the path of the operation.
    pub fn path(&self) -> &Path {
        match self {
            Op::Add { path, .. } => path,
            Op::Remove { path, .. } => path,
            Op::Replace { path, .. } => path,
            Op::Copy { path, .. } => path,
            Op::Move { path, .. } => path,
            Op::Test { path, .. } => path,
            Op::StrIns { path, .. } => path,
            Op::StrDel { path, .. } => path,
            Op::Flip { path, .. } => path,
            Op::Inc { path, .. } => path,
            Op::Split { path, .. } => path,
            Op::Merge { path, .. } => path,
            Op::Extend { path, .. } => path,
            Op::Defined { path, .. } => path,
            Op::Undefined { path, .. } => path,
            Op::Contains { path, .. } => path,
            Op::Ends { path, .. } => path,
            Op::Starts { path, .. } => path,
            Op::In { path, .. } => path,
            Op::Less { path, .. } => path,
            Op::More { path, .. } => path,
            Op::Matches { path, .. } => path,
            Op::TestType { path, .. } => path,
            Op::TestString { path, .. } => path,
            Op::TestStringLen { path, .. } => path,
            Op::Type { path, .. } => path,
            Op::And { path, .. } => path,
            Op::Not { path, .. } => path,
            Op::Or { path, .. } => path,
        }
    }

    /// Returns true if this is a predicate operation.
    pub fn is_predicate(&self) -> bool {
        matches!(
            self,
            Op::Test { .. }
                | Op::Defined { .. }
                | Op::Undefined { .. }
                | Op::Contains { .. }
                | Op::Ends { .. }
                | Op::Starts { .. }
                | Op::In { .. }
                | Op::Less { .. }
                | Op::More { .. }
                | Op::Matches { .. }
                | Op::TestType { .. }
                | Op::TestString { .. }
                | Op::TestStringLen { .. }
                | Op::Type { .. }
                | Op::And { .. }
                | Op::Not { .. }
                | Op::Or { .. }
        )
    }
}

// ── Result types ──────────────────────────────────────────────────────────

/// Result of applying a single operation.
#[derive(Debug, Clone)]
pub struct OpResult {
    /// The document after applying the operation.
    pub doc: Value,
    /// The value at the path before the operation, if applicable.
    pub old: Option<Value>,
}

/// Result of applying a full patch.
#[derive(Debug, Clone)]
pub struct PatchResult {
    pub doc: Value,
    pub res: Vec<OpResult>,
}

/// Options for `apply_patch`.
#[derive(Debug, Clone, Default)]
pub struct ApplyPatchOptions {
    /// If true, mutate the document in place (passed by value).
    /// If false, clone the document before applying.
    pub mutate: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── PatchError display ──────────────────────────────────────────────

    #[test]
    fn patch_error_not_found_display() {
        assert_eq!(PatchError::NotFound.to_string(), "NOT_FOUND");
    }

    #[test]
    fn patch_error_test_display() {
        assert_eq!(PatchError::Test.to_string(), "TEST");
    }

    #[test]
    fn patch_error_not_a_string_display() {
        assert_eq!(PatchError::NotAString.to_string(), "NOT_A_STRING");
    }

    #[test]
    fn patch_error_invalid_index_display() {
        assert_eq!(PatchError::InvalidIndex.to_string(), "INVALID_INDEX");
    }

    #[test]
    fn patch_error_invalid_target_display() {
        assert_eq!(PatchError::InvalidTarget.to_string(), "INVALID_TARGET");
    }

    #[test]
    fn patch_error_invalid_op_display() {
        let err = PatchError::InvalidOp("bad".into());
        assert_eq!(err.to_string(), "INVALID_OP: bad");
    }

    #[test]
    fn patch_error_equality() {
        assert_eq!(PatchError::NotFound, PatchError::NotFound);
        assert_ne!(PatchError::NotFound, PatchError::Test);
    }

    // ── JsonPatchType as_str / parse_str ────────────────────────────────

    #[test]
    fn json_patch_type_as_str_all() {
        assert_eq!(JsonPatchType::String.as_str(), "string");
        assert_eq!(JsonPatchType::Number.as_str(), "number");
        assert_eq!(JsonPatchType::Boolean.as_str(), "boolean");
        assert_eq!(JsonPatchType::Object.as_str(), "object");
        assert_eq!(JsonPatchType::Integer.as_str(), "integer");
        assert_eq!(JsonPatchType::Array.as_str(), "array");
        assert_eq!(JsonPatchType::Null.as_str(), "null");
    }

    #[test]
    fn json_patch_type_parse_str_all() {
        assert_eq!(
            JsonPatchType::parse_str("string").unwrap(),
            JsonPatchType::String
        );
        assert_eq!(
            JsonPatchType::parse_str("number").unwrap(),
            JsonPatchType::Number
        );
        assert_eq!(
            JsonPatchType::parse_str("boolean").unwrap(),
            JsonPatchType::Boolean
        );
        assert_eq!(
            JsonPatchType::parse_str("object").unwrap(),
            JsonPatchType::Object
        );
        assert_eq!(
            JsonPatchType::parse_str("integer").unwrap(),
            JsonPatchType::Integer
        );
        assert_eq!(
            JsonPatchType::parse_str("array").unwrap(),
            JsonPatchType::Array
        );
        assert_eq!(
            JsonPatchType::parse_str("null").unwrap(),
            JsonPatchType::Null
        );
    }

    #[test]
    fn json_patch_type_parse_str_unknown() {
        let err = JsonPatchType::parse_str("foo").unwrap_err();
        match err {
            PatchError::InvalidOp(s) => assert!(s.contains("unknown type")),
            _ => panic!("expected InvalidOp"),
        }
    }

    #[test]
    fn json_patch_type_from_str() {
        let t: JsonPatchType = "number".parse().unwrap();
        assert_eq!(t, JsonPatchType::Number);
        assert!("garbage".parse::<JsonPatchType>().is_err());
    }

    // ── JsonPatchType::matches_value ────────────────────────────────────

    #[test]
    fn matches_value_string() {
        assert!(JsonPatchType::String.matches_value(&json!("hello")));
        assert!(!JsonPatchType::String.matches_value(&json!(42)));
    }

    #[test]
    fn matches_value_number() {
        assert!(JsonPatchType::Number.matches_value(&json!(3.25)));
        assert!(JsonPatchType::Number.matches_value(&json!(42)));
        assert!(!JsonPatchType::Number.matches_value(&json!("x")));
    }

    #[test]
    fn matches_value_boolean() {
        assert!(JsonPatchType::Boolean.matches_value(&json!(true)));
        assert!(JsonPatchType::Boolean.matches_value(&json!(false)));
        assert!(!JsonPatchType::Boolean.matches_value(&json!(1)));
    }

    #[test]
    fn matches_value_object() {
        assert!(JsonPatchType::Object.matches_value(&json!({"a": 1})));
        assert!(!JsonPatchType::Object.matches_value(&json!([1])));
    }

    #[test]
    fn matches_value_integer() {
        assert!(JsonPatchType::Integer.matches_value(&json!(42)));
        assert!(JsonPatchType::Integer.matches_value(&json!(0)));
        assert!(!JsonPatchType::Integer.matches_value(&json!(3.25)));
        assert!(!JsonPatchType::Integer.matches_value(&json!("x")));
    }

    #[test]
    fn matches_value_array() {
        assert!(JsonPatchType::Array.matches_value(&json!([1, 2])));
        assert!(!JsonPatchType::Array.matches_value(&json!({"a": 1})));
    }

    #[test]
    fn matches_value_null() {
        assert!(JsonPatchType::Null.matches_value(&json!(null)));
        assert!(!JsonPatchType::Null.matches_value(&json!(0)));
    }

    // ── Op::op_name ────────────────────────────────────────────────────

    #[test]
    fn op_name_all_variants() {
        let ops: Vec<Op> = vec![
            Op::Add {
                path: vec![],
                value: json!(1),
            },
            Op::Remove {
                path: vec![],
                old_value: None,
            },
            Op::Replace {
                path: vec![],
                value: json!(1),
                old_value: None,
            },
            Op::Copy {
                path: vec![],
                from: vec![],
            },
            Op::Move {
                path: vec![],
                from: vec![],
            },
            Op::Test {
                path: vec![],
                value: json!(1),
                not: false,
            },
            Op::StrIns {
                path: vec![],
                pos: 0,
                str_val: String::new(),
            },
            Op::StrDel {
                path: vec![],
                pos: 0,
                str_val: None,
                len: None,
            },
            Op::Flip { path: vec![] },
            Op::Inc {
                path: vec![],
                inc: 0.0,
            },
            Op::Split {
                path: vec![],
                pos: 0,
                props: None,
            },
            Op::Merge {
                path: vec![],
                pos: 0,
                props: None,
            },
            Op::Extend {
                path: vec![],
                props: Map::new(),
                delete_null: false,
            },
            Op::Defined { path: vec![] },
            Op::Undefined { path: vec![] },
            Op::Contains {
                path: vec![],
                value: String::new(),
                ignore_case: false,
            },
            Op::Ends {
                path: vec![],
                value: String::new(),
                ignore_case: false,
            },
            Op::Starts {
                path: vec![],
                value: String::new(),
                ignore_case: false,
            },
            Op::In {
                path: vec![],
                value: vec![],
            },
            Op::Less {
                path: vec![],
                value: 0.0,
            },
            Op::More {
                path: vec![],
                value: 0.0,
            },
            Op::Matches {
                path: vec![],
                value: String::new(),
                ignore_case: false,
            },
            Op::TestType {
                path: vec![],
                type_vals: vec![],
            },
            Op::TestString {
                path: vec![],
                pos: 0,
                str_val: String::new(),
                not: false,
            },
            Op::TestStringLen {
                path: vec![],
                len: 0,
                not: false,
            },
            Op::Type {
                path: vec![],
                value: JsonPatchType::Null,
            },
            Op::And {
                path: vec![],
                ops: vec![],
            },
            Op::Not {
                path: vec![],
                ops: vec![],
            },
            Op::Or {
                path: vec![],
                ops: vec![],
            },
        ];
        let expected_names = [
            "add",
            "remove",
            "replace",
            "copy",
            "move",
            "test",
            "str_ins",
            "str_del",
            "flip",
            "inc",
            "split",
            "merge",
            "extend",
            "defined",
            "undefined",
            "contains",
            "ends",
            "starts",
            "in",
            "less",
            "more",
            "matches",
            "test_type",
            "test_string",
            "test_string_len",
            "type",
            "and",
            "not",
            "or",
        ];
        for (op, expected) in ops.iter().zip(expected_names.iter()) {
            assert_eq!(op.op_name(), *expected);
        }
    }

    // ── Op::path ────────────────────────────────────────────────────────

    #[test]
    fn op_path_returns_path() {
        let op = Op::Add {
            path: vec!["a".to_string(), "b".to_string()],
            value: json!(1),
        };
        assert_eq!(op.path(), &vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn op_path_all_variants_return_their_path() {
        let path = vec!["test".to_string()];
        let ops: Vec<Op> = vec![
            Op::Add {
                path: path.clone(),
                value: json!(1),
            },
            Op::Remove {
                path: path.clone(),
                old_value: None,
            },
            Op::Replace {
                path: path.clone(),
                value: json!(1),
                old_value: None,
            },
            Op::Copy {
                path: path.clone(),
                from: vec![],
            },
            Op::Move {
                path: path.clone(),
                from: vec![],
            },
            Op::Test {
                path: path.clone(),
                value: json!(1),
                not: false,
            },
            Op::StrIns {
                path: path.clone(),
                pos: 0,
                str_val: String::new(),
            },
            Op::StrDel {
                path: path.clone(),
                pos: 0,
                str_val: None,
                len: None,
            },
            Op::Flip { path: path.clone() },
            Op::Inc {
                path: path.clone(),
                inc: 0.0,
            },
            Op::Split {
                path: path.clone(),
                pos: 0,
                props: None,
            },
            Op::Merge {
                path: path.clone(),
                pos: 0,
                props: None,
            },
            Op::Extend {
                path: path.clone(),
                props: Map::new(),
                delete_null: false,
            },
            Op::Defined { path: path.clone() },
            Op::Undefined { path: path.clone() },
            Op::Contains {
                path: path.clone(),
                value: String::new(),
                ignore_case: false,
            },
            Op::Ends {
                path: path.clone(),
                value: String::new(),
                ignore_case: false,
            },
            Op::Starts {
                path: path.clone(),
                value: String::new(),
                ignore_case: false,
            },
            Op::In {
                path: path.clone(),
                value: vec![],
            },
            Op::Less {
                path: path.clone(),
                value: 0.0,
            },
            Op::More {
                path: path.clone(),
                value: 0.0,
            },
            Op::Matches {
                path: path.clone(),
                value: String::new(),
                ignore_case: false,
            },
            Op::TestType {
                path: path.clone(),
                type_vals: vec![],
            },
            Op::TestString {
                path: path.clone(),
                pos: 0,
                str_val: String::new(),
                not: false,
            },
            Op::TestStringLen {
                path: path.clone(),
                len: 0,
                not: false,
            },
            Op::Type {
                path: path.clone(),
                value: JsonPatchType::Null,
            },
            Op::And {
                path: path.clone(),
                ops: vec![],
            },
            Op::Not {
                path: path.clone(),
                ops: vec![],
            },
            Op::Or {
                path: path.clone(),
                ops: vec![],
            },
        ];
        for op in &ops {
            assert_eq!(op.path(), &path, "path() mismatch for {}", op.op_name());
        }
    }

    // ── Op::is_predicate ────────────────────────────────────────────────

    #[test]
    fn is_predicate_true_for_predicate_ops() {
        let predicates = vec![
            Op::Test {
                path: vec![],
                value: json!(1),
                not: false,
            },
            Op::Defined { path: vec![] },
            Op::Undefined { path: vec![] },
            Op::Contains {
                path: vec![],
                value: String::new(),
                ignore_case: false,
            },
            Op::Ends {
                path: vec![],
                value: String::new(),
                ignore_case: false,
            },
            Op::Starts {
                path: vec![],
                value: String::new(),
                ignore_case: false,
            },
            Op::In {
                path: vec![],
                value: vec![],
            },
            Op::Less {
                path: vec![],
                value: 0.0,
            },
            Op::More {
                path: vec![],
                value: 0.0,
            },
            Op::Matches {
                path: vec![],
                value: String::new(),
                ignore_case: false,
            },
            Op::TestType {
                path: vec![],
                type_vals: vec![],
            },
            Op::TestString {
                path: vec![],
                pos: 0,
                str_val: String::new(),
                not: false,
            },
            Op::TestStringLen {
                path: vec![],
                len: 0,
                not: false,
            },
            Op::Type {
                path: vec![],
                value: JsonPatchType::Null,
            },
            Op::And {
                path: vec![],
                ops: vec![],
            },
            Op::Not {
                path: vec![],
                ops: vec![],
            },
            Op::Or {
                path: vec![],
                ops: vec![],
            },
        ];
        for op in &predicates {
            assert!(op.is_predicate(), "{} should be a predicate", op.op_name());
        }
    }

    #[test]
    fn is_predicate_false_for_non_predicate_ops() {
        let non_predicates = vec![
            Op::Add {
                path: vec![],
                value: json!(1),
            },
            Op::Remove {
                path: vec![],
                old_value: None,
            },
            Op::Replace {
                path: vec![],
                value: json!(1),
                old_value: None,
            },
            Op::Copy {
                path: vec![],
                from: vec![],
            },
            Op::Move {
                path: vec![],
                from: vec![],
            },
            Op::StrIns {
                path: vec![],
                pos: 0,
                str_val: String::new(),
            },
            Op::StrDel {
                path: vec![],
                pos: 0,
                str_val: None,
                len: None,
            },
            Op::Flip { path: vec![] },
            Op::Inc {
                path: vec![],
                inc: 0.0,
            },
            Op::Split {
                path: vec![],
                pos: 0,
                props: None,
            },
            Op::Merge {
                path: vec![],
                pos: 0,
                props: None,
            },
            Op::Extend {
                path: vec![],
                props: Map::new(),
                delete_null: false,
            },
        ];
        for op in &non_predicates {
            assert!(
                !op.is_predicate(),
                "{} should not be a predicate",
                op.op_name()
            );
        }
    }

    // ── ApplyPatchOptions ───────────────────────────────────────────────

    #[test]
    fn apply_patch_options_default() {
        let opts = ApplyPatchOptions::default();
        assert!(!opts.mutate);
    }

    // ── OpResult / PatchResult ──────────────────────────────────────────

    #[test]
    fn op_result_construction() {
        let r = OpResult {
            doc: json!({"a": 1}),
            old: Some(json!(0)),
        };
        assert!(r.old.is_some());
        let r2 = r.clone();
        assert_eq!(r2.doc, json!({"a": 1}));
    }

    #[test]
    fn patch_result_construction() {
        let r = PatchResult {
            doc: json!({}),
            res: vec![OpResult {
                doc: json!(1),
                old: None,
            }],
        };
        assert_eq!(r.res.len(), 1);
        let r2 = r.clone();
        assert_eq!(r2.res.len(), 1);
    }
}
