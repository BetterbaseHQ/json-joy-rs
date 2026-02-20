use json_joy_json_path::{JsonPathCodegen, JsonPathEval, JsonPathParser, ValueNode};
use serde_json::{json, Value};

fn demo_ast_fixture() -> Value {
    json!({
        "type": "Program",
        "body": [
            null,
            null,
            null,
            {
                "declaration": {
                    "body": {
                        "body": [
                            null,
                            {
                                "value": {
                                    "body": {
                                        "body": [
                                            null,
                                            null,
                                            null,
                                            null,
                                            {
                                                "body": {
                                                    "body": [
                                                        {
                                                            "expression": {
                                                                "right": {
                                                                    "arguments": [
                                                                        null,
                                                                        {
                                                                            "type": "BinaryExpression",
                                                                            "operator": "+",
                                                                            "left": {"type": "Identifier", "name": "x"},
                                                                            "right": {"type": "Literal", "value": 1},
                                                                            "range": [455, 462]
                                                                        }
                                                                    ]
                                                                }
                                                            }
                                                        },
                                                        {
                                                            "expression": {
                                                                "right": {
                                                                    "arguments": [
                                                                        null,
                                                                        {
                                                                            "type": "BinaryExpression",
                                                                            "operator": "-",
                                                                            "left": {"type": "Identifier", "name": "y"},
                                                                            "right": {"type": "Literal", "value": 2},
                                                                            "range": [470, 477]
                                                                        }
                                                                    ]
                                                                }
                                                            }
                                                        },
                                                        {
                                                            "expression": {
                                                                "arguments": [
                                                                    null,
                                                                    {
                                                                        "type": "BinaryExpression",
                                                                        "operator": "+",
                                                                        "left": {"type": "Identifier", "name": "z"},
                                                                        "right": {"type": "Literal", "value": 3},
                                                                        "range": [515, 522]
                                                                    }
                                                                ]
                                                            }
                                                        }
                                                    ]
                                                }
                                            }
                                        ]
                                    }
                                }
                            }
                        ]
                    }
                }
            }
        ]
    })
}

#[test]
fn demo_typescript_ast_query_matrix() {
    let query = "$..[?@.type == \"BinaryExpression\" && @.operator == \"+\" && @..left.type == 'Identifier'].range";
    let ast = demo_ast_fixture();
    let parsed =
        JsonPathParser::parse(query).unwrap_or_else(|e| panic!("parse failed for '{query}': {e}"));

    let result = JsonPathEval::eval_query(&parsed, &ast);
    assert_eq!(result.values.len(), 2);
    assert_eq!(result.values[0], &json!([455, 462]));
    assert_eq!(result.values[1], &json!([515, 522]));

    let node0 = ValueNode::new(result.values[0], result.paths[0].clone());
    assert_eq!(
        node0.json_path(),
        "$['body'][3]['declaration']['body']['body'][1]['value']['body']['body'][4]['body']['body'][0]['expression']['right']['arguments'][1]['range']"
    );

    let node1 = ValueNode::new(result.values[1], result.paths[1].clone());
    assert_eq!(
        node1.json_path(),
        "$['body'][3]['declaration']['body']['body'][1]['value']['body']['body'][4]['body']['body'][2]['expression']['arguments'][1]['range']"
    );
}

#[test]
fn demo_typescript_ast_codegen_eval_parity_matrix() {
    let query = "$..[?@.type == \"BinaryExpression\" && @.operator == \"+\" && @..left.type == 'Identifier'].range";
    let ast = demo_ast_fixture();
    let parsed =
        JsonPathParser::parse(query).unwrap_or_else(|e| panic!("parse failed for '{query}': {e}"));

    let eval_values: Vec<Value> = JsonPathEval::eval(&parsed, &ast)
        .into_iter()
        .cloned()
        .collect();
    let codegen_values: Vec<Value> = JsonPathCodegen::run(query, &ast)
        .unwrap_or_else(|e| panic!("codegen failed for '{query}': {e}"))
        .into_iter()
        .cloned()
        .collect();

    assert_eq!(codegen_values, eval_values);
    assert_eq!(eval_values, vec![json!([455, 462]), json!([515, 522])]);
}
