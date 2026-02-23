//! Type class implementations.
//!
//! Each struct here corresponds to a TypeScript class extending AbsType<S>.

use serde_json::Value;
use std::sync::Arc;

use super::abs_type::BaseInfo;
use super::module_type::ModuleType;
use crate::schema::*;
use crate::type_def::discriminator::Discriminator;

// -------------------------------------------------------------------------
// AnyType

#[derive(Debug, Clone, Default)]
pub struct AnyType {
    pub base: BaseInfo,
}

impl AnyType {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn sys(mut self, system: Option<Arc<ModuleType>>) -> Self {
        self.base.system = system;
        self
    }
    pub fn get_schema(&self) -> Schema {
        Schema::Any(AnySchema {
            base: SchemaBase::default(),
        })
    }
    pub fn kind(&self) -> &'static str {
        "any"
    }
}

// -------------------------------------------------------------------------
// BoolType

#[derive(Debug, Clone, Default)]
pub struct BoolType {
    pub base: BaseInfo,
}

impl BoolType {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn sys(mut self, system: Option<Arc<ModuleType>>) -> Self {
        self.base.system = system;
        self
    }
    pub fn get_schema(&self) -> Schema {
        Schema::Bool(BoolSchema {
            base: SchemaBase::default(),
        })
    }
    pub fn kind(&self) -> &'static str {
        "bool"
    }
}

// -------------------------------------------------------------------------
// NumType

#[derive(Debug, Clone, Default)]
pub struct NumType {
    pub schema: NumSchema,
    pub base: BaseInfo,
}

impl NumType {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn sys(mut self, system: Option<Arc<ModuleType>>) -> Self {
        self.base.system = system;
        self
    }
    pub fn format(mut self, format: NumFormat) -> Self {
        self.schema.format = Some(format);
        self
    }
    pub fn gt(mut self, v: f64) -> Self {
        self.schema.gt = Some(v);
        self
    }
    pub fn gte(mut self, v: f64) -> Self {
        self.schema.gte = Some(v);
        self
    }
    pub fn lt(mut self, v: f64) -> Self {
        self.schema.lt = Some(v);
        self
    }
    pub fn lte(mut self, v: f64) -> Self {
        self.schema.lte = Some(v);
        self
    }
    pub fn get_schema(&self) -> Schema {
        Schema::Num(self.schema.clone())
    }
    pub fn kind(&self) -> &'static str {
        "num"
    }
}

// -------------------------------------------------------------------------
// StrType

#[derive(Debug, Clone, Default)]
pub struct StrType {
    pub schema: StrSchema,
    pub base: BaseInfo,
}

impl StrType {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn sys(mut self, system: Option<Arc<ModuleType>>) -> Self {
        self.base.system = system;
        self
    }
    pub fn format(mut self, format: StrFormat) -> Self {
        self.schema.format = Some(format);
        self
    }
    pub fn min(mut self, v: u64) -> Self {
        self.schema.min = Some(v);
        self
    }
    pub fn max(mut self, v: u64) -> Self {
        self.schema.max = Some(v);
        self
    }
    pub fn ascii(mut self, v: bool) -> Self {
        self.schema.ascii = Some(v);
        self
    }
    pub fn get_schema(&self) -> Schema {
        Schema::Str(self.schema.clone())
    }
    pub fn kind(&self) -> &'static str {
        "str"
    }
}

// -------------------------------------------------------------------------
// BinType

#[derive(Debug, Clone)]
pub struct BinType {
    pub inner_type: Box<super::TypeNode>,
    pub schema: BinSchema,
    pub base: BaseInfo,
}

impl BinType {
    pub fn new(inner_type: super::TypeNode) -> Self {
        let schema = BinSchema {
            base: SchemaBase::default(),
            type_: Box::new(inner_type.get_schema()),
            format: None,
            min: None,
            max: None,
        };
        Self {
            inner_type: Box::new(inner_type),
            schema,
            base: BaseInfo::default(),
        }
    }
    pub fn sys(mut self, system: Option<Arc<ModuleType>>) -> Self {
        self.base.system = system;
        self
    }
    pub fn min(mut self, v: u64) -> Self {
        self.schema.min = Some(v);
        self
    }
    pub fn max(mut self, v: u64) -> Self {
        self.schema.max = Some(v);
        self
    }
    pub fn get_schema(&self) -> Schema {
        Schema::Bin(BinSchema {
            base: SchemaBase::default(),
            type_: Box::new(self.inner_type.get_schema()),
            format: self.schema.format,
            min: self.schema.min,
            max: self.schema.max,
        })
    }
    pub fn kind(&self) -> &'static str {
        "bin"
    }
}

