//! Binary buffer reader with cursor tracking.

use std::str;

/// A binary buffer reader that reads data from a byte slice.
///
/// The reader maintains a cursor position and provides methods for reading
/// various integer types and strings.
///
/// # Example
///
/// ```
/// use json_joy_buffers::Reader;
///
/// let data = [0x01, 0x02, 0x03, 0x04];
/// let mut reader = Reader::new(&data);
///
/// assert_eq!(reader.u8(), 0x01);
/// assert_eq!(reader.u16(), 0x0203);
/// ```
pub struct Reader<'a> {
    /// The underlying byte slice.
    pub uint8: &'a [u8],
    /// Current cursor position.
    pub x: usize,
    /// End position (exclusive).
    pub end: usize,
}

impl<'a> Reader<'a> {
    /// Creates a new reader for the given byte slice.
    pub fn new(uint8: &'a [u8]) -> Self {
        let end = uint8.len();
        Self {
            uint8,
            x: 0,
            end,
        }
    }

    /// Creates a reader from a slice with custom start and end positions.
    pub fn from_slice(uint8: &'a [u8], x: usize, end: usize) -> Self {
        Self { uint8, x, end }
    }

    /// Resets the reader with a new byte slice.
    pub fn reset(&mut self, uint8: &'a [u8]) {
        self.x = 0;
        self.end = uint8.len();
        self.uint8 = uint8;
    }

    /// Returns the number of remaining bytes.
    pub fn size(&self) -> usize {
        self.end - self.x
    }

    /// Peeks at the current byte without advancing the cursor.
    pub fn peek(&self) -> u8 {
        self.uint8[self.x]
    }

    /// @deprecated Use peek() instead.
    pub fn peak(&self) -> u8 {
        self.peek()
    }

    /// Advances the cursor by the given number of bytes.
    pub fn skip(&mut self, length: usize) {
        self.x += length;
    }

