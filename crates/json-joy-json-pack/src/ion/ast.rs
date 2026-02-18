//! Amazon Ion AST node types for binary encoding.
//!
//! Upstream reference: `json-pack/src/ion/ast.ts`

use crate::PackValue;

/// Number of bytes needed to encode `n` as an Ion VUint.
pub fn vuint_len(n: usize) -> usize {
    if n <= 0x7f { 1 }
    else if n <= 0x3fff { 2 }
    else if n <= 0x1f_ffff { 3 }
    else if n <= 0x0fff_ffff { 4 }
    else if n <= 0x07_ffff_ffff { 5 }
    else { 6 }
}

/// An Ion AST node (value ready for binary encoding).
#[derive(Debug, Clone)]
pub enum IonAstNode {
    Null,
    Bool(bool),
    UInt(u64),
    /// Negative integer stored as its magnitude.
    NInt(u64),
    Float(f64),
    Str(String),
    Bin(Vec<u8>),
    Array(Vec<IonAstNode>),
    /// Struct: ordered list of (symbol_id, value) pairs.
    Struct(Vec<(usize, IonAstNode)>),
    Annotation {
        node: Box<IonAstNode>,
        annotations: Vec<usize>,
    },
}

impl IonAstNode {
    /// Byte length of the value body (excluding type descriptor and length prefix).
    pub fn content_len(&self) -> usize {
        match self {
            IonAstNode::Null => 0,
            IonAstNode::Bool(_) => 0, // encoded in the length nibble
            IonAstNode::UInt(n) => uint_byte_len(*n),
            IonAstNode::NInt(n) => uint_byte_len(*n),
            IonAstNode::Float(_) => 8,
            IonAstNode::Str(s) => s.as_bytes().len(),
            IonAstNode::Bin(b) => b.len(),
            IonAstNode::Array(arr) => arr.iter().map(|n| n.byte_length()).sum(),
            IonAstNode::Struct(fields) => {
                fields.iter().map(|(sid, n)| vuint_len(*sid) + n.byte_length()).sum()
            }
            IonAstNode::Annotation { node, annotations } => {
                let annot_payload: usize = annotations.iter().map(|a| vuint_len(*a)).sum();
                vuint_len(annot_payload) + annot_payload + node.byte_length()
            }
        }
    }

    /// Total byte length including type descriptor (and VUint length prefix if needed).
    pub fn byte_length(&self) -> usize {
        match self {
            // Null: type | 0xf — 1 byte total
            IonAstNode::Null => 1,
            // Bool: type | 0 or 1 — 1 byte total
            IonAstNode::Bool(_) => 1,
            // UInt(0): type | 0 — 1 byte, no body
            IonAstNode::UInt(0) => 1,
            _ => {
                let len = self.content_len();
                if len < 14 { 1 + len } else { 1 + vuint_len(len) + len }
            }
        }
    }
}

fn uint_byte_len(n: u64) -> usize {
    if n == 0 { 0 }
    else if n <= 0xff { 1 }
    else if n <= 0xffff { 2 }
    else if n <= 0xff_ffff { 3 }
    else if n <= 0xffff_ffff { 4 }
    else if n <= 0xff_ffff_ffff { 5 }
    else if n <= 0xffff_ffff_ffff { 6 }
    else { 7 }
}

/// Convert a `PackValue` to an Ion AST node, recording any object keys in the symbol table.
pub fn to_ast(value: &PackValue, symbols: &mut super::encoder::SymbolTracker) -> IonAstNode {
    match value {
        PackValue::Null | PackValue::Undefined => IonAstNode::Null,
        PackValue::Bool(b) => IonAstNode::Bool(*b),
        PackValue::Integer(i) => {
            if *i >= 0 { IonAstNode::UInt(*i as u64) } else { IonAstNode::NInt((-i) as u64) }
        }
        PackValue::UInteger(u) => IonAstNode::UInt(*u),
        PackValue::Float(f) => IonAstNode::Float(*f),
        PackValue::BigInt(n) => {
            if *n >= 0 { IonAstNode::UInt(*n as u64) } else { IonAstNode::NInt((-n) as u64) }
        }
        PackValue::Str(s) => IonAstNode::Str(s.clone()),
        PackValue::Bytes(b) => IonAstNode::Bin(b.clone()),
        PackValue::Array(arr) => {
            IonAstNode::Array(arr.iter().map(|v| to_ast(v, symbols)).collect())
        }
        PackValue::Object(obj) => {
            let fields = obj
                .iter()
                .map(|(k, v)| {
                    let sid = symbols.add(k);
                    (sid, to_ast(v, symbols))
                })
                .collect();
            IonAstNode::Struct(fields)
        }
        _ => IonAstNode::Null,
    }
}
