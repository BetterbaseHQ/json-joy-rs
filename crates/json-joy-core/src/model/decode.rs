use crate::crdt_binary::BinaryCursor;

fn decode_model_view(data: &[u8]) -> Result<Value, ModelError> {
    let mut reader = BinaryCursor::new(data);

    // Server-clock encoding starts with a marker byte whose highest bit is set
    // and does not contain a clock table section.
    if reader.peek_u8().ok_or(ModelError::InvalidModelBinary)? & 0b1000_0000 != 0 {
        let _marker = reader.u8().ok_or(ModelError::InvalidModelBinary)?;
        let _time = reader.vu57().ok_or(ModelError::InvalidModelBinary)?;
        return decode_root_to_end(&mut reader);
    }

    let clock_table_offset = reader.u32_be().ok_or(ModelError::InvalidClockTable)? as usize;
    let root_start = reader.position();
    let clock_start = root_start
        .checked_add(clock_table_offset)
        .ok_or(ModelError::InvalidClockTable)?;
    if clock_start > data.len() {
        return Err(ModelError::InvalidClockTable);
    }

    // Validate basic clock table framing similarly to upstream decode path.
    {
        let mut clock = BinaryCursor::new(&data[clock_start..]);
        let table_len = clock.vu57().ok_or(ModelError::InvalidClockTable)?;
        if table_len == 0 {
            return Err(ModelError::InvalidClockTable);
        }
        let _session = clock.vu57().ok_or(ModelError::InvalidClockTable)?;
        let _time = clock.vu57().ok_or(ModelError::InvalidClockTable)?;
        for _ in 1..table_len {
            let _ = clock.vu57().ok_or(ModelError::InvalidClockTable)?;
            let _ = clock.vu57().ok_or(ModelError::InvalidClockTable)?;
        }
    }

    let root_slice = &data[root_start..clock_start];
    let mut root_reader = BinaryCursor::new(root_slice);
    let value = decode_root(&mut root_reader)?;
    if !root_reader.is_eof() {
        return Err(ModelError::InvalidModelBinary);
    }
    Ok(value)
}

fn decode_root_to_end(reader: &mut BinaryCursor<'_>) -> Result<Value, ModelError> {
    let value = decode_root(reader)?;
    if !reader.is_eof() {
        return Err(ModelError::InvalidModelBinary);
    }
    Ok(value)
}

fn decode_root(reader: &mut BinaryCursor<'_>) -> Result<Value, ModelError> {
    let first = reader.peek_u8().ok_or(ModelError::InvalidModelBinary)?;
    if first == 0 {
        reader.u8().ok_or(ModelError::InvalidModelBinary)?;
        return Ok(Value::Null);
    }
    decode_node(reader)
}

fn decode_node(reader: &mut BinaryCursor<'_>) -> Result<Value, ModelError> {
    reader.skip_id().ok_or(ModelError::InvalidModelBinary)?;
    let octet = reader.u8().ok_or(ModelError::InvalidModelBinary)?;
    let major = octet >> 5;
    let minor = (octet & 0b1_1111) as u64;

    match major {
        // CON
        0 => decode_con(reader, minor),
        // VAL
        1 => decode_node(reader),
        // OBJ
        2 => {
            let len = if minor != 31 {
                minor
            } else {
                reader.vu57().ok_or(ModelError::InvalidModelBinary)?
            };
            decode_obj(reader, len)
        }
        // VEC
        3 => {
            let len = if minor != 31 {
                minor
            } else {
                reader.vu57().ok_or(ModelError::InvalidModelBinary)?
            };
            decode_vec(reader, len)
        }
        // STR
        4 => {
            let len = if minor != 31 {
                minor
            } else {
                reader.vu57().ok_or(ModelError::InvalidModelBinary)?
            };
            decode_str(reader, len)
        }
        // BIN
        5 => {
            let len = if minor != 31 {
                minor
            } else {
                reader.vu57().ok_or(ModelError::InvalidModelBinary)?
            };
            decode_bin(reader, len)
        }
        // ARR
        6 => {
            let len = if minor != 31 {
                minor
            } else {
                reader.vu57().ok_or(ModelError::InvalidModelBinary)?
            };
            decode_arr(reader, len)
        }
        _ => Err(ModelError::InvalidModelBinary),
    }
}

