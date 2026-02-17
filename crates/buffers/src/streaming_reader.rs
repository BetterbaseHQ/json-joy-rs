//! Streaming reader with internal buffer management.

use crate::{Reader, Writer};

/// A streaming reader that internally manages a growing buffer.
///
/// Data chunks are pushed into the reader and can be consumed incrementally.
pub struct StreamingReader {
    writer: Writer,
    /// Offset from the start of the buffer (x0 in Writer).
    dx: usize,
}

impl Default for StreamingReader {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamingReader {
    /// Creates a new streaming reader with default allocation size.
    pub fn new() -> Self {
        Self::with_alloc_size(16 * 1024)
    }

    /// Creates a new streaming reader with custom allocation size.
    pub fn with_alloc_size(alloc_size: usize) -> Self {
        Self {
            writer: Writer::with_alloc_size(alloc_size),
            dx: 0,
        }
    }

    /// Returns the number of bytes remaining to be read.
    pub fn size(&self) -> usize {
        self.writer.x - self.x()
    }

    fn assert_size(&self, size: usize) {
        if size > self.size() {
            panic!("OUT_OF_BOUNDS");
        }
    }

    /// Adds a chunk of data to be read.
    pub fn push(&mut self, data: &[u8]) {
        self.writer.buf(data);
    }

    /// Marks the current position as consumed, freeing memory for reuse.
    pub fn consume(&mut self) {
        self.writer.x0 += self.dx;
        self.dx = 0;
    }

    /// Returns the current cursor position.
    pub fn x(&self) -> usize {
        self.writer.x0 + self.dx
    }

    /// Sets the cursor position.
    pub fn set_x(&mut self, x: usize) {
        self.dx = x - self.writer.x0;
    }

    /// Peeks at the next byte without advancing.
    pub fn peek(&self) -> u8 {
        self.assert_size(1);
        self.writer.uint8[self.x()]
    }

    /// Skips the given number of bytes.
    pub fn skip(&mut self, length: usize) {
        self.assert_size(length);
        self.dx += length;
    }

    /// Reads bytes into a new vector.
    pub fn buf(&mut self, size: usize) -> Vec<u8> {
        self.assert_size(size);
        let x = self.x();
        let end = x + size;
        let result = self.writer.uint8[x..end].to_vec();
        self.dx += size;
        result
    }

    /// Returns a subarray without advancing.
    pub fn subarray(&self, start: usize, end: Option<usize>) -> &[u8] {
        let x = self.x();
        let actual_start = x + start;
        let actual_end = end.map(|e| x + e).unwrap_or(self.size() + x - start);
        &self.writer.uint8[actual_start..actual_end]
    }

    /// Creates a Reader slice from the current position.
    pub fn slice(&self, start: usize, end: Option<usize>) -> Reader<'_> {
        let x = self.x();
        let actual_start = x + start;
        let actual_end = end.map(|e| x + e).unwrap_or(self.size() + x - start);
        Reader::from_slice(&self.writer.uint8, actual_start, actual_end)
    }

    /// Reads and returns bytes, advancing the cursor.
    ///
    /// This is like `cut` but returns the data directly instead of a Reader.
    pub fn cut(&mut self, size: usize) -> Vec<u8> {
        self.buf(size)
    }

    /// Reads an unsigned 8-bit integer.
    pub fn u8(&mut self) -> u8 {
        self.assert_size(1);
        let val = self.writer.uint8[self.x()];
        self.dx += 1;
        val
    }

    /// Reads a signed 8-bit integer.
    pub fn i8(&mut self) -> i8 {
        self.assert_size(1);
        let val = self.writer.uint8[self.x()] as i8;
        self.dx += 1;
        val
    }

    /// Reads an unsigned 16-bit integer (big-endian).
    pub fn u16(&mut self) -> u16 {
        self.assert_size(2);
        let x = self.x();
        let val = u16::from_be_bytes([self.writer.uint8[x], self.writer.uint8[x + 1]]);
        self.dx += 2;
        val
    }

    /// Reads a signed 16-bit integer (big-endian).
    pub fn i16(&mut self) -> i16 {
        self.assert_size(2);
        let x = self.x();
        let val = i16::from_be_bytes([self.writer.uint8[x], self.writer.uint8[x + 1]]);
        self.dx += 2;
        val
    }

