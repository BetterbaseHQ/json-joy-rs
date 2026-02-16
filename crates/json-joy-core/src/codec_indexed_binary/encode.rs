use std::collections::HashMap;

use crate::crdt_binary::write_b1vu56;
use crate::model_runtime::types::{ConCell, Id, RuntimeNode};
use crate::model_runtime::RuntimeModel;

use super::types::{
    encode_clock_table, encode_indexed_id, group_arr_chunks, group_bin_chunks, group_str_chunks,
    to_base36, write_json_pack_any, write_text_like_json_pack, write_type_len, write_uint_major,
    IndexedBinaryCodecError, IndexedFields,
};

pub fn encode_model_binary_to_fields(
    model_binary: &[u8],
) -> Result<IndexedFields, IndexedBinaryCodecError> {
    let runtime = RuntimeModel::from_model_binary(model_binary)?;
    if runtime.server_clock_time.is_some() {
        return Err(IndexedBinaryCodecError::InvalidFields);
    }
    if runtime.clock_table.is_empty() {
        return Err(IndexedBinaryCodecError::InvalidFields);
    }

    let mut out = IndexedFields::new();
    out.insert("c".to_string(), encode_clock_table(&runtime.clock_table));

    let mut index_by_sid: HashMap<u64, u64> = HashMap::new();
    for (idx, c) in runtime.clock_table.iter().enumerate() {
        index_by_sid.insert(c.sid, idx as u64);
    }

    if let Some(root) = runtime.root {
        if root.sid != 0 {
            out.insert("r".to_string(), encode_indexed_id(root, &index_by_sid)?);
        }
    }

    let mut nodes: Vec<(&Id, &RuntimeNode)> = runtime.nodes.iter().collect();
    nodes.sort_by_key(|(id, _)| (id.sid, id.time));
    for (id, node) in nodes {
        let field = format!(
            "{}_{}",
            to_base36(
                *index_by_sid
                    .get(&id.sid)
                    .ok_or(IndexedBinaryCodecError::InvalidFields)?
            ),
            to_base36(id.time)
        );
        let payload = encode_node_payload(node, &index_by_sid)?;
        out.insert(field, payload);
    }

    Ok(out)
}

fn encode_node_payload(
    node: &RuntimeNode,
    index_by_sid: &HashMap<u64, u64>,
) -> Result<Vec<u8>, IndexedBinaryCodecError> {
    let mut out = Vec::new();
    match node {
        RuntimeNode::Con(ConCell::Json(v)) => {
            write_type_len(&mut out, 0, 0);
            write_json_pack_any(&mut out, v);
        }
        RuntimeNode::Con(ConCell::Ref(id)) => {
            write_type_len(&mut out, 0, 1);
            out.extend_from_slice(&encode_indexed_id(*id, index_by_sid)?);
        }
        RuntimeNode::Con(ConCell::Undef) => {
            write_type_len(&mut out, 0, 0);
            out.push(0xf7);
        }
        RuntimeNode::Val(child) => {
            write_type_len(&mut out, 1, 0);
            out.extend_from_slice(&encode_indexed_id(*child, index_by_sid)?);
        }
        RuntimeNode::Obj(entries) => {
            write_type_len(&mut out, 2, entries.len() as u64);
            for (k, v) in entries {
                write_text_like_json_pack(&mut out, k);
                out.extend_from_slice(&encode_indexed_id(*v, index_by_sid)?);
            }
        }
        RuntimeNode::Vec(elements) => {
            let len = elements.keys().max().map(|v| v + 1).unwrap_or(0);
            write_type_len(&mut out, 3, len);
            for i in 0..len {
                if let Some(id) = elements.get(&i) {
                    out.push(1);
                    out.extend_from_slice(&encode_indexed_id(*id, index_by_sid)?);
                } else {
                    out.push(0);
                }
            }
        }
        RuntimeNode::Str(atoms) => {
            let chunks = group_str_chunks(atoms);
            write_type_len(&mut out, 4, chunks.len() as u64);
            for ch in chunks {
                out.extend_from_slice(&encode_indexed_id(ch.id, index_by_sid)?);
                if let Some(text) = ch.text {
                    write_text_like_json_pack(&mut out, &text);
                } else {
                    write_uint_major(&mut out, 0, ch.span);
                }
            }
        }
        RuntimeNode::Bin(atoms) => {
            let chunks = group_bin_chunks(atoms);
            write_type_len(&mut out, 5, chunks.len() as u64);
            for ch in chunks {
                out.extend_from_slice(&encode_indexed_id(ch.id, index_by_sid)?);
                match ch.bytes {
                    Some(bytes) => {
                        write_b1vu56(&mut out, 0, ch.span);
                        out.extend_from_slice(&bytes);
                    }
                    None => write_b1vu56(&mut out, 1, ch.span),
                }
            }
        }
        RuntimeNode::Arr(atoms) => {
            let chunks = group_arr_chunks(atoms);
            write_type_len(&mut out, 6, chunks.len() as u64);
            for ch in chunks {
                out.extend_from_slice(&encode_indexed_id(ch.id, index_by_sid)?);
                match ch.values {
                    Some(values) => {
                        write_b1vu56(&mut out, 0, ch.span);
                        for v in values {
                            out.extend_from_slice(&encode_indexed_id(v, index_by_sid)?);
                        }
                    }
                    None => write_b1vu56(&mut out, 1, ch.span),
                }
            }
        }
    }
    Ok(out)
}
