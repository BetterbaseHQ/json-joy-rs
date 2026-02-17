//! Binary buffer utilities for json-joy.
//!
//! This crate provides efficient binary buffer reading and writing utilities,
//! ported from the TypeScript `@jsonjoy.com/buffers` package.
//!
//! # Overview
//!
//! ## Core Types
//! - [`Reader`] - Reads binary data from a byte slice with cursor tracking
//! - [`Writer`] - Writes binary data to an auto-growing buffer
//! - [`Slice`] - A view into a buffer (deprecated, use Reader instead)
//!
//! ## Streaming Readers
//! - [`StreamingReader`] - Streaming reader with internal buffer management
//! - [`StreamingOctetReader`] - Streaming reader for chunked data
//!
//! ## Utilities
//! - [`cmp_uint8_array`], [`cmp_uint8_array2`], [`cmp_uint8_array3`] - Byte slice comparison
//! - [`concat`], [`concat_list`], [`list_to_uint8`] - Concatenation
//! - [`copy_slice`] - Copy byte slices
//! - [`decode_f16`] - Half-precision float decoder
//! - [`is_float32`] - Float32 precision check
//! - [`ascii`], [`utf8`] - String encoding utilities
//! - [`print_octets`] - Debug hex output
//!
//! # Example
//!
//! ```
//! use json_joy_buffers::{Reader, Writer};
//!
//! // Write some data
//! let mut writer = Writer::new();
//! writer.u8(0x01);
//! writer.u16(0x0203);
//! writer.utf8("hello");
//! let data = writer.flush();
//!
//! // Read it back
//! let mut reader = Reader::new(&data);
//! assert_eq!(reader.u8(), 0x01);
//! assert_eq!(reader.u16(), 0x0203);
//! assert_eq!(reader.utf8(5), "hello");
//! ```

mod cmp;
mod concat;
mod copy;
mod f16;
mod is_float32;
mod print_octets;
mod reader;
mod slice;
mod streaming_octet_reader;
mod streaming_reader;
mod strings;
mod uint8_array_cut;
mod writer;

// Re-export all public items
pub use cmp::{cmp_uint8_array, cmp_uint8_array2, cmp_uint8_array3};
pub use concat::{concat, concat_list, list_to_uint8};
pub use copy::copy_slice;
pub use f16::decode_f16;
pub use is_float32::is_float32;
pub use print_octets::{print_octets, print_octets_default};
pub use reader::Reader;
pub use slice::Slice;
pub use streaming_octet_reader::StreamingOctetReader;
pub use streaming_reader::StreamingReader;
pub use strings::{ascii, utf8};
pub use uint8_array_cut::Uint8ArrayCut;
pub use writer::Writer;

/// Error type for buffer operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BufferError {
    /// Attempted to read past the end of the buffer.
    EndOfBuffer,
    /// Invalid UTF-8 sequence.
    InvalidUtf8,
    /// Buffer overflow during write.
    Overflow,
}

impl std::fmt::Display for BufferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BufferError::EndOfBuffer => write!(f, "end of buffer"),
            BufferError::InvalidUtf8 => write!(f, "invalid UTF-8 sequence"),
            BufferError::Overflow => write!(f, "buffer overflow"),
        }
    }
}

impl std::error::Error for BufferError {}
