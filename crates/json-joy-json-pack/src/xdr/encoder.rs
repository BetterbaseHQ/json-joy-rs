//! XDR primitive encoder.
//!
//! Upstream reference: `json-pack/src/xdr/XdrEncoder.ts`
//! Reference: RFC 4506 — all integers big-endian, 4-byte alignment.

use json_joy_buffers::Writer;

/// XDR primitive encoder.
///
/// Writes XDR-encoded primitives using big-endian byte order and 4-byte alignment.
pub struct XdrEncoder {
    pub writer: Writer,
}

impl Default for XdrEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl XdrEncoder {
    pub fn new() -> Self {
        Self {
            writer: Writer::new(),
        }
    }

    // ---------------------------------------------------------------- helpers

    /// Writes zero-byte padding to align to a 4-byte boundary.
    fn write_padding(&mut self, data_len: usize) {
        let rem = data_len % 4;
        if rem != 0 {
            let pad = 4 - rem;
            for _ in 0..pad {
                self.writer.u8(0);
            }
        }
    }

    // ---------------------------------------------------------------- primitives

    pub fn write_void(&mut self) {
        // No bytes for void.
    }

    pub fn write_boolean(&mut self, b: bool) {
        self.writer.u32(if b { 1 } else { 0 });
    }

    pub fn write_int(&mut self, n: i32) {
        self.writer.u32(n as u32);
    }

    pub fn write_unsigned_int(&mut self, n: u32) {
        self.writer.u32(n);
    }

    pub fn write_hyper(&mut self, n: i64) {
        let bytes = n.to_be_bytes();
        self.writer.buf(&bytes);
    }

    pub fn write_unsigned_hyper(&mut self, n: u64) {
        let bytes = n.to_be_bytes();
        self.writer.buf(&bytes);
    }

    pub fn write_float(&mut self, f: f32) {
        self.writer.u32(f.to_bits());
    }

    pub fn write_double(&mut self, f: f64) {
        let bytes = f.to_be_bytes();
        self.writer.buf(&bytes);
    }

    /// Writes fixed-size opaque data with padding to 4-byte boundary.
    pub fn write_opaque(&mut self, data: &[u8]) {
        self.writer.buf(data);
        self.write_padding(data.len());
    }

    /// Writes variable-length opaque: [length: u32][data][padding].
    pub fn write_varlen_opaque(&mut self, data: &[u8]) {
        self.writer.u32(data.len() as u32);
        self.write_opaque(data);
    }

    /// Writes a string: [length: u32][utf8 bytes][padding].
    pub fn write_str(&mut self, s: &str) {
        let bytes = s.as_bytes();
        self.writer.u32(bytes.len() as u32);
        self.writer.buf(bytes);
        self.write_padding(bytes.len());
    }
}
