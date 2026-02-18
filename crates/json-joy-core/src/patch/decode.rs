use crate::crdt_binary::BinaryCursor;
type Reader<'a> = BinaryCursor<'a>;

fn cbor_to_json(v: PackValue) -> Result<serde_json::Value, PatchError> {
    Ok(json_joy_json_pack::cbor_to_json_owned(v))
}

fn decode_id(reader: &mut BinaryCursor<'_>, patch_sid: u64) -> Result<Timestamp, PatchError> {
    let (flag, time) = reader.b1vu56().ok_or(PatchError::Overflow)?;
    if flag == 1 {
        let sid = reader.vu57().ok_or(PatchError::Overflow)?;
        Ok(Timestamp { sid, time })
    } else {
        Ok(Timestamp {
            sid: patch_sid,
            time,
        })
    }
}

type DecodedPatchPayload = (u64, u64, u64, u64, Vec<u8>, Vec<DecodedOp>);

fn decode_patch(reader: &mut BinaryCursor<'_>) -> Result<DecodedPatchPayload, PatchError> {
    let sid = reader.vu57().ok_or(PatchError::Overflow)?;
    let time = reader.vu57().ok_or(PatchError::Overflow)?;

    // meta is a CBOR value (typically undefined or [meta])
    let _meta = reader.read_one_cbor().ok_or(PatchError::InvalidCbor)?;

    let ops_len = reader.vu57().ok_or(PatchError::Overflow)?;
    let mut span: u64 = 0;
    let mut opcodes = Vec::with_capacity(ops_len as usize);
    let mut decoded_ops = Vec::with_capacity(ops_len as usize);
    let mut op_time = time;
    for _ in 0..ops_len {
        let op_id = Timestamp { sid, time: op_time };
        let (opcode, decoded, op_span) = decode_op(reader, sid, op_id)?;
        opcodes.push(opcode);
        decoded_ops.push(decoded);
        span = span.checked_add(op_span).ok_or(PatchError::Overflow)?;
        op_time = op_time.checked_add(op_span).ok_or(PatchError::Overflow)?;
    }
    Ok((sid, time, ops_len, span, opcodes, decoded_ops))
}

fn read_len_from_low3_or_var(
    reader: &mut BinaryCursor<'_>,
    octet: u8,
) -> Result<u64, PatchError> {
    let low = (octet & 0b111) as u64;
    if low == 0 {
        reader.vu57().ok_or(PatchError::Overflow)
    } else {
        Ok(low)
    }
}

