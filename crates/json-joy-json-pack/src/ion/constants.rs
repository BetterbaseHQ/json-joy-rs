//! Ion binary format constants.
//!
//! Upstream reference: `json-pack/src/ion/constants.ts`
#![allow(non_snake_case)]

/// Ion binary type identifiers (high 4 bits of type descriptor byte).
pub mod Type {
    pub const NULL: u8 = 0b0000;
    pub const BOOL: u8 = 0b0001;
    pub const UINT: u8 = 0b0010;
    pub const NINT: u8 = 0b0011;
    pub const FLOT: u8 = 0b0100;
    pub const STRI: u8 = 0b1000;
    pub const BINA: u8 = 0b1010;
    pub const LIST: u8 = 0b1011;
    pub const STRU: u8 = 0b1101;
    pub const ANNO: u8 = 0b1110;
}

/// Type overlay — type shifted into high 4 bits for type descriptor byte.
pub mod TypeOverlay {
    pub const NULL: u8 = super::Type::NULL << 4;
    pub const BOOL: u8 = super::Type::BOOL << 4;
    pub const UINT: u8 = super::Type::UINT << 4;
    pub const NINT: u8 = super::Type::NINT << 4;
    pub const FLOT: u8 = super::Type::FLOT << 4;
    pub const STRI: u8 = super::Type::STRI << 4;
    pub const BINA: u8 = super::Type::BINA << 4;
    pub const LIST: u8 = super::Type::LIST << 4;
    pub const STRU: u8 = super::Type::STRU << 4;
    pub const ANNO: u8 = super::Type::ANNO << 4;
}

/// Ion Binary Version Marker (IVM): 4 bytes 0xe0 0x01 0x00 0xea.
pub const ION_BVM: [u8; 4] = [0xe0, 0x01, 0x00, 0xea];

/// System symbol table (1-indexed; index 0 unused).
pub const SYSTEM_SYMBOLS: &[&str] = &[
    "",                         // 0: unused
    "$ion",                     // 1
    "$ion_1_0",                 // 2
    "$ion_symbol_table",        // 3
    "name",                     // 4
    "version",                  // 5
    "imports",                  // 6
    "symbols",                  // 7
    "max_id",                   // 8
    "$ion_shared_symbol_table", // 9
];

/// System symbol ID for '$ion_symbol_table'.
pub const SID_ION_SYMBOL_TABLE: u32 = 3;
/// System symbol ID for 'symbols'.
pub const SID_SYMBOLS: u32 = 7;
