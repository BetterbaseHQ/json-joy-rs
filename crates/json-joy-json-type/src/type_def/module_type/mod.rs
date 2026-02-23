//! ModuleType — a namespace of named type aliases.
//!
//! Upstream reference: json-type/src/type/classes/ModuleType/index.ts

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::schema::{KeySchema, ModuleSchema, ObjSchema, Schema};

/// An alias entry in a module.
#[derive(Debug, Clone)]
pub struct AliasEntry {
    pub id: String,
    /// The type schema for this alias.
    pub schema: Schema,
}

/// Inner state of a module (aliases map).
#[derive(Debug, Default)]
pub struct ModuleTypeInner {
    pub aliases: HashMap<String, AliasEntry>,
}

/// A module/namespace of named type aliases.
///
/// Wraps `ModuleTypeInner` in an `Arc<RwLock<>>` for shared ownership.
#[derive(Debug, Clone, Default)]
pub struct ModuleType {
    pub inner: Arc<RwLock<ModuleTypeInner>>,
}

impl ModuleType {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from an upstream `ModuleSchema`.
    pub fn from_module_schema(module: &ModuleSchema) -> Self {
        let mt = Self::new();
        mt.import(module);
        mt
    }

    /// Register a named alias with the given schema. If already exists, returns the existing.
    pub fn alias(&self, id: impl Into<String>, schema: Schema) -> AliasEntry {
        let id = id.into();
        {
            let inner = self.inner.read().unwrap();
            if let Some(existing) = inner.aliases.get(&id) {
                return existing.clone();
            }
        }
        let entry = AliasEntry {
            id: id.clone(),
            schema,
        };
        let mut inner = self.inner.write().unwrap();
        inner.aliases.insert(id, entry.clone());
        entry
    }

    /// Look up an alias by ID.
    pub fn unalias(&self, id: &str) -> Result<AliasEntry, String> {
        let inner = self.inner.read().unwrap();
        inner
            .aliases
            .get(id)
            .cloned()
            .ok_or_else(|| format!("Alias not found: {}", id))
    }

    /// Check if an alias exists.
    pub fn has_alias(&self, id: &str) -> bool {
        let inner = self.inner.read().unwrap();
        inner.aliases.contains_key(id)
    }

    /// Resolve an alias, following ref chains.
    pub fn resolve(&self, id: &str) -> Result<AliasEntry, String> {
        let entry = self.unalias(id)?;
        match &entry.schema {
            Schema::Ref(r) => self.resolve(&r.ref_.clone()),
            _ => Ok(entry),
        }
    }

    /// Import a module schema, expanding `extends` and registering all aliases.
    pub fn import(&self, module: &ModuleSchema) {
        let mut type_map: HashMap<String, Schema> = HashMap::new();
        for alias in &module.keys {
            type_map.insert(alias.key.clone(), *alias.value.clone());
        }

        // Expand obj extends
        let mut expanded_map: HashMap<String, Schema> = HashMap::new();
        for (key, schema) in &type_map {
            if let Schema::Obj(obj) = schema {
                if obj.extends.is_some() {
                    let expanded = expand_obj_extends(obj, &type_map);
                    expanded_map.insert(key.clone(), Schema::Obj(expanded));
                } else {
                    expanded_map.insert(key.clone(), schema.clone());
                }
            } else {
                expanded_map.insert(key.clone(), schema.clone());
            }
        }

        for (id, schema) in expanded_map {
            self.alias(id, schema);
        }
    }

    /// Import a map of named schemas.
    pub fn import_types(&self, aliases: HashMap<String, Schema>) {
        for (id, schema) in aliases {
            self.alias(id, schema);
        }
    }

    /// Export all aliases as a map of schemas.
    pub fn export_types(&self) -> HashMap<String, Schema> {
        let inner = self.inner.read().unwrap();
        inner
            .aliases
            .iter()
            .map(|(k, v)| (k.clone(), v.schema.clone()))
            .collect()
    }
}