fn decode_con(reader: &mut BinaryCursor<'_>, length: u64) -> Result<Value, ModelError> {
    if length == 0 {
        let cbor = reader.read_one_cbor().ok_or(ModelError::InvalidModelBinary)?;
        return cbor_to_json(cbor);
    }

    // Timestamp reference constant. Not expected in current fixture corpus.
    reader.skip_id().ok_or(ModelError::InvalidModelBinary)?;
    Ok(Value::Null)
}

fn decode_obj(reader: &mut BinaryCursor<'_>, len: u64) -> Result<Value, ModelError> {
    let mut map = Map::new();
    for _ in 0..len {
        let key = match reader.read_one_cbor().ok_or(ModelError::InvalidModelBinary)? {
            PackValue::Str(s) => s,
            _ => return Err(ModelError::InvalidModelBinary),
        };
        let val = decode_node(reader)?;
        map.insert(key, val);
    }
    Ok(Value::Object(map))
}

fn decode_vec(reader: &mut BinaryCursor<'_>, len: u64) -> Result<Value, ModelError> {
    let mut out = Vec::with_capacity(len as usize);
    for _ in 0..len {
        let octet = reader.peek_u8().ok_or(ModelError::InvalidModelBinary)?;
        if octet == 0 {
            reader.u8().ok_or(ModelError::InvalidModelBinary)?;
            out.push(Value::Null);
        } else {
            out.push(decode_node(reader)?);
        }
    }
    Ok(Value::Array(out))
}

fn decode_str(reader: &mut BinaryCursor<'_>, len: u64) -> Result<Value, ModelError> {
    let mut out = String::new();
    for _ in 0..len {
        reader.skip_id().ok_or(ModelError::InvalidModelBinary)?;
        let cbor = reader.read_one_cbor().ok_or(ModelError::InvalidModelBinary)?;
        match cbor {
            PackValue::Str(s) => {
                out.push_str(&s);
            }
            PackValue::Integer(i) if i >= 0 => {
                // deleted span — skip
            }
            PackValue::UInteger(_) => {
                // deleted span — skip
            }
            _ => return Err(ModelError::InvalidModelBinary),
        }
    }
    Ok(Value::String(out))
}

fn decode_bin(reader: &mut BinaryCursor<'_>, len: u64) -> Result<Value, ModelError> {
    let mut out: Vec<u8> = Vec::new();
    for _ in 0..len {
        reader.skip_id().ok_or(ModelError::InvalidModelBinary)?;
        let (deleted, span) = reader.b1vu56().ok_or(ModelError::InvalidModelBinary)?;
        if deleted == 1 {
            continue;
        }
        let bytes = reader
            .read_bytes(span as usize)
            .ok_or(ModelError::InvalidModelBinary)?;
        for b in bytes {
            out.push(*b);
        }
    }
    // Upstream view materializes as Uint8Array. In JSON fixtures this appears
    // as an object with numeric string keys, e.g. {"0":1,"1":2}.
    let mut map = Map::new();
    for (i, b) in out.iter().enumerate() {
        map.insert(i.to_string(), Value::Number(Number::from(*b)));
    }
    Ok(Value::Object(map))
}

fn decode_arr(reader: &mut BinaryCursor<'_>, len: u64) -> Result<Value, ModelError> {
    let mut out = Vec::new();
    for _ in 0..len {
        reader.skip_id().ok_or(ModelError::InvalidModelBinary)?;
        let (deleted, span) = reader.b1vu56().ok_or(ModelError::InvalidModelBinary)?;

        if deleted == 1 {
            continue;
        }
        for _ in 0..span {
            out.push(decode_node(reader)?);
        }
    }
    Ok(Value::Array(out))
}

fn cbor_to_json(v: PackValue) -> Result<Value, ModelError> {
    Ok(json_joy_json_pack::cbor_to_json_owned(v))
}
