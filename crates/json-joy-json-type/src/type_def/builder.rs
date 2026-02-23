//! TypeBuilder — factory for constructing TypeNode instances.
//!
//! Upstream reference: json-type/src/type/TypeBuilder.ts

use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use super::classes::*;
use super::module_type::ModuleType;
use super::TypeNode;
use crate::schema::Schema;

/// Factory for constructing TypeNode instances.
///
/// Mirrors the TypeScript `TypeBuilder` class.
#[derive(Debug, Clone, Default)]
pub struct TypeBuilder {
    pub system: Option<Arc<ModuleType>>,
}

#[allow(non_snake_case)]
impl TypeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_system(system: Arc<ModuleType>) -> Self {
        Self {
            system: Some(system),
        }
    }

    fn sys(&self) -> Option<Arc<ModuleType>> {
        self.system.clone()
    }

    // ------------------------------------------------------------------
    // Shorthand getters

    pub fn any(&self) -> TypeNode {
        self.Any(None)
    }

    pub fn bool(&self) -> TypeNode {
        self.Boolean(None)
    }

    pub fn num(&self) -> TypeNode {
        self.Number(None)
    }

    pub fn str(&self) -> TypeNode {
        self.String(None)
    }

    pub fn bin(&self) -> TypeNode {
        self.Binary(self.any(), None)
    }

    pub fn arr(&self) -> TypeNode {
        self.Array(self.any(), None)
    }

    pub fn obj(&self) -> TypeNode {
        self.Object(vec![])
    }

    pub fn map(&self) -> TypeNode {
        self.Map(self.any(), None, None)
    }

    pub fn undef(&self) -> TypeNode {
        self.Const(Value::Null, None)
    }

    pub fn nil(&self) -> TypeNode {
        self.Const(Value::Null, None)
    }

    pub fn fn_(&self) -> TypeNode {
        self.Function(self.undef(), self.undef(), None)
    }

    pub fn fn_rx(&self) -> TypeNode {
        self.function_streaming(self.undef(), self.undef(), None)
    }

    // ------------------------------------------------------------------
    // Factory methods

    pub fn Any(&self, _opts: Option<()>) -> TypeNode {
        TypeNode::Any(AnyType::new().sys(self.sys()))
    }

    pub fn Boolean(&self, _opts: Option<()>) -> TypeNode {
        TypeNode::Bool(BoolType::new().sys(self.sys()))
    }

    pub fn Number(&self, _opts: Option<()>) -> TypeNode {
        TypeNode::Num(NumType::new().sys(self.sys()))
    }

    pub fn String(&self, _opts: Option<()>) -> TypeNode {
        TypeNode::Str(StrType::new().sys(self.sys()))
    }

    pub fn Binary(&self, type_: TypeNode, _opts: Option<()>) -> TypeNode {
        TypeNode::Bin(BinType::new(type_).sys(self.sys()))
    }

    pub fn Array(&self, type_: TypeNode, _opts: Option<()>) -> TypeNode {
        TypeNode::Arr(ArrType::new(Some(type_), vec![], vec![]).sys(self.sys()))
    }

    pub fn Tuple(
        &self,
        head: Vec<TypeNode>,
        type_: Option<TypeNode>,
        tail: Option<Vec<TypeNode>>,
    ) -> TypeNode {
        TypeNode::Arr(ArrType::new(type_, head, tail.unwrap_or_default()).sys(self.sys()))
    }

    pub fn Object(&self, keys: Vec<KeyType>) -> TypeNode {
        TypeNode::Obj(ObjType::new(keys).sys(self.sys()))
    }

    pub fn Key(&self, key: impl Into<String>, value: TypeNode) -> TypeNode {
        TypeNode::Key(KeyType::new(key, value).sys(self.sys()))
    }

    pub fn KeyOpt(&self, key: impl Into<String>, value: TypeNode) -> TypeNode {
        TypeNode::Key(KeyType::new_opt(key, value).sys(self.sys()))
    }

    pub fn Map(&self, val: TypeNode, key: Option<TypeNode>, _opts: Option<()>) -> TypeNode {
        TypeNode::Map(MapType::new(val, key).sys(self.sys()))
    }

    pub fn Or(&self, types: Vec<TypeNode>) -> TypeNode {
        TypeNode::Or(OrType::new(types).sys(self.sys()))
    }

    pub fn Ref(&self, ref_: impl Into<String>) -> TypeNode {
        TypeNode::Ref(RefType::new(ref_).sys(self.sys()))
    }

    pub fn Const(&self, value: Value, _opts: Option<()>) -> TypeNode {
        TypeNode::Con(ConType::new(value).sys(self.sys()))
    }

    pub fn Function(&self, req: TypeNode, res: TypeNode, _opts: Option<()>) -> TypeNode {
        TypeNode::Fn(FnType::new(req, res).sys(self.sys()))
    }

    pub fn function_streaming(&self, req: TypeNode, res: TypeNode, _opts: Option<()>) -> TypeNode {
        TypeNode::FnRx(FnRxType::new(req, res).sys(self.sys()))
    }

    // ------------------------------------------------------------------
    // Higher-level helpers

    /// Create a union type from a list of const values.
    pub fn enum_<T: Into<Value> + Clone>(&self, values: Vec<T>) -> TypeNode {
        let types = values
            .into_iter()
            .map(|v| self.Const(v.into(), None))
            .collect();
        self.Or(types)
    }

    /// Create an "optional" union (T | undefined).
    pub fn maybe(&self, type_: TypeNode) -> TypeNode {
        self.Or(vec![type_, self.undef()])
    }

    /// Create an object type from a `key → TypeNode` map.
    pub fn object(&self, record: HashMap<String, TypeNode>) -> TypeNode {
        let mut keys: Vec<_> = record.into_iter().collect();
        keys.sort_by(|a, b| a.0.cmp(&b.0));
        let key_types: Vec<KeyType> = keys.into_iter().map(|(k, v)| KeyType::new(k, v)).collect();
        self.Object(key_types)
    }

    /// Create a tuple type from a list of element types.
    pub fn tuple(&self, types: Vec<TypeNode>) -> TypeNode {
        self.Tuple(types, None, None)
    }

    /// Import a `Schema` into a `TypeNode`.
    pub fn import(&self, schema: &Schema) -> TypeNode {
        match schema {
            Schema::Any(_) => self.Any(None),
            Schema::Bool(_) => self.Boolean(None),
            Schema::Num(s) => {
                let mut n = NumType::new().sys(self.sys());
                n.schema = s.clone();
                TypeNode::Num(n)
            }
            Schema::Str(s) => {
                let mut st = StrType::new().sys(self.sys());
                st.schema = s.clone();
                TypeNode::Str(st)
            }
            Schema::Bin(s) => {
                let inner = self.import(&s.type_);
                TypeNode::Bin(BinType::new(inner).sys(self.sys()))
            }
            Schema::Con(s) => self.Const(s.value.clone(), None),
            Schema::Arr(s) => {
                let head: Vec<TypeNode> = s
                    .head
                    .as_deref()
                    .unwrap_or(&[])
                    .iter()
                    .map(|h| self.import(h))
                    .collect();
                let type_ = s.type_.as_deref().map(|t| self.import(t));
                let tail: Vec<TypeNode> = s
                    .tail
                    .as_deref()
                    .unwrap_or(&[])
                    .iter()
                    .map(|t| self.import(t))
                    .collect();
                let mut arr = ArrType::new(type_, head, tail);
                arr.schema.min = s.min;
                arr.schema.max = s.max;
                TypeNode::Arr(arr.sys(self.sys()))
            }
            Schema::Obj(s) => {
                let keys: Vec<KeyType> = s
                    .keys
                    .iter()
                    .map(|k| {
                        let val = self.import(&k.value);
                        if k.optional == Some(true) {
                            KeyType::new_opt(k.key.clone(), val)
                        } else {
                            KeyType::new(k.key.clone(), val)
                        }
                    })
                    .collect();
                let mut obj = ObjType::new(keys).sys(self.sys());
                obj.schema.decode_unknown_keys = s.decode_unknown_keys;
                obj.schema.encode_unknown_keys = s.encode_unknown_keys;
                TypeNode::Obj(obj)
            }
            Schema::Key(s) => {
                let val = self.import(&s.value);
                if s.optional == Some(true) {
                    TypeNode::Key(KeyType::new_opt(s.key.clone(), val).sys(self.sys()))
                } else {
                    TypeNode::Key(KeyType::new(s.key.clone(), val).sys(self.sys()))
                }
            }
            Schema::Map(s) => {
                let val = self.import(&s.value);
                let key = s.key.as_deref().map(|k| self.import(k));
                self.Map(val, key, None)
            }
            Schema::Ref(s) => self.Ref(s.ref_.clone()),
            Schema::Or(s) => {
                let types: Vec<TypeNode> = s.types.iter().map(|t| self.import(t)).collect();
                TypeNode::Or(OrType::new(types).sys(self.sys()))
            }
            Schema::Fn(s) => {
                let req = self.import(&s.req);
                let res = self.import(&s.res);
                self.Function(req, res, None)
            }
            Schema::FnRx(s) => {
                let req = self.import(&s.req);
                let res = self.import(&s.res);
                self.function_streaming(req, res, None)
            }
            Schema::Alias(s) => self.import(&s.value),
            Schema::Module(_) => {
                // Modules are not directly representable as a TypeNode
                self.Any(None)
            }
        }
    }

    /// Infer a TypeNode from a JSON value.
    pub fn from_value(&self, value: &Value) -> TypeNode {
        match value {
            Value::Null => self.nil(),
            Value::Bool(_) => self.bool(),
            Value::Number(_) => self.num(),
            Value::String(_) => self.str(),
            Value::Array(arr) => {
                if arr.is_empty() {
                    return self.arr();
                }
                let first_type = self.from_value(&arr[0]);
                let first_kind = first_type.kind().to_string();
                let all_same = arr.iter().all(|v| self.from_value(v).kind() == first_kind);
                if all_same {
                    self.Array(first_type, None)
                } else {
                    let types = arr.iter().map(|v| self.from_value(v)).collect();
                    self.tuple(types)
                }
            }
            Value::Object(map) => {
                let keys: Vec<KeyType> = map
                    .iter()
                    .map(|(k, v)| KeyType::new(k.clone(), self.from_value(v)))
                    .collect();
                self.Object(keys)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{self, SchemaBase};
    use serde_json::json;

    fn t() -> TypeBuilder {
        TypeBuilder::new()
    }

    // -- Shorthand getters --

    #[test]
    fn shorthand_any() {
        assert_eq!(t().any().kind(), "any");
    }

    #[test]
    fn shorthand_bool() {
        assert_eq!(t().bool().kind(), "bool");
    }

    #[test]
    fn shorthand_num() {
        assert_eq!(t().num().kind(), "num");
    }

    #[test]
    fn shorthand_str() {
        assert_eq!(t().str().kind(), "str");
    }

    #[test]
    fn shorthand_bin() {
        assert_eq!(t().bin().kind(), "bin");
    }

    #[test]
    fn shorthand_arr() {
        assert_eq!(t().arr().kind(), "arr");
    }

    #[test]
    fn shorthand_obj() {
        assert_eq!(t().obj().kind(), "obj");
    }

    #[test]
    fn shorthand_map() {
        assert_eq!(t().map().kind(), "map");
    }

    #[test]
    fn shorthand_undef() {
        assert_eq!(t().undef().kind(), "con");
    }

    #[test]
    fn shorthand_nil() {
        assert_eq!(t().nil().kind(), "con");
    }

    #[test]
    fn shorthand_fn() {
        assert_eq!(t().fn_().kind(), "fn");
    }

    #[test]
    fn shorthand_fn_rx() {
        assert_eq!(t().fn_rx().kind(), "fn$");
    }

    // -- Factory methods --

    #[test]
    fn factory_const() {
        let node = t().Const(json!(42), None);
        assert_eq!(node.kind(), "con");
    }

    #[test]
    fn factory_binary() {
        let node = t().Binary(t().str(), None);
        assert_eq!(node.kind(), "bin");
    }

    #[test]
    fn factory_array() {
        let node = t().Array(t().num(), None);
        assert_eq!(node.kind(), "arr");
    }

    #[test]
    fn factory_tuple() {
        let node = t().Tuple(vec![t().str()], Some(t().num()), Some(vec![t().bool()]));
        assert_eq!(node.kind(), "arr");
    }

    #[test]
    fn factory_object() {
        let node = t().Object(vec![KeyType::new("x", t().num())]);
        assert_eq!(node.kind(), "obj");
    }

    #[test]
    fn factory_key() {
        let node = t().Key("k", t().str());
        assert_eq!(node.kind(), "key");
    }

    #[test]
    fn factory_key_opt() {
        let node = t().KeyOpt("k", t().str());
        if let TypeNode::Key(k) = &node {
            assert!(k.optional);
        } else {
            panic!("Expected Key");
        }
    }

    #[test]
    fn factory_map() {
        let node = t().Map(t().num(), Some(t().str()), None);
        assert_eq!(node.kind(), "map");
    }

    #[test]
    fn factory_or() {
        let node = t().Or(vec![t().str(), t().num()]);
        assert_eq!(node.kind(), "or");
    }

    #[test]
    fn factory_ref() {
        let node = t().Ref("MyType");
        assert_eq!(node.kind(), "ref");
    }

    #[test]
    fn factory_function() {
        let node = t().Function(t().str(), t().num(), None);
        assert_eq!(node.kind(), "fn");
    }

    #[test]
    fn factory_function_streaming() {
        let node = t().function_streaming(t().str(), t().num(), None);
        assert_eq!(node.kind(), "fn$");
    }

    // -- Higher-level helpers --

    #[test]
    fn enum_creates_or_of_consts() {
        let node = t().enum_(vec!["a", "b", "c"]);
        if let TypeNode::Or(or) = &node {
            assert_eq!(or.types.len(), 3);
            for ty in &or.types {
                assert_eq!(ty.kind(), "con");
            }
        } else {
            panic!("Expected Or");
        }
    }

    #[test]
    fn maybe_creates_union_with_undef() {
        let node = t().maybe(t().str());
        if let TypeNode::Or(or) = &node {
            assert_eq!(or.types.len(), 2);
            assert_eq!(or.types[0].kind(), "str");
            assert_eq!(or.types[1].kind(), "con");
        } else {
            panic!("Expected Or");
        }
    }

    #[test]
    fn object_from_hashmap() {
        let mut record = HashMap::new();
        record.insert("b".into(), t().num());
        record.insert("a".into(), t().str());
        let node = t().object(record);
        if let TypeNode::Obj(obj) = &node {
            // Keys should be sorted
            assert_eq!(obj.keys[0].key, "a");
            assert_eq!(obj.keys[1].key, "b");
        } else {
            panic!("Expected Obj");
        }
    }

    #[test]
    fn tuple_helper() {
        let node = t().tuple(vec![t().str(), t().num()]);
        if let TypeNode::Arr(arr) = &node {
            assert_eq!(arr.head.len(), 2);
            assert!(arr.type_.is_none());
        } else {
            panic!("Expected Arr");
        }
    }

    // -- with_system --

    #[test]
    fn with_system_stores_system() {
        let module = Arc::new(ModuleType::new());
        let tb = TypeBuilder::with_system(module.clone());
        assert!(tb.system.is_some());
        // All constructed types should carry the system
        let node = tb.any();
        assert!(node.base().system.is_some());
    }

    // -- import --

    #[test]
    fn import_any() {
        let s = Schema::Any(schema::AnySchema::default());
        let node = t().import(&s);
        assert_eq!(node.kind(), "any");
    }

    #[test]
    fn import_bool() {
        let s = Schema::Bool(schema::BoolSchema::default());
        assert_eq!(t().import(&s).kind(), "bool");
    }

    #[test]
    fn import_num() {
        let s = Schema::Num(schema::NumSchema {
            format: Some(schema::NumFormat::I32),
            gt: Some(0.0),
            ..Default::default()
        });
        let node = t().import(&s);
        if let TypeNode::Num(n) = &node {
            assert_eq!(n.schema.format, Some(schema::NumFormat::I32));
            assert_eq!(n.schema.gt, Some(0.0));
        } else {
            panic!("Expected Num");
        }
    }

    #[test]
    fn import_str() {
        let s = Schema::Str(schema::StrSchema {
            min: Some(5),
            ..Default::default()
        });
        let node = t().import(&s);
        if let TypeNode::Str(st) = &node {
            assert_eq!(st.schema.min, Some(5));
        } else {
            panic!("Expected Str");
        }
    }

    #[test]
    fn import_bin() {
        let s = Schema::Bin(schema::BinSchema {
            base: SchemaBase::default(),
            type_: Box::new(Schema::Any(schema::AnySchema::default())),
            format: None,
            min: None,
            max: None,
        });
        assert_eq!(t().import(&s).kind(), "bin");
    }

    #[test]
    fn import_con() {
        let s = Schema::Con(schema::ConSchema {
            base: SchemaBase::default(),
            value: json!("hello"),
        });
        assert_eq!(t().import(&s).kind(), "con");
    }

    #[test]
    fn import_arr() {
        let s = Schema::Arr(schema::ArrSchema {
            type_: Some(Box::new(Schema::Num(schema::NumSchema::default()))),
            ..Default::default()
        });
        assert_eq!(t().import(&s).kind(), "arr");
    }

    #[test]
    fn import_arr_with_head_tail() {
        let s = Schema::Arr(schema::ArrSchema {
            head: Some(vec![Schema::Str(schema::StrSchema::default())]),
            type_: Some(Box::new(Schema::Num(schema::NumSchema::default()))),
            tail: Some(vec![Schema::Bool(schema::BoolSchema::default())]),
            min: Some(1),
            max: Some(10),
            ..Default::default()
        });
        let node = t().import(&s);
        if let TypeNode::Arr(arr) = &node {
            assert_eq!(arr.head.len(), 1);
            assert!(arr.type_.is_some());
            assert_eq!(arr.tail.len(), 1);
            assert_eq!(arr.schema.min, Some(1));
            assert_eq!(arr.schema.max, Some(10));
        } else {
            panic!("Expected Arr");
        }
    }

    #[test]
    fn import_obj() {
        let s = Schema::Obj(schema::ObjSchema {
            keys: vec![schema::KeySchema {
                base: SchemaBase::default(),
                key: "name".into(),
                value: Box::new(Schema::Str(schema::StrSchema::default())),
                optional: Some(true),
            }],
            decode_unknown_keys: Some(true),
            ..Default::default()
        });
        let node = t().import(&s);
        if let TypeNode::Obj(obj) = &node {
            assert_eq!(obj.keys.len(), 1);
            assert!(obj.keys[0].optional);
            assert_eq!(obj.schema.decode_unknown_keys, Some(true));
        } else {
            panic!("Expected Obj");
        }
    }

    #[test]
    fn import_key_required() {
        let s = Schema::Key(schema::KeySchema {
            base: SchemaBase::default(),
            key: "x".into(),
            value: Box::new(Schema::Num(schema::NumSchema::default())),
            optional: None,
        });
        let node = t().import(&s);
        if let TypeNode::Key(k) = &node {
            assert!(!k.optional);
        } else {
            panic!("Expected Key");
        }
    }

    #[test]
    fn import_key_optional() {
        let s = Schema::Key(schema::KeySchema {
            base: SchemaBase::default(),
            key: "x".into(),
            value: Box::new(Schema::Num(schema::NumSchema::default())),
            optional: Some(true),
        });
        let node = t().import(&s);
        if let TypeNode::Key(k) = &node {
            assert!(k.optional);
        } else {
            panic!("Expected Key");
        }
    }

    #[test]
    fn import_map() {
        let s = Schema::Map(schema::MapSchema {
            base: SchemaBase::default(),
            key: Some(Box::new(Schema::Str(schema::StrSchema::default()))),
            value: Box::new(Schema::Num(schema::NumSchema::default())),
        });
        assert_eq!(t().import(&s).kind(), "map");
    }

    #[test]
    fn import_ref() {
        let s = Schema::Ref(schema::RefSchema {
            base: SchemaBase::default(),
            ref_: "Foo".into(),
        });
        assert_eq!(t().import(&s).kind(), "ref");
    }

    #[test]
    fn import_or() {
        let s = Schema::Or(schema::OrSchema {
            base: SchemaBase::default(),
            types: vec![
                Schema::Str(schema::StrSchema::default()),
                Schema::Num(schema::NumSchema::default()),
            ],
            discriminator: json!(null),
        });
        assert_eq!(t().import(&s).kind(), "or");
    }

    #[test]
    fn import_fn() {
        let s = Schema::Fn(schema::FnSchema {
            base: SchemaBase::default(),
            req: Box::new(Schema::Str(schema::StrSchema::default())),
            res: Box::new(Schema::Num(schema::NumSchema::default())),
        });
        assert_eq!(t().import(&s).kind(), "fn");
    }

    #[test]
    fn import_fn_rx() {
        let s = Schema::FnRx(schema::FnRxSchema {
            base: SchemaBase::default(),
            req: Box::new(Schema::Any(schema::AnySchema::default())),
            res: Box::new(Schema::Any(schema::AnySchema::default())),
        });
        assert_eq!(t().import(&s).kind(), "fn$");
    }

    #[test]
    fn import_alias() {
        let s = Schema::Alias(schema::AliasSchema {
            base: SchemaBase::default(),
            key: "Foo".into(),
            value: Box::new(Schema::Str(schema::StrSchema::default())),
            optional: None,
            pub_: None,
        });
        // Alias import delegates to the inner value
        assert_eq!(t().import(&s).kind(), "str");
    }

    #[test]
    fn import_module() {
        let s = Schema::Module(schema::ModuleSchema::default());
        // Module import falls back to any
        assert_eq!(t().import(&s).kind(), "any");
    }

    // -- from_value --

    #[test]
    fn from_value_null() {
        assert_eq!(t().from_value(&json!(null)).kind(), "con");
    }

    #[test]
    fn from_value_bool() {
        assert_eq!(t().from_value(&json!(true)).kind(), "bool");
    }

    #[test]
    fn from_value_number() {
        assert_eq!(t().from_value(&json!(42)).kind(), "num");
    }

    #[test]
    fn from_value_string() {
        assert_eq!(t().from_value(&json!("hello")).kind(), "str");
    }

    #[test]
    fn from_value_empty_array() {
        assert_eq!(t().from_value(&json!([])).kind(), "arr");
    }

    #[test]
    fn from_value_homogeneous_array() {
        let node = t().from_value(&json!([1, 2, 3]));
        assert_eq!(node.kind(), "arr");
        // Should be Array, not Tuple
        if let TypeNode::Arr(arr) = &node {
            assert!(arr.type_.is_some());
            assert!(arr.head.is_empty());
        } else {
            panic!("Expected Arr");
        }
    }

    #[test]
    fn from_value_heterogeneous_array() {
        let node = t().from_value(&json!([1, "two", true]));
        assert_eq!(node.kind(), "arr");
        // Should be Tuple (head elements, no type_)
        if let TypeNode::Arr(arr) = &node {
            assert!(!arr.head.is_empty());
            assert!(arr.type_.is_none());
        } else {
            panic!("Expected Arr");
        }
    }

    #[test]
    fn from_value_object() {
        let node = t().from_value(&json!({"name": "Alice", "age": 30}));
        assert_eq!(node.kind(), "obj");
        if let TypeNode::Obj(obj) = &node {
            assert_eq!(obj.keys.len(), 2);
        } else {
            panic!("Expected Obj");
        }
    }
}
