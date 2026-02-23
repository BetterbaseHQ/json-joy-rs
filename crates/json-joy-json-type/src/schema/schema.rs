use serde_json::Value;

/// Number format specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumFormat {
    I,
    U,
    F,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
}

impl NumFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::I => "i",
            Self::U => "u",
            Self::F => "f",
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::F32 => "f32",
            Self::F64 => "f64",
        }
    }

    pub fn is_integer(self) -> bool {
        matches!(
            self,
            Self::I
                | Self::I8
                | Self::I16
                | Self::I32
                | Self::I64
                | Self::U
                | Self::U8
                | Self::U16
                | Self::U32
                | Self::U64
        )
    }

    pub fn is_unsigned(self) -> bool {
        matches!(self, Self::U | Self::U8 | Self::U16 | Self::U32 | Self::U64)
    }

    pub fn is_float(self) -> bool {
        matches!(self, Self::F | Self::F32 | Self::F64)
    }
}

/// String format specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrFormat {
    Ascii,
    Utf8,
}

impl StrFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ascii => "ascii",
            Self::Utf8 => "utf8",
        }
    }
}

/// Binary encoding format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinFormat {
    Json,
    Cbor,
    Msgpack,
    Resp3,
    Ion,
    Bson,
    Ubjson,
    Bencode,
}

impl BinFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Cbor => "cbor",
            Self::Msgpack => "msgpack",
            Self::Resp3 => "resp3",
            Self::Ion => "ion",
            Self::Bson => "bson",
            Self::Ubjson => "ubjson",
            Self::Bencode => "bencode",
        }
    }
}

/// Example of how a value of a given type could look.
#[derive(Debug, Clone)]
pub struct SchemaExample {
    pub value: Value,
    pub title: Option<String>,
    pub intro: Option<String>,
    pub description: Option<String>,
}

/// Deprecation information.
#[derive(Debug, Clone)]
pub struct Deprecated {
    pub info: Option<String>,
}

/// Fields common to all schema nodes.
#[derive(Debug, Clone, Default)]
pub struct SchemaBase {
    pub title: Option<String>,
    pub intro: Option<String>,
    pub description: Option<String>,
    pub meta: Option<Value>,
    pub default: Option<Value>,
    pub examples: Vec<SchemaExample>,
    pub deprecated: Option<Deprecated>,
}

/// Represents any value (unknown type).
#[derive(Debug, Clone, Default)]
pub struct AnySchema {
    pub base: SchemaBase,
}

/// Represents a constant value.
#[derive(Debug, Clone)]
pub struct ConSchema {
    pub base: SchemaBase,
    pub value: Value,
}

/// Represents a JSON boolean.
#[derive(Debug, Clone, Default)]
pub struct BoolSchema {
    pub base: SchemaBase,
}

/// Represents a JSON number with optional format and range constraints.
#[derive(Debug, Clone, Default)]
pub struct NumSchema {
    pub base: SchemaBase,
    pub format: Option<NumFormat>,
    pub gt: Option<f64>,
    pub gte: Option<f64>,
    pub lt: Option<f64>,
    pub lte: Option<f64>,
}

/// Represents a JSON string.
#[derive(Debug, Clone, Default)]
pub struct StrSchema {
    pub base: SchemaBase,
    pub format: Option<StrFormat>,
    pub ascii: Option<bool>,
    pub no_json_escape: Option<bool>,
    pub min: Option<u64>,
    pub max: Option<u64>,
}

/// Represents binary data (encoded value).
#[derive(Debug, Clone)]
pub struct BinSchema {
    pub base: SchemaBase,
    /// Type of value encoded in the binary data.
    pub type_: Box<Schema>,
    pub format: Option<BinFormat>,
    pub min: Option<u64>,
    pub max: Option<u64>,
}

/// Represents a JSON array.
#[derive(Debug, Clone, Default)]
pub struct ArrSchema {
    pub base: SchemaBase,
    /// Element type for homogeneous arrays.
    pub type_: Option<Box<Schema>>,
    /// Head tuple types (fixed prefix elements).
    pub head: Option<Vec<Schema>>,
    /// Tail tuple types (fixed suffix elements).
    pub tail: Option<Vec<Schema>>,
    pub min: Option<u64>,
    pub max: Option<u64>,
}