/// Expand the `extends` field of an ObjSchema, merging parent fields.
fn expand_obj_extends(obj: &ObjSchema, type_map: &HashMap<String, Schema>) -> ObjSchema {
    let mut result_keys: Vec<KeySchema> = Vec::new();
    let mut seen: HashMap<String, usize> = HashMap::new();

    let add_key =
        |result_keys: &mut Vec<KeySchema>, seen: &mut HashMap<String, usize>, key: KeySchema| {
            if let Some(&idx) = seen.get(&key.key) {
                result_keys[idx] = key;
            } else {
                seen.insert(key.key.clone(), result_keys.len());
                result_keys.push(key);
            }
        };

    if let Some(extends) = &obj.extends {
        for parent_id in extends {
            if let Some(Schema::Obj(parent)) = type_map.get(parent_id) {
                let parent_expanded = if parent.extends.is_some() {
                    expand_obj_extends(parent, type_map)
                } else {
                    parent.clone()
                };
                for key in parent_expanded.keys {
                    add_key(&mut result_keys, &mut seen, key);
                }
            }
        }
    }
    for key in &obj.keys {
        add_key(&mut result_keys, &mut seen, key.clone());
    }

    ObjSchema {
        base: obj.base.clone(),
        keys: result_keys,
        extends: None,
        decode_unknown_keys: obj.decode_unknown_keys,
        encode_unknown_keys: obj.encode_unknown_keys,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{AliasSchema, AnySchema, SchemaBase, StrSchema};

    fn str_schema() -> Schema {
        Schema::Str(StrSchema::default())
    }

    fn any_schema() -> Schema {
        Schema::Any(AnySchema::default())
    }

    fn key_schema(name: &str, schema: Schema) -> KeySchema {
        KeySchema {
            base: SchemaBase::default(),
            key: name.to_string(),
            value: Box::new(schema),
            optional: None,
        }
    }

    #[test]
    fn module_type_new() {
        let m = ModuleType::new();
        let inner = m.inner.read().unwrap();
        assert!(inner.aliases.is_empty());
    }

    #[test]
    fn module_type_alias_and_unalias() {
        let m = ModuleType::new();
        m.alias("Foo", str_schema());
        let entry = m.unalias("Foo").unwrap();
        assert_eq!(entry.id, "Foo");
        assert_eq!(entry.schema.kind(), "str");
    }

    #[test]
    fn module_type_alias_idempotent() {
        let m = ModuleType::new();
        m.alias("Foo", str_schema());
        // Re-registering returns existing
        let entry = m.alias("Foo", any_schema());
        assert_eq!(entry.schema.kind(), "str"); // original, not new
    }

    #[test]
    fn module_type_unalias_missing() {
        let m = ModuleType::new();
        let result = m.unalias("Missing");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing"));
    }

    #[test]
    fn module_type_has_alias() {
        let m = ModuleType::new();
        assert!(!m.has_alias("Foo"));
        m.alias("Foo", str_schema());
        assert!(m.has_alias("Foo"));
    }

    #[test]
    fn module_type_resolve_direct() {
        let m = ModuleType::new();
        m.alias("Foo", str_schema());
        let resolved = m.resolve("Foo").unwrap();
        assert_eq!(resolved.schema.kind(), "str");
    }

    #[test]
    fn module_type_resolve_ref_chain() {
        let m = ModuleType::new();
        m.alias("Base", str_schema());
        m.alias(
            "Alias",
            Schema::Ref(crate::schema::RefSchema {
                base: SchemaBase::default(),
                ref_: "Base".into(),
            }),
        );
        let resolved = m.resolve("Alias").unwrap();
        assert_eq!(resolved.id, "Base");
        assert_eq!(resolved.schema.kind(), "str");
    }

    #[test]
    fn module_type_resolve_missing() {
        let m = ModuleType::new();
        assert!(m.resolve("Missing").is_err());
    }

    #[test]
    fn module_type_from_module_schema() {
        let ms = ModuleSchema {
            base: SchemaBase::default(),
            keys: vec![AliasSchema {
                base: SchemaBase::default(),
                key: "MyType".into(),
                value: Box::new(str_schema()),
                optional: None,
                pub_: Some(true),
            }],
        };
        let m = ModuleType::from_module_schema(&ms);
        assert!(m.has_alias("MyType"));
    }

    #[test]
    fn module_type_import_with_extends() {
        // Parent object
        let parent = Schema::Obj(ObjSchema {
            keys: vec![key_schema("id", any_schema())],
            ..Default::default()
        });
        // Child extends parent
        let child = Schema::Obj(ObjSchema {
            keys: vec![key_schema("name", str_schema())],
            extends: Some(vec!["Parent".into()]),
            ..Default::default()
        });
        let ms = ModuleSchema {
            base: SchemaBase::default(),
            keys: vec![
                AliasSchema {
                    base: SchemaBase::default(),
                    key: "Parent".into(),
                    value: Box::new(parent),
                    optional: None,
                    pub_: None,
                },
                AliasSchema {
                    base: SchemaBase::default(),
                    key: "Child".into(),
                    value: Box::new(child),
                    optional: None,
                    pub_: None,
                },
            ],
        };
        let m = ModuleType::new();
        m.import(&ms);

        let child_entry = m.unalias("Child").unwrap();
        if let Schema::Obj(obj) = &child_entry.schema {
            let key_names: Vec<&str> = obj.keys.iter().map(|k| k.key.as_str()).collect();
            assert!(key_names.contains(&"id"));
            assert!(key_names.contains(&"name"));
            assert!(obj.extends.is_none()); // extends should be removed
        } else {
            panic!("Expected Obj schema for Child");
        }
    }

    #[test]
    fn module_type_import_types() {
        let m = ModuleType::new();
        let mut types = HashMap::new();
        types.insert("A".into(), str_schema());
        types.insert("B".into(), any_schema());
        m.import_types(types);
        assert!(m.has_alias("A"));
        assert!(m.has_alias("B"));
    }

    #[test]
    fn module_type_export_types() {
        let m = ModuleType::new();
        m.alias("X", str_schema());
        m.alias("Y", any_schema());
        let exported = m.export_types();
        assert_eq!(exported.len(), 2);
        assert!(exported.contains_key("X"));
        assert!(exported.contains_key("Y"));
    }

    #[test]
    fn expand_obj_extends_no_extends() {
        let obj = ObjSchema {
            keys: vec![key_schema("a", str_schema())],
            ..Default::default()
        };
        let type_map = HashMap::new();
        let result = expand_obj_extends(&obj, &type_map);
        assert_eq!(result.keys.len(), 1);
        assert_eq!(result.keys[0].key, "a");
    }

    #[test]
    fn expand_obj_extends_with_override() {
        let parent_obj = ObjSchema {
            keys: vec![key_schema("id", any_schema())],
            ..Default::default()
        };
        let child_obj = ObjSchema {
            keys: vec![key_schema("id", str_schema())],
            extends: Some(vec!["Parent".into()]),
            ..Default::default()
        };
        let mut type_map = HashMap::new();
        type_map.insert("Parent".into(), Schema::Obj(parent_obj));

        let result = expand_obj_extends(&child_obj, &type_map);
        // Child's "id" should override parent's "id"
        assert_eq!(result.keys.len(), 1);
        assert_eq!(result.keys[0].key, "id");
        assert_eq!(result.keys[0].value.kind(), "str");
    }

    #[test]
    fn expand_obj_extends_preserves_decode_unknown() {
        let obj = ObjSchema {
            keys: vec![],
            decode_unknown_keys: Some(true),
            encode_unknown_keys: Some(false),
            ..Default::default()
        };
        let result = expand_obj_extends(&obj, &HashMap::new());
        assert_eq!(result.decode_unknown_keys, Some(true));
        assert_eq!(result.encode_unknown_keys, Some(false));
    }

    #[test]
    fn expand_obj_extends_chained() {
        let grandparent = ObjSchema {
            keys: vec![key_schema("gp_field", any_schema())],
            ..Default::default()
        };
        let parent = ObjSchema {
            keys: vec![key_schema("p_field", str_schema())],
            extends: Some(vec!["Grandparent".into()]),
            ..Default::default()
        };
        let child = ObjSchema {
            keys: vec![key_schema("c_field", str_schema())],
            extends: Some(vec!["Parent".into()]),
            ..Default::default()
        };
        let mut type_map = HashMap::new();
        type_map.insert("Grandparent".into(), Schema::Obj(grandparent));
        type_map.insert("Parent".into(), Schema::Obj(parent));

        let result = expand_obj_extends(&child, &type_map);
        let key_names: Vec<&str> = result.keys.iter().map(|k| k.key.as_str()).collect();
        assert!(key_names.contains(&"gp_field"));
        assert!(key_names.contains(&"p_field"));
        assert!(key_names.contains(&"c_field"));
    }
}
