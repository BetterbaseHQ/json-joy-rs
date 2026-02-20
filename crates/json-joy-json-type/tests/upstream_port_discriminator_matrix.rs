use json_joy_json_type::type_def::discriminator::Discriminator;
use json_joy_json_type::type_def::TypeBuilder;
use json_joy_json_type::{validate, ErrorMode, ValidationResult, ValidatorOptions};
use serde_json::json;

fn t() -> TypeBuilder {
    TypeBuilder::new()
}

fn opts_bool() -> ValidatorOptions {
    ValidatorOptions {
        errors: ErrorMode::Boolean,
        ..Default::default()
    }
}

#[test]
fn discriminator_find_const_at_root_matrix() {
    let d1 = Discriminator::find(&t().Const(json!("foo"), None));
    let d2 = Discriminator::find(&t().Const(json!(123), None));
    let d3 = Discriminator::find(&t().Const(json!([true, false]), None));

    assert_eq!(d1.to_specifier(), r#"["","con","foo"]"#);
    assert_eq!(d2.to_specifier(), r#"["","con",123]"#);
    assert_eq!(d3.to_specifier(), r#"["","con",[true,false]]"#);
}

#[test]
fn discriminator_find_const_in_tuple_matrix() {
    let d1 = Discriminator::find(&t().tuple(vec![t().Const(json!("foo"), None)]));
    let d2 =
        Discriminator::find(&t().tuple(vec![t().Const(json!("add"), None), t().str(), t().any()]));
    let d3 = Discriminator::find(&t().tuple(vec![
        t().map(),
        t().obj(),
        t().Const(json!(null), None),
        t().num(),
    ]));

    assert_eq!(d1.to_specifier(), r#"["/0","con","foo"]"#);
    assert_eq!(d2.to_specifier(), r#"["/0","con","add"]"#);
    assert_eq!(d3.to_specifier(), r#"["/2","con",null]"#);
}

#[test]
fn discriminator_find_const_in_object_matrix() {
    let ty = t().Object(vec![
        json_joy_json_type::type_def::KeyType::new("op", t().Const(json!("replace"), None)),
        json_joy_json_type::type_def::KeyType::new("value", t().num()),
        json_joy_json_type::type_def::KeyType::new("path", t().str()),
    ]);
    let d = Discriminator::find(&ty);
    assert_eq!(d.to_specifier(), r#"["/op","con","replace"]"#);
}

#[test]
fn discriminator_uses_node_type_when_no_const_matrix() {
    let d1 = Discriminator::find(&t().Map(t().str(), None, None));
    let d2 = Discriminator::find(&t().obj());
    let d3 = Discriminator::find(&t().str());

    assert_eq!(d1.to_specifier(), r#"["","obj",0]"#);
    assert_eq!(d2.to_specifier(), r#"["","obj",0]"#);
    assert_eq!(d3.to_specifier(), r#"["","str",0]"#);
}

#[test]
fn discriminator_find_nested_const_matrix() {
    let d1 = Discriminator::find(&t().tuple(vec![
        t().str(),
        t().tuple(vec![t().num(), t().Const(json!("foo"), None)]),
    ]));
    let d2 = Discriminator::find(&t().Object(vec![
        json_joy_json_type::type_def::KeyType::new(
            "type",
            t().tuple(vec![t().Const(json!(25), None), t().str(), t().any()]),
        ),
        json_joy_json_type::type_def::KeyType::new("value", t().num()),
    ]));

    assert_eq!(d1.to_specifier(), r#"["/1/1","con","foo"]"#);
    assert_eq!(d2.to_specifier(), r#"["/type/0","con",25]"#);
}

#[test]
fn or_type_infers_discriminator_expression_matrix() {
    let or = t().Or(vec![t().str(), t().num()]);
    let schema = or.get_schema();
    match schema {
        json_joy_json_type::schema::Schema::Or(or_schema) => {
            assert_eq!(
                or_schema.discriminator,
                json!(["?", ["==", ["type", ["$", ""]], "number"], 1, 0])
            );
        }
        _ => panic!("expected Or schema"),
    }
}

#[test]
fn or_validation_matrix_discriminator_coverage() {
    let or = t().Or(vec![
        t().Object(vec![
            json_joy_json_type::type_def::KeyType::new("op", t().Const(json!("replace"), None)),
            json_joy_json_type::type_def::KeyType::new("path", t().str()),
            json_joy_json_type::type_def::KeyType::new("value", t().any()),
        ]),
        t().Object(vec![
            json_joy_json_type::type_def::KeyType::new("op", t().Const(json!("add"), None)),
            json_joy_json_type::type_def::KeyType::new("path", t().str()),
            json_joy_json_type::type_def::KeyType::new("value", t().any()),
        ]),
        t().Object(vec![
            json_joy_json_type::type_def::KeyType::new("op", t().Const(json!("remove"), None)),
            json_joy_json_type::type_def::KeyType::new("path", t().str()),
        ]),
    ]);

    let ok_values = vec![
        json!({"op":"replace","path":"/foo","value":123}),
        json!({"op":"add","path":"/foo","value":{"x":1}}),
        json!({"op":"remove","path":"/foo"}),
    ];
    for value in ok_values {
        assert!(validate(&value, &or, &opts_bool(), &[]).is_ok());
    }

    for value in [
        json!({"op":"replace2","path":"/foo","value":123}),
        json!({"op":"add","path":123,"value":{"x":1}}),
        json!({"op":"remove","path":"/foo","from":"/bar"}),
        json!([]),
        json!({}),
        json!(123),
    ] {
        let result = validate(&value, &or, &opts_bool(), &[]);
        assert!(
            matches!(result, ValidationResult::BoolError),
            "expected BoolError for {value:?}, got {result:?}"
        );
    }
}
