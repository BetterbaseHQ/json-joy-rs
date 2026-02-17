//! Slice type for buffer views.
//!
//! **Note:** This type is deprecated. Use [`Reader`](crate::Reader) instead.

/// A slice view into a buffer.
///
/// **Deprecated:** Use [`Reader`](crate::Reader) instead.
#[derive(Debug, Clone)]
pub struct Slice<'a> {
    /// The underlying byte slice.
    pub uint8: &'a [u8],
    /// Start position.
    pub start: usize,
    /// End position.
    pub end: usize,
}

impl<'a> Slice<'a> {
    /// Creates a new slice.
    pub fn new(uint8: &'a [u8], start: usize, end: usize) -> Self {
        Self { uint8, start, end }
    }

    /// Returns the slice as a subarray.
    pub fn subarray(&self) -> &'a [u8] {
        &self.uint8[self.start..self.end]
    }
}