// -------------------------------------------------------------------------
// ConType

#[derive(Debug, Clone)]
pub struct ConType {
    pub value: Value,
    pub base: BaseInfo,
}

impl ConType {
    pub fn new(value: Value) -> Self {
        Self {
            value,
            base: BaseInfo::default(),
        }
    }
    pub fn sys(mut self, system: Option<Arc<ModuleType>>) -> Self {
        self.base.system = system;
        self
    }
    pub fn literal(&self) -> &Value {
        &self.value
    }
    pub fn get_schema(&self) -> Schema {
        Schema::Con(ConSchema {
            base: SchemaBase::default(),
            value: self.value.clone(),
        })
    }
    pub fn kind(&self) -> &'static str {
        "con"
    }
}

// -------------------------------------------------------------------------
// ArrType

#[derive(Debug, Clone, Default)]
pub struct ArrType {
    pub type_: Option<Box<super::TypeNode>>,
    pub head: Vec<super::TypeNode>,
    pub tail: Vec<super::TypeNode>,
    pub schema: ArrSchema,
    pub base: BaseInfo,
}

impl ArrType {
    pub fn new(
        type_: Option<super::TypeNode>,
        head: Vec<super::TypeNode>,
        tail: Vec<super::TypeNode>,
    ) -> Self {
        Self {
            type_: type_.map(Box::new),
            head,
            tail,
            schema: ArrSchema::default(),
            base: BaseInfo::default(),
        }
    }
    pub fn sys(mut self, system: Option<Arc<ModuleType>>) -> Self {
        self.base.system = system;
        self
    }
    pub fn min(mut self, v: u64) -> Self {
        self.schema.min = Some(v);
        self
    }
    pub fn max(mut self, v: u64) -> Self {
        self.schema.max = Some(v);
        self
    }
    pub fn get_schema(&self) -> Schema {
        let mut arr = self.schema.clone();
        if let Some(t) = &self.type_ {
            arr.type_ = Some(Box::new(t.get_schema()));
        }
        if !self.head.is_empty() {
            arr.head = Some(self.head.iter().map(|t| t.get_schema()).collect());
        }
        if !self.tail.is_empty() {
            arr.tail = Some(self.tail.iter().map(|t| t.get_schema()).collect());
        }
        Schema::Arr(arr)
    }
    pub fn kind(&self) -> &'static str {
        "arr"
    }
}

// -------------------------------------------------------------------------
// KeyType / KeyOptType

#[derive(Debug, Clone)]
pub struct KeyType {
    pub key: String,
    pub val: Box<super::TypeNode>,
    pub optional: bool,
    pub base: BaseInfo,
}

impl KeyType {
    pub fn new(key: impl Into<String>, val: super::TypeNode) -> Self {
        Self {
            key: key.into(),
            val: Box::new(val),
            optional: false,
            base: BaseInfo::default(),
        }
    }
    pub fn new_opt(key: impl Into<String>, val: super::TypeNode) -> Self {
        Self {
            key: key.into(),
            val: Box::new(val),
            optional: true,
            base: BaseInfo::default(),
        }
    }
    pub fn sys(mut self, system: Option<Arc<ModuleType>>) -> Self {
        self.base.system = system;
        self
    }
    pub fn get_schema(&self) -> Schema {
        Schema::Key(KeySchema {
            base: SchemaBase::default(),
            key: self.key.clone(),
            value: Box::new(self.val.get_schema()),
            optional: if self.optional { Some(true) } else { None },
        })
    }
    pub fn kind(&self) -> &'static str {
        "key"
    }
}

// -------------------------------------------------------------------------
// ObjType

#[derive(Debug, Clone, Default)]
pub struct ObjType {
    pub keys: Vec<KeyType>,
    pub schema: ObjSchema,
    pub base: BaseInfo,
}

