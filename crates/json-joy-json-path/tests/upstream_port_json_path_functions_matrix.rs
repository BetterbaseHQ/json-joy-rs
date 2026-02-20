use json_joy_json_path::{JsonPathEval, JsonPathParser};
use serde_json::{json, Value};

fn test_data() -> Value {
    json!({
        "store": {
            "book": [
                {
                    "category": "reference",
                    "author": "Nigel Rees",
                    "title": "Sayings of the Century",
                    "price": 8.95
                },
                {
                    "category": "fiction",
                    "author": "Evelyn Waugh",
                    "title": "Sword of Honour",
                    "price": 12.99
                },
                {
                    "category": "fiction",
                    "author": "Herman Melville",
                    "title": "Moby Dick",
                    "isbn": "0-553-21311-3",
                    "price": 8.99
                },
                {
                    "category": "fiction",
                    "author": "J. R. R. Tolkien",
                    "title": "The Lord of the Rings",
                    "isbn": "0-395-19395-8",
                    "price": 22.99
                }
            ],
            "bicycle": {
                "color": "red",
                "price": 19.95
            }
        },
        "authors": ["John", "Jane", "Bob"],
        "info": {
            "name": "Test Store",
            "location": "City",
            "contacts": {
                "email": "test@store.com",
                "phone": "123-456-7890"
            }
        }
    })
}

fn eval_values(path: &str, data: &Value) -> Vec<Value> {
    let parsed =
        JsonPathParser::parse(path).unwrap_or_else(|e| panic!("parse failed for '{path}': {e}"));
    JsonPathEval::eval(&parsed, data)
        .into_iter()
        .cloned()
        .collect()
}

#[test]
fn function_length_and_count_matrix() {
    let data = test_data();

    assert_eq!(eval_values("$[?length(@.info.name) == 10]", &data).len(), 1);
    assert_eq!(eval_values("$[?length(@.authors) == 3]", &data).len(), 1);
    assert_eq!(
        eval_values("$[?length(@.info.contacts) == 2]", &data).len(),
        1
    );
    assert_eq!(
        eval_values("$[?length(@.nonexistent) == 0]", &data).len(),
        0
    );

    assert_eq!(
        eval_values("$[?count(@.store.book[*]) == 4]", &data).len(),
        1
    );
    assert_eq!(
        eval_values("$[?count(@.store.book[?@.isbn]) == 2]", &data).len(),
        1
    );
    assert_eq!(eval_values("$[?count(@..price) == 5]", &data).len(), 1);
}

#[test]
fn function_match_and_search_matrix() {
    let data = test_data();

    assert_eq!(
        eval_values("$.store.book[?match(@.category, \"fiction\")]", &data).len(),
        3
    );
    assert_eq!(
        eval_values("$.store.book[?match(@.title, \".*Lord.*\")]", &data).len(),
        1
    );
    assert_eq!(
        eval_values("$.store.book[?match(@.title, \"[\")]", &data).len(),
        0
    );

    assert_eq!(
        eval_values("$.store.book[?search(@.title, \"Lord\")]", &data).len(),
        1
    );
    assert_eq!(
        eval_values("$.authors[?search(@, \"^Bob$\")]", &data),
        vec![json!("Bob")]
    );
}

#[test]
fn function_value_matrix() {
    let data = test_data();

    assert_eq!(
        eval_values("$[?value(@..color) == \"red\"]", &data).len(),
        1
    );
    assert_eq!(
        eval_values("$.store.book[?value(@.price) < 10]", &data).len(),
        2
    );
    assert_eq!(eval_values("$[?value(@..price) == 8.95]", &data).len(), 0);
    assert_eq!(
        eval_values("$[?value(@.nonexistent) == null]", &data).len(),
        0
    );
    assert_eq!(
        eval_values("$.store.book[?value(@.isbn) != null]", &data).len(),
        2
    );
}

#[test]
fn function_combined_usage_matrix() {
    let data = test_data();

    assert_eq!(
        eval_values("$[?length(@.authors) == count(@.authors[*])]", &data).len(),
        1
    );

    let match_result = eval_values("$.store.book[?match(@.title, \"Lord\")]", &data);
    let search_result = eval_values("$.store.book[?search(@.title, \"Lord\")]", &data);
    assert!(match_result.is_empty());
    assert_eq!(search_result.len(), 1);

    assert_eq!(
        eval_values("$[?count(@.store.book[?length(@.title) > 15]) == 2]", &data).len(),
        1
    );
    assert_eq!(
        eval_values("$.store.book[?length(value(@.title)) > 10]", &data).len(),
        3
    );
    assert_eq!(
        eval_values(
            "$.store.book[?length(@.title) > 10 && search(@.category, \"fiction\")]",
            &data,
        )
        .len(),
        2
    );
}
