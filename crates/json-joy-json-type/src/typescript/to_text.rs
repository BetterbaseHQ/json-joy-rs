//! Converts TypeScript AST nodes to source text.
//!
//! Upstream reference: json-type/src/typescript/toText.ts

use super::types::{TsDeclaration, TsMember, TsParam, TsType};

const TAB: &str = "  ";

fn format_comment(comment: &str, indent: &str) -> String {
    let lines: Vec<&str> = comment.lines().collect();
    let mut out = format!("{}/**\n", indent);
    for line in &lines {
        out.push_str(&format!("{} * {}\n", indent, line));
    }
    out.push_str(&format!("{} */\n", indent));
    out
}

fn is_simple_type(t: &TsType) -> bool {
    matches!(
        t,
        TsType::Any
            | TsType::Boolean
            | TsType::Number
            | TsType::String
            | TsType::Null
            | TsType::Object
            | TsType::Unknown
            | TsType::TypeReference { .. }
    )
}

fn needs_quotes(name: &str) -> bool {
    name.is_empty()
        || name
            .chars()
            .any(|c| !c.is_alphanumeric() && c != '_' && c != '$')
        || name
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
}

fn normalize_key(name: &str) -> String {
    if needs_quotes(name) {
        format!("\"{}\"", name.replace('"', "\\\""))
    } else {
        name.to_string()
    }
}

/// Convert a `TsType` to TypeScript source text.
pub fn ts_type_to_text(t: &TsType, indent: &str) -> String {
    match t {
        TsType::Any => "any".into(),
        TsType::Boolean => "boolean".into(),
        TsType::Number => "number".into(),
        TsType::String => "string".into(),
        TsType::Null => "null".into(),
        TsType::Object => "object".into(),
        TsType::Unknown => "unknown".into(),
        TsType::True => "true".into(),
        TsType::False => "false".into(),
        TsType::StringLiteral(s) => {
            // JSON-encode the string to produce a valid TS string literal
            format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
        }
        TsType::NumericLiteral(n) => n.clone(),
        TsType::Array(elem) => {
            let inner = ts_type_to_text(elem, indent);
            if is_simple_type(elem) {
                format!("{}[]", inner)
            } else {
                format!("Array<{}>", inner)
            }
        }
        TsType::Tuple(elements) => {
            let has_complex = elements
                .iter()
                .any(|e| matches!(e, TsType::TypeLiteral { .. }));
            if has_complex {
                let inner_indent = format!("{}{}", indent, TAB);
                let parts: Vec<String> = elements
                    .iter()
                    .map(|e| format!("{}{}", inner_indent, ts_type_to_text(e, &inner_indent)))
                    .collect();
                format!("[\n{}\n{}]", parts.join(",\n"), indent)
            } else {
                let parts: Vec<String> = elements
                    .iter()
                    .map(|e| ts_type_to_text(e, indent))
                    .collect();
                format!("[{}]", parts.join(", "))
            }
        }
        TsType::Rest(inner) => format!("...{}", ts_type_to_text(inner, indent)),
        TsType::TypeLiteral { members, comment } => {
            if members.is_empty() {
                return "{}".into();
            }
            let inner_indent = format!("{}{}", indent, TAB);
            let mut out = String::new();
            if let Some(c) = comment {
                out.push_str(&format_comment(c, indent));
            }
            out.push_str("{\n");
            for member in members {
                out.push_str(&member_to_text(member, &inner_indent));
            }
            out.push_str(&format!("{}}}", indent));
            out
        }
        TsType::Union(types) => {
            let parts: Vec<String> = types.iter().map(|t| ts_type_to_text(t, indent)).collect();
            parts.join(" | ")
        }
        TsType::TypeReference { name, type_args } => {
            if type_args.is_empty() {
                name.clone()
            } else {
                let args: Vec<String> = type_args
                    .iter()
                    .map(|a| ts_type_to_text(a, indent))
                    .collect();
                format!("{}<{}>", name, args.join(", "))
            }
        }
        TsType::FnType {
            params,
            return_type,
        } => {
            let param_strs: Vec<String> = params.iter().map(|p| param_to_text(p, indent)).collect();
            format!(
                "({}) => {}",
                param_strs.join(", "),
                ts_type_to_text(return_type, indent)
            )
        }
    }
}

fn param_to_text(p: &TsParam, indent: &str) -> String {
    format!("{}: {}", p.name, ts_type_to_text(&p.type_, indent))
}