impl ObjType {
    pub fn new(keys: Vec<KeyType>) -> Self {
        Self {
            keys,
            schema: ObjSchema::default(),
            base: BaseInfo::default(),
        }
    }
    pub fn sys(mut self, system: Option<Arc<ModuleType>>) -> Self {
        self.base.system = system;
        self
    }
    pub fn prop(mut self, key: impl Into<String>, val: super::TypeNode) -> Self {
        self.keys.push(KeyType::new(key, val));
        self
    }
    pub fn opt(mut self, key: impl Into<String>, val: super::TypeNode) -> Self {
        self.keys.push(KeyType::new_opt(key, val));
        self
    }
    pub fn extend(mut self, other: ObjType) -> Self {
        self.keys.extend(other.keys);
        self
    }
    pub fn omit(mut self, key: &str) -> Self {
        self.keys.retain(|k| k.key != key);
        self
    }
    pub fn get_field(&self, key: &str) -> Option<&KeyType> {
        self.keys.iter().find(|k| k.key == key)
    }
    pub fn get_schema(&self) -> Schema {
        let mut obj = self.schema.clone();
        obj.keys = self
            .keys
            .iter()
            .map(|k| KeySchema {
                base: SchemaBase::default(),
                key: k.key.clone(),
                value: Box::new(k.val.get_schema()),
                optional: if k.optional { Some(true) } else { None },
            })
            .collect();
        Schema::Obj(obj)
    }
    pub fn kind(&self) -> &'static str {
        "obj"
    }
}

// -------------------------------------------------------------------------
// MapType

#[derive(Debug, Clone)]
pub struct MapType {
    pub value: Box<super::TypeNode>,
    pub key: Option<Box<super::TypeNode>>,
    pub base: BaseInfo,
}

impl MapType {
    pub fn new(value: super::TypeNode, key: Option<super::TypeNode>) -> Self {
        Self {
            value: Box::new(value),
            key: key.map(Box::new),
            base: BaseInfo::default(),
        }
    }
    pub fn sys(mut self, system: Option<Arc<ModuleType>>) -> Self {
        self.base.system = system;
        self
    }
    pub fn get_schema(&self) -> Schema {
        Schema::Map(MapSchema {
            base: SchemaBase::default(),
            key: self.key.as_ref().map(|k| Box::new(k.get_schema())),
            value: Box::new(self.value.get_schema()),
        })
    }
    pub fn kind(&self) -> &'static str {
        "map"
    }
}

// -------------------------------------------------------------------------
// RefType

#[derive(Debug, Clone)]
pub struct RefType {
    pub ref_: String,
    pub base: BaseInfo,
}

impl RefType {
    pub fn new(ref_: impl Into<String>) -> Self {
        Self {
            ref_: ref_.into(),
            base: BaseInfo::default(),
        }
    }
    pub fn sys(mut self, system: Option<Arc<ModuleType>>) -> Self {
        self.base.system = system;
        self
    }
    pub fn ref_name(&self) -> &str {
        &self.ref_
    }
    pub fn get_schema(&self) -> Schema {
        Schema::Ref(RefSchema {
            base: SchemaBase::default(),
            ref_: self.ref_.clone(),
        })
    }
    pub fn kind(&self) -> &'static str {
        "ref"
    }
}

// -------------------------------------------------------------------------
// OrType

#[derive(Debug, Clone)]
pub struct OrType {
    pub types: Vec<super::TypeNode>,
    pub discriminator: Value,
    pub base: BaseInfo,
}

impl OrType {
    pub fn new(types: Vec<super::TypeNode>) -> Self {
        let discriminator = compute_discriminator(&types);
        Self {
            types,
            discriminator,
            base: BaseInfo::default(),
        }
    }
    pub fn sys(mut self, system: Option<Arc<ModuleType>>) -> Self {
        self.base.system = system;
        self
    }
    pub fn get_schema(&self) -> Schema {
        Schema::Or(OrSchema {
            base: SchemaBase::default(),
            types: self.types.iter().map(|t| t.get_schema()).collect(),
            discriminator: self.discriminator.clone(),
        })
    }
    pub fn kind(&self) -> &'static str {
        "or"
    }
}

