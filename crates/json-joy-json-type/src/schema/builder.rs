//! Schema builder — port of SchemaBuilder.ts.
//!
//! Provides a fluent API for constructing schema values.

use serde_json::Value;

use super::schema::*;

/// Builder for constructing schema values.
///
/// Upstream reference: json-type/src/schema/SchemaBuilder.ts
#[derive(Debug, Clone, Default)]
pub struct SchemaBuilder;

#[allow(non_snake_case)]
impl SchemaBuilder {
    pub fn new() -> Self {
        Self
    }

    // ------------------------------------------------------------------
    // Shorthand property accessors (no options)

    pub fn str(&self) -> Schema {
        self.String(None)
    }

    pub fn num(&self) -> Schema {
        self.Number(None)
    }

    pub fn bool(&self) -> Schema {
        self.Boolean(None)
    }

    pub fn any(&self) -> Schema {
        self.Any(None)
    }

    pub fn arr(&self) -> Schema {
        self.Array(self.any(), None)
    }

    pub fn obj(&self) -> Schema {
        self.Object(vec![], None)
    }

    pub fn map(&self) -> Schema {
        self.Map(self.any(), None, None)
    }

    pub fn bin(&self) -> Schema {
        self.Binary(self.any(), None, None)
    }

    pub fn fn_(&self) -> Schema {
        self.Function(self.any(), self.any(), None)
    }

    pub fn fn_rx(&self) -> Schema {
        self.function_streaming(self.any(), self.any(), None)
    }

    pub fn undef(&self) -> Schema {
        self.Const(Value::Null, None)
    }

    pub fn nil(&self) -> Schema {
        self.Const(Value::Null, None)
    }

    // ------------------------------------------------------------------
    // Named constructors

    pub fn Boolean(&self, base: Option<SchemaBase>) -> Schema {
        Schema::Bool(BoolSchema {
            base: base.unwrap_or_default(),
        })
    }

    pub fn Number(&self, base: Option<SchemaBase>) -> Schema {
        Schema::Num(NumSchema {
            base: base.unwrap_or_default(),
            ..Default::default()
        })
    }

    pub fn String(&self, base: Option<SchemaBase>) -> Schema {
        Schema::Str(StrSchema {
            base: base.unwrap_or_default(),
            ..Default::default()
        })
    }

    pub fn Any(&self, base: Option<SchemaBase>) -> Schema {
        Schema::Any(AnySchema {
            base: base.unwrap_or_default(),
        })
    }

    pub fn Const(&self, value: Value, base: Option<SchemaBase>) -> Schema {
        Schema::Con(ConSchema {
            base: base.unwrap_or_default(),
            value,
        })
    }

    pub fn Binary(
        &self,
        type_: Schema,
        format: Option<BinFormat>,
        base: Option<SchemaBase>,
    ) -> Schema {
        Schema::Bin(BinSchema {
            base: base.unwrap_or_default(),
            type_: Box::new(type_),
            format,
            min: None,
            max: None,
        })
    }

    pub fn Array(&self, type_: Schema, base: Option<SchemaBase>) -> Schema {
        Schema::Arr(ArrSchema {
            base: base.unwrap_or_default(),
            type_: Some(Box::new(type_)),
            ..Default::default()
        })
    }

    pub fn Tuple(
        &self,
        head: Vec<Schema>,
        type_: Option<Schema>,
        tail: Option<Vec<Schema>>,
    ) -> Schema {
        Schema::Arr(ArrSchema {
            base: SchemaBase::default(),
            type_: type_.map(Box::new),
            head: Some(head),
            tail,
            ..Default::default()
        })
    }

    pub fn Object(&self, keys: Vec<KeySchema>, base: Option<SchemaBase>) -> Schema {
        Schema::Obj(ObjSchema {
            base: base.unwrap_or_default(),
            keys,
            ..Default::default()
        })
    }

    pub fn Key(&self, key: impl Into<String>, value: Schema) -> KeySchema {
        KeySchema {
            base: SchemaBase::default(),
            key: key.into(),
            value: Box::new(value),
            optional: None,
        }
    }

