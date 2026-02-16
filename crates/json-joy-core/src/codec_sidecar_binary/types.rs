use std::collections::HashMap;

use ciborium::value::Value as CborValue;
use serde_json::Value;
use thiserror::Error;

use crate::crdt_binary::{read_b1vu56, read_vu57, write_b1vu56, write_vu57, LogicalClockBase};
use crate::model::ModelError;
use crate::model_runtime::types::{ArrAtom, BinAtom, Id, StrAtom};

#[derive(Debug, Error)]
pub enum SidecarBinaryCodecError {
    #[error("invalid sidecar payload")]
    InvalidPayload,
    #[error("model runtime failure: {0}")]
    Model(#[from] ModelError),
}

pub(super) struct ClockEncCtx {
    pub table: Vec<LogicalClockBase>,
    by_sid: HashMap<u64, usize>,
    local_base: u64,
}

impl ClockEncCtx {
    pub(super) fn new(clock_table: &[LogicalClockBase]) -> Result<Self, SidecarBinaryCodecError> {
        let local = clock_table
            .first()
            .ok_or(SidecarBinaryCodecError::InvalidPayload)?;
        let mut table = Vec::with_capacity(clock_table.len());
        let mut by_sid = HashMap::new();
        for (idx, c) in clock_table.iter().enumerate() {
            table.push(*c);
            by_sid.insert(c.sid, idx + 1);
        }
        Ok(Self {
            table,
            by_sid,
            local_base: local.time,
        })
    }

    pub(super) fn append(
        &mut self,
        id: Id,
        out: &mut Vec<u8>,
    ) -> Result<(), SidecarBinaryCodecError> {
        if id.sid == 0 {
            write_sidecar_id(out, 0, id.time);
            return Ok(());
        }
        let idx = match self.by_sid.get(&id.sid) {
            Some(v) => *v,
            None => {
                self.table.push(LogicalClockBase {
                    sid: id.sid,
                    time: self.local_base,
                });
                let n = self.table.len();
                self.by_sid.insert(id.sid, n);
                n
            }
        };
        let base = self.table[idx - 1].time;
        let diff = base
            .checked_sub(id.time)
            .ok_or(SidecarBinaryCodecError::InvalidPayload)?;
        write_sidecar_id(out, idx as u64, diff);
        Ok(())
    }
}

pub(super) fn write_sidecar_id(out: &mut Vec<u8>, session_index: u64, time_diff: u64) {
    if session_index <= 0b111 && time_diff <= 0b1111 {
        out.push(((session_index as u8) << 4) | (time_diff as u8));
    } else {
        write_b1vu56(out, 1, session_index);
        write_vu57(out, time_diff);
    }
}

pub(super) struct MetaCursor<'a> {
    pub data: &'a [u8],
    pub pos: usize,
}

impl<'a> MetaCursor<'a> {
    pub(super) fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub(super) fn u8(&mut self) -> Result<u8, ()> {
        let b = *self.data.get(self.pos).ok_or(())?;
        self.pos += 1;
        Ok(b)
    }

    pub(super) fn peek(&self) -> Option<u8> {
        self.data.get(self.pos).copied()
    }

    pub(super) fn read_len(&mut self, minor: u8) -> Result<u64, SidecarBinaryCodecError> {
        Ok(match minor {
            0..=23 => minor as u64,
            24 => self
                .u8()
                .map_err(|_| SidecarBinaryCodecError::InvalidPayload)? as u64,
            25 => {
                let a = self
                    .u8()
                    .map_err(|_| SidecarBinaryCodecError::InvalidPayload)?
                    as u64;
                let b = self
                    .u8()
                    .map_err(|_| SidecarBinaryCodecError::InvalidPayload)?
                    as u64;
                (a << 8) | b
            }
            26 => {
                let a = self
                    .u8()
                    .map_err(|_| SidecarBinaryCodecError::InvalidPayload)?
                    as u64;
                let b = self
                    .u8()
                    .map_err(|_| SidecarBinaryCodecError::InvalidPayload)?
                    as u64;
                let c = self
                    .u8()
                    .map_err(|_| SidecarBinaryCodecError::InvalidPayload)?
                    as u64;
                let d = self
                    .u8()
                    .map_err(|_| SidecarBinaryCodecError::InvalidPayload)?
                    as u64;
                (a << 24) | (b << 16) | (c << 8) | d
            }
            _ => return Err(SidecarBinaryCodecError::InvalidPayload),
        })
    }

    pub(super) fn is_eof(&self) -> bool {
        self.pos == self.data.len()
    }
}

