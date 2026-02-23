//! Converts TypeNode to TypeScript AST.
//!
//! Upstream reference: json-type/src/typescript/converter.ts

use serde_json::Value;

use super::types::{TsDeclaration, TsMember, TsParam, TsType};
use crate::type_def::TypeNode;

/// Convert a `TypeNode` to a TypeScript AST type node.
///
/// Ports `toTypeScriptAst` from `json-type/src/typescript/converter.ts`.
pub fn to_typescript_ast(type_: &TypeNode) -> TsType {
    match type_ {
        TypeNode::Any(_) => TsType::Any,

        TypeNode::Bool(_) => TsType::Boolean,

        TypeNode::Num(_) => TsType::Number,

        TypeNode::Str(_) => TsType::String,

        TypeNode::Bin(_) => TsType::TypeReference {
            name: "Uint8Array".into(),
            type_args: vec![],
        },

        TypeNode::Con(t) => match &t.value {
            Value::Bool(true) => TsType::True,
            Value::Bool(false) => TsType::False,
            Value::String(s) => TsType::StringLiteral(s.clone()),
            Value::Number(n) => TsType::NumericLiteral(n.to_string()),
            Value::Null => TsType::Null,
            _ => TsType::Object,
        },

        TypeNode::Arr(t) => {
            // Tuple when head or tail items are present
            if !t.head.is_empty() || !t.tail.is_empty() {
                let mut elements: Vec<TsType> = Vec::new();
                for h in &t.head {
                    elements.push(to_typescript_ast(h));
                }
                if let Some(body) = &t.type_ {
                    elements.push(TsType::Rest(Box::new(to_typescript_ast(body))));
                }
                for tail_item in &t.tail {
                    elements.push(to_typescript_ast(tail_item));
                }
                TsType::Tuple(elements)
            } else if let Some(item_type) = &t.type_ {
                TsType::Array(Box::new(to_typescript_ast(item_type)))
            } else {
                TsType::Array(Box::new(TsType::Unknown))
            }
        }

        TypeNode::Obj(t) => {
            let members: Vec<TsMember> = t
                .keys
                .iter()
                .map(|key| {
                    let comment =
                        build_comment(key.base.title.as_deref(), key.base.description.as_deref());
                    TsMember::Property {
                        name: key.key.clone(),
                        type_: to_typescript_ast(&key.val),
                        optional: key.optional,
                        comment,
                    }
                })
                .collect();
            let mut all_members = members;
            if t.schema.decode_unknown_keys.unwrap_or(false)
                || t.schema.encode_unknown_keys.unwrap_or(false)
            {
                all_members.push(TsMember::Index {
                    type_: TsType::Unknown,
                });
            }
            let comment = {
                let base = &t.schema.base;
                build_comment(base.title.as_deref(), base.description.as_deref())
            };
            TsType::TypeLiteral {
                members: all_members,
                comment,
            }
        }

        TypeNode::Map(t) => TsType::TypeReference {
            name: "Record".into(),
            type_args: vec![TsType::String, to_typescript_ast(&t.value)],
        },

        TypeNode::Or(t) => TsType::Union(t.types.iter().map(to_typescript_ast).collect()),

        TypeNode::Ref(t) => TsType::TypeReference {
            name: t.ref_.clone(),
            type_args: vec![],
        },

        TypeNode::Fn(t) => TsType::FnType {
            params: vec![TsParam {
                name: "request".into(),
                type_: to_typescript_ast(&t.req),
            }],
            return_type: Box::new(TsType::TypeReference {
                name: "Promise".into(),
                type_args: vec![to_typescript_ast(&t.res)],
            }),
        },

        TypeNode::FnRx(t) => TsType::FnType {
            params: vec![TsParam {
                name: "request$".into(),
                type_: TsType::TypeReference {
                    name: "Observable".into(),
                    type_args: vec![to_typescript_ast(&t.req)],
                },
            }],
            return_type: Box::new(TsType::TypeReference {
                name: "Observable".into(),
                type_args: vec![to_typescript_ast(&t.res)],
            }),
        },

        TypeNode::Key(t) => to_typescript_ast(&t.val),

        TypeNode::Alias(alias) => to_typescript_ast(alias.get_type()),
    }
}

/// Build a JSDoc-style comment string from title and description.
fn build_comment(title: Option<&str>, description: Option<&str>) -> Option<String> {
    match (title, description) {
        (None, None) => None,
        (Some(t), None) => Some(t.to_string()),
        (None, Some(d)) => Some(d.to_string()),
        (Some(t), Some(d)) => Some(format!("{}\n\n{}", t, d)),
    }
}

