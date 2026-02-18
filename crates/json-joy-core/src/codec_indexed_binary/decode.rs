use std::collections::HashMap;

use json_joy_json_pack::{decode_cbor_value_with_consumed, PackValue};
use serde_json::Value;

use crate::crdt_binary::{read_b1vu56, read_vu57, LogicalClockBase};
use crate::model_runtime::types::{ArrAtom, BinAtom, ConCell, Id, RuntimeNode, StrAtom};
use crate::model_runtime::RuntimeModel;

use super::types::{
    decode_clock_table, decode_indexed_id, from_base36, json_from_cbor, IndexedBinaryCodecError,
    IndexedFields,
};

pub fn decode_fields_to_model_binary(
    fields: &IndexedFields,
) -> Result<Vec<u8>, IndexedBinaryCodecError> {
    let c = fields
        .get("c")
        .ok_or(IndexedBinaryCodecError::InvalidFields)?;
    let clock_table = decode_clock_table(c)?;
    if clock_table.is_empty() {
        return Err(IndexedBinaryCodecError::InvalidFields);
    }

    let root = match fields.get("r") {
        Some(bytes) => Some(decode_indexed_id(bytes, &clock_table)?),
        None => None,
    };

    let mut nodes: HashMap<Id, RuntimeNode> = HashMap::new();
    for (field, payload) in fields {
        if field == "c" || field == "r" {
            continue;
        }
        let id = parse_field_id(field, &clock_table)?;
        let node = decode_node_payload(payload, &clock_table)?;
        nodes.insert(id, node);
    }

    let runtime = RuntimeModel {
        nodes,
        root,
        clock: Default::default(),
        fallback_view: Value::Null,
        infer_empty_object_root: false,
        clock_table,
        server_clock_time: None,
    };

    runtime
        .to_model_binary_like()
        .map_err(IndexedBinaryCodecError::from)
}

fn parse_field_id(
    field: &str,
    clock_table: &[LogicalClockBase],
) -> Result<Id, IndexedBinaryCodecError> {
    let (sid_idx_s, time_s) = field
        .split_once('_')
        .ok_or(IndexedBinaryCodecError::InvalidFields)?;
    let sid_idx = from_base36(sid_idx_s).ok_or(IndexedBinaryCodecError::InvalidFields)? as usize;
    let time = from_base36(time_s).ok_or(IndexedBinaryCodecError::InvalidFields)?;
    let sid = clock_table
        .get(sid_idx)
        .ok_or(IndexedBinaryCodecError::InvalidFields)?
        .sid;
    Ok(Id { sid, time })
}

struct DecodeCursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> DecodeCursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn u8(&mut self) -> Result<u8, IndexedBinaryCodecError> {
        let b = *self
            .data
            .get(self.pos)
            .ok_or(IndexedBinaryCodecError::InvalidNode)?;
        self.pos += 1;
        Ok(b)
    }

    fn read_indexed_id(
        &mut self,
        table: &[LogicalClockBase],
    ) -> Result<Id, IndexedBinaryCodecError> {
        let first = self.u8()?;
        let (x, y) = if first <= 0x7f {
            ((first >> 4) as u64, (first & 0x0f) as u64)
        } else {
            self.pos -= 1;
            let (flag, x) = read_b1vu56(self.data, &mut self.pos)
                .ok_or(IndexedBinaryCodecError::InvalidNode)?;
            if flag != 1 {
                return Err(IndexedBinaryCodecError::InvalidNode);
            }
            let y =
                read_vu57(self.data, &mut self.pos).ok_or(IndexedBinaryCodecError::InvalidNode)?;
            (x, y)
        };
        let sid = table
            .get(x as usize)
            .ok_or(IndexedBinaryCodecError::InvalidNode)?
            .sid;
        Ok(Id { sid, time: y })
    }

    fn read_len(&mut self, minor: u8) -> Result<u64, IndexedBinaryCodecError> {
        match minor {
            0..=23 => Ok(minor as u64),
            24 => Ok(self.u8()? as u64),
            25 => {
                let a = self.u8()? as u64;
                let b = self.u8()? as u64;
                Ok((a << 8) | b)
            }
            26 => {
                let a = self.u8()? as u64;
                let b = self.u8()? as u64;
                let c = self.u8()? as u64;
                let d = self.u8()? as u64;
                Ok((a << 24) | (b << 16) | (c << 8) | d)
            }
            _ => Err(IndexedBinaryCodecError::InvalidNode),
        }
    }

    fn read_one_cbor(&mut self) -> Result<PackValue, IndexedBinaryCodecError> {
        let (value, consumed) = decode_cbor_value_with_consumed(&self.data[self.pos..])
            .map_err(|_| IndexedBinaryCodecError::InvalidNode)?;
        self.pos += consumed;
        Ok(value)
    }

    fn is_eof(&self) -> bool {
        self.pos == self.data.len()
    }
}