    /// Returns a subarray of the given size and advances the cursor.
    pub fn buf(&mut self, size: usize) -> &'a [u8] {
        let x = self.x;
        let end = x + size;
        let bin = &self.uint8[x..end];
        self.x = end;
        bin
    }

    /// Returns a subarray without advancing the cursor.
    pub fn subarray(&self, start: usize, end: Option<usize>) -> &'a [u8] {
        let x = self.x;
        let actual_start = x + start;
        let actual_end = end.map(|e| x + e).unwrap_or(self.end);
        &self.uint8[actual_start..actual_end]
    }

    /// Creates a new Reader that references the same underlying memory.
    pub fn slice(&self, start: usize, end: Option<usize>) -> Reader<'a> {
        let x = self.x;
        let actual_start = x + start;
        let actual_end = end.map(|e| x + e).unwrap_or(self.end);
        Reader::from_slice(self.uint8, actual_start, actual_end)
    }

    /// Creates a new Reader from the current position and advances the cursor.
    pub fn cut(&mut self, size: usize) -> Reader<'a> {
        let slice = self.slice(0, Some(size));
        self.skip(size);
        slice
    }

    /// Reads an unsigned 8-bit integer.
    #[inline]
    pub fn u8(&mut self) -> u8 {
        let val = self.uint8[self.x];
        self.x += 1;
        val
    }

    /// Reads a signed 8-bit integer.
    #[inline]
    pub fn i8(&mut self) -> i8 {
        let val = self.uint8[self.x] as i8;
        self.x += 1;
        val
    }

    /// Reads an unsigned 16-bit integer (big-endian).
    #[inline]
    pub fn u16(&mut self) -> u16 {
        let x = self.x;
        let val = ((self.uint8[x] as u16) << 8) | (self.uint8[x + 1] as u16);
        self.x += 2;
        val
    }

    /// Reads a signed 16-bit integer (big-endian).
    #[inline]
    pub fn i16(&mut self) -> i16 {
        let val = i16::from_be_bytes([self.uint8[self.x], self.uint8[self.x + 1]]);
        self.x += 2;
        val
    }

    /// Reads an unsigned 32-bit integer (big-endian).
    #[inline]
    pub fn u32(&mut self) -> u32 {
        let val = u32::from_be_bytes([
            self.uint8[self.x],
            self.uint8[self.x + 1],
            self.uint8[self.x + 2],
            self.uint8[self.x + 3],
        ]);
        self.x += 4;
        val
    }

    /// Reads a signed 32-bit integer (big-endian).
    #[inline]
    pub fn i32(&mut self) -> i32 {
        let val = i32::from_be_bytes([
            self.uint8[self.x],
            self.uint8[self.x + 1],
            self.uint8[self.x + 2],
            self.uint8[self.x + 3],
        ]);
        self.x += 4;
        val
    }

    /// Reads an unsigned 64-bit integer (big-endian).
    #[inline]
    pub fn u64(&mut self) -> u64 {
        let val = u64::from_be_bytes([
            self.uint8[self.x],
            self.uint8[self.x + 1],
            self.uint8[self.x + 2],
            self.uint8[self.x + 3],
            self.uint8[self.x + 4],
            self.uint8[self.x + 5],
            self.uint8[self.x + 6],
            self.uint8[self.x + 7],
        ]);
        self.x += 8;
        val
    }

    /// Reads a signed 64-bit integer (big-endian).
    #[inline]
    pub fn i64(&mut self) -> i64 {
        let val = i64::from_be_bytes([
            self.uint8[self.x],
            self.uint8[self.x + 1],
            self.uint8[self.x + 2],
            self.uint8[self.x + 3],
            self.uint8[self.x + 4],
            self.uint8[self.x + 5],
            self.uint8[self.x + 6],
            self.uint8[self.x + 7],
        ]);
        self.x += 8;
        val
    }

    /// Reads a 32-bit floating point number (big-endian).
    #[inline]
    pub fn f32(&mut self) -> f32 {
        let val = f32::from_be_bytes([
            self.uint8[self.x],
            self.uint8[self.x + 1],
            self.uint8[self.x + 2],
            self.uint8[self.x + 3],
        ]);
        self.x += 4;
        val
    }

    /// Reads a 64-bit floating point number (big-endian).
    #[inline]
    pub fn f64(&mut self) -> f64 {
        let val = f64::from_be_bytes([
            self.uint8[self.x],
            self.uint8[self.x + 1],
            self.uint8[self.x + 2],
            self.uint8[self.x + 3],
            self.uint8[self.x + 4],
            self.uint8[self.x + 5],
            self.uint8[self.x + 6],
            self.uint8[self.x + 7],
        ]);
        self.x += 8;
        val
    }

    /// Reads a UTF-8 string of the given size.
    pub fn utf8(&mut self, size: usize) -> &'a str {
        let start = self.x;
        self.x += size;
        str::from_utf8(&self.uint8[start..self.x]).unwrap_or("")
    }

    /// Reads an ASCII string of the given length.
    pub fn ascii(&mut self, length: usize) -> &'a str {
        let start = self.x;
        self.x += length;
        // ASCII is a subset of UTF-8, so this is safe
        str::from_utf8(&self.uint8[start..self.x]).unwrap_or("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8() {
        let data = [0x01, 0x02, 0x03];
        let mut reader = Reader::new(&data);
        assert_eq!(reader.u8(), 0x01);
        assert_eq!(reader.u8(), 0x02);
        assert_eq!(reader.u8(), 0x03);
    }

    #[test]
    fn test_u16() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = Reader::new(&data);
        assert_eq!(reader.u16(), 0x0102);
        assert_eq!(reader.u16(), 0x0304);
    }

    #[test]
    fn test_u32() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = Reader::new(&data);
        assert_eq!(reader.u32(), 0x01020304);
    }

    #[test]
    fn test_skip() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = Reader::new(&data);
        reader.skip(2);
        assert_eq!(reader.u8(), 0x03);
    }

    #[test]
    fn test_slice() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05];
        let reader = Reader::new(&data);
        let mut slice = reader.slice(1, Some(4));
        assert_eq!(slice.u8(), 0x02);
    }

    #[test]
    fn test_utf8() {
        let data = b"hello world";
        let mut reader = Reader::new(data);
        assert_eq!(reader.utf8(5), "hello");
        assert_eq!(reader.utf8(6), " world");
    }
}