/// Represents a single field of an object.
#[derive(Debug, Clone)]
pub struct KeySchema {
    pub base: SchemaBase,
    pub key: String,
    pub value: Box<Schema>,
    pub optional: Option<bool>,
}

/// Represents a JSON object with defined keys.
#[derive(Debug, Clone, Default)]
pub struct ObjSchema {
    pub base: SchemaBase,
    pub keys: Vec<KeySchema>,
    pub extends: Option<Vec<String>>,
    pub decode_unknown_keys: Option<bool>,
    pub encode_unknown_keys: Option<bool>,
}

/// Represents an object treated as a map (all values same type).
#[derive(Debug, Clone)]
pub struct MapSchema {
    pub base: SchemaBase,
    pub key: Option<Box<Schema>>,
    pub value: Box<Schema>,
}

/// Reference to another named type.
#[derive(Debug, Clone)]
pub struct RefSchema {
    pub base: SchemaBase,
    pub ref_: String,
}

/// Union of multiple types.
#[derive(Debug, Clone)]
pub struct OrSchema {
    pub base: SchemaBase,
    pub types: Vec<Schema>,
    pub discriminator: Value,
}

/// RPC function type (request/response).
#[derive(Debug, Clone)]
pub struct FnSchema {
    pub base: SchemaBase,
    pub req: Box<Schema>,
    pub res: Box<Schema>,
}

/// Streaming RPC function type (Observable request/response).
#[derive(Debug, Clone)]
pub struct FnRxSchema {
    pub base: SchemaBase,
    pub req: Box<Schema>,
    pub res: Box<Schema>,
}

/// Named alias in a module.
#[derive(Debug, Clone)]
pub struct AliasSchema {
    pub base: SchemaBase,
    pub key: String,
    pub value: Box<Schema>,
    pub optional: Option<bool>,
    pub pub_: Option<bool>,
}

/// Module containing named type aliases.
#[derive(Debug, Clone, Default)]
pub struct ModuleSchema {
    pub base: SchemaBase,
    pub keys: Vec<AliasSchema>,
}

/// The unified Schema enum covering all schema kinds.
#[derive(Debug, Clone)]
pub enum Schema {
    Any(AnySchema),
    Bool(BoolSchema),
    Num(NumSchema),
    Str(StrSchema),
    Bin(BinSchema),
    Con(ConSchema),
    Arr(ArrSchema),
    Obj(ObjSchema),
    Key(KeySchema),
    Map(MapSchema),
    Ref(RefSchema),
    Or(OrSchema),
    Fn(FnSchema),
    FnRx(FnRxSchema),
    Alias(AliasSchema),
    Module(ModuleSchema),
}