    pub fn KeyOpt(&self, key: impl Into<String>, value: Schema) -> KeySchema {
        KeySchema {
            base: SchemaBase::default(),
            key: key.into(),
            value: Box::new(value),
            optional: Some(true),
        }
    }

    pub fn Map(&self, value: Schema, key: Option<Schema>, base: Option<SchemaBase>) -> Schema {
        Schema::Map(MapSchema {
            base: base.unwrap_or_default(),
            key: key.map(Box::new),
            value: Box::new(value),
        })
    }

    pub fn Ref(&self, ref_: impl Into<String>) -> Schema {
        Schema::Ref(RefSchema {
            base: SchemaBase::default(),
            ref_: ref_.into(),
        })
    }

    pub fn Or(&self, types: Vec<Schema>) -> Schema {
        Schema::Or(OrSchema {
            base: SchemaBase::default(),
            types,
            discriminator: serde_json::json!(["num", -1]),
        })
    }

    pub fn Function(&self, req: Schema, res: Schema, base: Option<SchemaBase>) -> Schema {
        Schema::Fn(FnSchema {
            base: base.unwrap_or_default(),
            req: Box::new(req),
            res: Box::new(res),
        })
    }

    /// Streaming function (`fn$` in upstream TypeScript).
    pub fn function_streaming(&self, req: Schema, res: Schema, base: Option<SchemaBase>) -> Schema {
        Schema::FnRx(FnRxSchema {
            base: base.unwrap_or_default(),
            req: Box::new(req),
            res: Box::new(res),
        })
    }
}

