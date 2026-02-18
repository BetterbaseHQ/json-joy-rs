//! json-walk — recursive JSON value visitor.
//!
//! Mirrors `packages/json-joy/src/json-walk/index.ts`.

use serde_json::Value;

/// Walk every node in a JSON value tree, calling `callback` on each.
///
/// The callback is called on the root value first, then on every nested value
/// (arrays and object values are descended into).
pub fn walk<F>(value: &Value, callback: &mut F)
where
    F: FnMut(&Value),
{
    callback(value);
    match value {
        Value::Array(arr) => {
            for item in arr {
                walk(item, callback);
            }
        }
        Value::Object(obj) => {
            for (_key, val) in obj {
                walk(val, callback);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn walks_scalar() {
        let mut visited = vec![];
        walk(&json!(42), &mut |v| visited.push(v.clone()));
        assert_eq!(visited, vec![json!(42)]);
    }

    #[test]
    fn walks_array() {
        let mut visited = vec![];
        walk(&json!([1, 2, 3]), &mut |v| visited.push(v.clone()));
        assert_eq!(visited.len(), 4); // root + 3 items
        assert_eq!(visited[0], json!([1, 2, 3]));
        assert_eq!(visited[1], json!(1));
        assert_eq!(visited[2], json!(2));
        assert_eq!(visited[3], json!(3));
    }

    #[test]
    fn walks_nested_object() {
        let val = json!({"a": 1, "b": [2, 3]});
        let mut count = 0;
        walk(&val, &mut |_| count += 1);
        // root + "a":1 + "b":[2,3] + 2 + 3 = 5
        assert_eq!(count, 5);
    }

    #[test]
    fn walks_null() {
        let mut visited = vec![];
        walk(&json!(null), &mut |v| visited.push(v.clone()));
        assert_eq!(visited, vec![json!(null)]);
    }
}