impl Schema {
    /// Returns the "kind" string identifier for this schema node.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Any(_) => "any",
            Self::Bool(_) => "bool",
            Self::Num(_) => "num",
            Self::Str(_) => "str",
            Self::Bin(_) => "bin",
            Self::Con(_) => "con",
            Self::Arr(_) => "arr",
            Self::Obj(_) => "obj",
            Self::Key(_) => "key",
            Self::Map(_) => "map",
            Self::Ref(_) => "ref",
            Self::Or(_) => "or",
            Self::Fn(_) => "fn",
            Self::FnRx(_) => "fn$",
            Self::Alias(_) => "key",
            Self::Module(_) => "module",
        }
    }

    /// Returns the base schema fields.
    pub fn base(&self) -> &SchemaBase {
        match self {
            Self::Any(s) => &s.base,
            Self::Bool(s) => &s.base,
            Self::Num(s) => &s.base,
            Self::Str(s) => &s.base,
            Self::Bin(s) => &s.base,
            Self::Con(s) => &s.base,
            Self::Arr(s) => &s.base,
            Self::Obj(s) => &s.base,
            Self::Key(s) => &s.base,
            Self::Map(s) => &s.base,
            Self::Ref(s) => &s.base,
            Self::Or(s) => &s.base,
            Self::Fn(s) => &s.base,
            Self::FnRx(s) => &s.base,
            Self::Alias(s) => &s.base,
            Self::Module(s) => &s.base,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn schema_kind_returns_correct_strings() {
        assert_eq!(Schema::Any(AnySchema::default()).kind(), "any");
        assert_eq!(Schema::Bool(BoolSchema::default()).kind(), "bool");
        assert_eq!(Schema::Num(NumSchema::default()).kind(), "num");
        assert_eq!(Schema::Str(StrSchema::default()).kind(), "str");
        assert_eq!(
            Schema::Con(ConSchema {
                base: SchemaBase::default(),
                value: json!(42),
            })
            .kind(),
            "con"
        );
        assert_eq!(Schema::Arr(ArrSchema::default()).kind(), "arr");
        assert_eq!(Schema::Obj(ObjSchema::default()).kind(), "obj");
        assert_eq!(
            Schema::Key(KeySchema {
                base: SchemaBase::default(),
                key: "k".into(),
                value: Box::new(Schema::Any(AnySchema::default())),
                optional: None,
            })
            .kind(),
            "key"
        );
        assert_eq!(
            Schema::Map(MapSchema {
                base: SchemaBase::default(),
                key: None,
                value: Box::new(Schema::Any(AnySchema::default())),
            })
            .kind(),
            "map"
        );
        assert_eq!(
            Schema::Ref(RefSchema {
                base: SchemaBase::default(),
                ref_: "Foo".into(),
            })
            .kind(),
            "ref"
        );
        assert_eq!(
            Schema::Or(OrSchema {
                base: SchemaBase::default(),
                types: vec![],
                discriminator: json!(null),
            })
            .kind(),
            "or"
        );
        assert_eq!(
            Schema::Fn(FnSchema {
                base: SchemaBase::default(),
                req: Box::new(Schema::Any(AnySchema::default())),
                res: Box::new(Schema::Any(AnySchema::default())),
            })
            .kind(),
            "fn"
        );
        assert_eq!(
            Schema::FnRx(FnRxSchema {
                base: SchemaBase::default(),
                req: Box::new(Schema::Any(AnySchema::default())),
                res: Box::new(Schema::Any(AnySchema::default())),
            })
            .kind(),
            "fn$"
        );
        assert_eq!(
            Schema::Alias(AliasSchema {
                base: SchemaBase::default(),
                key: "A".into(),
                value: Box::new(Schema::Any(AnySchema::default())),
                optional: None,
                pub_: None,
            })
            .kind(),
            "key"
        );
        assert_eq!(Schema::Module(ModuleSchema::default()).kind(), "module");
    }

    #[test]
    fn schema_base_returns_base_for_all_variants() {
        let base = SchemaBase {
            title: Some("test".into()),
            ..Default::default()
        };
        let s = Schema::Any(AnySchema { base: base.clone() });
        assert_eq!(s.base().title.as_deref(), Some("test"));

        let s = Schema::Bool(BoolSchema { base: base.clone() });
        assert_eq!(s.base().title.as_deref(), Some("test"));

        let s = Schema::Num(NumSchema {
            base: base.clone(),
            ..Default::default()
        });
        assert_eq!(s.base().title.as_deref(), Some("test"));

        let s = Schema::Str(StrSchema {
            base: base.clone(),
            ..Default::default()
        });
        assert_eq!(s.base().title.as_deref(), Some("test"));
    }

    #[test]
    fn num_format_as_str() {
        assert_eq!(NumFormat::I.as_str(), "i");
        assert_eq!(NumFormat::U.as_str(), "u");
        assert_eq!(NumFormat::F.as_str(), "f");
        assert_eq!(NumFormat::I8.as_str(), "i8");
        assert_eq!(NumFormat::I16.as_str(), "i16");
        assert_eq!(NumFormat::I32.as_str(), "i32");
        assert_eq!(NumFormat::I64.as_str(), "i64");
        assert_eq!(NumFormat::U8.as_str(), "u8");
        assert_eq!(NumFormat::U16.as_str(), "u16");
        assert_eq!(NumFormat::U32.as_str(), "u32");
        assert_eq!(NumFormat::U64.as_str(), "u64");
        assert_eq!(NumFormat::F32.as_str(), "f32");
        assert_eq!(NumFormat::F64.as_str(), "f64");
    }

    #[test]
    fn num_format_is_integer() {
        assert!(NumFormat::I.is_integer());
        assert!(NumFormat::I8.is_integer());
        assert!(NumFormat::U.is_integer());
        assert!(NumFormat::U64.is_integer());
        assert!(!NumFormat::F.is_integer());
        assert!(!NumFormat::F32.is_integer());
        assert!(!NumFormat::F64.is_integer());
    }

    #[test]
    fn num_format_is_unsigned() {
        assert!(NumFormat::U.is_unsigned());
        assert!(NumFormat::U8.is_unsigned());
        assert!(NumFormat::U16.is_unsigned());
        assert!(NumFormat::U32.is_unsigned());
        assert!(NumFormat::U64.is_unsigned());
        assert!(!NumFormat::I.is_unsigned());
        assert!(!NumFormat::F.is_unsigned());
    }

    #[test]
    fn num_format_is_float() {
        assert!(NumFormat::F.is_float());
        assert!(NumFormat::F32.is_float());
        assert!(NumFormat::F64.is_float());
        assert!(!NumFormat::I.is_float());
        assert!(!NumFormat::U.is_float());
    }

    #[test]
    fn str_format_as_str() {
        assert_eq!(StrFormat::Ascii.as_str(), "ascii");
        assert_eq!(StrFormat::Utf8.as_str(), "utf8");
    }

    #[test]
    fn bin_format_as_str() {
        assert_eq!(BinFormat::Json.as_str(), "json");
        assert_eq!(BinFormat::Cbor.as_str(), "cbor");
        assert_eq!(BinFormat::Msgpack.as_str(), "msgpack");
        assert_eq!(BinFormat::Resp3.as_str(), "resp3");
        assert_eq!(BinFormat::Ion.as_str(), "ion");
        assert_eq!(BinFormat::Bson.as_str(), "bson");
        assert_eq!(BinFormat::Ubjson.as_str(), "ubjson");
        assert_eq!(BinFormat::Bencode.as_str(), "bencode");
    }

    #[test]
    fn schema_base_default_all_none() {
        let base = SchemaBase::default();
        assert!(base.title.is_none());
        assert!(base.intro.is_none());
        assert!(base.description.is_none());
        assert!(base.meta.is_none());
        assert!(base.default.is_none());
        assert!(base.examples.is_empty());
        assert!(base.deprecated.is_none());
    }

    #[test]
    fn schema_example_holds_value() {
        let ex = SchemaExample {
            value: json!({"hello": "world"}),
            title: Some("Example".into()),
            intro: None,
            description: Some("A simple object".into()),
        };
        assert_eq!(ex.value, json!({"hello": "world"}));
        assert_eq!(ex.title.as_deref(), Some("Example"));
        assert!(ex.intro.is_none());
    }

    #[test]
    fn deprecated_holds_info() {
        let d = Deprecated {
            info: Some("Use v2 instead".into()),
        };
        assert_eq!(d.info.as_deref(), Some("Use v2 instead"));
        let d2 = Deprecated { info: None };
        assert!(d2.info.is_none());
    }

    #[test]
    fn bin_schema_construction() {
        let s = Schema::Bin(BinSchema {
            base: SchemaBase::default(),
            type_: Box::new(Schema::Any(AnySchema::default())),
            format: Some(BinFormat::Json),
            min: Some(0),
            max: Some(1024),
        });
        assert_eq!(s.kind(), "bin");
        if let Schema::Bin(bin) = &s {
            assert_eq!(bin.format, Some(BinFormat::Json));
            assert_eq!(bin.min, Some(0));
            assert_eq!(bin.max, Some(1024));
        }
    }
}