pub(super) fn decode_sidecar_id(
    cur: &mut MetaCursor<'_>,
    table: &[LogicalClockBase],
) -> Result<Id, SidecarBinaryCodecError> {
    let first = cur
        .u8()
        .map_err(|_| SidecarBinaryCodecError::InvalidPayload)?;
    let (session_index, time_diff) = if first <= 0x7f {
        ((first >> 4) as u64, (first & 0x0f) as u64)
    } else {
        cur.pos -= 1;
        let (flag, x) =
            read_b1vu56(cur.data, &mut cur.pos).ok_or(SidecarBinaryCodecError::InvalidPayload)?;
        if flag != 1 {
            return Err(SidecarBinaryCodecError::InvalidPayload);
        }
        let y = read_vu57(cur.data, &mut cur.pos).ok_or(SidecarBinaryCodecError::InvalidPayload)?;
        (x, y)
    };

    if session_index == 0 {
        return Ok(Id {
            sid: 0,
            time: time_diff,
        });
    }
    let base = table
        .get(session_index as usize - 1)
        .ok_or(SidecarBinaryCodecError::InvalidPayload)?;
    let time = base
        .time
        .checked_sub(time_diff)
        .ok_or(SidecarBinaryCodecError::InvalidPayload)?;
    Ok(Id {
        sid: base.sid,
        time,
    })
}

pub(super) fn write_type_len(out: &mut Vec<u8>, major: u8, len: u64) {
    if len < 24 {
        out.push((major << 5) | (len as u8));
    } else if len <= 0xff {
        out.push((major << 5) | 24);
        out.push(len as u8);
    } else if len <= 0xffff {
        out.push((major << 5) | 25);
        out.push(((len >> 8) & 0xff) as u8);
        out.push((len & 0xff) as u8);
    } else {
        out.push((major << 5) | 26);
        out.push(((len >> 24) & 0xff) as u8);
        out.push(((len >> 16) & 0xff) as u8);
        out.push(((len >> 8) & 0xff) as u8);
        out.push((len & 0xff) as u8);
    }
}

pub(super) fn cbor_from_json(v: &Value) -> CborValue {
    json_joy_json_pack::json_to_cbor(v)
}

pub(super) fn json_from_cbor(v: &CborValue) -> Result<Value, SidecarBinaryCodecError> {
    json_joy_json_pack::cbor_to_json(v).map_err(|_| SidecarBinaryCodecError::InvalidPayload)
}

#[derive(Debug)]
pub(super) struct StrChunk {
    pub id: Id,
    pub span: u64,
    pub text: Option<String>,
}
#[derive(Debug)]
pub(super) struct BinChunk {
    pub id: Id,
    pub span: u64,
    pub bytes: Option<Vec<u8>>,
}
#[derive(Debug)]
pub(super) struct ArrChunk {
    pub id: Id,
    pub span: u64,
    pub values: Option<Vec<Id>>,
}

pub(super) fn group_str_chunks(atoms: &[StrAtom]) -> Vec<StrChunk> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < atoms.len() {
        let start = &atoms[i];
        let mut j = i + 1;
        while j < atoms.len()
            && atoms[j - 1].slot.sid == atoms[j].slot.sid
            && atoms[j - 1].ch.is_some() == atoms[j].ch.is_some()
            && atoms[j].slot.time == atoms[j - 1].slot.time + 1
        {
            j += 1;
        }
        if start.ch.is_some() {
            let mut s = String::new();
            for a in &atoms[i..j] {
                if let Some(ch) = a.ch {
                    s.push(ch);
                }
            }
            out.push(StrChunk {
                id: start.slot,
                span: (j - i) as u64,
                text: Some(s),
            });
        } else {
            out.push(StrChunk {
                id: start.slot,
                span: (j - i) as u64,
                text: None,
            });
        }
        i = j;
    }
    out
}

pub(super) fn group_bin_chunks(atoms: &[BinAtom]) -> Vec<BinChunk> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < atoms.len() {
        let start = &atoms[i];
        let mut j = i + 1;
        while j < atoms.len()
            && atoms[j - 1].slot.sid == atoms[j].slot.sid
            && atoms[j - 1].byte.is_some() == atoms[j].byte.is_some()
            && atoms[j].slot.time == atoms[j - 1].slot.time + 1
        {
            j += 1;
        }
        if start.byte.is_some() {
            let mut bytes = Vec::new();
            for a in &atoms[i..j] {
                if let Some(b) = a.byte {
                    bytes.push(b);
                }
            }
            out.push(BinChunk {
                id: start.slot,
                span: (j - i) as u64,
                bytes: Some(bytes),
            });
        } else {
            out.push(BinChunk {
                id: start.slot,
                span: (j - i) as u64,
                bytes: None,
            });
        }
        i = j;
    }
    out
}

pub(super) fn group_arr_chunks(atoms: &[ArrAtom]) -> Vec<ArrChunk> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < atoms.len() {
        let start = &atoms[i];
        let mut j = i + 1;
        while j < atoms.len()
            && atoms[j - 1].slot.sid == atoms[j].slot.sid
            && atoms[j - 1].value.is_some() == atoms[j].value.is_some()
            && atoms[j].slot.time == atoms[j - 1].slot.time + 1
        {
            j += 1;
        }
        if start.value.is_some() {
            let mut values = Vec::new();
            for a in &atoms[i..j] {
                if let Some(v) = a.value {
                    values.push(v);
                }
            }
            out.push(ArrChunk {
                id: start.slot,
                span: (j - i) as u64,
                values: Some(values),
            });
        } else {
            out.push(ArrChunk {
                id: start.slot,
                span: (j - i) as u64,
                values: None,
            });
        }
        i = j;
    }
    out
}
