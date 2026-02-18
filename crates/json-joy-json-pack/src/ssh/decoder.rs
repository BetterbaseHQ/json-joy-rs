//! SSH 2.0 binary decoder (RFC 4251).
//!
//! Upstream reference: `json-pack/src/ssh/SshDecoder.ts`

use crate::JsonPackMpint;

/// SSH 2.0 binary decoder.
///
/// Wraps a [`Reader`] and exposes typed read methods for RFC 4251 types.
/// Unlike most decoders, `read_any()` is not meaningful for SSH because the
/// format is schema-driven — use the explicit typed methods instead.
pub struct SshDecoder {
    pub reader: Vec<u8>,
    pub x: usize,
}

impl Default for SshDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl SshDecoder {
    pub fn new() -> Self {
        Self {
            reader: Vec::new(),
            x: 0,
        }
    }

    /// Resets the decoder with a new byte slice to decode from.
    pub fn reset(&mut self, data: &[u8]) {
        self.reader = data.to_vec();
        self.x = 0;
    }

    fn assert_size(&self, n: usize) {
        if self.x + n > self.reader.len() {
            panic!("OUT_OF_BOUNDS");
        }
    }

    /// Reads an SSH boolean (1 byte; non-zero = true).
    pub fn read_boolean(&mut self) -> bool {
        self.assert_size(1);
        let val = self.reader[self.x];
        self.x += 1;
        val != 0
    }

    /// Reads a single raw byte.
    pub fn read_byte(&mut self) -> u8 {
        self.assert_size(1);
        let val = self.reader[self.x];
        self.x += 1;
        val
    }

    /// Reads a big-endian uint32.
    pub fn read_uint32(&mut self) -> u32 {
        self.assert_size(4);
        let val = u32::from_be_bytes([
            self.reader[self.x],
            self.reader[self.x + 1],
            self.reader[self.x + 2],
            self.reader[self.x + 3],
        ]);
        self.x += 4;
        val
    }

    /// Reads a big-endian uint64.
    pub fn read_uint64(&mut self) -> u64 {
        self.assert_size(8);
        let val = u64::from_be_bytes([
            self.reader[self.x],
            self.reader[self.x + 1],
            self.reader[self.x + 2],
            self.reader[self.x + 3],
            self.reader[self.x + 4],
            self.reader[self.x + 5],
            self.reader[self.x + 6],
            self.reader[self.x + 7],
        ]);
        self.x += 8;
        val
    }

    /// Reads an SSH binary string (uint32 length + raw bytes).
    pub fn read_bin_str(&mut self) -> Vec<u8> {
        let length = self.read_uint32() as usize;
        self.assert_size(length);
        let data = self.reader[self.x..self.x + length].to_vec();
        self.x += length;
        data
    }

    /// Reads an SSH UTF-8 string (uint32 length + UTF-8 bytes).
    pub fn read_str(&mut self) -> String {
        let bytes = self.read_bin_str();
        String::from_utf8(bytes).unwrap_or_default()
    }

    /// Reads an SSH ASCII string (uint32 length + ASCII bytes).
    pub fn read_ascii_str(&mut self) -> String {
        let length = self.read_uint32() as usize;
        self.assert_size(length);
        let s: String = self.reader[self.x..self.x + length]
            .iter()
            .map(|&b| b as char)
            .collect();
        self.x += length;
        s
    }

    /// Reads an SSH mpint (uint32 length + two's-complement MSB-first bytes).
    pub fn read_mpint(&mut self) -> JsonPackMpint {
        let bytes = self.read_bin_str();
        JsonPackMpint { data: bytes }
    }

    /// Reads an SSH name-list (comma-separated ASCII names).
    pub fn read_name_list(&mut self) -> Vec<String> {
        let s = self.read_ascii_str();
        if s.is_empty() {
            return Vec::new();
        }
        s.split(',').map(|s| s.to_string()).collect()
    }

    /// Reads binary data as an SSH string.
    pub fn read_bin(&mut self) -> Vec<u8> {
        self.read_bin_str()
    }
}