fn compute_discriminator(types: &[super::TypeNode]) -> Value {
    // Preserve a usable fallback when discriminator derivation fails, but prefer
    // the upstream expression builder for parity.
    Discriminator::create_expression(types).unwrap_or_else(|_| serde_json::json!(["num", -1]))
}

// -------------------------------------------------------------------------
// FnType / FnRxType

#[derive(Debug, Clone)]
pub struct FnType {
    pub req: Box<super::TypeNode>,
    pub res: Box<super::TypeNode>,
    pub base: BaseInfo,
}

impl FnType {
    pub fn new(req: super::TypeNode, res: super::TypeNode) -> Self {
        Self {
            req: Box::new(req),
            res: Box::new(res),
            base: BaseInfo::default(),
        }
    }
    pub fn sys(mut self, system: Option<Arc<ModuleType>>) -> Self {
        self.base.system = system;
        self
    }
    pub fn get_schema(&self) -> Schema {
        Schema::Fn(FnSchema {
            base: SchemaBase::default(),
            req: Box::new(self.req.get_schema()),
            res: Box::new(self.res.get_schema()),
        })
    }
    pub fn kind(&self) -> &'static str {
        "fn"
    }
}

#[derive(Debug, Clone)]
pub struct FnRxType {
    pub req: Box<super::TypeNode>,
    pub res: Box<super::TypeNode>,
    pub base: BaseInfo,
}

impl FnRxType {
    pub fn new(req: super::TypeNode, res: super::TypeNode) -> Self {
        Self {
            req: Box::new(req),
            res: Box::new(res),
            base: BaseInfo::default(),
        }
    }
    pub fn sys(mut self, system: Option<Arc<ModuleType>>) -> Self {
        self.base.system = system;
        self
    }
    pub fn get_schema(&self) -> Schema {
        Schema::FnRx(FnRxSchema {
            base: SchemaBase::default(),
            req: Box::new(self.req.get_schema()),
            res: Box::new(self.res.get_schema()),
        })
    }
    pub fn kind(&self) -> &'static str {
        "fn$"
    }
}

// -------------------------------------------------------------------------
// AliasType

#[derive(Debug, Clone)]
pub struct AliasType {
    pub id: String,
    pub type_: Box<super::TypeNode>,
    pub system: Arc<ModuleType>,
    pub base: BaseInfo,
}

impl AliasType {
    pub fn new(system: Arc<ModuleType>, id: impl Into<String>, type_: super::TypeNode) -> Self {
        Self {
            id: id.into(),
            type_: Box::new(type_),
            system,
            base: BaseInfo::default(),
        }
    }
    pub fn get_type(&self) -> &super::TypeNode {
        &self.type_
    }
    pub fn get_schema(&self) -> Schema {
        self.type_.get_schema()
    }
    pub fn kind(&self) -> &'static str {
        "alias"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Arc;

    // -- AnyType --

    #[test]
    fn any_type_new_default() {
        let t = AnyType::new();
        assert_eq!(t.kind(), "any");
        assert!(t.base.system.is_none());
    }

    #[test]
    fn any_type_sys() {
        let module = Arc::new(ModuleType::new());
        let t = AnyType::new().sys(Some(module));
        assert!(t.base.system.is_some());
    }

    #[test]
    fn any_type_get_schema() {
        let s = AnyType::new().get_schema();
        assert_eq!(s.kind(), "any");
    }

    // -- BoolType --

    #[test]
    fn bool_type_new_default() {
        let t = BoolType::new();
        assert_eq!(t.kind(), "bool");
    }

    #[test]
    fn bool_type_sys() {
        let module = Arc::new(ModuleType::new());
        let t = BoolType::new().sys(Some(module));
        assert!(t.base.system.is_some());
    }

    #[test]
    fn bool_type_get_schema() {
        let s = BoolType::new().get_schema();
        assert_eq!(s.kind(), "bool");
    }

    // -- NumType --

    #[test]
    fn num_type_new_default() {
        let t = NumType::new();
        assert_eq!(t.kind(), "num");
        assert!(t.schema.format.is_none());
    }

    #[test]
    fn num_type_format() {
        let t = NumType::new().format(NumFormat::I32);
        assert_eq!(t.schema.format, Some(NumFormat::I32));
    }

