//! CBOR constants.
//!
//! Mirrors `cbor/constants.ts` from upstream.

// MAJOR type values (bits 7-5 of the initial byte)
pub const MAJOR_UIN: u8 = 0b000;
pub const MAJOR_NIN: u8 = 0b001;
pub const MAJOR_BIN: u8 = 0b010;
pub const MAJOR_STR: u8 = 0b011;
pub const MAJOR_ARR: u8 = 0b100;
pub const MAJOR_MAP: u8 = 0b101;
pub const MAJOR_TAG: u8 = 0b110;
pub const MAJOR_TKN: u8 = 0b111;

// MAJOR type overlays (major shifted to bits 7-5)
pub const OVERLAY_UIN: u8 = 0b000_00000;
pub const OVERLAY_NIN: u8 = 0b001_00000;
pub const OVERLAY_BIN: u8 = 0b010_00000;
pub const OVERLAY_STR: u8 = 0b011_00000;
pub const OVERLAY_ARR: u8 = 0b100_00000;
pub const OVERLAY_MAP: u8 = 0b101_00000;
pub const OVERLAY_TAG: u8 = 0b110_00000;
#[allow(dead_code)] // upstream parity: cbor/constants.ts
pub const OVERLAY_TKN: u8 = 0b111_00000;

pub const MINOR_MASK: u8 = 0b11111;

/// Maximum safe integer representable as f64 without precision loss.
#[allow(dead_code)] // upstream parity: cbor/constants.ts
pub const MAX_UINT: u64 = 9007199254740991; // Number.MAX_SAFE_INTEGER

/// CBOR "break" stop code.
pub const CBOR_END: u8 = 0xff;

/// Returns `true` if `f` can be losslessly represented as an `f32`.
#[inline]
pub fn is_f32_roundtrip(f: f64) -> bool {
    (f as f32) as f64 == f
}