fn member_to_text(member: &TsMember, indent: &str) -> String {
    match member {
        TsMember::Property {
            name,
            type_,
            optional,
            comment,
        } => {
            let mut out = String::new();
            if let Some(c) = comment {
                out.push_str(&format_comment(c, indent));
            }
            let opt = if *optional { "?" } else { "" };
            let key = normalize_key(name);
            out.push_str(&format!(
                "{}{}{}: {};\n",
                indent,
                key,
                opt,
                ts_type_to_text(type_, indent)
            ));
            out
        }
        TsMember::Index { type_ } => {
            format!(
                "{}[key: string]: {};\n",
                indent,
                ts_type_to_text(type_, indent)
            )
        }
    }
}

/// Convert a top-level `TsDeclaration` to TypeScript source text.
pub fn declaration_to_text(decl: &TsDeclaration, indent: &str) -> String {
    match decl {
        TsDeclaration::Interface {
            name,
            members,
            comment,
        } => {
            let inner_indent = format!("{}{}", indent, TAB);
            let mut out = String::new();
            if let Some(c) = comment {
                out.push_str(&format_comment(c, indent));
            }
            out.push_str(&format!("{}export interface {} {{\n", indent, name));
            for member in members {
                out.push_str(&member_to_text(member, &inner_indent));
            }
            out.push_str(&format!("{}}}\n", indent));
            out
        }
        TsDeclaration::TypeAlias {
            name,
            type_,
            comment,
        } => {
            let mut out = String::new();
            if let Some(c) = comment {
                out.push_str(&format_comment(c, indent));
            }
            out.push_str(&format!(
                "{}export type {} = {};\n",
                indent,
                name,
                ts_type_to_text(type_, indent)
            ));
            out
        }
        TsDeclaration::Module { name, statements } => {
            let inner_indent = format!("{}{}", indent, TAB);
            let mut out = format!("{}export namespace {} {{\n", indent, name);
            for stmt in statements {
                out.push_str(&declaration_to_text(stmt, &inner_indent));
            }
            out.push_str(&format!("{}}}\n", indent));
            out
        }
    }
}

