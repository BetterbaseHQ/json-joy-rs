//! Shared CRDT binary primitives used by model/patch/runtime code.
//!
//! These helpers mirror `json-joy@17.67.0` `CrdtReader/CrdtWriter` behavior
//! for `vu57`, `b1vu56`, and logical clock-table/id handling.

use ciborium::value::Value as CborValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogicalClockBase {
    pub sid: u64,
    pub time: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct BinaryCursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> BinaryCursor<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn is_eof(&self) -> bool {
        self.pos == self.data.len()
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn peek_u8(&self) -> Option<u8> {
        self.data.get(self.pos).copied()
    }

    pub fn u8(&mut self) -> Option<u8> {
        let b = self.peek_u8()?;
        self.pos += 1;
        Some(b)
    }

    pub fn u32_be(&mut self) -> Option<u32> {
        if self.remaining() < 4 {
            return None;
        }
        let out = u32::from_be_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        Some(out)
    }

    pub fn skip(&mut self, n: usize) -> Option<()> {
        if self.remaining() < n {
            return None;
        }
        self.pos += n;
        Some(())
    }

    pub fn read_bytes(&mut self, n: usize) -> Option<&'a [u8]> {
        if self.remaining() < n {
            return None;
        }
        let start = self.pos;
        self.pos += n;
        Some(&self.data[start..start + n])
    }

    pub fn vu57(&mut self) -> Option<u64> {
        read_vu57(self.data, &mut self.pos)
    }

    pub fn b1vu56(&mut self) -> Option<(u8, u64)> {
        read_b1vu56(self.data, &mut self.pos)
    }

    pub fn skip_id(&mut self) -> Option<()> {
        let byte = self.u8()?;
        if byte <= 0b0111_1111 {
            return Some(());
        }
        self.pos = self.pos.saturating_sub(1);
        let _ = self.b1vu56()?;
        let _ = self.vu57()?;
        Some(())
    }

    pub fn read_one_cbor(&mut self) -> Option<CborValue> {
        let slice = &self.data[self.pos..];
        let (val, consumed) = json_joy_json_pack::decode_cbor_value_with_consumed(slice).ok()?;
        self.skip(consumed)?;
        Some(val)
    }
}

pub fn write_vu57(out: &mut Vec<u8>, mut value: u64) {
    for _ in 0..7 {
        let mut b = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            out.push(b);
            return;
        }
        b |= 0x80;
        out.push(b);
    }
    out.push((value & 0xff) as u8);
}

pub fn read_vu57(data: &[u8], pos: &mut usize) -> Option<u64> {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;
    for i in 0..8 {
        let b = *data.get(*pos)?;
        *pos += 1;
        if i < 7 {
            let part = (b & 0x7f) as u64;
            result |= part.checked_shl(shift)?;
            if (b & 0x80) == 0 {
                return Some(result);
            }
            shift += 7;
        } else {
            result |= (b as u64).checked_shl(49)?;
            return Some(result);
        }
    }
    None
}

pub fn write_b1vu56(out: &mut Vec<u8>, flag: u8, mut value: u64) {
    let low6 = (value & 0x3f) as u8;
    value >>= 6;
    let mut first = ((flag & 1) << 7) | low6;
    if value == 0 {
        out.push(first);
        return;
    }
    first |= 0x40;
    out.push(first);

    for _ in 0..6 {
        let mut b = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            out.push(b);
            return;
        }
        b |= 0x80;
        out.push(b);
    }
    out.push((value & 0xff) as u8);
}

pub fn read_b1vu56(data: &[u8], pos: &mut usize) -> Option<(u8, u64)> {
    let first = *data.get(*pos)?;
    *pos += 1;
    let flag = (first >> 7) & 1;
    let mut result: u64 = (first & 0x3f) as u64;
    if (first & 0x40) == 0 {
        return Some((flag, result));
    }

    let mut shift: u32 = 6;
    for i in 0..7 {
        let b = *data.get(*pos)?;
        *pos += 1;
        if i < 6 {
            result |= ((b & 0x7f) as u64).checked_shl(shift)?;
            if (b & 0x80) == 0 {
                return Some((flag, result));
            }
            shift += 7;
        } else {
            result |= (b as u64).checked_shl(48)?;
            return Some((flag, result));
        }
    }
    None
}

pub fn parse_logical_clock_table(data: &[u8]) -> Option<(usize, Vec<LogicalClockBase>)> {
    if data.is_empty() || (data[0] & 0x80) != 0 || data.len() < 4 {
        return None;
    }
    let offset = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let mut pos = 4usize.checked_add(offset)?;
    let len = read_vu57(data, &mut pos)? as usize;
    if len == 0 {
        return None;
    }

    let mut table = Vec::with_capacity(len);
    for _ in 0..len {
        let sid = read_vu57(data, &mut pos)?;
        let time = read_vu57(data, &mut pos)?;
        table.push(LogicalClockBase { sid, time });
    }

    Some((offset, table))
}

pub fn first_logical_clock_sid_time(data: &[u8]) -> Option<(u64, u64)> {
    let (_, table) = parse_logical_clock_table(data)?;
    let first = table.first()?;
    Some((first.sid, first.time))
}

/// Returns the first model clock `(sid, time)` for both logical and server
/// structural model encodings.
///
/// - Logical model preamble: 4-byte clock-table offset followed by table.
/// - Server model preamble: marker byte with MSB set, then `vu57(time)`.
pub fn first_model_clock_sid_time(data: &[u8]) -> Option<(u64, u64)> {
    let first = *data.first()?;
    if (first & 0x80) != 0 {
        let mut pos = 1usize;
        let time = read_vu57(data, &mut pos)?;
        // Upstream server-clock session id is fixed to 1.
        return Some((1, time));
    }
    first_logical_clock_sid_time(data)
}
