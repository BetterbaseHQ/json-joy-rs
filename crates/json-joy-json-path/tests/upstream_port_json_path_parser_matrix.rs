use json_joy_json_path::{
    ComparisonOperator, FilterExpression, JsonPathParser, LogicalOperator, Selector,
    ValueExpression,
};

#[test]
fn parser_union_selector_matrix() {
    let path = JsonPathParser::parse("$['a','b','c']").unwrap();
    assert_eq!(path.segments.len(), 1);
    assert_eq!(path.segments[0].selectors.len(), 3);
    assert!(matches!(path.segments[0].selectors[0], Selector::Name(_)));
    assert!(matches!(path.segments[0].selectors[1], Selector::Name(_)));
    assert!(matches!(path.segments[0].selectors[2], Selector::Name(_)));

    let path = JsonPathParser::parse("$[0, 'name', 2]").unwrap();
    assert_eq!(path.segments.len(), 1);
    assert_eq!(path.segments[0].selectors.len(), 3);
    assert!(matches!(path.segments[0].selectors[0], Selector::Index(0)));
    assert!(matches!(path.segments[0].selectors[1], Selector::Name(_)));
    assert!(matches!(path.segments[0].selectors[2], Selector::Index(2)));

    let path = JsonPathParser::parse("$[0:2, 5, 'key']").unwrap();
    assert_eq!(path.segments.len(), 1);
    assert_eq!(path.segments[0].selectors.len(), 3);
    assert!(matches!(
        path.segments[0].selectors[0],
        Selector::Slice { .. }
    ));
    assert!(matches!(path.segments[0].selectors[1], Selector::Index(5)));
    assert!(matches!(path.segments[0].selectors[2], Selector::Name(_)));

    let path = JsonPathParser::parse("$[*, 0, 'key']").unwrap();
    assert_eq!(path.segments.len(), 1);
    assert_eq!(path.segments[0].selectors.len(), 3);
    assert!(matches!(path.segments[0].selectors[0], Selector::Wildcard));
    assert!(matches!(path.segments[0].selectors[1], Selector::Index(0)));
    assert!(matches!(path.segments[0].selectors[2], Selector::Name(_)));

    let path = JsonPathParser::parse("$[ 0 , 'name' , 2 ]").unwrap();
    assert_eq!(path.segments.len(), 1);
    assert_eq!(path.segments[0].selectors.len(), 3);
    assert!(matches!(path.segments[0].selectors[0], Selector::Index(0)));
    assert!(matches!(path.segments[0].selectors[1], Selector::Name(_)));
    assert!(matches!(path.segments[0].selectors[2], Selector::Index(2)));

    let path = JsonPathParser::parse("$[-1, -2, 0]").unwrap();
    assert_eq!(path.segments.len(), 1);
    assert_eq!(path.segments[0].selectors.len(), 3);
    assert!(matches!(path.segments[0].selectors[0], Selector::Index(-1)));
    assert!(matches!(path.segments[0].selectors[1], Selector::Index(-2)));
    assert!(matches!(path.segments[0].selectors[2], Selector::Index(0)));

    let path = JsonPathParser::parse("$.store['book', 'bicycle'][0, -1, 'title']").unwrap();
    assert_eq!(path.segments.len(), 3);
    assert!(matches!(path.segments[0].selectors[0], Selector::Name(_)));
    assert_eq!(path.segments[1].selectors.len(), 2);
    assert!(matches!(path.segments[1].selectors[0], Selector::Name(_)));
    assert!(matches!(path.segments[1].selectors[1], Selector::Name(_)));
    assert_eq!(path.segments[2].selectors.len(), 3);
    assert!(matches!(path.segments[2].selectors[0], Selector::Index(0)));
    assert!(matches!(path.segments[2].selectors[1], Selector::Index(-1)));
    assert!(matches!(path.segments[2].selectors[2], Selector::Name(_)));
}

#[test]
fn parser_filter_existence_path_matrix() {
    let path = JsonPathParser::parse("$[?@.nested.property]").unwrap();
    let selector = &path.segments[0].selectors[0];
    match selector {
        Selector::Filter(FilterExpression::Existence { path }) => {
            assert_eq!(path.segments.len(), 2);
            assert!(matches!(path.segments[0].selectors[0], Selector::Name(_)));
            assert!(matches!(path.segments[1].selectors[0], Selector::Name(_)));
        }
        other => panic!("expected existence filter, got {other:?}"),
    }

    let path = JsonPathParser::parse("$[?@['key with spaces']]").unwrap();
    let selector = &path.segments[0].selectors[0];
    match selector {
        Selector::Filter(FilterExpression::Existence { path }) => {
            assert_eq!(path.segments.len(), 1);
            assert!(matches!(path.segments[0].selectors[0], Selector::Name(_)));
        }
        other => panic!("expected existence filter, got {other:?}"),
    }
}

