//! Streaming RESP decoder.
//!
//! Upstream reference: `json-pack/src/resp/RespStreamingDecoder.ts`

use super::{RespDecodeError, RespDecoder};
use crate::PackValue;

/// Incremental RESP decoder that accepts chunked input and emits decoded values.
pub struct RespStreamingDecoder {
    buffer: Vec<u8>,
    offset: usize,
    decoder: RespDecoder,
}

impl Default for RespStreamingDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl RespStreamingDecoder {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            offset: 0,
            decoder: RespDecoder::new(),
        }
    }

    pub fn try_utf8(&self) -> bool {
        self.decoder.try_utf8
    }

    pub fn set_try_utf8(&mut self, value: bool) {
        self.decoder.try_utf8 = value;
    }

    pub fn push(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    pub fn read(&mut self) -> Result<Option<PackValue>, RespDecodeError> {
        if self.offset >= self.buffer.len() {
            return Ok(None);
        }
        let input = &self.buffer[self.offset..];
        self.decoder.reset(input);
        match self.decoder.read_any() {
            Ok(value) => {
                self.offset += self.decoder.position();
                self.compact();
                Ok(Some(value))
            }
            Err(RespDecodeError::EndOfInput) => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub fn read_cmd(&mut self) -> Result<Option<Vec<Vec<u8>>>, RespDecodeError> {
        if self.offset >= self.buffer.len() {
            return Ok(None);
        }
        let input = &self.buffer[self.offset..];
        self.decoder.reset(input);
        match self.decoder.read_cmd() {
            Ok(value) => {
                self.offset += self.decoder.position();
                self.compact();
                Ok(Some(value))
            }
            Err(RespDecodeError::EndOfInput) => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub fn skip(&mut self) -> Result<Option<()>, RespDecodeError> {
        if self.offset >= self.buffer.len() {
            return Ok(None);
        }
        let input = &self.buffer[self.offset..];
        self.decoder.reset(input);
        match self.decoder.skip_any() {
            Ok(()) => {
                self.offset += self.decoder.position();
                self.compact();
                Ok(Some(()))
            }
            Err(RespDecodeError::EndOfInput) => Ok(None),
            Err(err) => Err(err),
        }
    }

    fn compact(&mut self) {
        if self.offset == 0 {
            return;
        }
        if self.offset == self.buffer.len() {
            self.buffer.clear();
            self.offset = 0;
            return;
        }
        if self.offset >= 8192 || self.offset * 2 >= self.buffer.len() {
            self.buffer.drain(..self.offset);
            self.offset = 0;
        }
    }
}
