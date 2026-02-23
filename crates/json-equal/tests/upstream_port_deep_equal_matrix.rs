//! Upstream: json-joy/packages/json-equal/src/
//!
//! Deep equality comparison matrix tests covering symmetry, reflexivity,
//! type mismatches, nested structures, null handling, and number edge cases.

use json_joy_json_equal::deep_equal;
use serde_json::json;

// ---------------------------------------------------------------------------
// Reflexivity
// ---------------------------------------------------------------------------

#[test]
fn reflexivity_null() {
    let v = json!(null);
    assert!(deep_equal(&v, &v));
}

#[test]
fn reflexivity_bool() {
    let v = json!(true);
    assert!(deep_equal(&v, &v));
}

#[test]
fn reflexivity_number() {
    let v = json!(42);
    assert!(deep_equal(&v, &v));
}

#[test]
fn reflexivity_string() {
    let v = json!("hello");
    assert!(deep_equal(&v, &v));
}

#[test]
fn reflexivity_array() {
    let v = json!([1, 2, 3]);
    assert!(deep_equal(&v, &v));
}

#[test]
fn reflexivity_object() {
    let v = json!({"a": 1, "b": [2, 3]});
    assert!(deep_equal(&v, &v));
}

#[test]
fn reflexivity_complex_nested() {
    let v = json!({"complex": [1, 2, {"nested": true}]});
    assert!(deep_equal(&v, &v));
}

// ---------------------------------------------------------------------------
// Symmetry
// ---------------------------------------------------------------------------

#[test]
fn symmetry_equal_objects() {
    let a = json!({"x": 1});
    let b = json!({"x": 1});
    assert!(deep_equal(&a, &b));
    assert!(deep_equal(&b, &a));
}

#[test]
fn symmetry_unequal_objects() {
    let a = json!({"x": 1});
    let b = json!({"x": 2});
    assert!(!deep_equal(&a, &b));
    assert!(!deep_equal(&b, &a));
}

#[test]
fn symmetry_type_mismatch() {
    let a = json!(1);
    let b = json!("1");
    assert!(!deep_equal(&a, &b));
    assert!(!deep_equal(&b, &a));
}

// ---------------------------------------------------------------------------
// Null handling
// ---------------------------------------------------------------------------

#[test]
fn null_equals_null() {
    assert!(deep_equal(&json!(null), &json!(null)));
}

#[test]
fn null_not_equal_zero() {
    assert!(!deep_equal(&json!(null), &json!(0)));
}

#[test]
fn null_not_equal_false() {
    assert!(!deep_equal(&json!(null), &json!(false)));
}

#[test]
fn null_not_equal_empty_string() {
    assert!(!deep_equal(&json!(null), &json!("")));
}

#[test]
fn null_not_equal_empty_array() {
    assert!(!deep_equal(&json!(null), &json!([])));
}

#[test]
fn null_not_equal_empty_object() {
    assert!(!deep_equal(&json!(null), &json!({})));
}

// ---------------------------------------------------------------------------
// Type mismatches
// ---------------------------------------------------------------------------

#[test]
fn type_mismatch_number_vs_bool() {
    assert!(!deep_equal(&json!(1), &json!(true)));
    assert!(!deep_equal(&json!(0), &json!(false)));
}

#[test]
fn type_mismatch_number_vs_string() {
    assert!(!deep_equal(&json!(1), &json!("1")));
}

#[test]
fn type_mismatch_number_vs_array() {
    assert!(!deep_equal(&json!(1), &json!([])));
    assert!(!deep_equal(&json!(1), &json!([1])));
}

#[test]
fn type_mismatch_string_vs_array() {
    assert!(!deep_equal(&json!("a"), &json!(["a"])));
}

#[test]
fn type_mismatch_object_vs_array() {
    assert!(!deep_equal(&json!({}), &json!([])));
}

#[test]
fn type_mismatch_bool_vs_string() {
    assert!(!deep_equal(&json!(true), &json!("true")));
}

// ---------------------------------------------------------------------------
// Number edge cases
// ---------------------------------------------------------------------------

#[test]
fn number_zero_variants() {
    assert!(deep_equal(&json!(0), &json!(0)));
    // serde_json treats 0.0 as a float and 0 as an integer — different Number types
    assert!(!deep_equal(&json!(0.0), &json!(0)));
}

#[test]
fn number_equal_integers() {
    assert!(deep_equal(&json!(42), &json!(42)));
}

#[test]
fn number_unequal_integers() {
    assert!(!deep_equal(&json!(42), &json!(43)));
}

#[test]
fn number_negative() {
    assert!(deep_equal(&json!(-1), &json!(-1)));
    assert!(!deep_equal(&json!(-1), &json!(1)));
}

#[test]
fn number_large() {
    assert!(deep_equal(&json!(999999999), &json!(999999999)));
    assert!(!deep_equal(&json!(999999999), &json!(999999998)));
}

