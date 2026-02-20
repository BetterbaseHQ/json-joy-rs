//! JSONPath evaluator.

use crate::{
    ComparisonOperator, FilterExpression, FunctionArg, JSONPath, JsonPathParser, LogicalOperator,
    PathComponent, PathSegment, QueryResult, Selector, Value as JsonPathValue, ValueExpression,
};
use regex::Regex;
use serde_json::Value as JsonValue;

/// JSONPath evaluator.
pub struct JsonPathEval;

impl JsonPathEval {
    /// Evaluate a JSONPath against a JSON document.
    ///
    /// Returns a vector of references to matching values.
    pub fn eval<'a>(path: &JSONPath, doc: &'a JsonValue) -> Vec<&'a JsonValue> {
        Self::eval_query(path, doc).values
    }

    /// Evaluate a JSONPath and also return normalized path components.
    pub fn eval_query<'a>(path: &JSONPath, doc: &'a JsonValue) -> QueryResult<'a> {
        let mut results = vec![doc];
        let mut paths: Vec<Vec<PathComponent>> = vec![vec![]];

        for segment in &path.segments {
            let mut new_results = Vec::new();
            let mut new_paths = Vec::new();

            for (idx, value) in results.iter().enumerate() {
                let current_path = &paths[idx];

                if segment.recursive {
                    Self::eval_recursive(
                        value,
                        &segment.selectors,
                        current_path,
                        doc,
                        &mut new_results,
                        &mut new_paths,
                    );
                } else {
                    Self::eval_segment(
                        value,
                        segment,
                        current_path,
                        doc,
                        &mut new_results,
                        &mut new_paths,
                    );
                }
            }

            results = new_results;
            paths = new_paths;
        }

        QueryResult {
            values: results,
            paths,
        }
    }

    /// Upstream-compatible convenience API that parses and evaluates a path string.
    pub fn run(path: &str, data: &JsonValue) -> Vec<JsonPathValue> {
        let parsed = JsonPathParser::parse(path);
        if !parsed.success || parsed.path.is_none() || parsed.error.is_some() {
            panic!(
                "Invalid JSONPath: {} [position = {:?}, path = {}]",
                parsed
                    .error
                    .unwrap_or_else(|| "unknown parse error".to_string()),
                parsed.position,
                path
            );
        }

        let ast = parsed.path.expect("parse result path unexpectedly missing");
        Self::run_ast(&ast, data)
    }

    /// Evaluate a pre-parsed JSONPath and return values with path metadata.
    pub fn run_ast(path: &JSONPath, data: &JsonValue) -> Vec<JsonPathValue> {
        let query = Self::eval_query(path, data);
        query
            .values
            .into_iter()
            .zip(query.paths)
            .map(|(value, path)| JsonPathValue::from_query_path(&path, value.clone()))
            .collect()
    }

    fn eval_segment<'a>(
        value: &'a JsonValue,
        segment: &PathSegment,
        current_path: &[PathComponent],
        root: &'a JsonValue,
        results: &mut Vec<&'a JsonValue>,
        paths: &mut Vec<Vec<PathComponent>>,
    ) {
        for selector in &segment.selectors {
            Self::eval_selector(value, selector, current_path, root, results, paths);
        }
    }

    fn eval_recursive<'a>(
        value: &'a JsonValue,
        selectors: &[Selector],
        current_path: &[PathComponent],
        root: &'a JsonValue,
        results: &mut Vec<&'a JsonValue>,
        paths: &mut Vec<Vec<PathComponent>>,
    ) {
        // Match selectors at current node first.
        for selector in selectors {
            Self::eval_selector(value, selector, current_path, root, results, paths);
        }

        // Then recurse through descendants.
        match value {
            JsonValue::Object(map) => {
                for (key, child) in map {
                    let mut new_path = current_path.to_vec();
                    new_path.push(PathComponent::Key(key.clone()));
                    Self::eval_recursive(child, selectors, &new_path, root, results, paths);
                }
            }
            JsonValue::Array(arr) => {
                for (idx, child) in arr.iter().enumerate() {
                    let mut new_path = current_path.to_vec();
                    new_path.push(PathComponent::Index(idx));
                    Self::eval_recursive(child, selectors, &new_path, root, results, paths);
                }
            }
            _ => {}
        }
    }

    fn eval_selector<'a>(
        value: &'a JsonValue,
        selector: &Selector,
        current_path: &[PathComponent],
        root: &'a JsonValue,
        results: &mut Vec<&'a JsonValue>,
        paths: &mut Vec<Vec<PathComponent>>,
    ) {
        match selector {
            Selector::Name(name) => {
                if let JsonValue::Object(map) = value {
                    if let Some(child) = map.get(name) {
                        let mut new_path = current_path.to_vec();
                        new_path.push(PathComponent::Key(name.clone()));
                        results.push(child);
                        paths.push(new_path);
                    }
                }
            }
            Selector::Index(index) => {
                if let JsonValue::Array(arr) = value {
                    let idx_opt = if *index < 0 {
                        arr.len().checked_sub(index.unsigned_abs())
                    } else {
                        Some(*index as usize)
                    };
                    if let Some(idx) = idx_opt {
                        if let Some(child) = arr.get(idx) {
                            let mut new_path = current_path.to_vec();
                            new_path.push(PathComponent::Index(idx));
                            results.push(child);
                            paths.push(new_path);
                        }
                    }
                }
            }
            Selector::Wildcard => match value {
                JsonValue::Object(map) => {
                    for (key, child) in map {
                        let mut new_path = current_path.to_vec();
                        new_path.push(PathComponent::Key(key.clone()));
                        results.push(child);
                        paths.push(new_path);
                    }
                }
                JsonValue::Array(arr) => {
                    for (idx, child) in arr.iter().enumerate() {
                        let mut new_path = current_path.to_vec();
                        new_path.push(PathComponent::Index(idx));
                        results.push(child);
                        paths.push(new_path);
                    }
                }
                _ => {}
            },
            Selector::Slice { start, end, step } => {
                if let JsonValue::Array(arr) = value {
                    let len = arr.len() as isize;
                    let step_val = step.unwrap_or(1);
                    if step_val == 0 {
                        return;
                    }

                    let (mut start_idx, mut end_idx) = if step_val > 0 {
                        (
                            start.unwrap_or(0),
                            end.unwrap_or_else(|| arr.len() as isize),
                        )
                    } else {
                        (start.unwrap_or(len - 1), end.unwrap_or(-len - 1))
                    };

                    start_idx = if start_idx < 0 {
                        len + start_idx
                    } else {
                        start_idx
                    };
                    end_idx = if end_idx < 0 { len + end_idx } else { end_idx };

                    if step_val > 0 {
                        let lower = start_idx.clamp(0, len) as usize;
                        let upper = end_idx.clamp(0, len) as usize;
                        let mut idx = lower;
                        while idx < upper {
                            if let Some(child) = arr.get(idx) {
                                let mut new_path = current_path.to_vec();
                                new_path.push(PathComponent::Index(idx));
                                results.push(child);
                                paths.push(new_path);
                            }
                            idx = idx.saturating_add(step_val as usize);
                        }
                    } else {
                        let upper = start_idx.clamp(-1, len - 1);
                        let lower = end_idx.clamp(-1, len - 1);
                        let mut idx = upper;
                        while idx > lower {
                            let uidx = idx as usize;
                            if let Some(child) = arr.get(uidx) {
                                let mut new_path = current_path.to_vec();
                                new_path.push(PathComponent::Index(uidx));
                                results.push(child);
                                paths.push(new_path);
                            }
                            idx += step_val;
                        }
                    }
                }
            }
            Selector::Filter(expr) => {
                // Upstream behavior: root-level filter on an object evaluates the object itself.
                if current_path.is_empty() && value.is_object() {
                    if Self::eval_filter(expr, value, root) {
                        results.push(value);
                        paths.push(current_path.to_vec());
                        return;
                    }
                }

                match value {
                    JsonValue::Object(map) => {
                        for (key, child) in map {
                            if Self::eval_filter(expr, child, root) {
                                let mut new_path = current_path.to_vec();
                                new_path.push(PathComponent::Key(key.clone()));
                                results.push(child);
                                paths.push(new_path);
                            }
                        }
                    }
                    JsonValue::Array(arr) => {
                        for (idx, child) in arr.iter().enumerate() {
                            if Self::eval_filter(expr, child, root) {
                                let mut new_path = current_path.to_vec();
                                new_path.push(PathComponent::Index(idx));
                                results.push(child);
                                paths.push(new_path);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn eval_filter(expr: &FilterExpression, current: &JsonValue, root: &JsonValue) -> bool {
        match expr {
            FilterExpression::Existence { path } => {
                let results = Self::eval(path, current);
                !results.is_empty()
            }
            FilterExpression::Comparison {
                operator,
                left,
                right,
            } => {
                let left_val = Self::eval_value_expr(left, current, root);
                let right_val = Self::eval_value_expr(right, current, root);
                Self::compare(operator, &left_val, &right_val)
            }
            FilterExpression::Logical {
                operator,
                left,
                right,
            } => {
                let left_result = Self::eval_filter(left, current, root);
                let right_result = Self::eval_filter(right, current, root);
                match operator {
                    LogicalOperator::And => left_result && right_result,
                    LogicalOperator::Or => left_result || right_result,
                }
            }
            FilterExpression::Negation(expr) => !Self::eval_filter(expr, current, root),
            FilterExpression::Paren(expr) => Self::eval_filter(expr, current, root),
            FilterExpression::Function { name, args } => {
                let result = Self::eval_function_value(name, args, current, root);
                Self::value_truthy(&result)
            }
        }
    }

    fn value_truthy(value: &Option<JsonValue>) -> bool {
        match value {
            Some(JsonValue::Bool(v)) => *v,
            Some(JsonValue::Number(v)) => v.as_f64().map(|n| n != 0.0).unwrap_or(false),
            Some(JsonValue::Array(v)) => !v.is_empty(),
            Some(JsonValue::Null) | None => false,
            Some(_) => true,
        }
    }

    fn eval_value_expr(
        expr: &ValueExpression,
        current: &JsonValue,
        root: &JsonValue,
    ) -> Option<JsonValue> {
        match expr {
            ValueExpression::Current => Some(current.clone()),
            ValueExpression::Root => Some(root.clone()),
            ValueExpression::Literal(v) => Some(v.clone()),
            ValueExpression::Path(path) => {
                let results = Self::eval(path, current);
                results.first().map(|v| (*v).clone())
            }
            ValueExpression::Function { name, args } => {
                Self::eval_function_value(name, args, current, root)
            }
        }
    }

    fn eval_function_value(
        name: &str,
        args: &[FunctionArg],
        current: &JsonValue,
        root: &JsonValue,
    ) -> Option<JsonValue> {
        match name {
            "length" => {
                if args.len() != 1 {
                    return None;
                }
                let value = Self::eval_function_arg_single(&args[0], current, root)?;
                let out = match value {
                    JsonValue::String(s) => s.chars().count() as u64,
                    JsonValue::Array(arr) => arr.len() as u64,
                    JsonValue::Object(map) => map.len() as u64,
                    _ => return None,
                };
                Some(JsonValue::Number(serde_json::Number::from(out)))
            }
            "count" => {
                if args.len() != 1 {
                    return Some(JsonValue::Number(serde_json::Number::from(0u64)));
                }
                let arg = &args[0];
                let count = if Self::function_arg_is_nodes(arg) {
                    Self::eval_function_arg_nodes(arg, current, root).len() as u64
                } else if Self::eval_function_arg_single(arg, current, root).is_some() {
                    1
                } else {
                    0
                };
                Some(JsonValue::Number(serde_json::Number::from(count)))
            }
            "match" => {
                if args.len() != 2 {
                    return Some(JsonValue::Bool(false));
                }
                let string_arg = Self::eval_function_arg_single(&args[0], current, root);
                let regex_arg = Self::eval_function_arg_single(&args[1], current, root);

                let string = match string_arg {
                    Some(JsonValue::String(s)) => s,
                    _ => return Some(JsonValue::Bool(false)),
                };
                let regex = match regex_arg {
                    Some(JsonValue::String(s)) => s,
                    _ => return Some(JsonValue::Bool(false)),
                };

                let pattern = format!("^(?:{regex})$");
                let matched = Regex::new(&pattern)
                    .map(|r| r.is_match(&string))
                    .unwrap_or(false);
                Some(JsonValue::Bool(matched))
            }
            "search" => {
                if args.len() != 2 {
                    return Some(JsonValue::Bool(false));
                }
                let string_arg = Self::eval_function_arg_single(&args[0], current, root);
                let regex_arg = Self::eval_function_arg_single(&args[1], current, root);

                let string = match string_arg {
                    Some(JsonValue::String(s)) => s,
                    _ => return Some(JsonValue::Bool(false)),
                };
                let regex = match regex_arg {
                    Some(JsonValue::String(s)) => s,
                    _ => return Some(JsonValue::Bool(false)),
                };

                let matched = Regex::new(&regex)
                    .map(|r| r.is_match(&string))
                    .unwrap_or(false);
                Some(JsonValue::Bool(matched))
            }
            "value" => {
                if args.len() != 1 {
                    return None;
                }
                let arg = &args[0];
                if Self::function_arg_is_nodes(arg) {
                    let nodes = Self::eval_function_arg_nodes(arg, current, root);
                    if nodes.len() == 1 {
                        return nodes.into_iter().next();
                    }
                    None
                } else {
                    Self::eval_function_arg_single(arg, current, root)
                }
            }
            _ => Some(JsonValue::Bool(false)),
        }
    }

    fn function_arg_is_nodes(arg: &FunctionArg) -> bool {
        matches!(arg, FunctionArg::Path(_))
            || matches!(arg, FunctionArg::Value(ValueExpression::Path(_)))
    }

    fn eval_function_arg_nodes(
        arg: &FunctionArg,
        current: &JsonValue,
        root: &JsonValue,
    ) -> Vec<JsonValue> {
        match arg {
            FunctionArg::Path(path) => Self::eval(path, root).into_iter().cloned().collect(),
            FunctionArg::Value(ValueExpression::Path(path)) => {
                Self::eval(path, current).into_iter().cloned().collect()
            }
            FunctionArg::Value(expr) => Self::eval_value_expr(expr, current, root)
                .into_iter()
                .collect(),
            FunctionArg::Filter(filter) => {
                vec![JsonValue::Bool(Self::eval_filter(filter, current, root))]
            }
        }
    }

    fn eval_function_arg_single(
        arg: &FunctionArg,
        current: &JsonValue,
        root: &JsonValue,
    ) -> Option<JsonValue> {
        let nodes = Self::eval_function_arg_nodes(arg, current, root);
        if nodes.len() == 1 {
            return nodes.into_iter().next();
        }

        if Self::function_arg_is_nodes(arg) {
            None
        } else {
            nodes.into_iter().next()
        }
    }

    fn compare(
        operator: &ComparisonOperator,
        left: &Option<JsonValue>,
        right: &Option<JsonValue>,
    ) -> bool {
        match (left, right) {
            (None, None) => match operator {
                ComparisonOperator::Equal => true,
                ComparisonOperator::NotEqual => false,
                _ => false,
            },
            (Some(l), Some(r)) => {
                let ord = Self::compare_values(l, r);
                match operator {
                    ComparisonOperator::Equal => {
                        if let (JsonValue::Number(_), JsonValue::Number(_)) = (l, r) {
                            ord == Some(std::cmp::Ordering::Equal)
                        } else {
                            l == r
                        }
                    }
                    ComparisonOperator::NotEqual => {
                        if let (JsonValue::Number(_), JsonValue::Number(_)) = (l, r) {
                            ord != Some(std::cmp::Ordering::Equal)
                        } else {
                            l != r
                        }
                    }
                    ComparisonOperator::Less => ord == Some(std::cmp::Ordering::Less),
                    ComparisonOperator::LessEqual => matches!(
                        ord,
                        Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)
                    ),
                    ComparisonOperator::Greater => ord == Some(std::cmp::Ordering::Greater),
                    ComparisonOperator::GreaterEqual => matches!(
                        ord,
                        Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)
                    ),
                }
            }
            _ => false,
        }
    }

    fn compare_values(a: &JsonValue, b: &JsonValue) -> Option<std::cmp::Ordering> {
        match (a, b) {
            (JsonValue::Number(a), JsonValue::Number(b)) => {
                if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
                    a.partial_cmp(&b)
                } else {
                    None
                }
            }
            (JsonValue::String(a), JsonValue::String(b)) => Some(a.cmp(b)),
            (JsonValue::Bool(a), JsonValue::Bool(b)) => Some(a.cmp(b)),
            _ => None,
        }
    }
}
