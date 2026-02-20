//! JSONPath types and interfaces based on RFC 9535.

use serde_json::Value;
use std::fmt;

/// Selector types for JSONPath.
#[derive(Debug, Clone, PartialEq)]
pub enum Selector {
    /// Named selector for property access: `.name`, `['key']`
    Name(String),
    /// Index selector for array element access: `[0]`, `[-1]`
    Index(isize),
    /// Slice selector for array slicing: `[start:end:step]`
    Slice {
        start: Option<isize>,
        end: Option<isize>,
        step: Option<isize>,
    },
    /// Wildcard selector for selecting all elements: `.*`, `[*]`
    Wildcard,
    /// Filter expression for conditional selection: `[?(@.price < 10)]`
    Filter(FilterExpression),
}

/// Normalized path segment, mirrors upstream `(string | number)[]`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NormalizedPathSegment {
    Key(String),
    Index(i64),
}

impl From<&str> for NormalizedPathSegment {
    fn from(value: &str) -> Self {
        Self::Key(value.to_string())
    }
}

impl From<String> for NormalizedPathSegment {
    fn from(value: String) -> Self {
        Self::Key(value)
    }
}

impl From<usize> for NormalizedPathSegment {
    fn from(value: usize) -> Self {
        Self::Index(value as i64)
    }
}

impl From<i64> for NormalizedPathSegment {
    fn from(value: i64) -> Self {
        Self::Index(value)
    }
}

impl From<i32> for NormalizedPathSegment {
    fn from(value: i32) -> Self {
        Self::Index(value as i64)
    }
}

impl fmt::Display for NormalizedPathSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(k) => f.write_str(k),
            Self::Index(i) => write!(f, "{i}"),
        }
    }
}

/// Normalized JSONPath result path.
pub type NormalizedPath = Vec<NormalizedPathSegment>;

/// Path segment containing one or more selectors.
#[derive(Debug, Clone, PartialEq)]
pub struct PathSegment {
    /// Selectors in this segment.
    pub selectors: Vec<Selector>,
    /// Whether this is a recursive descent segment (`..`).
    pub recursive: bool,
}

impl PathSegment {
    pub fn new(selectors: Vec<Selector>, recursive: bool) -> Self {
        Self {
            selectors,
            recursive,
        }
    }
}

/// Complete JSONPath expression.
#[derive(Debug, Clone, PartialEq)]
pub struct JSONPath {
    /// Path segments.
    pub segments: Vec<PathSegment>,
}

impl JSONPath {
    pub fn new(segments: Vec<PathSegment>) -> Self {
        Self { segments }
    }
}

/// Filter expression types.
#[derive(Debug, Clone, PartialEq)]
pub enum FilterExpression {
    /// Comparison expression: `@.price < 10`
    Comparison {
        operator: ComparisonOperator,
        left: ValueExpression,
        right: ValueExpression,
    },
    /// Logical expression: `@.a && @.b`
    Logical {
        operator: LogicalOperator,
        left: Box<FilterExpression>,
        right: Box<FilterExpression>,
    },
    /// Existence test: `@.name`
    Existence { path: JSONPath },
    /// Function call: `length(@)`
    Function {
        name: String,
        args: Vec<FunctionArg>,
    },
    /// Parenthesized expression: `(@.a || @.b)`
    Paren(Box<FilterExpression>),
    /// Negation: `!@.flag`
    Negation(Box<FilterExpression>),
}

/// Comparison operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOperator {
    Equal,        // ==
    NotEqual,     // !=
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=
}

/// Logical operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalOperator {
    And, // &&
    Or,  // ||
}

/// Value expressions in filters.
#[derive(Debug, Clone, PartialEq)]
pub enum ValueExpression {
    /// Current node: `@`
    Current,
    /// Root node: `$`
    Root,
    /// Literal value: `"string"`, `42`, `true`, `null`
    Literal(Value),
    /// Path expression: `@.name`
    Path(JSONPath),
    /// Function call: `length(@)`
    Function {
        name: String,
        args: Vec<FunctionArg>,
    },
}

/// Function argument types.
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionArg {
    Value(ValueExpression),
    Filter(FilterExpression),
    Path(JSONPath),
}

/// Result of JSONPath query evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryResult<'a> {
    /// The matched values.
    pub values: Vec<&'a Value>,
    /// Normalized paths to the matched values.
    pub paths: Vec<Vec<PathComponent>>,
}

/// A component of a normalized path.
#[derive(Debug, Clone, PartialEq)]
pub enum PathComponent {
    Key(String),
    Index(usize),
}

/// JSONPath parse result shape, mirrors upstream `IParseResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseResult {
    pub success: bool,
    pub path: Option<JSONPath>,
    pub error: Option<String>,
    pub position: Option<usize>,
}

impl ParseResult {
    /// Compatibility helper for existing `Result`-style call sites.
    pub fn unwrap(self) -> JSONPath {
        self.expect("called ParseResult::unwrap() on an unsuccessful parse result")
    }

    /// Compatibility helper for existing `Result`-style call sites.
    pub fn expect(self, msg: &str) -> JSONPath {
        if self.success {
            return self.path.expect("successful parse result missing path");
        }
        panic!(
            "{msg}: error={:?}, position={:?}",
            self.error, self.position
        );
    }

    /// Compatibility helper for existing `Result`-style call sites.
    pub fn unwrap_or_else<F>(self, op: F) -> JSONPath
    where
        F: FnOnce(String) -> JSONPath,
    {
        if self.success {
            return self.path.expect("successful parse result missing path");
        }
        let err = self
            .error
            .unwrap_or_else(|| "unknown parse error".to_string());
        op(err)
    }
}