/// Global default schema builder.
pub static S: SchemaBuilder = SchemaBuilder;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn s() -> SchemaBuilder {
        SchemaBuilder::new()
    }

    #[test]
    fn new_creates_default() {
        let b = SchemaBuilder::new();
        let _ = format!("{:?}", b);
    }

    #[test]
    fn str_returns_str_schema() {
        assert_eq!(s().str().kind(), "str");
    }

    #[test]
    fn num_returns_num_schema() {
        assert_eq!(s().num().kind(), "num");
    }

    #[test]
    fn bool_returns_bool_schema() {
        assert_eq!(s().bool().kind(), "bool");
    }

    #[test]
    fn any_returns_any_schema() {
        assert_eq!(s().any().kind(), "any");
    }

    #[test]
    fn arr_returns_arr_schema() {
        assert_eq!(s().arr().kind(), "arr");
    }

    #[test]
    fn obj_returns_obj_schema() {
        assert_eq!(s().obj().kind(), "obj");
    }

    #[test]
    fn map_returns_map_schema() {
        assert_eq!(s().map().kind(), "map");
    }

    #[test]
    fn bin_returns_bin_schema() {
        assert_eq!(s().bin().kind(), "bin");
    }

    #[test]
    fn fn_returns_fn_schema() {
        assert_eq!(s().fn_().kind(), "fn");
    }

    #[test]
    fn fn_rx_returns_fn_rx_schema() {
        assert_eq!(s().fn_rx().kind(), "fn$");
    }

    #[test]
    fn undef_returns_con_null() {
        let schema = s().undef();
        assert_eq!(schema.kind(), "con");
        if let Schema::Con(con) = &schema {
            assert_eq!(con.value, json!(null));
        } else {
            panic!("Expected Con");
        }
    }

    #[test]
    fn nil_returns_con_null() {
        let schema = s().nil();
        if let Schema::Con(con) = &schema {
            assert_eq!(con.value, json!(null));
        } else {
            panic!("Expected Con");
        }
    }

    #[test]
    fn boolean_with_base() {
        let base = SchemaBase {
            title: Some("My Bool".into()),
            ..Default::default()
        };
        let schema = s().Boolean(Some(base));
        assert_eq!(schema.base().title.as_deref(), Some("My Bool"));
    }

    #[test]
    fn number_with_base() {
        let base = SchemaBase {
            title: Some("Num".into()),
            ..Default::default()
        };
        let schema = s().Number(Some(base));
        assert_eq!(schema.base().title.as_deref(), Some("Num"));
    }

    #[test]
    fn string_with_base() {
        let schema = s().String(None);
        assert_eq!(schema.kind(), "str");
    }

    #[test]
    fn any_with_base() {
        let schema = s().Any(None);
        assert_eq!(schema.kind(), "any");
    }

    #[test]
    fn const_with_value() {
        let schema = s().Const(json!("hello"), None);
        if let Schema::Con(con) = &schema {
            assert_eq!(con.value, json!("hello"));
        } else {
            panic!("Expected Con");
        }
    }

    #[test]
    fn binary_with_format() {
        let schema = s().Binary(s().str(), Some(BinFormat::Json), None);
        if let Schema::Bin(bin) = &schema {
            assert_eq!(bin.format, Some(BinFormat::Json));
        } else {
            panic!("Expected Bin");
        }
    }

    #[test]
    fn array_wraps_type() {
        let schema = s().Array(s().num(), None);
        if let Schema::Arr(arr) = &schema {
            assert!(arr.type_.is_some());
            assert_eq!(arr.type_.as_ref().unwrap().kind(), "num");
        } else {
            panic!("Expected Arr");
        }
    }

    #[test]
    fn tuple_constructs_head_and_tail() {
        let schema = s().Tuple(vec![s().str()], Some(s().num()), Some(vec![s().bool()]));
        if let Schema::Arr(arr) = &schema {
            assert_eq!(arr.head.as_ref().unwrap().len(), 1);
            assert!(arr.type_.is_some());
            assert_eq!(arr.tail.as_ref().unwrap().len(), 1);
        } else {
            panic!("Expected Arr");
        }
    }

    #[test]
    fn object_with_keys() {
        let schema = s().Object(vec![s().Key("name", s().str())], None);
        if let Schema::Obj(obj) = &schema {
            assert_eq!(obj.keys.len(), 1);
            assert_eq!(obj.keys[0].key, "name");
        } else {
            panic!("Expected Obj");
        }
    }

    #[test]
    fn key_creates_required() {
        let k = s().Key("age", s().num());
        assert_eq!(k.key, "age");
        assert!(k.optional.is_none());
    }

    #[test]
    fn key_opt_creates_optional() {
        let k = s().KeyOpt("nickname", s().str());
        assert_eq!(k.key, "nickname");
        assert_eq!(k.optional, Some(true));
    }

    #[test]
    fn map_with_key_schema() {
        let schema = s().Map(s().num(), Some(s().str()), None);
        if let Schema::Map(map) = &schema {
            assert!(map.key.is_some());
            assert_eq!(map.key.as_ref().unwrap().kind(), "str");
            assert_eq!(map.value.kind(), "num");
        } else {
            panic!("Expected Map");
        }
    }

    #[test]
    fn ref_creates_ref_schema() {
        let schema = s().Ref("MyType");
        if let Schema::Ref(r) = &schema {
            assert_eq!(r.ref_, "MyType");
        } else {
            panic!("Expected Ref");
        }
    }

    #[test]
    fn or_creates_union() {
        let schema = s().Or(vec![s().str(), s().num()]);
        if let Schema::Or(or) = &schema {
            assert_eq!(or.types.len(), 2);
        } else {
            panic!("Expected Or");
        }
    }

    #[test]
    fn function_creates_fn_schema() {
        let schema = s().Function(s().str(), s().num(), None);
        if let Schema::Fn(f) = &schema {
            assert_eq!(f.req.kind(), "str");
            assert_eq!(f.res.kind(), "num");
        } else {
            panic!("Expected Fn");
        }
    }

    #[test]
    fn function_streaming_creates_fn_rx_schema() {
        let schema = s().function_streaming(s().str(), s().num(), None);
        if let Schema::FnRx(f) = &schema {
            assert_eq!(f.req.kind(), "str");
            assert_eq!(f.res.kind(), "num");
        } else {
            panic!("Expected FnRx");
        }
    }

    #[test]
    fn global_static_s_works() {
        assert_eq!(S.str().kind(), "str");
        assert_eq!(S.num().kind(), "num");
    }
}