/// Convert an `AliasType` to a top-level TypeScript declaration.
///
/// Ports `aliasToTs` from `json-type/src/typescript/converter.ts`.
pub fn alias_to_ts(type_: &TypeNode, name: &str) -> TsDeclaration {
    match type_ {
        TypeNode::Obj(obj) => {
            let members: Vec<TsMember> = obj
                .keys
                .iter()
                .map(|key| {
                    let comment =
                        build_comment(key.base.title.as_deref(), key.base.description.as_deref());
                    TsMember::Property {
                        name: key.key.clone(),
                        type_: to_typescript_ast(&key.val),
                        optional: key.optional,
                        comment,
                    }
                })
                .collect();
            TsDeclaration::Interface {
                name: name.to_string(),
                members,
                comment: None,
            }
        }
        _ => TsDeclaration::TypeAlias {
            name: name.to_string(),
            type_: to_typescript_ast(type_),
            comment: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_def::classes::*;
    use crate::type_def::TypeBuilder;
    use serde_json::json;

    fn t() -> TypeBuilder {
        TypeBuilder::new()
    }

    #[test]
    fn convert_any() {
        let node = t().any();
        assert!(matches!(to_typescript_ast(&node), TsType::Any));
    }

    #[test]
    fn convert_bool() {
        let node = t().bool();
        assert!(matches!(to_typescript_ast(&node), TsType::Boolean));
    }

    #[test]
    fn convert_num() {
        let node = t().num();
        assert!(matches!(to_typescript_ast(&node), TsType::Number));
    }

    #[test]
    fn convert_str() {
        let node = t().str();
        assert!(matches!(to_typescript_ast(&node), TsType::String));
    }

    #[test]
    fn convert_bin() {
        let node = t().bin();
        if let TsType::TypeReference { name, type_args } = to_typescript_ast(&node) {
            assert_eq!(name, "Uint8Array");
            assert!(type_args.is_empty());
        } else {
            panic!("Expected TypeReference for bin");
        }
    }

    #[test]
    fn convert_con_true() {
        let node = t().Const(json!(true), None);
        assert!(matches!(to_typescript_ast(&node), TsType::True));
    }

    #[test]
    fn convert_con_false() {
        let node = t().Const(json!(false), None);
        assert!(matches!(to_typescript_ast(&node), TsType::False));
    }

    #[test]
    fn convert_con_string() {
        let node = t().Const(json!("hello"), None);
        if let TsType::StringLiteral(s) = to_typescript_ast(&node) {
            assert_eq!(s, "hello");
        } else {
            panic!("Expected StringLiteral");
        }
    }

    #[test]
    fn convert_con_number() {
        let node = t().Const(json!(42), None);
        if let TsType::NumericLiteral(s) = to_typescript_ast(&node) {
            assert_eq!(s, "42");
        } else {
            panic!("Expected NumericLiteral");
        }
    }

    #[test]
    fn convert_con_null() {
        let node = t().Const(json!(null), None);
        assert!(matches!(to_typescript_ast(&node), TsType::Null));
    }

    #[test]
    fn convert_con_object_fallback() {
        // Arrays and objects in Con fall back to Object
        let node = t().Const(json!([1, 2, 3]), None);
        assert!(matches!(to_typescript_ast(&node), TsType::Object));
    }

    #[test]
    fn convert_arr_simple() {
        let node = t().Array(t().num(), None);
        if let TsType::Array(inner) = to_typescript_ast(&node) {
            assert!(matches!(*inner, TsType::Number));
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn convert_arr_no_type_returns_unknown_array() {
        let node = TypeNode::Arr(ArrType::new(None, vec![], vec![]));
        if let TsType::Array(inner) = to_typescript_ast(&node) {
            assert!(matches!(*inner, TsType::Unknown));
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn convert_arr_tuple_with_head() {
        let node = t().Tuple(vec![t().str(), t().num()], None, None);
        if let TsType::Tuple(elements) = to_typescript_ast(&node) {
            assert_eq!(elements.len(), 2);
            assert!(matches!(elements[0], TsType::String));
            assert!(matches!(elements[1], TsType::Number));
        } else {
            panic!("Expected Tuple");
        }
    }

    #[test]
    fn convert_arr_tuple_with_head_body_tail() {
        let node = t().Tuple(vec![t().str()], Some(t().num()), Some(vec![t().bool()]));
        if let TsType::Tuple(elements) = to_typescript_ast(&node) {
            assert_eq!(elements.len(), 3);
            assert!(matches!(elements[0], TsType::String));
            assert!(matches!(elements[1], TsType::Rest(_)));
            assert!(matches!(elements[2], TsType::Boolean));
        } else {
            panic!("Expected Tuple");
        }
    }

    #[test]
    fn convert_obj_empty() {
        let node = t().obj();
        if let TsType::TypeLiteral { members, comment } = to_typescript_ast(&node) {
            assert!(members.is_empty());
            assert!(comment.is_none());
        } else {
            panic!("Expected TypeLiteral");
        }
    }

    #[test]
    fn convert_obj_with_keys() {
        let node = t().Object(vec![
            KeyType::new("name", t().str()),
            KeyType::new_opt("age", t().num()),
        ]);
        if let TsType::TypeLiteral { members, .. } = to_typescript_ast(&node) {
            assert_eq!(members.len(), 2);
            if let TsMember::Property { name, optional, .. } = &members[0] {
                assert_eq!(name, "name");
                assert!(!optional);
            }
            if let TsMember::Property { name, optional, .. } = &members[1] {
                assert_eq!(name, "age");
                assert!(optional);
            }
        } else {
            panic!("Expected TypeLiteral");
        }
    }

    #[test]
    fn convert_obj_with_unknown_keys_adds_index() {
        let mut obj = ObjType::new(vec![]);
        obj.schema.decode_unknown_keys = Some(true);
        let node = TypeNode::Obj(obj);
        if let TsType::TypeLiteral { members, .. } = to_typescript_ast(&node) {
            assert_eq!(members.len(), 1);
            assert!(matches!(members[0], TsMember::Index { .. }));
        } else {
            panic!("Expected TypeLiteral");
        }
    }

    #[test]
    fn convert_map() {
        let node = t().Map(t().num(), None, None);
        if let TsType::TypeReference { name, type_args } = to_typescript_ast(&node) {
            assert_eq!(name, "Record");
            assert_eq!(type_args.len(), 2);
            assert!(matches!(type_args[0], TsType::String));
            assert!(matches!(type_args[1], TsType::Number));
        } else {
            panic!("Expected TypeReference");
        }
    }

    #[test]
    fn convert_or() {
        let node = t().Or(vec![t().str(), t().num()]);
        if let TsType::Union(types) = to_typescript_ast(&node) {
            assert_eq!(types.len(), 2);
        } else {
            panic!("Expected Union");
        }
    }

    #[test]
    fn convert_ref() {
        let node = t().Ref("MyType");
        if let TsType::TypeReference { name, type_args } = to_typescript_ast(&node) {
            assert_eq!(name, "MyType");
            assert!(type_args.is_empty());
        } else {
            panic!("Expected TypeReference");
        }
    }

    #[test]
    fn convert_fn() {
        let node = t().Function(t().str(), t().num(), None);
        if let TsType::FnType {
            params,
            return_type,
        } = to_typescript_ast(&node)
        {
            assert_eq!(params.len(), 1);
            assert_eq!(params[0].name, "request");
            if let TsType::TypeReference { name, type_args } = return_type.as_ref() {
                assert_eq!(name, "Promise");
                assert_eq!(type_args.len(), 1);
            } else {
                panic!("Expected Promise return type");
            }
        } else {
            panic!("Expected FnType");
        }
    }

    #[test]
    fn convert_fn_rx() {
        let node = t().function_streaming(t().str(), t().num(), None);
        if let TsType::FnType {
            params,
            return_type,
        } = to_typescript_ast(&node)
        {
            assert_eq!(params.len(), 1);
            assert_eq!(params[0].name, "request$");
            if let TsType::TypeReference { name, .. } = &params[0].type_ {
                assert_eq!(name, "Observable");
            } else {
                panic!("Expected Observable param type");
            }
            if let TsType::TypeReference { name, .. } = return_type.as_ref() {
                assert_eq!(name, "Observable");
            } else {
                panic!("Expected Observable return type");
            }
        } else {
            panic!("Expected FnType");
        }
    }

    #[test]
    fn convert_key_delegates_to_val() {
        let node = TypeNode::Key(KeyType::new("x", t().num()));
        assert!(matches!(to_typescript_ast(&node), TsType::Number));
    }

    // -- build_comment --

    #[test]
    fn build_comment_none_none() {
        assert!(build_comment(None, None).is_none());
    }

    #[test]
    fn build_comment_title_only() {
        assert_eq!(build_comment(Some("Title"), None).unwrap(), "Title");
    }

    #[test]
    fn build_comment_description_only() {
        assert_eq!(build_comment(None, Some("Desc")).unwrap(), "Desc");
    }

    #[test]
    fn build_comment_both() {
        let result = build_comment(Some("Title"), Some("Desc")).unwrap();
        assert_eq!(result, "Title\n\nDesc");
    }

    // -- alias_to_ts --

    #[test]
    fn alias_to_ts_obj_creates_interface() {
        let node = t().Object(vec![KeyType::new("id", t().num())]);
        if let TsDeclaration::Interface { name, members, .. } = alias_to_ts(&node, "User") {
            assert_eq!(name, "User");
            assert_eq!(members.len(), 1);
        } else {
            panic!("Expected Interface");
        }
    }

    #[test]
    fn alias_to_ts_non_obj_creates_type_alias() {
        let node = t().str();
        if let TsDeclaration::TypeAlias { name, .. } = alias_to_ts(&node, "Name") {
            assert_eq!(name, "Name");
        } else {
            panic!("Expected TypeAlias");
        }
    }

    #[test]
    fn alias_to_ts_union_creates_type_alias() {
        let node = t().Or(vec![t().str(), t().num()]);
        if let TsDeclaration::TypeAlias { name, type_, .. } = alias_to_ts(&node, "StringOrNum") {
            assert_eq!(name, "StringOrNum");
            assert!(matches!(type_, TsType::Union(_)));
        } else {
            panic!("Expected TypeAlias");
        }
    }
}
