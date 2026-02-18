//! Record Marshalling (RM) frame decoder.
//!
//! Upstream reference: `json-pack/src/rm/RmRecordDecoder.ts`

use json_joy_buffers::StreamingReader;

/// Record Marshalling frame decoder.
///
/// Accepts pushed byte chunks and assembles complete records from RM frames.
/// Call [`push`] to feed data and [`read_record`] to receive reassembled records.
pub struct RmRecordDecoder {
    pub reader: StreamingReader,
    fragments: Vec<Vec<u8>>,
}

impl Default for RmRecordDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl RmRecordDecoder {
    pub fn new() -> Self {
        Self {
            reader: StreamingReader::new(),
            fragments: Vec::new(),
        }
    }

    /// Pushes a chunk of bytes into the internal buffer.
    pub fn push(&mut self, data: &[u8]) {
        self.reader.push(data);
    }

    /// Attempts to read a complete RM record from the internal buffer.
    ///
    /// Returns `Some(bytes)` when a full record has been assembled, or `None`
    /// when more data is needed.
    ///
    /// Panics and state-resets if the buffer is corrupt — matching upstream
    /// behaviour of catching `RangeError` and restoring `reader.x`.
    pub fn read_record(&mut self) -> Option<Vec<u8>> {
        let size = self.reader.size();
        if size < 4 {
            return None;
        }
        let saved_x = self.reader.x();
        // Use a closure so we can restore position on failure (mirrors the
        // TypeScript try/catch RangeError pattern).
        match self.try_read_fragment() {
            Ok(result) => result,
            Err(()) => {
                self.reader.set_x(saved_x);
                None
            }
        }
    }

    fn try_read_fragment(&mut self) -> Result<Option<Vec<u8>>, ()> {
        let size = self.reader.size();
        if size < 4 {
            return Ok(None);
        }
        let header = {
            // Temporarily snapshot position to detect underflow
            let saved = self.reader.x();
            let h = self.reader.u32();
            let _ = saved;
            h
        };
        let fin = (header & 0x8000_0000) != 0;
        let len = (header & 0x7fff_ffff) as usize;
        if self.reader.size() < len {
            return Err(()); // not enough data — restore
        }
        self.reader.consume();
        let fragments = &mut self.fragments;
        if fin {
            if fragments.is_empty() {
                let data = self.reader.buf(len);
                if data.is_empty() {
                    return Ok(None);
                }
                return Ok(Some(data));
            }
            let chunk = self.reader.buf(len);
            fragments.push(chunk);
            let record: Vec<u8> = fragments.concat();
            self.fragments = Vec::new();
            if record.is_empty() {
                return Ok(None);
            }
            Ok(Some(record))
        } else {
            let chunk = self.reader.buf(len);
            self.fragments.push(chunk);
            Ok(None)
        }
    }
}
