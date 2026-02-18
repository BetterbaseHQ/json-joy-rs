//! TypeScript AST node types.
//!
//! Upstream reference: json-type/src/typescript/types.ts

/// A TypeScript type AST node.
#[derive(Debug, Clone)]
pub enum TsType {
    /// `any`
    Any,
    /// `boolean`
    Boolean,
    /// `number`
    Number,
    /// `string`
    String,
    /// `null`
    Null,
    /// `object`
    Object,
    /// `unknown`
    Unknown,
    /// `true`
    True,
    /// `false`
    False,
    /// string literal, e.g. `"hello"`
    StringLiteral(String),
    /// numeric literal, e.g. `42`
    NumericLiteral(String),
    /// Array type `T[]` or `Array<T>`
    Array(Box<TsType>),
    /// Tuple type `[A, B, ...C[], D]`
    Tuple(Vec<TsType>),
    /// Rest type `...T` (used inside tuples)
    Rest(Box<TsType>),
    /// Object type literal `{ key: T; ... }`
    TypeLiteral {
        members: Vec<TsMember>,
        comment: Option<String>,
    },
    /// Union type `A | B`
    Union(Vec<TsType>),
    /// Generic type reference `Name<T, U>`
    TypeReference {
        name: String,
        type_args: Vec<TsType>,
    },
    /// Function type `(param: T) => R`
    FnType {
        params: Vec<TsParam>,
        return_type: Box<TsType>,
    },
}

/// A property or index signature inside a type literal.
#[derive(Debug, Clone)]
pub enum TsMember {
    /// `key?: T;`
    Property {
        name: String,
        type_: TsType,
        optional: bool,
        comment: Option<String>,
    },
    /// `[key: string]: T;`
    Index { type_: TsType },
}

/// A function parameter.
#[derive(Debug, Clone)]
pub struct TsParam {
    pub name: String,
    pub type_: TsType,
}

/// A top-level TypeScript declaration.
#[derive(Debug, Clone)]
pub enum TsDeclaration {
    /// `export interface Name { ... }`
    Interface {
        name: String,
        members: Vec<TsMember>,
        comment: Option<String>,
    },
    /// `export type Name = T`
    TypeAlias {
        name: String,
        type_: TsType,
        comment: Option<String>,
    },
    /// `export namespace Name { ... }`
    Module {
        name: String,
        statements: Vec<TsDeclaration>,
    },
}