    #[test]
    fn num_type_gt() {
        let t = NumType::new().gt(5.0);
        assert_eq!(t.schema.gt, Some(5.0));
    }

    #[test]
    fn num_type_gte() {
        let t = NumType::new().gte(5.0);
        assert_eq!(t.schema.gte, Some(5.0));
    }

    #[test]
    fn num_type_lt() {
        let t = NumType::new().lt(10.0);
        assert_eq!(t.schema.lt, Some(10.0));
    }

    #[test]
    fn num_type_lte() {
        let t = NumType::new().lte(10.0);
        assert_eq!(t.schema.lte, Some(10.0));
    }

    #[test]
    fn num_type_get_schema() {
        let t = NumType::new().format(NumFormat::F64).gt(0.0);
        let s = t.get_schema();
        if let Schema::Num(num) = &s {
            assert_eq!(num.format, Some(NumFormat::F64));
            assert_eq!(num.gt, Some(0.0));
        } else {
            panic!("Expected Num");
        }
    }

    // -- StrType --

    #[test]
    fn str_type_new_default() {
        let t = StrType::new();
        assert_eq!(t.kind(), "str");
    }

    #[test]
    fn str_type_format() {
        let t = StrType::new().format(StrFormat::Ascii);
        assert_eq!(t.schema.format, Some(StrFormat::Ascii));
    }

    #[test]
    fn str_type_min_max() {
        let t = StrType::new().min(1).max(100);
        assert_eq!(t.schema.min, Some(1));
        assert_eq!(t.schema.max, Some(100));
    }

    #[test]
    fn str_type_ascii() {
        let t = StrType::new().ascii(true);
        assert_eq!(t.schema.ascii, Some(true));
    }

    #[test]
    fn str_type_get_schema() {
        let t = StrType::new().min(5).max(50);
        let s = t.get_schema();
        if let Schema::Str(str_s) = &s {
            assert_eq!(str_s.min, Some(5));
            assert_eq!(str_s.max, Some(50));
        } else {
            panic!("Expected Str");
        }
    }

    // -- BinType --

    #[test]
    fn bin_type_new() {
        let inner = super::super::TypeNode::Any(AnyType::new());
        let t = BinType::new(inner);
        assert_eq!(t.kind(), "bin");
    }

    #[test]
    fn bin_type_min_max() {
        let inner = super::super::TypeNode::Any(AnyType::new());
        let t = BinType::new(inner).min(0).max(1024);
        assert_eq!(t.schema.min, Some(0));
        assert_eq!(t.schema.max, Some(1024));
    }

    #[test]
    fn bin_type_get_schema() {
        let inner = super::super::TypeNode::Any(AnyType::new());
        let t = BinType::new(inner);
        let s = t.get_schema();
        assert_eq!(s.kind(), "bin");
    }

    // -- ConType --

    #[test]
    fn con_type_new() {
        let t = ConType::new(json!(42));
        assert_eq!(t.kind(), "con");
        assert_eq!(t.value, json!(42));
    }

    #[test]
    fn con_type_literal() {
        let t = ConType::new(json!("hello"));
        assert_eq!(t.literal(), &json!("hello"));
    }

    #[test]
    fn con_type_get_schema() {
        let t = ConType::new(json!(true));
        let s = t.get_schema();
        if let Schema::Con(con) = &s {
            assert_eq!(con.value, json!(true));
        } else {
            panic!("Expected Con");
        }
    }

    // -- ArrType --

    #[test]
    fn arr_type_new() {
        let inner = super::super::TypeNode::Num(NumType::new());
        let t = ArrType::new(Some(inner), vec![], vec![]);
        assert_eq!(t.kind(), "arr");
    }

    #[test]
    fn arr_type_min_max() {
        let inner = super::super::TypeNode::Num(NumType::new());
        let t = ArrType::new(Some(inner), vec![], vec![]).min(1).max(10);
        assert_eq!(t.schema.min, Some(1));
        assert_eq!(t.schema.max, Some(10));
    }

    #[test]
    fn arr_type_get_schema_with_head_and_tail() {
        let head = vec![super::super::TypeNode::Str(StrType::new())];
        let inner = super::super::TypeNode::Num(NumType::new());
        let tail = vec![super::super::TypeNode::Bool(BoolType::new())];
        let t = ArrType::new(Some(inner), head, tail);
        let s = t.get_schema();
        if let Schema::Arr(arr) = &s {
            assert!(arr.head.is_some());
            assert!(arr.type_.is_some());
            assert!(arr.tail.is_some());
        } else {
            panic!("Expected Arr");
        }
    }

