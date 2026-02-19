//! RESP3 protocol constants.
//!
//! Upstream reference: `json-pack/src/resp/constants.ts`
#![allow(non_snake_case)]

/// RESP3 type-prefix bytes and special values.
pub mod Resp {
    pub const R: u8 = 0x0d; // \r
    pub const N: u8 = 0x0a; // \n
    pub const RN: u16 = 0x0d0a; // \r\n as u16

    pub const NULL: u8 = 95; // _
    pub const BOOL: u8 = 35; // #
    pub const INT: u8 = 58; // :
    pub const BIG: u8 = 40; // (
    pub const FLOAT: u8 = 44; // ,
    pub const STR_SIMPLE: u8 = 43; // +
    pub const STR_BULK: u8 = 36; // $
    pub const STR_VERBATIM: u8 = 61; // =
    pub const ERR_SIMPLE: u8 = 45; // -
    pub const ERR_BULK: u8 = 33; // !
    pub const ARR: u8 = 42; // *
    pub const SET: u8 = 126; // ~
    pub const OBJ: u8 = 37; // %
    pub const PUSH: u8 = 62; // >
    pub const ATTR: u8 = 124; // |

    pub const PLUS: u8 = 43; // +
    pub const MINUS: u8 = 45; // -
}

/// Extension tags used for RESP-specific value types (not in core RESP constants).
pub const RESP_EXTENSION_PUSH: u64 = 1;
pub const RESP_EXTENSION_ATTRIBUTES: u64 = 2;
pub const RESP_EXTENSION_VERBATIM_STRING: u64 = 3;