#[test]
fn number_float() {
    assert!(deep_equal(&json!(1.5), &json!(1.5)));
    assert!(!deep_equal(&json!(1.5), &json!(1.6)));
}

// ---------------------------------------------------------------------------
// String tests
// ---------------------------------------------------------------------------

#[test]
fn string_equal() {
    assert!(deep_equal(&json!("hello"), &json!("hello")));
}

#[test]
fn string_unequal() {
    assert!(!deep_equal(&json!("hello"), &json!("world")));
}

#[test]
fn string_empty_vs_nonempty() {
    assert!(!deep_equal(&json!(""), &json!("a")));
}

#[test]
fn string_unicode() {
    assert!(deep_equal(&json!("\u{1F600}"), &json!("\u{1F600}")));
    assert!(!deep_equal(&json!("\u{1F600}"), &json!("\u{1F601}")));
}

// ---------------------------------------------------------------------------
// Array tests
// ---------------------------------------------------------------------------

#[test]
fn array_empty() {
    assert!(deep_equal(&json!([]), &json!([])));
}

#[test]
fn array_equal() {
    assert!(deep_equal(&json!([1, 2, 3]), &json!([1, 2, 3])));
}

#[test]
fn array_different_element() {
    assert!(!deep_equal(&json!([1, 2, 3]), &json!([1, 2, 4])));
}

#[test]
fn array_different_length() {
    assert!(!deep_equal(&json!([1, 2]), &json!([1, 2, 3])));
    assert!(!deep_equal(&json!([1, 2, 3]), &json!([1, 2])));
}

#[test]
fn array_different_order() {
    assert!(!deep_equal(&json!([1, 2, 3]), &json!([3, 2, 1])));
}

#[test]
fn array_nested_objects() {
    assert!(deep_equal(
        &json!([{"a": "a"}, {"b": "b"}]),
        &json!([{"a": "a"}, {"b": "b"}])
    ));
    assert!(!deep_equal(
        &json!([{"a": "a"}, {"b": "b"}]),
        &json!([{"a": "a"}, {"b": "c"}])
    ));
}

// ---------------------------------------------------------------------------
// Object tests
// ---------------------------------------------------------------------------

#[test]
fn object_empty() {
    assert!(deep_equal(&json!({}), &json!({})));
}

#[test]
fn object_equal_same_order() {
    assert!(deep_equal(
        &json!({"a": 1, "b": "2"}),
        &json!({"a": 1, "b": "2"})
    ));
}

#[test]
fn object_equal_different_order() {
    assert!(deep_equal(
        &json!({"a": 1, "b": "2"}),
        &json!({"b": "2", "a": 1})
    ));
}

#[test]
fn object_extra_key() {
    assert!(!deep_equal(&json!({"a": 1}), &json!({"a": 1, "b": 2})));
}

#[test]
fn object_different_value() {
    assert!(!deep_equal(&json!({"a": 1}), &json!({"a": 2})));
}

#[test]
fn object_different_key() {
    assert!(!deep_equal(&json!({"a": 1}), &json!({"b": 1})));
}

// ---------------------------------------------------------------------------
// Deeply nested structures
// ---------------------------------------------------------------------------

#[test]
fn deeply_nested_equal() {
    let a = json!({
        "prop1": "value1",
        "prop2": "value2",
        "prop3": "value3",
        "prop4": {
            "subProp1": "sub value1",
            "subProp2": {
                "subSubProp1": "sub sub value1",
                "subSubProp2": [1, 2, {"prop2": 1, "prop": 2}, 4, 5]
            }
        },
        "prop5": 1000
    });
    let b = json!({
        "prop5": 1000,
        "prop3": "value3",
        "prop1": "value1",
        "prop2": "value2",
        "prop4": {
            "subProp2": {
                "subSubProp1": "sub sub value1",
                "subSubProp2": [1, 2, {"prop2": 1, "prop": 2}, 4, 5]
            },
            "subProp1": "sub value1"
        }
    });
    assert!(deep_equal(&a, &b));
}

#[test]
fn deeply_nested_unequal_leaf() {
    let a = json!({"a": {"b": {"c": 1}}});
    let b = json!({"a": {"b": {"c": 2}}});
    assert!(!deep_equal(&a, &b));
}

#[test]
fn nested_array_in_object() {
    assert!(deep_equal(
        &json!({"a": [{"b": "c"}]}),
        &json!({"a": [{"b": "c"}]})
    ));
    assert!(!deep_equal(
        &json!({"a": [{"b": "c"}]}),
        &json!({"a": [{"b": "d"}]})
    ));
}

// ---------------------------------------------------------------------------
// Boolean tests
// ---------------------------------------------------------------------------

#[test]
fn bool_equal() {
    assert!(deep_equal(&json!(true), &json!(true)));
    assert!(deep_equal(&json!(false), &json!(false)));
}

#[test]
fn bool_unequal() {
    assert!(!deep_equal(&json!(true), &json!(false)));
    assert!(!deep_equal(&json!(false), &json!(true)));
}
