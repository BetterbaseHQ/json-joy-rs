//! XDR type definitions.
//!
//! Upstream reference: `json-pack/src/xdr/types.ts`
//! Reference: RFC 4506

/// XDR schema — describes the type of a value.
#[derive(Debug, Clone)]
pub enum XdrSchema {
    Void,
    Int,
    UnsignedInt,
    Hyper,
    UnsignedHyper,
    Float,
    Double,
    Quadruple,
    Boolean,
    Enum(Vec<(String, i32)>),
    Opaque(u32),
    VarOpaque(Option<u32>),
    Str(Option<u32>),
    Array {
        element: Box<XdrSchema>,
        size: u32,
    },
    VarArray {
        element: Box<XdrSchema>,
        max_size: Option<u32>,
    },
    Struct(Vec<(Box<XdrSchema>, String)>),
    Union {
        arms: Vec<(XdrDiscriminant, Box<XdrSchema>)>,
        default: Option<Box<XdrSchema>>,
    },
    Optional(Box<XdrSchema>),
    Const(i32),
}

/// Discriminant value for XDR union arms.
#[derive(Debug, Clone, PartialEq)]
pub enum XdrDiscriminant {
    Int(i32),
    Bool(bool),
    Str(String),
}

/// XDR value — runtime value corresponding to a schema.
#[derive(Debug, Clone, PartialEq)]
pub enum XdrValue {
    Void,
    Int(i32),
    UnsignedInt(u32),
    Hyper(i64),
    UnsignedHyper(u64),
    Float(f32),
    Double(f64),
    Bool(bool),
    Bytes(Vec<u8>),
    Str(String),
    Array(Vec<XdrValue>),
    Struct(Vec<(String, XdrValue)>),
    Union(Box<XdrUnionValue>),
    Optional(Option<Box<XdrValue>>),
    Enum(String),
}

/// An XDR union value with discriminant and payload.
#[derive(Debug, Clone, PartialEq)]
pub struct XdrUnionValue {
    pub discriminant: XdrDiscriminant,
    pub value: XdrValue,
}
