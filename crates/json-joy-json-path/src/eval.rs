//! JSONPath evaluator.

use regex::Regex;
use serde_json::Value;

use crate::types::*;

/// JSONPath evaluator.
pub struct JsonPathEval;

impl JsonPathEval {
    /// Evaluate a JSONPath against a JSON document.
    ///
    /// Returns a vector of references to matching values.
    pub fn eval<'a>(path: &JSONPath, doc: &'a Value) -> Vec<&'a Value> {
        Self::eval_query(path, doc).values
    }

    /// Evaluate a JSONPath and return both matched values and normalized paths.
    pub fn eval_query<'a>(path: &JSONPath, doc: &'a Value) -> QueryResult<'a> {
        let mut results = vec![doc];
        let mut paths: Vec<Vec<PathComponent>> = vec![vec![]];

        for segment in &path.segments {
            let mut new_results = Vec::new();
            let mut new_paths = Vec::new();

            for (i, value) in results.iter().enumerate() {
                let current_path = &paths[i];

                if segment.recursive {
                    Self::eval_recursive(
                        value,
                        &segment.selectors,
                        current_path,
                        &mut new_results,
                        &mut new_paths,
                        doc,
                    );
                } else {
                    Self::eval_segment(
                        value,
                        segment,
                        current_path,
                        &mut new_results,
                        &mut new_paths,
                        doc,
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

    fn eval_segment<'a>(
        value: &'a Value,
        segment: &PathSegment,
        current_path: &[PathComponent],
        results: &mut Vec<&'a Value>,
        paths: &mut Vec<Vec<PathComponent>>,
        root: &'a Value,
    ) {
        for selector in &segment.selectors {
            Self::eval_selector(value, selector, current_path, results, paths, root);
        }
    }

    fn eval_recursive<'a>(
        value: &'a Value,
        selectors: &[Selector],
        current_path: &[PathComponent],
        results: &mut Vec<&'a Value>,
        paths: &mut Vec<Vec<PathComponent>>,
        root: &'a Value,
    ) {
        for selector in selectors {
            Self::eval_selector(value, selector, current_path, results, paths, root);
        }

        match value {
            Value::Object(map) => {
                for (key, child) in map {
                    let mut new_path = current_path.to_vec();
                    new_path.push(PathComponent::Key(key.clone()));
                    Self::eval_recursive(child, selectors, &new_path, results, paths, root);
                }
            }
            Value::Array(arr) => {
                for (idx, child) in arr.iter().enumerate() {
                    let mut new_path = current_path.to_vec();
                    new_path.push(PathComponent::Index(idx));
                    Self::eval_recursive(child, selectors, &new_path, results, paths, root);
                }
            }
            _ => {}
        }
    }

    fn eval_selector<'a>(
        value: &'a Value,
        selector: &Selector,
        current_path: &[PathComponent],
        results: &mut Vec<&'a Value>,
        paths: &mut Vec<Vec<PathComponent>>,
        root: &'a Value,
    ) {
        match selector {
            Selector::Name(name) => {
                if let Value::Object(map) = value {
                    if let Some(child) = map.get(name) {
                        let mut new_path = current_path.to_vec();
                        new_path.push(PathComponent::Key(name.clone()));
                        results.push(child);
                        paths.push(new_path);
                    }
                }
            }
            Selector::Index(index) => {
                if let Value::Array(arr) = value {
                    let idx = if *index < 0 {
                        (arr.len() as isize + index) as usize
                    } else {
                        *index as usize
                    };
                    if let Some(child) = arr.get(idx) {
                        let mut new_path = current_path.to_vec();
                        new_path.push(PathComponent::Index(idx));
                        results.push(child);
                        paths.push(new_path);
                    }
                }
            }
            Selector::Wildcard => match value {
                Value::Object(map) => {
                    for (key, child) in map {
                        let mut new_path = current_path.to_vec();
                        new_path.push(PathComponent::Key(key.clone()));
                        results.push(child);
                        paths.push(new_path);
                    }
                }
                Value::Array(arr) => {
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
                if let Value::Array(arr) = value {
                    let len = arr.len() as isize;
                    let step_val = step.unwrap_or(1);
                    if step_val == 0 {
                        return;
                    }

                    let (start_val, end_val) = if step_val > 0 {
                        (start.unwrap_or(0), end.unwrap_or(len))
                    } else {
                        (start.unwrap_or(len - 1), end.unwrap_or(-len - 1))
                    };

                    let normalize = |idx: isize| if idx >= 0 { idx } else { len + idx };
                    let normalized_start = normalize(start_val);
                    let normalized_end = normalize(end_val);

                    if step_val > 0 {
                        let lower = normalized_start.clamp(0, len);
                        let upper = normalized_end.clamp(0, len);
                        let mut idx = lower;
                        while idx < upper {
                            if let Some(child) = arr.get(idx as usize) {
                                let mut new_path = current_path.to_vec();
                                new_path.push(PathComponent::Index(idx as usize));
                                results.push(child);
                                paths.push(new_path);
                            }
                            idx += step_val;
                        }
                    } else {
                        let upper = normalized_start.clamp(-1, len - 1);
                        let lower = normalized_end.clamp(-1, len - 1);
                        let mut idx = upper;
                        while idx > lower {
                            if idx >= 0 {
                                if let Some(child) = arr.get(idx as usize) {
                                    let mut new_path = current_path.to_vec();
                                    new_path.push(PathComponent::Index(idx as usize));
                                    results.push(child);
                                    paths.push(new_path);
                                }
                            }
                            idx += step_val;
                        }
                    }
                }
            }
            Selector::Filter(expr) => match value {
                Value::Object(map) => {
                    // Match upstream behavior: a root-level filter on an object
                    // evaluates the object itself.
                    if current_path.is_empty() {
                        if Self::eval_filter(expr, value, root) {
                            results.push(value);
                            paths.push(current_path.to_vec());
                        }
                    } else {
                        for (key, child) in map {
                            if Self::eval_filter(expr, child, root) {
                                let mut new_path = current_path.to_vec();
                                new_path.push(PathComponent::Key(key.clone()));
                                results.push(child);
                                paths.push(new_path);
                            }
                        }
                    }
                }
                Value::Array(arr) => {
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
            },
        }
    }

    fn eval_filter(expr: &FilterExpression, current: &Value, root: &Value) -> bool {
        match expr {
            FilterExpression::Existence { path } => !Self::eval(path, current).is_empty(),
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
            } => match operator {
                LogicalOperator::And => {
                    Self::eval_filter(left, current, root)
                        && Self::eval_filter(right, current, root)
                }
                LogicalOperator::Or => {
                    Self::eval_filter(left, current, root)
                        || Self::eval_filter(right, current, root)
                }
            },
            FilterExpression::Negation(expr) => !Self::eval_filter(expr, current, root),
            FilterExpression::Paren(expr) => Self::eval_filter(expr, current, root),
            FilterExpression::Function { name, args } => {
                Self::truthy(Self::eval_function_expression(name, args, current, root))
            }
        }
    }

    fn eval_value_expr(expr: &ValueExpression, current: &Value, root: &Value) -> Option<Value> {
        let mut values = Self::eval_value_expr_nodes(expr, current, root);
        match values.len() {
            0 => None,
            1 => Some(values.remove(0)),
            _ => Some(Value::Array(values)),
        }
    }

    fn eval_value_expr_nodes(expr: &ValueExpression, current: &Value, root: &Value) -> Vec<Value> {
        match expr {
            ValueExpression::Current => vec![current.clone()],
            ValueExpression::Root => vec![root.clone()],
            ValueExpression::Literal(v) => vec![v.clone()],
            ValueExpression::Path(path) => Self::eval(path, current)
                .iter()
                .map(|v| (*v).clone())
                .collect(),
            ValueExpression::AbsolutePath(path) => Self::eval(path, root)
                .iter()
                .map(|v| (*v).clone())
                .collect(),
            ValueExpression::Function { name, args } => {
                Self::eval_function_expression(name, args, current, root)
                    .into_iter()
                    .collect()
            }
        }
    }

    fn eval_function_expression(
        name: &str,
        args: &[FunctionArg],
        current: &Value,
        root: &Value,
    ) -> Option<Value> {
        match name {
            "length" => {
                if args.len() != 1 {
                    return None;
                }
                let values = Self::eval_function_arg_nodes(&args[0], current, root);
                if values.len() != 1 {
                    return None;
                }
                let length = match &values[0] {
                    Value::String(s) => s.chars().count(),
                    Value::Array(arr) => arr.len(),
                    Value::Object(obj) => obj.len(),
                    _ => return None,
                };
                Some(Value::Number((length as u64).into()))
            }
            "count" => {
                if args.len() != 1 {
                    return None;
                }
                let values = Self::eval_function_arg_nodes(&args[0], current, root);
                Some(Value::Number((values.len() as u64).into()))
            }
            "value" => {
                if args.len() != 1 {
                    return None;
                }
                let mut values = Self::eval_function_arg_nodes(&args[0], current, root);
                if values.len() == 1 {
                    Some(values.remove(0))
                } else {
                    None
                }
            }
            "match" => {
                if args.len() != 2 {
                    return None;
                }
                let input = Self::function_arg_single_string(&args[0], current, root)?;
                let pattern = Self::function_arg_single_string(&args[1], current, root)?;
                let regex = Regex::new(&format!("^(?:{})$", pattern)).ok()?;
                Some(Value::Bool(regex.is_match(&input)))
            }
            "search" => {
                if args.len() != 2 {
                    return None;
                }
                let input = Self::function_arg_single_string(&args[0], current, root)?;
                let pattern = Self::function_arg_single_string(&args[1], current, root)?;
                let regex = Regex::new(&pattern).ok()?;
                Some(Value::Bool(regex.is_match(&input)))
            }
            _ => None,
        }
    }

    fn eval_function_arg_nodes(arg: &FunctionArg, current: &Value, root: &Value) -> Vec<Value> {
        match arg {
            FunctionArg::Value(expr) => Self::eval_value_expr_nodes(expr, current, root),
            FunctionArg::Filter(expr) => vec![Value::Bool(Self::eval_filter(expr, current, root))],
            FunctionArg::Path(path) => Self::eval(path, root)
                .iter()
                .map(|v| (*v).clone())
                .collect(),
        }
    }

    fn function_arg_single_string(
        arg: &FunctionArg,
        current: &Value,
        root: &Value,
    ) -> Option<String> {
        let mut values = Self::eval_function_arg_nodes(arg, current, root);
        if values.len() != 1 {
            return None;
        }
        match values.remove(0) {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    fn truthy(value: Option<Value>) -> bool {
        match value {
            None => false,
            Some(Value::Bool(v)) => v,
            Some(Value::Number(n)) => n.as_f64().is_some_and(|v| v != 0.0),
            Some(Value::Array(arr)) => !arr.is_empty(),
            Some(Value::Null) => false,
            Some(_) => true,
        }
    }

    fn compare(operator: &ComparisonOperator, left: &Option<Value>, right: &Option<Value>) -> bool {
        match (left, right) {
            (None, None) => matches!(operator, ComparisonOperator::Equal),
            (Some(l), Some(r)) => {
                let l = Self::unwrap_singleton_array(l);
                let r = Self::unwrap_singleton_array(r);
                let ord = Self::compare_values(l, r);
                match operator {
                    ComparisonOperator::Equal => {
                        if let (Value::Number(_), Value::Number(_)) = (l, r) {
                            ord == Some(std::cmp::Ordering::Equal)
                        } else {
                            l == r
                        }
                    }
                    ComparisonOperator::NotEqual => {
                        if let (Value::Number(_), Value::Number(_)) = (l, r) {
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

    fn unwrap_singleton_array(value: &Value) -> &Value {
        if let Value::Array(items) = value {
            if items.len() == 1 {
                return &items[0];
            }
        }
        value
    }

    fn compare_values(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
                    a.partial_cmp(&b)
                } else {
                    None
                }
            }
            (Value::String(a), Value::String(b)) => Some(a.cmp(b)),
            (Value::Bool(a), Value::Bool(b)) => Some(a.cmp(b)),
            _ => None,
        }
    }
}
