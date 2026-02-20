//! Discriminator inference and expression construction for `Or` types.
//!
//! Upstream reference: `json-type/src/type/discriminator.ts`.

use std::collections::HashSet;

use serde_json::{json, Value};

use super::{builder::TypeBuilder, TypeNode};

/// A discovered discriminator (path + type marker).
#[derive(Debug, Clone)]
pub struct Discriminator {
    pub path: String,
    pub type_: TypeNode,
}

impl Discriminator {
    pub fn new(path: impl Into<String>, type_: TypeNode) -> Self {
        Self {
            path: path.into(),
            type_,
        }
    }

    /// Attempts to find a constant discriminator inside the type.
    pub fn find_const(type_: &TypeNode) -> Option<Self> {
        match type_ {
            TypeNode::Con(_) => Some(Self::new("", type_.clone())),
            TypeNode::Arr(arr) => {
                // Upstream currently only scans tuple head entries here.
                for (i, t) in arr.head.iter().enumerate() {
                    if let Some(d) = Self::find_const(t) {
                        return Some(Self::new(format!("/{i}{}", d.path), d.type_));
                    }
                }
                None
            }
            TypeNode::Obj(obj) => {
                for field in &obj.keys {
                    if let Some(d) = Self::find_const(&field.val) {
                        return Some(Self::new(format!("/{}{}", field.key, d.path), d.type_));
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Finds the discriminator for a type, preferring constants.
    pub fn find(type_: &TypeNode) -> Self {
        Self::find_const(type_).unwrap_or_else(|| Self::new("", type_.clone()))
    }

    /// Creates a discriminator JSON-expression for a list of union member types.
    ///
    /// The resulting expression evaluates to an index in the flattened union.
    pub fn create_expression(types: &[TypeNode]) -> Value {
        let mut specifiers = HashSet::new();
        let mut expanded = Vec::new();
        for t in types {
            Self::expand(t, &mut expanded);
        }

        let mut discriminators = Vec::new();
        for t in expanded.iter().skip(1) {
            let d = Self::find(t);
            let specifier = d.to_specifier();
            if !specifiers.insert(specifier.clone()) {
                panic!("Duplicate discriminator: {specifier}");
            }
            discriminators.push(d);
        }

        let mut expr = json!(0);
        for (i, d) in discriminators.iter().enumerate() {
            expr = json!(["?", d.condition(), (i + 1) as i64, expr]);
        }
        expr
    }

    fn expand(type_: &TypeNode, out: &mut Vec<TypeNode>) {
        match type_ {
            TypeNode::Ref(ref_type) => {
                let system = ref_type
                    .base
                    .system
                    .as_ref()
                    .unwrap_or_else(|| panic!("NO_SYSTEM"));
                let alias = system
                    .resolve(&ref_type.ref_)
                    .unwrap_or_else(|e| panic!("{e}"));
                let resolved = TypeBuilder::new().import(&alias.schema);
                Self::expand(&resolved, out);
            }
            TypeNode::Key(key_type) => {
                Self::expand(&key_type.val, out);
            }
            TypeNode::Or(or_type) => {
                for inner in &or_type.types {
                    Self::expand(inner, out);
                }
            }
            _ => out.push(type_.clone()),
        }
    }

    pub fn condition(&self) -> Value {
        if let TypeNode::Con(con) = &self.type_ {
            let fallback = if con.literal().is_null() {
                Value::String(String::new())
            } else {
                Value::Null
            };
            return json!(["==", con.literal().clone(), ["$", self.path, fallback]]);
        }

        match self.type_specifier() {
            "bool" => json!(["==", ["type", ["$", self.path]], "boolean"]),
            "num" => json!(["==", ["type", ["$", self.path]], "number"]),
            "str" => json!(["==", ["type", ["$", self.path]], "string"]),
            "obj" => json!(["==", ["type", ["$", self.path]], "object"]),
            "arr" => json!(["==", ["type", ["$", self.path]], "array"]),
            _ => panic!(
                "Cannot create condition for discriminator: {}",
                self.to_specifier()
            ),
        }
    }

    pub fn type_specifier(&self) -> &'static str {
        match self.type_.kind() {
            "bool" => "bool",
            "str" => "str",
            "num" => "num",
            "con" => "con",
            "obj" | "map" => "obj",
            "arr" => "arr",
            "fn" | "fn$" => "fn",
            _ => "",
        }
    }

    pub fn to_specifier(&self) -> String {
        let value = match &self.type_ {
            TypeNode::Con(con) => con.literal().clone(),
            _ => json!(0),
        };
        serde_json::to_string(&json!([self.path, self.type_specifier(), value])).unwrap_or_default()
    }
}
