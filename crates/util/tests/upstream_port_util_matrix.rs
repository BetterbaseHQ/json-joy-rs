//! Upstream: json-joy/packages/util/src/
//!
//! Tests for json_size, json_clone, and string utilities.

use json_joy_util::json_clone::clone;
use json_joy_util::json_size::{json_size, json_size_approx, json_size_fast, utf8_size};
use json_joy_util::strings::{as_string, escape, is_letter, is_punctuation, is_whitespace};
use serde_json::json;

// ---------------------------------------------------------------------------
// utf8_size
// ---------------------------------------------------------------------------

#[test]
fn utf8_size_empty() {
    assert_eq!(utf8_size(""), 0);
}

#[test]
fn utf8_size_ascii() {
    assert_eq!(utf8_size("hello"), 5);
}

#[test]
fn utf8_size_multibyte() {
    assert_eq!(utf8_size("he\u{0301}llo"), 7); // combining accent = 2 bytes
    assert_eq!(utf8_size("\u{65E5}\u{672C}\u{8A9E}"), 9); // 3 CJK chars, 3 bytes each
}

#[test]
fn utf8_size_emoji() {
    assert_eq!(utf8_size("\u{1F600}"), 4); // emoji = 4 bytes
}

// ---------------------------------------------------------------------------
// json_size
// ---------------------------------------------------------------------------

#[test]
fn json_size_null() {
    assert_eq!(json_size(&json!(null)), 4);
}

#[test]
fn json_size_booleans() {
    assert_eq!(json_size(&json!(true)), 4);
    assert_eq!(json_size(&json!(false)), 5);
}

#[test]
fn json_size_numbers() {
    assert_eq!(json_size(&json!(0)), 1);
    assert_eq!(json_size(&json!(123)), 3);
    assert_eq!(json_size(&json!(-123)), 4);
}

#[test]
fn json_size_strings() {
    assert_eq!(json_size(&json!("")), 2);
    assert_eq!(json_size(&json!("hello")), 7);
}

#[test]
fn json_size_string_with_escape() {
    // "a\tb" serialized is "a\tb" which is 6 chars: " a \ t b "
    assert_eq!(json_size(&json!("a\tb")), 6);
}

#[test]
fn json_size_empty_array() {
    assert_eq!(json_size(&json!([])), 2);
}

#[test]
fn json_size_array_with_elements() {
    // [1,2,3] = 7
    assert_eq!(json_size(&json!([1, 2, 3])), 7);
}

#[test]
fn json_size_empty_object() {
    assert_eq!(json_size(&json!({})), 2);
}

#[test]
fn json_size_object_with_key() {
    // {"a":1} = 7
    assert_eq!(json_size(&json!({"a": 1})), 7);
}

#[test]
fn json_size_matches_serde_serialization() {
    let values = vec![
        json!(null),
        json!(true),
        json!(false),
        json!(0),
        json!(42),
        json!(-1),
        json!(""),
        json!("hello"),
        json!([]),
        json!([1, 2, 3]),
        json!({}),
        json!({"a": 1}),
        json!({"name": "test", "values": [1, 2, 3], "nested": {"a": true}}),
    ];
    for v in &values {
        let serialized = serde_json::to_string(v).unwrap();
        assert_eq!(json_size(v), serialized.len(), "mismatch for {v}");
    }
}

// ---------------------------------------------------------------------------
// json_size_approx
// ---------------------------------------------------------------------------

#[test]
fn json_size_approx_reasonable() {
    let value = json!({"name": "test"});
    let exact = json_size(&value);
    let approx = json_size_approx(&value);
    // Should be within a small margin
    assert!(
        approx >= exact.saturating_sub(2) && approx <= exact + 2,
        "approx={approx}, exact={exact}"
    );
}

// ---------------------------------------------------------------------------
// json_size_fast
// ---------------------------------------------------------------------------

#[test]
fn json_size_fast_scalars() {
    // json_size_fast estimates MessagePack encoding sizes, not JSON sizes
    assert_eq!(json_size_fast(&json!(null)), 1);
    assert_eq!(json_size_fast(&json!(true)), 1);
    assert_eq!(json_size_fast(&json!(42)), 9);
    assert_eq!(json_size_fast(&json!("hello")), 9); // 4 + 5
    assert_eq!(json_size_fast(&json!([])), 2);
    assert_eq!(json_size_fast(&json!({})), 2);
}