    /// Reads an unsigned 32-bit integer (big-endian).
    pub fn u32(&mut self) -> u32 {
        self.assert_size(4);
        let x = self.x();
        let val = u32::from_be_bytes([
            self.writer.uint8[x],
            self.writer.uint8[x + 1],
            self.writer.uint8[x + 2],
            self.writer.uint8[x + 3],
        ]);
        self.dx += 4;
        val
    }

    /// Reads a signed 32-bit integer (big-endian).
    pub fn i32(&mut self) -> i32 {
        self.assert_size(4);
        let x = self.x();
        let val = i32::from_be_bytes([
            self.writer.uint8[x],
            self.writer.uint8[x + 1],
            self.writer.uint8[x + 2],
            self.writer.uint8[x + 3],
        ]);
        self.dx += 4;
        val
    }

    /// Reads an unsigned 64-bit integer (big-endian).
    pub fn u64(&mut self) -> u64 {
        self.assert_size(8);
        let x = self.x();
        let val = u64::from_be_bytes([
            self.writer.uint8[x],
            self.writer.uint8[x + 1],
            self.writer.uint8[x + 2],
            self.writer.uint8[x + 3],
            self.writer.uint8[x + 4],
            self.writer.uint8[x + 5],
            self.writer.uint8[x + 6],
            self.writer.uint8[x + 7],
        ]);
        self.dx += 8;
        val
    }

    /// Reads a signed 64-bit integer (big-endian).
    pub fn i64(&mut self) -> i64 {
        self.assert_size(8);
        let x = self.x();
        let val = i64::from_be_bytes([
            self.writer.uint8[x],
            self.writer.uint8[x + 1],
            self.writer.uint8[x + 2],
            self.writer.uint8[x + 3],
            self.writer.uint8[x + 4],
            self.writer.uint8[x + 5],
            self.writer.uint8[x + 6],
            self.writer.uint8[x + 7],
        ]);
        self.dx += 8;
        val
    }

    /// Reads a 32-bit float (big-endian).
    pub fn f32(&mut self) -> f32 {
        self.assert_size(4);
        let x = self.x();
        let val = f32::from_be_bytes([
            self.writer.uint8[x],
            self.writer.uint8[x + 1],
            self.writer.uint8[x + 2],
            self.writer.uint8[x + 3],
        ]);
        self.dx += 4;
        val
    }

    /// Reads a 64-bit float (big-endian).
    pub fn f64(&mut self) -> f64 {
        self.assert_size(8);
        let x = self.x();
        let val = f64::from_be_bytes([
            self.writer.uint8[x],
            self.writer.uint8[x + 1],
            self.writer.uint8[x + 2],
            self.writer.uint8[x + 3],
            self.writer.uint8[x + 4],
            self.writer.uint8[x + 5],
            self.writer.uint8[x + 6],
            self.writer.uint8[x + 7],
        ]);
        self.dx += 8;
        val
    }

    /// Reads a UTF-8 string of the given size.
    pub fn utf8(&mut self, size: usize) -> &str {
        self.assert_size(size);
        let x = self.x();
        let s = std::str::from_utf8(&self.writer.uint8[x..x + size]).unwrap_or("");
        self.dx += size;
        s
    }

    /// Reads an ASCII string of the given length.
    pub fn ascii(&mut self, length: usize) -> &str {
        self.utf8(length)
    }

    /// Resets the reader with new data.
    pub fn reset(&mut self, data: &[u8]) {
        self.dx = 0;
        self.writer.reset();
        self.push(data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_read() {
        let mut reader = StreamingReader::new();
        reader.push(&[1, 2, 3, 4]);
        assert_eq!(reader.u8(), 1);
        assert_eq!(reader.u8(), 2);
        assert_eq!(reader.u16(), 0x0304);
    }

    #[test]
    fn test_peek() {
        let mut reader = StreamingReader::new();
        reader.push(&[42, 43]);
        assert_eq!(reader.peek(), 42);
        assert_eq!(reader.u8(), 42);
    }

    #[test]
    fn test_skip() {
        let mut reader = StreamingReader::new();
        reader.push(&[1, 2, 3, 4, 5]);
        reader.skip(2);
        assert_eq!(reader.u8(), 3);
    }

    #[test]
    fn test_buf() {
        let mut reader = StreamingReader::new();
        reader.push(&[1, 2, 3, 4, 5]);
        let buf = reader.buf(3);
        assert_eq!(buf, vec![1, 2, 3]);
    }
}
