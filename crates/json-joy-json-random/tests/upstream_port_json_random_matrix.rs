use json_joy_json_random::examples as ex;
use json_joy_json_random::number::{int, int64};
use json_joy_json_random::string::{random_string, Token};
use json_joy_json_random::structured::templates;
use json_joy_json_random::structured::{ObjectTemplateField, Template, TemplateJson};
use json_joy_json_random::{NodeOdds, RandomJson, RandomJsonOptions, RootNode};
use serde_json::Value;

#[test]
fn random_json_root_variants() {
    let object = RandomJson::generate(RandomJsonOptions::default());
    assert!(object.is_object());

    let array = RandomJson::generate(RandomJsonOptions {
        root_node: Some(RootNode::Array),
        ..Default::default()
    });
    assert!(array.is_array());

    let string = RandomJson::generate(RandomJsonOptions {
        root_node: Some(RootNode::String),
        ..Default::default()
    });
    assert!(string.is_string());
}

#[test]
fn random_string_token_matrix() {
    let pick = Token::pick(vec![
        Token::literal("apple"),
        Token::literal("banana"),
        Token::literal("cherry"),
    ]);
    let picked = random_string(&pick);
    assert!(["apple", "banana", "cherry"].contains(&picked.as_str()));

    let repeated = random_string(&Token::repeat(2, 5, Token::literal("x")));
    assert!((2..=5).contains(&repeated.len()));

    let chars = random_string(&Token::char_range(65, 90, Some(3)));
    assert_eq!(chars.chars().count(), 3);

    let list = Token::list(vec![
        Token::literal("prefix-"),
        Token::pick(vec![Token::literal("a"), Token::literal("b")]),
        Token::literal("-suffix"),
    ]);
    let listed = random_string(&list);
    assert!(listed.starts_with("prefix-"));
    assert!(listed.ends_with("-suffix"));
}

#[test]
fn template_json_object_and_map() {
    let template = Template::obj(vec![
        ObjectTemplateField::required_literal_key("id", Template::int(None, None)),
        ObjectTemplateField::required_literal_key("name", Template::str(None)),
        ObjectTemplateField::optional_literal_key("nickname", Template::str(None), 1.0),
    ]);

    let value = TemplateJson::gen(Some(template), None);
    let obj = value.as_object().expect("object");
    assert!(obj.get("id").is_some());
    assert!(obj.get("name").is_some());
    assert!(obj.get("nickname").is_none());

    let map = TemplateJson::gen(
        Some(Template::map(
            Some(templates::tokens_object_key()),
            Some(Template::bool(None)),
            Some(2),
            Some(4),
        )),
        None,
    );
    let map_obj = map.as_object().expect("map object");
    assert!((2..=4).contains(&map_obj.len()));
}

#[test]
fn number_helpers_clamp() {
    for _ in 0..50 {
        let n = int(-10, 10);
        assert!((-10..=10).contains(&n));

        let b = int64(-10, 10);
        assert!((-10..=10).contains(&b));
    }
}

#[test]
fn number_helpers_accept_reversed_bounds() {
    for _ in 0..50 {
        let n = int(10, -10);
        assert!((-10..=10).contains(&n));

        let b = int64(10, -10);
        assert!((-10..=10).contains(&b));
    }
}

#[test]
fn random_string_invalid_codepoint_falls_back() {
    let token = Token::char_range(0xD800, 0xD800, Some(1));
    let out = random_string(&token);
    assert_eq!(out, "\u{FFFD}");
}

#[test]
fn random_json_can_force_boolean_nodes() {
    let value = RandomJson::generate(RandomJsonOptions {
        root_node: Some(RootNode::Object),
        node_count: 8,
        odds: NodeOdds {
            null: 0,
            boolean: 1,
            number: 0,
            string: 0,
            binary: 0,
            array: 0,
            object: 0,
        },
        strings: None,
    });

    let obj = value.as_object().expect("object root");
    assert!(!obj.is_empty());
    assert!(obj.values().all(Value::is_boolean));
}

#[test]
fn random_json_can_force_binary_nodes() {
    let value = RandomJson::generate(RandomJsonOptions {
        root_node: Some(RootNode::Array),
        node_count: 6,
        odds: NodeOdds {
            null: 0,
            boolean: 0,
            number: 0,
            string: 0,
            binary: 1,
            array: 0,
            object: 0,
        },
        strings: None,
    });

    let arr = value.as_array().expect("array root");
    assert!(!arr.is_empty());
    for item in arr {
        let bytes = item.as_array().expect("binary is encoded as JSON array");
        assert!(bytes.iter().all(Value::is_number));
    }
}