    #[test]
    fn arr_type_get_schema_no_head_tail() {
        let t = ArrType::new(None, vec![], vec![]);
        let s = t.get_schema();
        if let Schema::Arr(arr) = &s {
            assert!(arr.head.is_none());
            assert!(arr.type_.is_none());
            assert!(arr.tail.is_none());
        } else {
            panic!("Expected Arr");
        }
    }

    // -- KeyType --

    #[test]
    fn key_type_new_required() {
        let val = super::super::TypeNode::Str(StrType::new());
        let k = KeyType::new("name", val);
        assert_eq!(k.key, "name");
        assert!(!k.optional);
        assert_eq!(k.kind(), "key");
    }

    #[test]
    fn key_type_new_opt() {
        let val = super::super::TypeNode::Str(StrType::new());
        let k = KeyType::new_opt("age", val);
        assert_eq!(k.key, "age");
        assert!(k.optional);
    }

    #[test]
    fn key_type_get_schema_required() {
        let val = super::super::TypeNode::Num(NumType::new());
        let k = KeyType::new("count", val);
        let s = k.get_schema();
        if let Schema::Key(key) = &s {
            assert_eq!(key.key, "count");
            assert!(key.optional.is_none());
        } else {
            panic!("Expected Key");
        }
    }

    #[test]
    fn key_type_get_schema_optional() {
        let val = super::super::TypeNode::Num(NumType::new());
        let k = KeyType::new_opt("count", val);
        let s = k.get_schema();
        if let Schema::Key(key) = &s {
            assert_eq!(key.optional, Some(true));
        } else {
            panic!("Expected Key");
        }
    }

    // -- ObjType --

    #[test]
    fn obj_type_new_empty() {
        let t = ObjType::new(vec![]);
        assert_eq!(t.kind(), "obj");
        assert!(t.keys.is_empty());
    }

    #[test]
    fn obj_type_prop() {
        let t = ObjType::new(vec![]).prop("name", super::super::TypeNode::Str(StrType::new()));
        assert_eq!(t.keys.len(), 1);
        assert_eq!(t.keys[0].key, "name");
        assert!(!t.keys[0].optional);
    }

    #[test]
    fn obj_type_opt() {
        let t = ObjType::new(vec![]).opt("age", super::super::TypeNode::Num(NumType::new()));
        assert_eq!(t.keys.len(), 1);
        assert!(t.keys[0].optional);
    }

    #[test]
    fn obj_type_extend() {
        let base = ObjType::new(vec![KeyType::new(
            "id",
            super::super::TypeNode::Num(NumType::new()),
        )]);
        let child = ObjType::new(vec![KeyType::new(
            "name",
            super::super::TypeNode::Str(StrType::new()),
        )])
        .extend(base);
        assert_eq!(child.keys.len(), 2);
    }

    #[test]
    fn obj_type_omit() {
        let t = ObjType::new(vec![
            KeyType::new("a", super::super::TypeNode::Str(StrType::new())),
            KeyType::new("b", super::super::TypeNode::Num(NumType::new())),
        ])
        .omit("a");
        assert_eq!(t.keys.len(), 1);
        assert_eq!(t.keys[0].key, "b");
    }

    #[test]
    fn obj_type_get_field() {
        let t = ObjType::new(vec![KeyType::new(
            "name",
            super::super::TypeNode::Str(StrType::new()),
        )]);
        assert!(t.get_field("name").is_some());
        assert!(t.get_field("missing").is_none());
    }

    #[test]
    fn obj_type_get_schema() {
        let t = ObjType::new(vec![KeyType::new(
            "x",
            super::super::TypeNode::Num(NumType::new()),
        )]);
        let s = t.get_schema();
        if let Schema::Obj(obj) = &s {
            assert_eq!(obj.keys.len(), 1);
            assert_eq!(obj.keys[0].key, "x");
        } else {
            panic!("Expected Obj");
        }
    }

    // -- MapType --

