//! JTD form types.
//!
//! Upstream reference: json-type/src/jtd/types.ts

use std::collections::HashMap;

/// A JSON Type Definition scalar type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JtdType {
    Boolean,
    Float32,
    Float64,
    Int8,
    Uint8,
    Int16,
    Uint16,
    Int32,
    Uint32,
    String,
    Timestamp,
}

impl JtdType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::Float32 => "float32",
            Self::Float64 => "float64",
            Self::Int8 => "int8",
            Self::Uint8 => "uint8",
            Self::Int16 => "int16",
            Self::Uint16 => "uint16",
            Self::Int32 => "int32",
            Self::Uint32 => "uint32",
            Self::String => "string",
            Self::Timestamp => "timestamp",
        }
    }
}

/// A JTD form — one of the eight JTD form kinds.
#[derive(Debug, Clone)]
pub enum JtdForm {
    /// Empty form: `{}` or `{nullable: bool}`
    Empty { nullable: bool },
    /// Ref form: `{ref: "TypeName"}`
    Ref { ref_: String },
    /// Type form: `{type: JtdType}`
    Type { type_: JtdType },
    /// Enum form: `{enum: [...]}`
    Enum { variants: Vec<String> },
    /// Elements form: `{elements: JtdForm}`
    Elements { elements: Box<JtdForm> },
    /// Properties form: `{properties: {...}, optionalProperties: {...}}`
    Properties {
        properties: HashMap<String, JtdForm>,
        optional_properties: HashMap<String, JtdForm>,
        additional_properties: bool,
    },
    /// Values form: `{values: JtdForm}`
    Values { values: Box<JtdForm> },
    /// Discriminator form: `{discriminator: "field", mapping: {...}}`
    Discriminator {
        discriminator: String,
        mapping: HashMap<String, JtdForm>,
    },
}