fn decode_node_payload(
    payload: &[u8],
    clock_table: &[LogicalClockBase],
) -> Result<RuntimeNode, IndexedBinaryCodecError> {
    let mut r = DecodeCursor::new(payload);
    let octet = r.u8()?;
    let major = octet >> 5;
    let len = r.read_len(octet & 0x1f)?;
    let node = match major {
        0 => {
            if len == 0 {
                if r.data.get(r.pos) == Some(&0xf7) {
                    r.pos += 1;
                    RuntimeNode::Con(ConCell::Undef)
                } else {
                    let cbor = r.read_one_cbor()?;
                    RuntimeNode::Con(ConCell::Json(json_from_cbor(&cbor)?))
                }
            } else {
                let id = r.read_indexed_id(clock_table)?;
                RuntimeNode::Con(ConCell::Ref(id))
            }
        }
        1 => {
            let child = r.read_indexed_id(clock_table)?;
            RuntimeNode::Val(child)
        }
        2 => {
            let mut entries = Vec::with_capacity(len as usize);
            for _ in 0..len {
                let key = match r.read_one_cbor()? {
                    PackValue::Str(s) => s,
                    _ => return Err(IndexedBinaryCodecError::InvalidNode),
                };
                let id = r.read_indexed_id(clock_table)?;
                entries.push((key, id));
            }
            RuntimeNode::Obj(entries)
        }
        3 => {
            let mut map = std::collections::BTreeMap::new();
            for i in 0..len {
                let flag = r.u8()?;
                if flag != 0 {
                    let id = r.read_indexed_id(clock_table)?;
                    map.insert(i, id);
                }
            }
            RuntimeNode::Vec(map)
        }
        4 => {
            let mut atoms = Vec::new();
            for _ in 0..len {
                let id = r.read_indexed_id(clock_table)?;
                match r.read_one_cbor()? {
                    PackValue::Str(s) => {
                        let chars: Vec<char> = s.chars().collect();
                        for (i, ch) in chars.iter().enumerate() {
                            atoms.push(StrAtom {
                                slot: Id {
                                    sid: id.sid,
                                    time: id.time + i as u64,
                                },
                                ch: Some(*ch),
                            });
                        }
                    }
                    PackValue::Integer(n) if n >= 0 => {
                        let span = n as u64;
                        for i in 0..span {
                            atoms.push(StrAtom {
                                slot: Id {
                                    sid: id.sid,
                                    time: id.time + i,
                                },
                                ch: None,
                            });
                        }
                    }
                    PackValue::UInteger(span) => {
                        for i in 0..span {
                            atoms.push(StrAtom {
                                slot: Id {
                                    sid: id.sid,
                                    time: id.time + i,
                                },
                                ch: None,
                            });
                        }
                    }
                    _ => return Err(IndexedBinaryCodecError::InvalidNode),
                }
            }
            RuntimeNode::Str(atoms)
        }
        5 => {
            let mut atoms = Vec::new();
            for _ in 0..len {
                let id = r.read_indexed_id(clock_table)?;
                let (deleted, span) =
                    read_b1vu56(r.data, &mut r.pos).ok_or(IndexedBinaryCodecError::InvalidNode)?;
                if deleted == 1 {
                    for i in 0..span {
                        atoms.push(BinAtom {
                            slot: Id {
                                sid: id.sid,
                                time: id.time + i,
                            },
                            byte: None,
                        });
                    }
                } else {
                    for i in 0..span {
                        let b = r.u8()?;
                        atoms.push(BinAtom {
                            slot: Id {
                                sid: id.sid,
                                time: id.time + i,
                            },
                            byte: Some(b),
                        });
                    }
                }
            }
            RuntimeNode::Bin(atoms)
        }
        6 => {
            let mut atoms = Vec::new();
            for _ in 0..len {
                let id = r.read_indexed_id(clock_table)?;
                let (deleted, span) =
                    read_b1vu56(r.data, &mut r.pos).ok_or(IndexedBinaryCodecError::InvalidNode)?;
                if deleted == 1 {
                    for i in 0..span {
                        atoms.push(ArrAtom {
                            slot: Id {
                                sid: id.sid,
                                time: id.time + i,
                            },
                            value: None,
                        });
                    }
                } else {
                    for i in 0..span {
                        let value = r.read_indexed_id(clock_table)?;
                        atoms.push(ArrAtom {
                            slot: Id {
                                sid: id.sid,
                                time: id.time + i,
                            },
                            value: Some(value),
                        });
                    }
                }
            }
            RuntimeNode::Arr(atoms)
        }
        _ => return Err(IndexedBinaryCodecError::InvalidNode),
    };
    if !r.is_eof() {
        return Err(IndexedBinaryCodecError::InvalidNode);
    }
    Ok(node)
}