fn decode_op(
    reader: &mut BinaryCursor<'_>,
    patch_sid: u64,
    op_id: Timestamp,
) -> Result<(u8, DecodedOp, u64), PatchError> {
    let octet = reader.u8().ok_or(PatchError::Overflow)?;
    let opcode = octet >> 3;

    match opcode {
        // new_con
        0 => {
            let low = octet & 0b111;
            let value = if low == 0 {
                // CBOR undefined (0xf7) is used by json-joy diffs for object-key
                // deletion semantics. Preserve it explicitly to allow canonical
                // binary parity in native patch encoding.
                if reader.peek_u8() == Some(0xf7) {
                    reader.u8().ok_or(PatchError::Overflow)?;
                    ConValue::Undef
                } else {
                    ConValue::Json(cbor_to_json(
                        reader.read_one_cbor().ok_or(PatchError::InvalidCbor)?,
                    )?)
                }
            } else {
                ConValue::Ref(decode_id(reader, patch_sid)?)
            };
            Ok((opcode, DecodedOp::NewCon { id: op_id, value }, 1))
        }
        // new_val
        1 => Ok((opcode, DecodedOp::NewVal { id: op_id }, 1)),
        // new_obj
        2 => Ok((opcode, DecodedOp::NewObj { id: op_id }, 1)),
        // new_vec
        3 => Ok((opcode, DecodedOp::NewVec { id: op_id }, 1)),
        // new_str
        4 => Ok((opcode, DecodedOp::NewStr { id: op_id }, 1)),
        // new_bin
        5 => Ok((opcode, DecodedOp::NewBin { id: op_id }, 1)),
        // new_arr
        6 => Ok((opcode, DecodedOp::NewArr { id: op_id }, 1)),
        // ins_val
        9 => {
            let obj = decode_id(reader, patch_sid)?;
            let val = decode_id(reader, patch_sid)?;
            Ok((
                opcode,
                DecodedOp::InsVal {
                    id: op_id,
                    obj,
                    val,
                },
                1,
            ))
        }
        // ins_obj
        10 => {
            let len = read_len_from_low3_or_var(reader, octet)?;
            let obj = decode_id(reader, patch_sid)?;
            let mut data = Vec::with_capacity(len as usize);
            for _ in 0..len {
                let key = match reader.read_one_cbor().ok_or(PatchError::InvalidCbor)? {
                    PackValue::Str(s) => s,
                    _ => return Err(PatchError::InvalidCbor),
                };
                let value = decode_id(reader, patch_sid)?;
                data.push((key, value));
            }
            Ok((
                opcode,
                DecodedOp::InsObj {
                    id: op_id,
                    obj,
                    data,
                },
                1,
            ))
        }
        // ins_vec
        11 => {
            let len = read_len_from_low3_or_var(reader, octet)?;
            let obj = decode_id(reader, patch_sid)?;
            let mut data = Vec::with_capacity(len as usize);
            for _ in 0..len {
                let idx = reader.u8().ok_or(PatchError::Overflow)? as u64;
                let value = decode_id(reader, patch_sid)?;
                data.push((idx, value));
            }
            Ok((
                opcode,
                DecodedOp::InsVec {
                    id: op_id,
                    obj,
                    data,
                },
                1,
            ))
        }
        // ins_str
        12 => {
            let len = read_len_from_low3_or_var(reader, octet)? as usize;
            let obj = decode_id(reader, patch_sid)?;
            let reference = decode_id(reader, patch_sid)?;
            let bytes = reader.read_bytes(len).ok_or(PatchError::Overflow)?;
            let data = String::from_utf8(bytes.to_vec()).map_err(|_| PatchError::InvalidCbor)?;
            // Upstream JS patch op span for strings is UTF-16 code unit length.
            let span = data.encode_utf16().count() as u64;
            Ok((
                opcode,
                DecodedOp::InsStr {
                    id: op_id,
                    obj,
                    reference,
                    data,
                },
                span,
            ))
        }
        // ins_bin
        13 => {
            let len = read_len_from_low3_or_var(reader, octet)? as usize;
            let obj = decode_id(reader, patch_sid)?;
            let reference = decode_id(reader, patch_sid)?;
            let data = reader.read_bytes(len).ok_or(PatchError::Overflow)?.to_vec();
            Ok((
                opcode,
                DecodedOp::InsBin {
                    id: op_id,
                    obj,
                    reference,
                    data,
                },
                len as u64,
            ))
        }
        // ins_arr
        14 => {
            let len = read_len_from_low3_or_var(reader, octet)?;
            let obj = decode_id(reader, patch_sid)?;
            let reference = decode_id(reader, patch_sid)?;
            let mut data = Vec::with_capacity(len as usize);
            for _ in 0..len {
                data.push(decode_id(reader, patch_sid)?);
            }
            Ok((
                opcode,
                DecodedOp::InsArr {
                    id: op_id,
                    obj,
                    reference,
                    data,
                },
                len,
            ))
        }
        // upd_arr
        15 => {
            let obj = decode_id(reader, patch_sid)?;
            let reference = decode_id(reader, patch_sid)?;
            let val = decode_id(reader, patch_sid)?;
            Ok((
                opcode,
                DecodedOp::UpdArr {
                    id: op_id,
                    obj,
                    reference,
                    val,
                },
                1,
            ))
        }
        // del
        16 => {
            let len = read_len_from_low3_or_var(reader, octet)?;
            let obj = decode_id(reader, patch_sid)?;
            let mut what = Vec::with_capacity(len as usize);
            for _ in 0..len {
                let start = decode_id(reader, patch_sid)?;
                let span = reader.vu57().ok_or(PatchError::Overflow)?;
                what.push(Timespan {
                    sid: start.sid,
                    time: start.time,
                    span,
                });
            }
            Ok((
                opcode,
                DecodedOp::Del {
                    id: op_id,
                    obj,
                    what,
                },
                1,
            ))
        }
        // nop
        17 => {
            let len = read_len_from_low3_or_var(reader, octet)?;
            Ok((opcode, DecodedOp::Nop { id: op_id, len }, len))
        }
        _ => Err(PatchError::UnknownOpcode(opcode)),
    }
}
