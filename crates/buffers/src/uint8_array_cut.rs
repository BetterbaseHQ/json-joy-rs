//! A cut from a byte array.

/// Represents a cut (view) into a byte slice with start position and size.
///
/// This is a lightweight type for referencing a portion of a byte array
/// without copying it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uint8ArrayCut<'a> {
    /// The underlying byte slice.
    pub uint8: &'a [u8],
    /// Start position of the cut.
    pub start: usize,
    /// Size of the cut in bytes.
    pub size: usize,
}

impl<'a> Uint8ArrayCut<'a> {
    /// Creates a new cut.
    pub fn new(uint8: &'a [u8], start: usize, size: usize) -> Self {
        Self { uint8, start, size }
    }

    /// Returns the cut as a slice.
    pub fn as_slice(&self) -> &'a [u8] {
        &self.uint8[self.start..self.start + self.size]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uint8_array_cut() {
        let data = vec![1, 2, 3, 4, 5];
        let cut = Uint8ArrayCut::new(&data, 1, 3);
        assert_eq!(cut.as_slice(), &[2, 3, 4]);
    }
}
