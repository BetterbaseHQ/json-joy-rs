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

    #[test]
    fn test_default() {
        let reader = StreamingReader::default();
        assert_eq!(reader.size(), 0);
    }

    #[test]
    fn test_with_alloc_size() {
        let mut reader = StreamingReader::with_alloc_size(64);
        reader.push(&[10, 20]);
        assert_eq!(reader.u8(), 10);
        assert_eq!(reader.u8(), 20);
    }

    #[test]
    fn test_size_tracks_remaining() {
        let mut reader = StreamingReader::new();
        reader.push(&[1, 2, 3, 4, 5]);
        assert_eq!(reader.size(), 5);
        reader.u8();
        assert_eq!(reader.size(), 4);
        reader.skip(2);
        assert_eq!(reader.size(), 2);
    }

    #[test]
    fn test_i8() {
        let mut reader = StreamingReader::new();
        reader.push(&[0xff]); // -1 as i8
        assert_eq!(reader.i8(), -1);
    }

    #[test]
    fn test_u16_big_endian() {
        let mut reader = StreamingReader::new();
        reader.push(&[0x01, 0x00]);
        assert_eq!(reader.u16(), 256);
    }

    #[test]
    fn test_i16_big_endian() {
        let mut reader = StreamingReader::new();
        reader.push(&[0xff, 0xfe]); // -2 in big-endian i16
        assert_eq!(reader.i16(), -2);
    }

    #[test]
    fn test_u32_big_endian() {
        let mut reader = StreamingReader::new();
        reader.push(&[0x00, 0x01, 0x00, 0x00]);
        assert_eq!(reader.u32(), 0x10000);
    }

    #[test]
    fn test_i32_big_endian() {
        let mut reader = StreamingReader::new();
        reader.push(&[0xff, 0xff, 0xff, 0xff]); // -1
        assert_eq!(reader.i32(), -1);
    }

    #[test]
    fn test_u64_big_endian() {
        let mut reader = StreamingReader::new();
        reader.push(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00]);
        assert_eq!(reader.u64(), 256);
    }

    #[test]
    fn test_i64_big_endian() {
        let mut reader = StreamingReader::new();
        reader.push(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe]); // -2
        assert_eq!(reader.i64(), -2);
    }

    #[test]
    fn test_f32_big_endian() {
        let mut reader = StreamingReader::new();
        let bytes = 1.5f32.to_be_bytes();
        reader.push(&bytes);
        assert_eq!(reader.f32(), 1.5);
    }

    #[test]
    fn test_f64_big_endian() {
        let mut reader = StreamingReader::new();
        let bytes = 1.5f64.to_be_bytes();
        reader.push(&bytes);
        assert_eq!(reader.f64(), 1.5);
    }

    #[test]
    fn test_utf8() {
        let mut reader = StreamingReader::new();
        reader.push(b"hello world");
        let s = reader.utf8(5);
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_ascii() {
        let mut reader = StreamingReader::new();
        reader.push(b"test");
        let s = reader.ascii(4);
        assert_eq!(s, "test");
    }

    #[test]
    fn test_cut() {
        let mut reader = StreamingReader::new();
        reader.push(&[10, 20, 30, 40]);
        let data = reader.cut(2);
        assert_eq!(data, vec![10, 20]);
        assert_eq!(reader.size(), 2);
    }

    #[test]
    fn test_consume_and_x() {
        let mut reader = StreamingReader::new();
        reader.push(&[1, 2, 3, 4]);
        let x_before = reader.x();
        reader.u8();
        reader.u8();
        reader.consume();
        // After consume, x0 advanced, dx reset
        assert_eq!(reader.size(), 2);
        // x() should still be consistent
        let _ = reader.u8();
        assert_eq!(reader.size(), 1);
        let _ = x_before; // suppress unused warning
    }

    #[test]
    fn test_set_x() {
        let mut reader = StreamingReader::new();
        reader.push(&[10, 20, 30, 40]);
        reader.u8(); // advance to position 1
        let x = reader.x();
        reader.u8(); // advance to position 2
        reader.set_x(x); // go back
        assert_eq!(reader.u8(), 20);
    }

    #[test]
    fn test_subarray() {
        let mut reader = StreamingReader::new();
        reader.push(&[10, 20, 30, 40, 50]);
        let sub = reader.subarray(1, Some(3));
        assert_eq!(sub, &[20, 30]);
    }

    #[test]
    fn test_subarray_no_end() {
        let mut reader = StreamingReader::new();
        reader.push(&[10, 20, 30, 40, 50]);
        // When end is None, uses (size - start) bytes from offset start
        // size=5, start=0 => 5 bytes from offset 0 = all
        let sub = reader.subarray(0, None);
        assert_eq!(sub, &[10, 20, 30, 40, 50]);
    }

    #[test]
    fn test_slice() {
        let mut reader = StreamingReader::new();
        reader.push(&[10, 20, 30, 40, 50]);
        let slice_reader = reader.slice(1, Some(3));
        // The slice reader should cover bytes 20, 30
        assert_eq!(slice_reader.size(), 2);
    }

    #[test]
    fn test_slice_no_end() {
        let mut reader = StreamingReader::new();
        reader.push(&[10, 20, 30, 40, 50]);
        // When end is None, uses (size - start) bytes from offset start
        let slice_reader = reader.slice(0, None);
        assert_eq!(slice_reader.size(), 5);
    }

    #[test]
    fn test_reset() {
        let mut reader = StreamingReader::new();
        reader.push(&[1, 2, 3]);
        reader.u8();
        reader.reset(&[10, 20]);
        assert_eq!(reader.size(), 2);
        assert_eq!(reader.u8(), 10);
    }

    #[test]
    fn test_multiple_pushes() {
        let mut reader = StreamingReader::new();
        reader.push(&[1, 2]);
        reader.push(&[3, 4]);
        assert_eq!(reader.size(), 4);
        assert_eq!(reader.u8(), 1);
        assert_eq!(reader.u8(), 2);
        assert_eq!(reader.u8(), 3);
        assert_eq!(reader.u8(), 4);
    }

    #[test]
    #[should_panic(expected = "OUT_OF_BOUNDS")]
    fn test_u8_out_of_bounds() {
        let mut reader = StreamingReader::new();
        reader.push(&[1]);
        reader.u8();
        reader.u8(); // should panic
    }

    #[test]
    #[should_panic(expected = "OUT_OF_BOUNDS")]
    fn test_peek_empty() {
        let reader = StreamingReader::new();
        reader.peek();
    }

    #[test]
    #[should_panic(expected = "OUT_OF_BOUNDS")]
    fn test_skip_out_of_bounds() {
        let mut reader = StreamingReader::new();
        reader.push(&[1, 2]);
        reader.skip(3);
    }

    #[test]
    #[should_panic(expected = "OUT_OF_BOUNDS")]
    fn test_buf_out_of_bounds() {
        let mut reader = StreamingReader::new();
        reader.push(&[1]);
        reader.buf(5);
    }
}
