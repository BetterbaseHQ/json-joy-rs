//! JSONPath utility functions (`util.ts`).

use crate::{JSONPath, Selector};

/// Convert a JSONPath AST to a readable string form.
pub fn json_path_to_string(json_path: &JSONPath) -> String {
    let mut result = String::from("$");

    for segment in &json_path.segments {
        if segment.selectors.len() == 1 {
            let selector = &segment.selectors[0];
            if segment.recursive {
                result.push_str(&recursive_selector_to_string(selector));
            } else {
                result.push_str(&selector_to_string(selector));
            }
        } else {
            result.push('[');
            for (idx, selector) in segment.selectors.iter().enumerate() {
                if idx > 0 {
                    result.push(',');
                }
                result.push_str(&selector_to_string(selector));
            }
            result.push(']');
        }
    }

    result
}

/// Check if two JSONPath AST values are equivalent.
pub fn json_path_equals(path1: &JSONPath, path2: &JSONPath) -> bool {
    path1 == path2
}

/// Returns property names touched by direct name selectors.
pub fn get_accessed_properties(json_path: &JSONPath) -> Vec<String> {
    let mut properties = Vec::new();

    for segment in &json_path.segments {
        for selector in &segment.selectors {
            if let Selector::Name(name) = selector {
                properties.push(name.clone());
            }
        }
    }

    properties
}

fn recursive_selector_to_string(selector: &Selector) -> String {
    match selector {
        Selector::Name(name) => format!("..{name}"),
        Selector::Index(index) => format!("..[{index}]"),
        Selector::Slice { start, end, step } => {
            let mut slice = String::from("..[");
            if let Some(start) = start {
                slice.push_str(&start.to_string());
            }
            slice.push(':');
            if let Some(end) = end {
                slice.push_str(&end.to_string());
            }
            if let Some(step) = step {
                slice.push(':');
                slice.push_str(&step.to_string());
            }
            slice.push(']');
            slice
        }
        Selector::Wildcard => "..*".to_string(),
        Selector::Filter(_) => "..[?(...)]".to_string(),
    }
}

fn selector_to_string(selector: &Selector) -> String {
    match selector {
        Selector::Name(name) => format!(".{name}"),
        Selector::Index(index) => format!("[{index}]"),
        Selector::Slice { start, end, step } => {
            let mut slice = String::from("[");
            if let Some(start) = start {
                slice.push_str(&start.to_string());
            }
            slice.push(':');
            if let Some(end) = end {
                slice.push_str(&end.to_string());
            }
            if let Some(step) = step {
                slice.push(':');
                slice.push_str(&step.to_string());
            }
            slice.push(']');
            slice
        }
        Selector::Wildcard => ".*".to_string(),
        Selector::Filter(_) => "[?(...)]".to_string(),
    }
}
