/// Validation error codes.
///
/// ATTENTION: Only add new error codes at the end of the list !!!
/// Upstream reference: json-type/src/constants.ts
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValidationError {
    Str = 0,
    Num = 1,
    Bool = 2,
    Arr = 3,
    Tup = 4,
    Obj = 5,
    Map = 6,
    Key = 7,
    Keys = 8,
    Bin = 9,
    Or = 10,
    Ref = 11,
    Enum = 12,
    Const = 13,
    Validation = 14,
    Int = 15,
    Uint = 16,
    StrLen = 17,
    ArrLen = 18,
    Gt = 19,
    Gte = 20,
    Lt = 21,
    Lte = 22,
    BinLen = 23,
}

impl ValidationError {
    pub fn name(self) -> &'static str {
        match self {
            Self::Str => "STR",
            Self::Num => "NUM",
            Self::Bool => "BOOL",
            Self::Arr => "ARR",
            Self::Tup => "TUP",
            Self::Obj => "OBJ",
            Self::Map => "MAP",
            Self::Key => "KEY",
            Self::Keys => "KEYS",
            Self::Bin => "BIN",
            Self::Or => "OR",
            Self::Ref => "REF",
            Self::Enum => "ENUM",
            Self::Const => "CONST",
            Self::Validation => "VALIDATION",
            Self::Int => "INT",
            Self::Uint => "UINT",
            Self::StrLen => "STR_LEN",
            Self::ArrLen => "ARR_LEN",
            Self::Gt => "GT",
            Self::Gte => "GTE",
            Self::Lt => "LT",
            Self::Lte => "LTE",
            Self::BinLen => "BIN_LEN",
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            Self::Str => "Not a string.",
            Self::Num => "Not a number.",
            Self::Bool => "Not a boolean.",
            Self::Arr => "Not an array.",
            Self::Tup => "Not a tuple.",
            Self::Obj => "Not an object.",
            Self::Map => "Not a map.",
            Self::Key => "Missing key.",
            Self::Keys => "Too many or missing object keys.",
            Self::Bin => "Not a binary.",
            Self::Or => "None of types matched.",
            Self::Ref => "Validation error in referenced type.",
            Self::Enum => "Not an enum value.",
            Self::Const => "Invalid constant.",
            Self::Validation => "Custom validator failed.",
            Self::Int => "Not an integer.",
            Self::Uint => "Not an unsigned integer.",
            Self::StrLen => "Invalid string length.",
            Self::BinLen => "Invalid binary length.",
            Self::ArrLen => "Invalid array length.",
            Self::Gt => "Value is too small.",
            Self::Gte => "Value is too small.",
            Self::Lt => "Value is too large.",
            Self::Lte => "Value is too large.",
        }
    }
}
