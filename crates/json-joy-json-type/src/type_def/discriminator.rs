//! Discriminator computation for `or` types.
//!
//! Upstream reference:
//! - `json-type/src/type/discriminator.ts`

use std::collections::HashSet;
use std::sync::Arc;

use serde_json::{json, Value};

use super::TypeNode;

/// A discriminator candidate (path + type) used to build union expressions.
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

    /// Finds the first constant discriminator in a type, if any.
    pub fn find_const(type_: &TypeNode) -> Option<Discriminator> {
        match type_ {
            TypeNode::Con(_) => Some(Discriminator::new("", type_.clone())),
            TypeNode::Arr(arr) => {
                for (i, child) in arr.head.iter().enumerate() {
                    if let Some(d) = Self::find_const(child) {
                        return Some(Discriminator::new(format!("/{i}{}", d.path), d.type_));
                    }
                }
                None
            }
            TypeNode::Obj(obj) => {
                for field in &obj.keys {
                    if let Some(d) = Self::find_const(&field.val) {
                        return Some(Discriminator::new(
                            format!("/{}{}", field.key, d.path),
                            d.type_,
                        ));
                    }
                }
                None
            }
            TypeNode::Key(key) => Self::find_const(&key.val),
            TypeNode::Ref(reference) => {
                let system = reference.base.system.as_ref()?;
                let alias = system.resolve(&reference.ref_).ok()?;
                let builder = super::builder::TypeBuilder::with_system(Arc::clone(system));
                let resolved = builder.import(&alias.schema);
                Self::find_const(&resolved)
            }
            _ => None,
        }
    }

    /// Finds a discriminator for a type (prefers constants, otherwise root type).
    pub fn find(type_: &TypeNode) -> Discriminator {
        Self::find_const(type_).unwrap_or_else(|| Discriminator::new("", type_.clone()))
    }

    /// Creates a JSON expression returning the matching union branch index.
    pub fn create_expression(types: &[TypeNode]) -> Result<Value, String> {
        let mut specifiers: HashSet<String> = HashSet::new();
        let mut expanded: Vec<TypeNode> = Vec::new();
        for type_ in types {
            expanded.extend(expand_type(type_)?);
        }

        let mut discriminators: Vec<Discriminator> = Vec::new();
        for type_ in expanded.iter().skip(1) {
            let d = Discriminator::find(type_);
            let specifier = d.to_specifier()?;
            if specifiers.contains(&specifier) {
                return Err(format!("Duplicate discriminator: {specifier}"));
            }
            specifiers.insert(specifier);
            discriminators.push(d);
        }

        let mut expr = json!(0);
        for (i, d) in discriminators.iter().enumerate() {
            expr = json!(["?", d.condition()?, (i + 1) as i64, expr]);
        }
        Ok(expr)
    }

    fn condition(&self) -> Result<Value, String> {
        if let TypeNode::Con(constant) = &self.type_ {
            let fallback = if constant.literal().is_null() {
                Value::String(String::new())
            } else {
                Value::Null
            };
            return Ok(json!([
                "==",
                constant.literal(),
                ["$", self.path, fallback]
            ]));
        }

        match self.type_specifier().as_str() {
            "bool" => Ok(json!(["==", ["type", ["$", self.path]], "boolean"])),
            "num" => Ok(json!(["==", ["type", ["$", self.path]], "number"])),
            "str" => Ok(json!(["==", ["type", ["$", self.path]], "string"])),
            "obj" => Ok(json!(["==", ["type", ["$", self.path]], "object"])),
            "arr" => Ok(json!(["==", ["type", ["$", self.path]], "array"])),
            _ => Err(format!(
                "Cannot create condition for discriminator: {}",
                self.to_specifier().unwrap_or_default()
            )),
        }
    }

    fn type_specifier(&self) -> String {
        match self.type_ {
            TypeNode::Bool(_) => "bool".to_string(),
            TypeNode::Str(_) => "str".to_string(),
            TypeNode::Num(_) => "num".to_string(),
            TypeNode::Con(_) => "con".to_string(),
            TypeNode::Obj(_) | TypeNode::Map(_) => "obj".to_string(),
            TypeNode::Arr(_) => "arr".to_string(),
            TypeNode::Fn(_) | TypeNode::FnRx(_) => "fn".to_string(),
            _ => String::new(),
        }
    }

    fn to_specifier(&self) -> Result<String, String> {
        let value = if let TypeNode::Con(constant) = &self.type_ {
            constant.literal().clone()
        } else {
            json!(0)
        };
        serde_json::to_string(&json!([self.path, self.type_specifier(), value]))
            .map_err(|e| e.to_string())
    }
}

fn expand_type(type_: &TypeNode) -> Result<Vec<TypeNode>, String> {
    let mut current = type_.clone();
    loop {
        match current {
            TypeNode::Ref(reference) => {
                let Some(system) = reference.base.system else {
                    return Ok(vec![TypeNode::Ref(reference)]);
                };
                let alias = system.resolve(&reference.ref_).map_err(|e| e.to_string())?;
                let builder = super::builder::TypeBuilder::with_system(system);
                current = builder.import(&alias.schema);
            }
            TypeNode::Key(key) => {
                current = (*key.val).clone();
            }
            TypeNode::Or(or_type) => {
                let mut out: Vec<TypeNode> = Vec::new();
                for child in or_type.types {
                    out.extend(expand_type(&child)?);
                }
                return Ok(out);
            }
            _ => return Ok(vec![current]),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::Discriminator;
    use crate::type_def::{KeyType, TypeBuilder, TypeNode};

    fn t() -> TypeBuilder {
        TypeBuilder::new()
    }

    #[test]
    fn create_expression_for_str_num_union() {
        let expr = Discriminator::create_expression(&[t().str(), t().num()]).expect("expression");
        assert_eq!(
            expr,
            json!(["?", ["==", ["type", ["$", ""]], "number"], 1, 0])
        );
    }

    #[test]
    fn create_expression_rejects_duplicate_discriminators() {
        let result = Discriminator::create_expression(&[t().str(), t().num(), t().num()]);
        assert!(result.is_err());
    }

    #[test]
    fn find_const_discovers_nested_object_constant() {
        let typ = TypeNode::Obj(crate::type_def::ObjType::new(vec![KeyType::new(
            "kind",
            t().Const(json!("user"), None),
        )]));
        let d = Discriminator::find_const(&typ).expect("const discriminator");
        assert_eq!(d.path, "/kind");
    }
}
