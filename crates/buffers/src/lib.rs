//! Binary buffer utilities for json-joy.
//!
//! This crate provides efficient binary buffer reading and writing utilities,
//! ported from the TypeScript `@jsonjoy.com/buffers` package.
//!
//! # Overview
//!
//! - [`Reader`] - Reads binary data from a byte slice with cursor tracking
//! - [`Writer`] - Writes binary data to an auto-growing buffer
//! - [`Slice`] - A view into a buffer (deprecated, use Reader instead)
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

mod reader;
mod slice;
mod writer;

pub use reader::Reader;
pub use slice::Slice;
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
