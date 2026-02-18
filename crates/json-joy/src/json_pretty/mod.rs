//! json-pretty — single-line pretty JSON printer.
//!
//! Mirrors `packages/json-joy/src/json-pretty/index.ts`.

use serde_json::Value;

/// Serialize `value` to a JSON string with a single space after every
/// `{`, `[`, `,`, `:` and before every `}`, `]`.
///
/// Equivalent to the upstream `prettyOneLine` function:
/// ```ts
/// let json = JSON.stringify(value);
/// json = json.replace(/([{[:,])/g, '$1 ').replace(/([}\]])/g, ' $1');
/// ```
pub fn pretty_one_line(value: &Value) -> String {
    let json = serde_json::to_string(value).unwrap_or_default();
    // Insert space after { [ : ,
    let step1 = insert_space_after(&json);
    // Insert space before } ]
    insert_space_before(&step1)
}

fn insert_space_after(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    for ch in s.chars() {
        out.push(ch);
        if matches!(ch, '{' | '[' | ':' | ',') {
            out.push(' ');
        }
    }
    out
}

fn insert_space_before(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        if matches!(ch, '}' | ']') {
            out.push(' ');
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn formats_scalar() {
        assert_eq!(pretty_one_line(&json!(42)), "42");
        assert_eq!(pretty_one_line(&json!("hello")), r#""hello""#);
        assert_eq!(pretty_one_line(&json!(null)), "null");
    }

    #[test]
    fn formats_empty_object() {
        // {} → step1 adds space after { → "{ }" → step2 adds space before } → "{  }"
        assert_eq!(pretty_one_line(&json!({})), "{  }");
    }

    #[test]
    fn formats_empty_array() {
        // [] → step1: "[ ]" → step2: "[  ]"
        assert_eq!(pretty_one_line(&json!([])), "[  ]");
    }

    #[test]
    fn formats_object() {
        // {"a":1} → step1: '{ "a": 1}' → step2: '{ "a": 1 }'
        let result = pretty_one_line(&json!({"a": 1}));
        assert_eq!(result, r#"{ "a": 1 }"#);
    }

    #[test]
    fn formats_array() {
        // [1,2,3] → step1: "[ 1, 2, 3]" → step2: "[ 1, 2, 3 ]"
        let result = pretty_one_line(&json!([1, 2, 3]));
        assert_eq!(result, "[ 1, 2, 3 ]");
    }
}
