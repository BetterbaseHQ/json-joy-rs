mod fixtures_json_path;

use fixtures_json_path::{array_data, bookstore, complex_data, data0, test_data};
use json_joy_json_path::{
    get_accessed_properties, json_path_equals, json_path_to_string, JsonPathCodegen, JsonPathEval,
    JsonPathParser,
};
use serde_json::json;

#[test]
fn json_path_parser_matrix() {
    let root = JsonPathParser::parse("$");
    assert!(root.success);
    assert_eq!(root.path.unwrap().segments.len(), 0);

    let dot = JsonPathParser::parse("$.store.book[0].title");
    assert!(dot.success);
    assert_eq!(dot.path.unwrap().segments.len(), 4);

    let bracket = JsonPathParser::parse("$['store']['book']");
    assert!(bracket.success);
    assert_eq!(bracket.path.unwrap().segments.len(), 2);

    let bad = JsonPathParser::parse("$.");
    assert!(!bad.success);
    assert!(bad.error.is_some());
    assert!(bad.position.is_some());
}

#[test]
fn json_path_eval_run_matrix() {
    let result = JsonPathEval::run("$.store.book[0].title", &data0());
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].data, json!("Harry Potter"));
    assert_eq!(result[0].pointer(), "$['store']['book'][0]['title']");
    assert_eq!(
        result[0].path(),
        vec![
            "$".into(),
            "store".into(),
            "book".into(),
            0.into(),
            "title".into()
        ]
    );

    let prices = JsonPathEval::run("$.store.book[?@.price < 10]", &data0());
    assert_eq!(prices.len(), 1);
    assert_eq!(prices[0].data["title"], "Harry Potter");

    let desc = JsonPathEval::run("$..price", &data0());
    let vals: Vec<f64> = desc.iter().map(|v| v.data.as_f64().unwrap()).collect();
    assert!(vals.contains(&8.95));
    assert!(vals.contains(&12.99));
    assert!(vals.contains(&399.0));
}

#[test]
fn json_path_eval_functions_matrix() {
    let len = JsonPathEval::run("$[?length(@.authors) == 3]", &test_data());
    assert_eq!(len.len(), 1);

    let cnt = JsonPathEval::run("$[?count(@.store.book[?@.isbn]) == 2]", &test_data());
    assert_eq!(cnt.len(), 1);

    let mtch = JsonPathEval::run(
        "$.store.book[?match(@.category, \"fiction\")]",
        &test_data(),
    );
    assert_eq!(mtch.len(), 3);

    let srch = JsonPathEval::run("$.store.book[?search(@.title, \"Lord\")]", &test_data());
    assert_eq!(srch.len(), 1);
    assert_eq!(srch[0].data["title"], "The Lord of the Rings");

    let val = JsonPathEval::run("$.store.book[?value(@.isbn) != null]", &test_data());
    assert_eq!(val.len(), 2);
}

#[test]
fn json_path_descendant_selector_matrix() {
    let data = json!({
        "store": {
            "book": [
                {"title": "Book 1", "price": 10},
                {"title": "Book 2", "price": 20}
            ],
            "bicycle": {"color": "red", "price": 100}
        }
    });

    let bad = std::panic::catch_unwind(|| JsonPathEval::run("$..", &data));
    assert!(bad.is_err());

    let all = JsonPathEval::run("$..*", &data);
    let values: Vec<serde_json::Value> = all.iter().map(|v| v.data.clone()).collect();
    assert!(values.contains(&json!("Book 1")));
    assert!(values.contains(&json!(100)));

    let prices = JsonPathEval::run("$..price", &data);
    let prices_vals: Vec<f64> = prices.iter().map(|v| v.data.as_f64().unwrap()).collect();
    assert_eq!(prices_vals.len(), 3);
    assert!(prices_vals.contains(&10.0));
    assert!(prices_vals.contains(&20.0));
    assert!(prices_vals.contains(&100.0));
}

#[test]
fn json_path_codegen_matrix() {
    let query = "$.store.book[*].author";
    let data = bookstore();
    let eval_result = JsonPathEval::run(query, &data);
    let codegen_result = JsonPathCodegen::run(query, &data);
    assert_eq!(
        codegen_result
            .iter()
            .map(|v| v.data.clone())
            .collect::<Vec<_>>(),
        eval_result
            .iter()
            .map(|v| v.data.clone())
            .collect::<Vec<_>>()
    );

    let compiled = JsonPathCodegen::compile("$..price");
    let compiled_result = compiled(&data);
    let eval_prices = JsonPathEval::run("$..price", &data);
    assert_eq!(compiled_result.len(), eval_prices.len());
}

#[test]
fn json_path_util_matrix() {
    let p1 = JsonPathParser::parse("$.store.book[0].title").path.unwrap();
    let p2 = JsonPathParser::parse("$['store']['book'][0]['title']")
        .path
        .unwrap();
    let p3 = JsonPathParser::parse("$.store.book[1].title").path.unwrap();

    assert!(json_path_equals(&p1, &p2));
    assert!(!json_path_equals(&p1, &p3));
    assert_eq!(json_path_to_string(&p1), "$.store.book[0].title");

    let props =
        get_accessed_properties(&JsonPathParser::parse("$.store.book[0].title").path.unwrap());
    assert_eq!(
        props,
        vec!["store".to_string(), "book".to_string(), "title".to_string()]
    );

    let props_recursive =
        get_accessed_properties(&JsonPathParser::parse("$..author").path.unwrap());
    assert_eq!(props_recursive, vec!["author".to_string()]);

    let _ = array_data();
    let _ = complex_data();
}