#[test]
fn random_json_zero_total_odds_defaults_to_null_nodes() {
    let value = RandomJson::generate(RandomJsonOptions {
        root_node: Some(RootNode::Object),
        node_count: 3,
        odds: NodeOdds {
            null: 0,
            boolean: 0,
            number: 0,
            string: 0,
            binary: 0,
            array: 0,
            object: 0,
        },
        strings: None,
    });
    let obj = value.as_object().expect("object root");
    assert!(!obj.is_empty());
    assert!(obj.values().all(Value::is_null));
}

#[test]
fn template_json_exercises_additional_template_variants() {
    let arr = TemplateJson::gen(
        Some(Template::arr(
            Some(1),
            Some(1),
            Some(Template::int(Some(1), Some(1))),
            vec![Template::lit(Value::String("head".into()))],
            vec![Template::lit(Value::String("tail".into()))],
        )),
        None,
    );
    let arr = arr.as_array().expect("array");
    assert_eq!(arr.first(), Some(&Value::String("head".into())));
    assert_eq!(arr.last(), Some(&Value::String("tail".into())));

    let lit = TemplateJson::gen(Some(Template::lit(serde_json::json!({"k": [1,2,3]}))), None);
    assert_eq!(lit["k"][1], Value::Number(2.into()));

    let or = TemplateJson::gen(
        Some(Template::or(vec![Template::lit(Value::String(
            "only".into(),
        ))])),
        None,
    );
    assert_eq!(or, Value::String("only".into()));

    let bin = TemplateJson::gen(
        Some(Template::bin(Some(3), Some(3), Some(42), Some(42))),
        None,
    );
    let bin = bin.as_array().expect("bin as json array");
    assert_eq!(bin.len(), 3);
    assert!(bin.iter().all(|v| *v == Value::Number(42.into())));
}

#[test]
fn examples_generators_are_callable() {
    let user = ex::gen_user();
    assert!(user.is_object());

    let api = ex::gen_api_response();
    assert!(api.is_object());

    let random = ex::gen_random_example();
    assert!(random.is_null() || random.is_object() || random.is_array() || random.is_string());
}

#[test]
fn examples_template_catalog_matrix() {
    let object_templates = vec![
        ex::api_response_detailed(),
        ex::service_config(),
        ex::product(),
        ex::order(),
        ex::user_token(),
        ex::user_role(),
        ex::log_entry(),
        ex::metric_data(),
        ex::address(),
        ex::location(),
        ex::transaction(),
        ex::bank_account(),
        ex::social_post(),
        ex::social_profile(),
        ex::sensor_reading(),
        ex::iot_device(),
        ex::patient(),
        ex::medical_record(),
        ex::student(),
        ex::course(),
        ex::grade(),
        ex::empty_structures(),
        ex::unicode_text(),
        ex::large_numbers(),
        ex::load_test_user(),
    ];
    for template in object_templates {
        let value = TemplateJson::gen(Some(template), None);
        assert!(value.is_object(), "expected object template");
    }

    let map_templates = vec![ex::config_map(), ex::permissions(), ex::translations()];
    for template in map_templates {
        let value = TemplateJson::gen(Some(template), None);
        assert!(value.is_object(), "expected map/object template");
    }

    let recursive_templates = vec![ex::tree(), ex::comment()];
    for template in recursive_templates {
        let value = TemplateJson::gen(Some(template), None);
        assert!(value.is_object(), "expected recursive object template");
    }

    let coords = TemplateJson::gen(Some(ex::coordinates()), None);
    assert!(coords.is_array());
    assert_eq!(coords.as_array().expect("coordinates array").len(), 3);
    assert_eq!(coords[2], Value::String("WGS84".to_string()));

    let perf = TemplateJson::gen(Some(ex::performance_test()), None);
    assert!(perf.is_array());
    assert_eq!(perf.as_array().expect("performance array").len(), 100);

    let mixed = TemplateJson::gen(Some(ex::mixed_types()), None);
    assert!(
        mixed.is_string()
            || mixed.is_number()
            || mixed.is_boolean()
            || mixed.is_null()
            || mixed.is_array()
            || mixed.is_object()
    );

    let any = TemplateJson::gen(Some(ex::all_examples()), None);
    assert!(
        any.is_null()
            || any.is_object()
            || any.is_array()
            || any.is_string()
            || any.is_number()
            || any.is_boolean()
    );
}