/// Convert a `TsType` or `TsDeclaration` to TypeScript source text.
///
/// Ports `toText` from `json-type/src/typescript/toText.ts`.
pub fn to_text(type_: &TsType) -> String {
    ts_type_to_text(type_, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Primitive types --

    #[test]
    fn to_text_any() {
        assert_eq!(to_text(&TsType::Any), "any");
    }

    #[test]
    fn to_text_boolean() {
        assert_eq!(to_text(&TsType::Boolean), "boolean");
    }

    #[test]
    fn to_text_number() {
        assert_eq!(to_text(&TsType::Number), "number");
    }

    #[test]
    fn to_text_string() {
        assert_eq!(to_text(&TsType::String), "string");
    }

    #[test]
    fn to_text_null() {
        assert_eq!(to_text(&TsType::Null), "null");
    }

    #[test]
    fn to_text_object() {
        assert_eq!(to_text(&TsType::Object), "object");
    }

    #[test]
    fn to_text_unknown() {
        assert_eq!(to_text(&TsType::Unknown), "unknown");
    }

    #[test]
    fn to_text_true() {
        assert_eq!(to_text(&TsType::True), "true");
    }

    #[test]
    fn to_text_false() {
        assert_eq!(to_text(&TsType::False), "false");
    }

    // -- Literals --

    #[test]
    fn to_text_string_literal() {
        assert_eq!(to_text(&TsType::StringLiteral("hello".into())), "\"hello\"");
    }

    #[test]
    fn to_text_string_literal_with_quotes() {
        assert_eq!(
            to_text(&TsType::StringLiteral("say \"hi\"".into())),
            "\"say \\\"hi\\\"\""
        );
    }

    #[test]
    fn to_text_string_literal_with_backslash() {
        assert_eq!(to_text(&TsType::StringLiteral("a\\b".into())), "\"a\\\\b\"");
    }

    #[test]
    fn to_text_numeric_literal() {
        assert_eq!(to_text(&TsType::NumericLiteral("42".into())), "42");
    }

    // -- Array --

    #[test]
    fn to_text_simple_array() {
        let t = TsType::Array(Box::new(TsType::Number));
        assert_eq!(to_text(&t), "number[]");
    }

    #[test]
    fn to_text_complex_array_uses_generic() {
        // Union inside array should use Array<> syntax
        let inner = TsType::Union(vec![TsType::String, TsType::Number]);
        let t = TsType::Array(Box::new(inner));
        assert_eq!(to_text(&t), "Array<string | number>");
    }

    // -- Tuple --

    #[test]
    fn to_text_simple_tuple() {
        let t = TsType::Tuple(vec![TsType::String, TsType::Number]);
        assert_eq!(to_text(&t), "[string, number]");
    }

    #[test]
    fn to_text_tuple_with_complex_element_uses_multiline() {
        let member = TsMember::Property {
            name: "x".into(),
            type_: TsType::Number,
            optional: false,
            comment: None,
        };
        let literal = TsType::TypeLiteral {
            members: vec![member],
            comment: None,
        };
        let t = TsType::Tuple(vec![literal]);
        let text = to_text(&t);
        assert!(text.contains('\n'));
    }

    // -- Rest --

    #[test]
    fn to_text_rest() {
        let t = TsType::Rest(Box::new(TsType::Number));
        assert_eq!(to_text(&t), "...number");
    }

    // -- TypeLiteral --

    #[test]
    fn to_text_empty_type_literal() {
        let t = TsType::TypeLiteral {
            members: vec![],
            comment: None,
        };
        assert_eq!(to_text(&t), "{}");
    }

    #[test]
    fn to_text_type_literal_with_property() {
        let t = TsType::TypeLiteral {
            members: vec![TsMember::Property {
                name: "name".into(),
                type_: TsType::String,
                optional: false,
                comment: None,
            }],
            comment: None,
        };
        let text = to_text(&t);
        assert!(text.contains("name: string;"));
    }

    #[test]
    fn to_text_type_literal_with_optional_property() {
        let t = TsType::TypeLiteral {
            members: vec![TsMember::Property {
                name: "age".into(),
                type_: TsType::Number,
                optional: true,
                comment: None,
            }],
            comment: None,
        };
        let text = to_text(&t);
        assert!(text.contains("age?: number;"));
    }

    #[test]
    fn to_text_type_literal_with_index_signature() {
        let t = TsType::TypeLiteral {
            members: vec![TsMember::Index {
                type_: TsType::Unknown,
            }],
            comment: None,
        };
        let text = to_text(&t);
        assert!(text.contains("[key: string]: unknown;"));
    }

    #[test]
    fn to_text_type_literal_with_comment() {
        let t = TsType::TypeLiteral {
            members: vec![TsMember::Property {
                name: "x".into(),
                type_: TsType::Number,
                optional: false,
                comment: None,
            }],
            comment: Some("A comment".into()),
        };
        let text = to_text(&t);
        assert!(text.contains("/**"));
        assert!(text.contains("A comment"));
        assert!(text.contains("*/"));
    }

    // -- Union --

    #[test]
    fn to_text_union() {
        let t = TsType::Union(vec![TsType::String, TsType::Number, TsType::Null]);
        assert_eq!(to_text(&t), "string | number | null");
    }

    // -- TypeReference --

    #[test]
    fn to_text_type_reference_no_args() {
        let t = TsType::TypeReference {
            name: "MyType".into(),
            type_args: vec![],
        };
        assert_eq!(to_text(&t), "MyType");
    }

    #[test]
    fn to_text_type_reference_with_args() {
        let t = TsType::TypeReference {
            name: "Record".into(),
            type_args: vec![TsType::String, TsType::Number],
        };
        assert_eq!(to_text(&t), "Record<string, number>");
    }

    // -- FnType --

    #[test]
    fn to_text_fn_type() {
        let t = TsType::FnType {
            params: vec![TsParam {
                name: "x".into(),
                type_: TsType::Number,
            }],
            return_type: Box::new(TsType::String),
        };
        assert_eq!(to_text(&t), "(x: number) => string");
    }

    #[test]
    fn to_text_fn_type_no_params() {
        let t = TsType::FnType {
            params: vec![],
            return_type: Box::new(TsType::Boolean),
        };
        assert_eq!(to_text(&t), "() => boolean");
    }

    // -- Member with comment --

    #[test]
    fn member_property_with_comment() {
        let t = TsType::TypeLiteral {
            members: vec![TsMember::Property {
                name: "id".into(),
                type_: TsType::Number,
                optional: false,
                comment: Some("Unique ID".into()),
            }],
            comment: None,
        };
        let text = to_text(&t);
        assert!(text.contains("Unique ID"));
        assert!(text.contains("id: number;"));
    }

    // -- needs_quotes / normalize_key --

    #[test]
    fn needs_quotes_for_empty_string() {
        assert!(needs_quotes(""));
    }

    #[test]
    fn needs_quotes_for_dash() {
        assert!(needs_quotes("my-key"));
    }

    #[test]
    fn needs_quotes_for_digit_start() {
        assert!(needs_quotes("0abc"));
    }

    #[test]
    fn no_quotes_for_normal_identifier() {
        assert!(!needs_quotes("myKey"));
        assert!(!needs_quotes("_private"));
        assert!(!needs_quotes("$special"));
    }

    #[test]
    fn normalize_key_quotes_when_needed() {
        assert_eq!(normalize_key("my-key"), "\"my-key\"");
        assert_eq!(normalize_key("normal"), "normal");
    }

    #[test]
    fn normalize_key_escapes_inner_quotes() {
        assert_eq!(normalize_key("say\"hi"), "\"say\\\"hi\"");
    }

    // -- declaration_to_text --

    #[test]
    fn declaration_interface() {
        let decl = TsDeclaration::Interface {
            name: "User".into(),
            members: vec![TsMember::Property {
                name: "name".into(),
                type_: TsType::String,
                optional: false,
                comment: None,
            }],
            comment: None,
        };
        let text = declaration_to_text(&decl, "");
        assert!(text.contains("export interface User {"));
        assert!(text.contains("name: string;"));
        assert!(text.contains("}"));
    }

    #[test]
    fn declaration_interface_with_comment() {
        let decl = TsDeclaration::Interface {
            name: "User".into(),
            members: vec![],
            comment: Some("A user type".into()),
        };
        let text = declaration_to_text(&decl, "");
        assert!(text.contains("/**"));
        assert!(text.contains("A user type"));
    }

    #[test]
    fn declaration_type_alias() {
        let decl = TsDeclaration::TypeAlias {
            name: "ID".into(),
            type_: TsType::Number,
            comment: None,
        };
        let text = declaration_to_text(&decl, "");
        assert_eq!(text, "export type ID = number;\n");
    }

    #[test]
    fn declaration_type_alias_with_comment() {
        let decl = TsDeclaration::TypeAlias {
            name: "ID".into(),
            type_: TsType::Number,
            comment: Some("An identifier".into()),
        };
        let text = declaration_to_text(&decl, "");
        assert!(text.contains("An identifier"));
        assert!(text.contains("export type ID = number;"));
    }

    #[test]
    fn declaration_module() {
        let inner = TsDeclaration::TypeAlias {
            name: "Foo".into(),
            type_: TsType::String,
            comment: None,
        };
        let decl = TsDeclaration::Module {
            name: "MyModule".into(),
            statements: vec![inner],
        };
        let text = declaration_to_text(&decl, "");
        assert!(text.contains("export namespace MyModule {"));
        assert!(text.contains("export type Foo = string;"));
        assert!(text.contains("}"));
    }

    // -- format_comment --

    #[test]
    fn format_comment_single_line() {
        let result = format_comment("Hello", "");
        assert!(result.contains("/**"));
        assert!(result.contains(" * Hello"));
        assert!(result.contains(" */"));
    }

    #[test]
    fn format_comment_multi_line() {
        let result = format_comment("Line 1\nLine 2", "  ");
        assert!(result.contains("  /**"));
        assert!(result.contains("   * Line 1"));
        assert!(result.contains("   * Line 2"));
        assert!(result.contains("   */"));
    }

    // -- is_simple_type --

    #[test]
    fn is_simple_type_primitives() {
        assert!(is_simple_type(&TsType::Any));
        assert!(is_simple_type(&TsType::Boolean));
        assert!(is_simple_type(&TsType::Number));
        assert!(is_simple_type(&TsType::String));
        assert!(is_simple_type(&TsType::Null));
        assert!(is_simple_type(&TsType::Object));
        assert!(is_simple_type(&TsType::Unknown));
    }

    #[test]
    fn is_simple_type_reference() {
        assert!(is_simple_type(&TsType::TypeReference {
            name: "Foo".into(),
            type_args: vec![],
        }));
    }

    #[test]
    fn is_not_simple_type_union() {
        assert!(!is_simple_type(&TsType::Union(vec![])));
    }

    #[test]
    fn is_not_simple_type_array() {
        assert!(!is_simple_type(&TsType::Array(Box::new(TsType::Any))));
    }
}