#[test]
fn parser_recursive_with_filter_matrix() {
    let path = JsonPathParser::parse("$..book[?@.isbn]").unwrap();
    assert_eq!(path.segments.len(), 2);
    assert!(path.segments[0].recursive);
    assert!(matches!(path.segments[0].selectors[0], Selector::Name(_)));
    assert!(!path.segments[1].recursive);
    assert!(matches!(
        path.segments[1].selectors[0],
        Selector::Filter(FilterExpression::Existence { .. })
    ));

    let path = JsonPathParser::parse("$..book[?@.price<10]").unwrap();
    assert_eq!(path.segments.len(), 2);
    let filter = &path.segments[1].selectors[0];
    match filter {
        Selector::Filter(FilterExpression::Comparison {
            operator,
            left,
            right,
        }) => {
            assert_eq!(*operator, ComparisonOperator::Less);
            assert!(matches!(left, ValueExpression::Path(_)));
            assert!(matches!(right, ValueExpression::Literal(_)));
        }
        other => panic!("expected comparison filter, got {other:?}"),
    }
}

#[test]
fn parser_logical_filter_matrix() {
    let path = JsonPathParser::parse("$[?@.isbn && @.price < 20]").unwrap();
    let selector = &path.segments[0].selectors[0];
    match selector {
        Selector::Filter(FilterExpression::Logical {
            operator,
            left,
            right,
        }) => {
            assert_eq!(*operator, LogicalOperator::And);
            assert!(matches!(left.as_ref(), FilterExpression::Existence { .. }));
            assert!(matches!(
                right.as_ref(),
                FilterExpression::Comparison { .. }
            ));
        }
        other => panic!("expected logical filter, got {other:?}"),
    }
}

#[test]
fn parser_function_and_nested_filter_matrix() {
    let path = JsonPathParser::parse("$[?length(@.name)]").unwrap();
    let selector = &path.segments[0].selectors[0];
    match selector {
        Selector::Filter(FilterExpression::Function { name, args }) => {
            assert_eq!(name, "length");
            assert_eq!(args.len(), 1);
        }
        other => panic!("expected function filter, got {other:?}"),
    }

    let path =
        JsonPathParser::parse("$[?((@.price < 10 || @.price > 100) && @.category == \"book\")]")
            .unwrap();
    let selector = &path.segments[0].selectors[0];
    match selector {
        Selector::Filter(FilterExpression::Logical {
            operator,
            left,
            right,
        }) => {
            assert_eq!(*operator, LogicalOperator::And);
            assert!(matches!(left.as_ref(), FilterExpression::Paren(_)));
            assert!(matches!(
                right.as_ref(),
                FilterExpression::Comparison {
                    operator: ComparisonOperator::Equal,
                    ..
                }
            ));
        }
        other => panic!("expected nested logical filter, got {other:?}"),
    }
}

#[test]
fn parser_edge_case_syntax_matrix() {
    let path = JsonPathParser::parse("$['']").unwrap();
    assert_eq!(path.segments.len(), 1);
    assert!(matches!(
        &path.segments[0].selectors[0],
        Selector::Name(name) if name.is_empty()
    ));

    let path = JsonPathParser::parse("$['key with spaces']").unwrap();
    assert_eq!(path.segments.len(), 1);
    assert!(matches!(
        &path.segments[0].selectors[0],
        Selector::Name(name) if name == "key with spaces"
    ));

    let path = JsonPathParser::parse("$['key\\'with\\'quotes']").unwrap();
    assert_eq!(path.segments.len(), 1);
    assert!(matches!(
        &path.segments[0].selectors[0],
        Selector::Name(name) if name == "key'with'quotes"
    ));

    let path = JsonPathParser::parse("$ . store [ 'book' ] [ 0 ] . title ").unwrap();
    assert_eq!(path.segments.len(), 4);
    assert!(matches!(path.segments[0].selectors[0], Selector::Name(_)));
    assert!(matches!(path.segments[1].selectors[0], Selector::Name(_)));
    assert!(matches!(path.segments[2].selectors[0], Selector::Index(0)));
    assert!(matches!(path.segments[3].selectors[0], Selector::Name(_)));
}

#[test]
fn parser_error_matrix() {
    assert!(JsonPathParser::parse(".name").is_err());
    assert!(JsonPathParser::parse("$['unterminated").is_err());
    assert!(JsonPathParser::parse("$[invalid]").is_err());
    assert!(JsonPathParser::parse("$[0").is_err());
}