    #[test]
    fn map_type_new() {
        let val = super::super::TypeNode::Num(NumType::new());
        let t = MapType::new(val, None);
        assert_eq!(t.kind(), "map");
        assert!(t.key.is_none());
    }

    #[test]
    fn map_type_with_key() {
        let val = super::super::TypeNode::Num(NumType::new());
        let key = super::super::TypeNode::Str(StrType::new());
        let t = MapType::new(val, Some(key));
        assert!(t.key.is_some());
    }

    #[test]
    fn map_type_get_schema() {
        let val = super::super::TypeNode::Any(AnyType::new());
        let t = MapType::new(val, None);
        let s = t.get_schema();
        if let Schema::Map(map) = &s {
            assert!(map.key.is_none());
        } else {
            panic!("Expected Map");
        }
    }

    // -- RefType --

    #[test]
    fn ref_type_new() {
        let t = RefType::new("MyType");
        assert_eq!(t.kind(), "ref");
        assert_eq!(t.ref_name(), "MyType");
    }

    #[test]
    fn ref_type_get_schema() {
        let t = RefType::new("Foo");
        let s = t.get_schema();
        if let Schema::Ref(r) = &s {
            assert_eq!(r.ref_, "Foo");
        } else {
            panic!("Expected Ref");
        }
    }

    // -- OrType --

    #[test]
    fn or_type_new() {
        let types = vec![
            super::super::TypeNode::Str(StrType::new()),
            super::super::TypeNode::Num(NumType::new()),
        ];
        let t = OrType::new(types);
        assert_eq!(t.kind(), "or");
        assert_eq!(t.types.len(), 2);
    }

    #[test]
    fn or_type_get_schema() {
        let types = vec![super::super::TypeNode::Str(StrType::new())];
        let t = OrType::new(types);
        let s = t.get_schema();
        if let Schema::Or(or) = &s {
            assert_eq!(or.types.len(), 1);
        } else {
            panic!("Expected Or");
        }
    }

    // -- FnType --

    #[test]
    fn fn_type_new() {
        let req = super::super::TypeNode::Str(StrType::new());
        let res = super::super::TypeNode::Num(NumType::new());
        let t = FnType::new(req, res);
        assert_eq!(t.kind(), "fn");
    }

    #[test]
    fn fn_type_get_schema() {
        let req = super::super::TypeNode::Any(AnyType::new());
        let res = super::super::TypeNode::Bool(BoolType::new());
        let t = FnType::new(req, res);
        let s = t.get_schema();
        if let Schema::Fn(f) = &s {
            assert_eq!(f.req.kind(), "any");
            assert_eq!(f.res.kind(), "bool");
        } else {
            panic!("Expected Fn");
        }
    }

    // -- FnRxType --

    #[test]
    fn fn_rx_type_new() {
        let req = super::super::TypeNode::Str(StrType::new());
        let res = super::super::TypeNode::Num(NumType::new());
        let t = FnRxType::new(req, res);
        assert_eq!(t.kind(), "fn$");
    }

    #[test]
    fn fn_rx_type_get_schema() {
        let req = super::super::TypeNode::Any(AnyType::new());
        let res = super::super::TypeNode::Any(AnyType::new());
        let t = FnRxType::new(req, res);
        let s = t.get_schema();
        assert_eq!(s.kind(), "fn$");
    }

    // -- AliasType --

    #[test]
    fn alias_type_new() {
        let system = Arc::new(ModuleType::new());
        let inner = super::super::TypeNode::Str(StrType::new());
        let t = AliasType::new(system, "MyAlias", inner);
        assert_eq!(t.kind(), "alias");
        assert_eq!(t.id, "MyAlias");
    }

    #[test]
    fn alias_type_get_type() {
        let system = Arc::new(ModuleType::new());
        let inner = super::super::TypeNode::Num(NumType::new());
        let t = AliasType::new(system, "NumAlias", inner);
        assert_eq!(t.get_type().kind(), "num");
    }

    #[test]
    fn alias_type_get_schema() {
        let system = Arc::new(ModuleType::new());
        let inner = super::super::TypeNode::Bool(BoolType::new());
        let t = AliasType::new(system, "BoolAlias", inner);
        let s = t.get_schema();
        assert_eq!(s.kind(), "bool");
    }
}