// ---------------------------------------------------------------------------
// clone
// ---------------------------------------------------------------------------

#[test]
fn clone_null() {
    assert_eq!(clone(&json!(null)), json!(null));
}

#[test]
fn clone_bool() {
    assert_eq!(clone(&json!(true)), json!(true));
    assert_eq!(clone(&json!(false)), json!(false));
}

#[test]
fn clone_number() {
    assert_eq!(clone(&json!(42)), json!(42));
    assert_eq!(clone(&json!(-1.5)), json!(-1.5));
}

#[test]
fn clone_string() {
    assert_eq!(clone(&json!("hello")), json!("hello"));
}

#[test]
fn clone_array() {
    let original = json!([1, "two", null, true]);
    let cloned = clone(&original);
    assert_eq!(original, cloned);
}

#[test]
fn clone_object() {
    let original = json!({"a": 1, "b": "two"});
    let cloned = clone(&original);
    assert_eq!(original, cloned);
}

#[test]
fn clone_deeply_nested() {
    let original = json!({
        "array": [1, 2, {"nested": true}],
        "object": {"a": "b"},
        "scalar": 42
    });
    let cloned = clone(&original);
    assert_eq!(original, cloned);
}

#[test]
fn clone_empty_containers() {
    assert_eq!(clone(&json!([])), json!([]));
    assert_eq!(clone(&json!({})), json!({}));
}

// ---------------------------------------------------------------------------
// String utilities: as_string
// ---------------------------------------------------------------------------

#[test]
fn as_string_simple() {
    assert_eq!(as_string("hello"), "\"hello\"");
}

#[test]
fn as_string_empty() {
    assert_eq!(as_string(""), "\"\"");
}

#[test]
fn as_string_with_quotes() {
    assert_eq!(as_string("say \"hi\""), "\"say \\\"hi\\\"\"");
}

#[test]
fn as_string_with_backslash() {
    assert_eq!(as_string("back\\slash"), "\"back\\\\slash\"");
}

#[test]
fn as_string_with_newline() {
    assert_eq!(as_string("line1\nline2"), "\"line1\\nline2\"");
}

#[test]
fn as_string_with_tab() {
    assert_eq!(as_string("tab\there"), "\"tab\\there\"");
}

#[test]
fn as_string_with_unicode() {
    assert_eq!(
        as_string("hello \u{65E5}\u{672C}\u{8A9E}"),
        "\"hello \u{65E5}\u{672C}\u{8A9E}\""
    );
}

// ---------------------------------------------------------------------------
// String utilities: escape
// ---------------------------------------------------------------------------

#[test]
fn escape_no_special_chars() {
    assert_eq!(escape("hello"), "hello");
}

#[test]
fn escape_with_quotes() {
    assert_eq!(escape("say \"hi\""), "say \\\"hi\\\"");
}

#[test]
fn escape_with_backslash() {
    assert_eq!(escape("back\\slash"), "back\\\\slash");
}

#[test]
fn escape_with_newline() {
    assert_eq!(escape("line1\nline2"), "line1\\nline2");
}

// ---------------------------------------------------------------------------
// String utilities: character predicates
// ---------------------------------------------------------------------------

#[test]
fn is_letter_ascii() {
    assert!(is_letter('a'));
    assert!(is_letter('Z'));
    // is_letter uses is_alphanumeric, so digits count as "letters"
    assert!(is_letter('1'));
    assert!(!is_letter(' '));
}

#[test]
fn is_whitespace_chars() {
    assert!(is_whitespace(' '));
    assert!(is_whitespace('\t'));
    assert!(is_whitespace('\n'));
    assert!(!is_whitespace('a'));
}

#[test]
fn is_punctuation_chars() {
    assert!(is_punctuation('.'));
    assert!(is_punctuation(','));
    assert!(is_punctuation('!'));
    assert!(!is_punctuation('a'));
    assert!(!is_punctuation(' '));
}
