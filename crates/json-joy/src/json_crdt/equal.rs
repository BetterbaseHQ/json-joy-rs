//! Node equality helpers.
//!
//! Mirrors:
//! - `json-crdt/equal/cmp.ts`      → [`cmp`]
//! - `json-crdt/equal/cmpNode.ts`  → [`cmp_node`]
//!
//! `cmp` checks structural + optional value equality, ignoring CRDT metadata.
//! `cmp_node` checks CRDT metadata (timestamps), ignoring deep values.

use super::nodes::{ArrNode, BinNode, CrdtNode, StrNode};
#[cfg(test)]
use super::nodes::{ConNode, ObjNode};
use super::nodes::{NodeIndex, TsKey};
use crate::json_crdt_patch::clock::{equal as ts_equal, Ts};

/// Resolve a node from the index by `Ts`.
#[inline]
fn get_node<'a>(index: &'a NodeIndex, id: &Ts) -> Option<&'a CrdtNode> {
    index.get(&TsKey::from(*id))
}

// ── cmp ──────────────────────────────────────────────────────────────────

/// Deeply checks if two JSON CRDT nodes have the same schema and optionally
/// the same values.
///
/// When `compare_content` is `false`, only structural type-parity is checked
/// (same node type, same keys/length).  When `true`, leaf values are also
/// compared.
///
/// Mirrors `cmp` in `cmp.ts`.
pub fn cmp(a: &CrdtNode, b: &CrdtNode, compare_content: bool, index: &NodeIndex) -> bool {
    if std::ptr::eq(a as *const _, b as *const _) {
        return true;
    }
    match (a, b) {
        (CrdtNode::Con(na), CrdtNode::Con(nb)) => {
            if !compare_content {
                return true;
            }
            na.val == nb.val
        }
        (CrdtNode::Val(na), CrdtNode::Val(nb)) => {
            // Resolve the values both registers point to and recurse.
            let va = get_node(index, &na.val);
            let vb = get_node(index, &nb.val);
            match (va, vb) {
                (Some(ca), Some(cb)) => cmp(ca, cb, compare_content, index),
                (None, None) => true,
                _ => false,
            }
        }
        (CrdtNode::Str(na), CrdtNode::Str(nb)) => {
            if !compare_content {
                return true;
            }
            let sa = na.view_str();
            let sb = nb.view_str();
            sa.len() == sb.len() && sa == sb
        }
        (CrdtNode::Bin(na), CrdtNode::Bin(nb)) => {
            if !compare_content {
                return true;
            }
            let ba = na.view();
            let bb = nb.view();
            ba.len() == bb.len() && ba == bb
        }
        (CrdtNode::Obj(na), CrdtNode::Obj(nb)) => {
            let len1 = na.keys.len();
            let len2 = nb.keys.len();
            if len1 != len2 {
                return false;
            }
            for (key, id_a) in &na.keys {
                let id_b = match nb.keys.get(key) {
                    Some(id) => id,
                    None => return false,
                };
                let node_a = get_node(index, id_a);
                let node_b = get_node(index, id_b);
                match (node_a, node_b) {
                    (Some(ca), Some(cb)) => {
                        if !cmp(ca, cb, compare_content, index) {
                            return false;
                        }
                    }
                    (None, None) => {}
                    _ => return false,
                }
            }
            true
        }
        (CrdtNode::Vec(na), CrdtNode::Vec(nb)) => {
            let len1 = na.elements.len();
            let len2 = nb.elements.len();
            if len1 != len2 {
                return false;
            }
            for i in 0..len1 {
                let ea = na.elements[i];
                let eb = nb.elements[i];
                match (ea, eb) {
                    (Some(ia), Some(ib)) => {
                        let ca = get_node(index, &ia);
                        let cb = get_node(index, &ib);
                        match (ca, cb) {
                            (Some(na), Some(nb)) => {
                                if !cmp(na, nb, compare_content, index) {
                                    return false;
                                }
                            }
                            (None, None) => {}
                            _ => return false,
                        }
                    }
                    (None, None) => {}
                    _ => return false,
                }
            }
            true
        }
        (CrdtNode::Arr(na), CrdtNode::Arr(nb)) => {
            let va: Vec<Ts> = na
                .rga
                .iter_live()
                .filter_map(|c| c.data.as_ref())
                .flat_map(|v| v.iter().copied())
                .collect();
            let vb: Vec<Ts> = nb
                .rga
                .iter_live()
                .filter_map(|c| c.data.as_ref())
                .flat_map(|v| v.iter().copied())
                .collect();
            if va.len() != vb.len() {
                return false;
            }
            if !compare_content {
                return true;
            }
            for (id_a, id_b) in va.iter().zip(vb.iter()) {
                let ca = get_node(index, id_a);
                let cb = get_node(index, id_b);
                match (ca, cb) {
                    (Some(na), Some(nb)) => {
                        if !cmp(na, nb, compare_content, index) {
                            return false;
                        }
                    }
                    (None, None) => {}
                    _ => return false,
                }
            }
            true
        }
        _ => false, // different types
    }
}

// ── cmp_node ──────────────────────────────────────────────────────────────

/// Performs type and metadata shallow check of two JSON CRDT nodes.
///
/// Compares node type and their timestamps / structural metadata (like the
/// max chunk ID and live length for RGA nodes).  Does not compare values.
///
/// Mirrors `cmpNode` in `cmpNode.ts`.
pub fn cmp_node(a: &CrdtNode, b: &CrdtNode) -> bool {
    if std::ptr::eq(a as *const _, b as *const _) {
        return true;
    }
    match (a, b) {
        (CrdtNode::Con(na), CrdtNode::Con(nb)) => ts_equal(na.id, nb.id),
        (CrdtNode::Val(na), CrdtNode::Val(nb)) => {
            ts_equal(na.id, nb.id) && ts_equal(na.val, nb.val)
        }
        (CrdtNode::Str(na), CrdtNode::Str(nb)) => {
            if !ts_equal(na.id, nb.id) {
                return false;
            }
            cmp_rga_str(na, nb)
        }
        (CrdtNode::Bin(na), CrdtNode::Bin(nb)) => {
            if !ts_equal(na.id, nb.id) {
                return false;
            }
            cmp_rga_bin(na, nb)
        }
        (CrdtNode::Obj(na), CrdtNode::Obj(nb)) => {
            if !ts_equal(na.id, nb.id) {
                return false;
            }
            if na.keys.len() != nb.keys.len() {
                return false;
            }
            for (key, ts_a) in &na.keys {
                match nb.keys.get(key) {
                    Some(ts_b) if ts_equal(*ts_a, *ts_b) => {}
                    _ => return false,
                }
            }
            true
        }
        (CrdtNode::Vec(na), CrdtNode::Vec(nb)) => {
            if !ts_equal(na.id, nb.id) {
                return false;
            }
            let len = na.elements.len();
            if len != nb.elements.len() {
                return false;
            }
            for i in 0..len {
                match (na.elements[i], nb.elements[i]) {
                    (Some(a), Some(b)) if ts_equal(a, b) => {}
                    (None, None) => {}
                    _ => return false,
                }
            }
            true
        }
        (CrdtNode::Arr(na), CrdtNode::Arr(nb)) => {
            if !ts_equal(na.id, nb.id) {
                return false;
            }
            cmp_rga_arr(na, nb)
        }
        _ => false,
    }
}

// ── RGA comparison helpers ────────────────────────────────────────────────

/// Compare two StrNode RGAs by max-chunk-ID and live length.
///
/// Mirrors `cmpRga` in `cmpNode.ts`:
/// - If both have a last chunk, their IDs must match.
/// - `size()` (live char count) and chunk count must match.
fn cmp_rga_str(a: &StrNode, b: &StrNode) -> bool {
    let max_a = a.rga.last_chunk();
    let max_b = b.rga.last_chunk();
    match (max_a, max_b) {
        (Some(ca), Some(cb)) => {
            if !ts_equal(ca.id, cb.id) {
                return false;
            }
        }
        (None, None) => {}
        _ => return false,
    }
    a.size() == b.size() && a.rga.chunk_count() == b.rga.chunk_count()
}

/// Compare two BinNode RGAs by max-chunk-ID and live length.
fn cmp_rga_bin(a: &BinNode, b: &BinNode) -> bool {
    let max_a = a.rga.last_chunk();
    let max_b = b.rga.last_chunk();
    match (max_a, max_b) {
        (Some(ca), Some(cb)) => {
            if !ts_equal(ca.id, cb.id) {
                return false;
            }
        }
        (None, None) => {}
        _ => return false,
    }
    let len_a: usize = a
        .rga
        .iter_live()
        .filter_map(|c| c.data.as_ref())
        .map(|v| v.len())
        .sum();
    let len_b: usize = b
        .rga
        .iter_live()
        .filter_map(|c| c.data.as_ref())
        .map(|v| v.len())
        .sum();
    len_a == len_b && a.rga.chunk_count() == b.rga.chunk_count()
}

/// Compare two ArrNode RGAs by max-chunk-ID and live length.
fn cmp_rga_arr(a: &ArrNode, b: &ArrNode) -> bool {
    let max_a = a.rga.last_chunk();
    let max_b = b.rga.last_chunk();
    match (max_a, max_b) {
        (Some(ca), Some(cb)) => {
            if !ts_equal(ca.id, cb.id) {
                return false;
            }
        }
        (None, None) => {}
        _ => return false,
    }
    a.size() == b.size() && a.rga.chunk_count() == b.rga.chunk_count()
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json_crdt_patch::clock::ts;
    use crate::json_crdt_patch::operations::ConValue;
    use json_joy_json_pack::PackValue;
    use std::collections::BTreeMap;

    fn sid() -> u64 {
        999
    }

    // ── cmp tests ────────────────────────────────────────────────────────

    #[test]
    fn cmp_con_same_value() {
        let index = BTreeMap::default();
        let a = CrdtNode::Con(ConNode::new(
            ts(sid(), 1),
            ConValue::Val(PackValue::Integer(42)),
        ));
        let b = CrdtNode::Con(ConNode::new(
            ts(sid(), 2),
            ConValue::Val(PackValue::Integer(42)),
        ));
        assert!(cmp(&a, &b, true, &index));
    }

    #[test]
    fn cmp_con_different_value() {
        let index = BTreeMap::default();
        let a = CrdtNode::Con(ConNode::new(
            ts(sid(), 1),
            ConValue::Val(PackValue::Integer(1)),
        ));
        let b = CrdtNode::Con(ConNode::new(
            ts(sid(), 2),
            ConValue::Val(PackValue::Integer(2)),
        ));
        assert!(!cmp(&a, &b, true, &index));
    }

    #[test]
    fn cmp_con_no_content() {
        // With compareContent=false, different values should be "equal".
        let index = BTreeMap::default();
        let a = CrdtNode::Con(ConNode::new(
            ts(sid(), 1),
            ConValue::Val(PackValue::Integer(1)),
        ));
        let b = CrdtNode::Con(ConNode::new(
            ts(sid(), 2),
            ConValue::Val(PackValue::Integer(2)),
        ));
        assert!(cmp(&a, &b, false, &index));
    }

    #[test]
    fn cmp_different_types_false() {
        let index = BTreeMap::default();
        let a = CrdtNode::Con(ConNode::new(
            ts(sid(), 1),
            ConValue::Val(PackValue::Integer(1)),
        ));
        let b = CrdtNode::Str(StrNode::new(ts(sid(), 1)));
        assert!(!cmp(&a, &b, false, &index));
    }

    // ── cmp_node tests ───────────────────────────────────────────────────

    #[test]
    fn cmp_node_same_con_id() {
        let a = CrdtNode::Con(ConNode::new(
            ts(sid(), 5),
            ConValue::Val(PackValue::Integer(1)),
        ));
        let b = CrdtNode::Con(ConNode::new(
            ts(sid(), 5),
            ConValue::Val(PackValue::Integer(2)),
        ));
        // Same ID → true (cmpNode ignores values)
        assert!(cmp_node(&a, &b));
    }

    #[test]
    fn cmp_node_different_con_id() {
        let a = CrdtNode::Con(ConNode::new(
            ts(sid(), 5),
            ConValue::Val(PackValue::Integer(1)),
        ));
        let b = CrdtNode::Con(ConNode::new(
            ts(sid(), 6),
            ConValue::Val(PackValue::Integer(1)),
        ));
        assert!(!cmp_node(&a, &b));
    }

    #[test]
    fn cmp_node_different_types() {
        let a = CrdtNode::Con(ConNode::new(
            ts(sid(), 5),
            ConValue::Val(PackValue::Integer(1)),
        ));
        let b = CrdtNode::Str(StrNode::new(ts(sid(), 5)));
        assert!(!cmp_node(&a, &b));
    }

    #[test]
    fn cmp_node_identical_str_nodes() {
        // Two empty StrNodes with same ID should be equal.
        let a = CrdtNode::Str(StrNode::new(ts(sid(), 1)));
        let b = CrdtNode::Str(StrNode::new(ts(sid(), 1)));
        assert!(cmp_node(&a, &b));
    }

    #[test]
    fn cmp_node_obj_same_keys() {
        let mut na = ObjNode::new(ts(sid(), 1));
        na.put("x", ts(sid(), 10));
        let mut nb = ObjNode::new(ts(sid(), 1));
        nb.put("x", ts(sid(), 10));
        let a = CrdtNode::Obj(na);
        let b = CrdtNode::Obj(nb);
        assert!(cmp_node(&a, &b));
    }

    #[test]
    fn cmp_node_obj_different_keys() {
        let mut na = ObjNode::new(ts(sid(), 1));
        na.put("x", ts(sid(), 10));
        let mut nb = ObjNode::new(ts(sid(), 1));
        nb.put("x", ts(sid(), 11));
        let a = CrdtNode::Obj(na);
        let b = CrdtNode::Obj(nb);
        assert!(!cmp_node(&a, &b));
    }

    // ── cmp: Val node tests ─────────────────────────────────────────────

    #[test]
    fn cmp_val_both_resolve_same_content() {
        let mut index = BTreeMap::default();
        // Two Val nodes pointing to different Ts but both resolve to same Con value.
        let con_a = CrdtNode::Con(ConNode::new(
            ts(sid(), 10),
            ConValue::Val(PackValue::Integer(99)),
        ));
        let con_b = CrdtNode::Con(ConNode::new(
            ts(sid(), 20),
            ConValue::Val(PackValue::Integer(99)),
        ));
        index.insert(TsKey::from(ts(sid(), 10)), con_a);
        index.insert(TsKey::from(ts(sid(), 20)), con_b);

        let mut va = super::super::nodes::ValNode::new(ts(sid(), 1));
        va.val = ts(sid(), 10);
        let mut vb = super::super::nodes::ValNode::new(ts(sid(), 2));
        vb.val = ts(sid(), 20);
        let a = CrdtNode::Val(va);
        let b = CrdtNode::Val(vb);
        assert!(cmp(&a, &b, true, &index));
    }

    #[test]
    fn cmp_val_both_resolve_different_content() {
        let mut index = BTreeMap::default();
        let con_a = CrdtNode::Con(ConNode::new(
            ts(sid(), 10),
            ConValue::Val(PackValue::Integer(1)),
        ));
        let con_b = CrdtNode::Con(ConNode::new(
            ts(sid(), 20),
            ConValue::Val(PackValue::Integer(2)),
        ));
        index.insert(TsKey::from(ts(sid(), 10)), con_a);
        index.insert(TsKey::from(ts(sid(), 20)), con_b);

        let mut va = super::super::nodes::ValNode::new(ts(sid(), 1));
        va.val = ts(sid(), 10);
        let mut vb = super::super::nodes::ValNode::new(ts(sid(), 2));
        vb.val = ts(sid(), 20);
        let a = CrdtNode::Val(va);
        let b = CrdtNode::Val(vb);
        assert!(!cmp(&a, &b, true, &index));
    }

    #[test]
    fn cmp_val_one_missing_from_index() {
        let mut index = BTreeMap::default();
        let con_a = CrdtNode::Con(ConNode::new(ts(sid(), 10), ConValue::Val(PackValue::Null)));
        index.insert(TsKey::from(ts(sid(), 10)), con_a);

        let mut va = super::super::nodes::ValNode::new(ts(sid(), 1));
        va.val = ts(sid(), 10);
        let mut vb = super::super::nodes::ValNode::new(ts(sid(), 2));
        vb.val = ts(sid(), 99); // not in index
        let a = CrdtNode::Val(va);
        let b = CrdtNode::Val(vb);
        assert!(!cmp(&a, &b, true, &index));
    }

    #[test]
    fn cmp_val_both_missing_from_index() {
        let index = BTreeMap::default();
        let mut va = super::super::nodes::ValNode::new(ts(sid(), 1));
        va.val = ts(sid(), 50);
        let mut vb = super::super::nodes::ValNode::new(ts(sid(), 2));
        vb.val = ts(sid(), 60);
        let a = CrdtNode::Val(va);
        let b = CrdtNode::Val(vb);
        // Both missing → true
        assert!(cmp(&a, &b, true, &index));
    }

    // ── cmp: Str node tests ─────────────────────────────────────────────

    #[test]
    fn cmp_str_same_content() {
        let index = BTreeMap::default();
        let mut sa = StrNode::new(ts(sid(), 1));
        sa.ins(ts(sid(), 1), ts(sid(), 10), "hello".into());
        let mut sb = StrNode::new(ts(sid(), 2));
        sb.ins(ts(sid(), 2), ts(sid(), 20), "hello".into());
        let a = CrdtNode::Str(sa);
        let b = CrdtNode::Str(sb);
        assert!(cmp(&a, &b, true, &index));
    }

    #[test]
    fn cmp_str_different_content() {
        let index = BTreeMap::default();
        let mut sa = StrNode::new(ts(sid(), 1));
        sa.ins(ts(sid(), 1), ts(sid(), 10), "hello".into());
        let mut sb = StrNode::new(ts(sid(), 2));
        sb.ins(ts(sid(), 2), ts(sid(), 20), "world".into());
        let a = CrdtNode::Str(sa);
        let b = CrdtNode::Str(sb);
        assert!(!cmp(&a, &b, true, &index));
    }

    #[test]
    fn cmp_str_no_content_same_length() {
        let index = BTreeMap::default();
        let mut sa = StrNode::new(ts(sid(), 1));
        sa.ins(ts(sid(), 1), ts(sid(), 10), "abc".into());
        let mut sb = StrNode::new(ts(sid(), 2));
        sb.ins(ts(sid(), 2), ts(sid(), 20), "xyz".into());
        let a = CrdtNode::Str(sa);
        let b = CrdtNode::Str(sb);
        // compare_content=false, same type → true
        assert!(cmp(&a, &b, false, &index));
    }

    // ── cmp: Bin node tests ─────────────────────────────────────────────

    #[test]
    fn cmp_bin_same_data() {
        let index = BTreeMap::default();
        let mut ba = BinNode::new(ts(sid(), 1));
        ba.ins(ts(sid(), 1), ts(sid(), 10), vec![1, 2, 3]);
        let mut bb = BinNode::new(ts(sid(), 2));
        bb.ins(ts(sid(), 2), ts(sid(), 20), vec![1, 2, 3]);
        let a = CrdtNode::Bin(ba);
        let b = CrdtNode::Bin(bb);
        assert!(cmp(&a, &b, true, &index));
    }

    #[test]
    fn cmp_bin_different_data() {
        let index = BTreeMap::default();
        let mut ba = BinNode::new(ts(sid(), 1));
        ba.ins(ts(sid(), 1), ts(sid(), 10), vec![1, 2, 3]);
        let mut bb = BinNode::new(ts(sid(), 2));
        bb.ins(ts(sid(), 2), ts(sid(), 20), vec![4, 5, 6]);
        let a = CrdtNode::Bin(ba);
        let b = CrdtNode::Bin(bb);
        assert!(!cmp(&a, &b, true, &index));
    }

    #[test]
    fn cmp_bin_no_content() {
        let index = BTreeMap::default();
        let mut ba = BinNode::new(ts(sid(), 1));
        ba.ins(ts(sid(), 1), ts(sid(), 10), vec![1, 2]);
        let mut bb = BinNode::new(ts(sid(), 2));
        bb.ins(ts(sid(), 2), ts(sid(), 20), vec![9, 8]);
        let a = CrdtNode::Bin(ba);
        let b = CrdtNode::Bin(bb);
        assert!(cmp(&a, &b, false, &index));
    }

    // ── cmp: Obj node tests ─────────────────────────────────────────────

    #[test]
    fn cmp_obj_same_keys_same_values() {
        let mut index = BTreeMap::default();
        let con_a = CrdtNode::Con(ConNode::new(
            ts(sid(), 10),
            ConValue::Val(PackValue::Integer(42)),
        ));
        let con_b = CrdtNode::Con(ConNode::new(
            ts(sid(), 20),
            ConValue::Val(PackValue::Integer(42)),
        ));
        index.insert(TsKey::from(ts(sid(), 10)), con_a);
        index.insert(TsKey::from(ts(sid(), 20)), con_b);

        let mut oa = ObjNode::new(ts(sid(), 1));
        oa.put("key", ts(sid(), 10));
        let mut ob = ObjNode::new(ts(sid(), 2));
        ob.put("key", ts(sid(), 20));
        let a = CrdtNode::Obj(oa);
        let b = CrdtNode::Obj(ob);
        assert!(cmp(&a, &b, true, &index));
    }

    #[test]
    fn cmp_obj_different_key_count() {
        let index = BTreeMap::default();
        let mut oa = ObjNode::new(ts(sid(), 1));
        oa.put("x", ts(sid(), 10));
        let ob = ObjNode::new(ts(sid(), 2));
        let a = CrdtNode::Obj(oa);
        let b = CrdtNode::Obj(ob);
        assert!(!cmp(&a, &b, true, &index));
    }

    #[test]
    fn cmp_obj_missing_key_in_b() {
        let index = BTreeMap::default();
        let mut oa = ObjNode::new(ts(sid(), 1));
        oa.put("x", ts(sid(), 10));
        let mut ob = ObjNode::new(ts(sid(), 2));
        ob.put("y", ts(sid(), 20));
        let a = CrdtNode::Obj(oa);
        let b = CrdtNode::Obj(ob);
        assert!(!cmp(&a, &b, true, &index));
    }

    #[test]
    fn cmp_obj_value_one_missing_from_index() {
        let mut index = BTreeMap::default();
        let con_a = CrdtNode::Con(ConNode::new(ts(sid(), 10), ConValue::Val(PackValue::Null)));
        index.insert(TsKey::from(ts(sid(), 10)), con_a);
        // ts(sid(), 20) is NOT in the index

        let mut oa = ObjNode::new(ts(sid(), 1));
        oa.put("k", ts(sid(), 10));
        let mut ob = ObjNode::new(ts(sid(), 2));
        ob.put("k", ts(sid(), 20));
        let a = CrdtNode::Obj(oa);
        let b = CrdtNode::Obj(ob);
        assert!(!cmp(&a, &b, true, &index));
    }

    // ── cmp: Vec node tests ─────────────────────────────────────────────

    #[test]
    fn cmp_vec_same_elements() {
        let mut index = BTreeMap::default();
        let con_a = CrdtNode::Con(ConNode::new(
            ts(sid(), 10),
            ConValue::Val(PackValue::Integer(1)),
        ));
        let con_b = CrdtNode::Con(ConNode::new(
            ts(sid(), 20),
            ConValue::Val(PackValue::Integer(1)),
        ));
        index.insert(TsKey::from(ts(sid(), 10)), con_a);
        index.insert(TsKey::from(ts(sid(), 20)), con_b);

        let mut va = super::super::nodes::VecNode::new(ts(sid(), 1));
        va.put(0, ts(sid(), 10));
        let mut vb = super::super::nodes::VecNode::new(ts(sid(), 2));
        vb.put(0, ts(sid(), 20));
        let a = CrdtNode::Vec(va);
        let b = CrdtNode::Vec(vb);
        assert!(cmp(&a, &b, true, &index));
    }

    #[test]
    fn cmp_vec_different_lengths() {
        let mut index = BTreeMap::default();
        let con = CrdtNode::Con(ConNode::new(ts(sid(), 10), ConValue::Val(PackValue::Null)));
        index.insert(TsKey::from(ts(sid(), 10)), con);

        let mut va = super::super::nodes::VecNode::new(ts(sid(), 1));
        va.put(0, ts(sid(), 10));
        let vb = super::super::nodes::VecNode::new(ts(sid(), 2));
        let a = CrdtNode::Vec(va);
        let b = CrdtNode::Vec(vb);
        assert!(!cmp(&a, &b, true, &index));
    }

    #[test]
    fn cmp_vec_none_elements_equal() {
        let index = BTreeMap::default();
        // Both have 1 element slot but it's None
        let mut va = super::super::nodes::VecNode::new(ts(sid(), 1));
        va.elements.push(None);
        let mut vb = super::super::nodes::VecNode::new(ts(sid(), 2));
        vb.elements.push(None);
        let a = CrdtNode::Vec(va);
        let b = CrdtNode::Vec(vb);
        assert!(cmp(&a, &b, true, &index));
    }

    #[test]
    fn cmp_vec_one_none_one_some() {
        let index = BTreeMap::default();
        let mut va = super::super::nodes::VecNode::new(ts(sid(), 1));
        va.elements.push(None);
        let mut vb = super::super::nodes::VecNode::new(ts(sid(), 2));
        vb.elements.push(Some(ts(sid(), 10)));
        let a = CrdtNode::Vec(va);
        let b = CrdtNode::Vec(vb);
        assert!(!cmp(&a, &b, true, &index));
    }

    // ── cmp: Arr node tests ─────────────────────────────────────────────

    #[test]
    fn cmp_arr_empty_equal() {
        let index = BTreeMap::default();
        let a = CrdtNode::Arr(super::super::nodes::ArrNode::new(ts(sid(), 1)));
        let b = CrdtNode::Arr(super::super::nodes::ArrNode::new(ts(sid(), 2)));
        assert!(cmp(&a, &b, true, &index));
    }

    #[test]
    fn cmp_arr_different_length() {
        let mut index = BTreeMap::default();
        let con = CrdtNode::Con(ConNode::new(ts(sid(), 50), ConValue::Val(PackValue::Null)));
        index.insert(TsKey::from(ts(sid(), 50)), con);

        let mut arr_a = super::super::nodes::ArrNode::new(ts(sid(), 1));
        arr_a.ins(ts(sid(), 1), ts(sid(), 10), vec![ts(sid(), 50)]);
        let arr_b = super::super::nodes::ArrNode::new(ts(sid(), 2));
        let a = CrdtNode::Arr(arr_a);
        let b = CrdtNode::Arr(arr_b);
        assert!(!cmp(&a, &b, true, &index));
    }

    #[test]
    fn cmp_arr_same_length_no_content() {
        let mut index = BTreeMap::default();
        let con_a = CrdtNode::Con(ConNode::new(
            ts(sid(), 50),
            ConValue::Val(PackValue::Integer(1)),
        ));
        let con_b = CrdtNode::Con(ConNode::new(
            ts(sid(), 60),
            ConValue::Val(PackValue::Integer(2)),
        ));
        index.insert(TsKey::from(ts(sid(), 50)), con_a);
        index.insert(TsKey::from(ts(sid(), 60)), con_b);

        let mut arr_a = super::super::nodes::ArrNode::new(ts(sid(), 1));
        arr_a.ins(ts(sid(), 1), ts(sid(), 10), vec![ts(sid(), 50)]);
        let mut arr_b = super::super::nodes::ArrNode::new(ts(sid(), 2));
        arr_b.ins(ts(sid(), 2), ts(sid(), 20), vec![ts(sid(), 60)]);
        let a = CrdtNode::Arr(arr_a);
        let b = CrdtNode::Arr(arr_b);
        // same length, compare_content=false → true
        assert!(cmp(&a, &b, false, &index));
    }

    #[test]
    fn cmp_arr_same_length_different_content() {
        let mut index = BTreeMap::default();
        let con_a = CrdtNode::Con(ConNode::new(
            ts(sid(), 50),
            ConValue::Val(PackValue::Integer(1)),
        ));
        let con_b = CrdtNode::Con(ConNode::new(
            ts(sid(), 60),
            ConValue::Val(PackValue::Integer(2)),
        ));
        index.insert(TsKey::from(ts(sid(), 50)), con_a);
        index.insert(TsKey::from(ts(sid(), 60)), con_b);

        let mut arr_a = super::super::nodes::ArrNode::new(ts(sid(), 1));
        arr_a.ins(ts(sid(), 1), ts(sid(), 10), vec![ts(sid(), 50)]);
        let mut arr_b = super::super::nodes::ArrNode::new(ts(sid(), 2));
        arr_b.ins(ts(sid(), 2), ts(sid(), 20), vec![ts(sid(), 60)]);
        let a = CrdtNode::Arr(arr_a);
        let b = CrdtNode::Arr(arr_b);
        assert!(!cmp(&a, &b, true, &index));
    }

    // ── cmp: pointer identity ───────────────────────────────────────────

    #[test]
    fn cmp_same_pointer_returns_true() {
        let index = BTreeMap::default();
        let a = CrdtNode::Con(ConNode::new(
            ts(sid(), 1),
            ConValue::Val(PackValue::Integer(42)),
        ));
        assert!(cmp(&a, &a, true, &index));
    }

    // ── cmp_node: Val ───────────────────────────────────────────────────

    #[test]
    fn cmp_node_val_same_id_same_val() {
        let mut va = super::super::nodes::ValNode::new(ts(sid(), 1));
        va.val = ts(sid(), 10);
        let mut vb = super::super::nodes::ValNode::new(ts(sid(), 1));
        vb.val = ts(sid(), 10);
        let a = CrdtNode::Val(va);
        let b = CrdtNode::Val(vb);
        assert!(cmp_node(&a, &b));
    }

    #[test]
    fn cmp_node_val_same_id_different_val() {
        let mut va = super::super::nodes::ValNode::new(ts(sid(), 1));
        va.val = ts(sid(), 10);
        let mut vb = super::super::nodes::ValNode::new(ts(sid(), 1));
        vb.val = ts(sid(), 20);
        let a = CrdtNode::Val(va);
        let b = CrdtNode::Val(vb);
        assert!(!cmp_node(&a, &b));
    }

    #[test]
    fn cmp_node_val_different_id() {
        let va = super::super::nodes::ValNode::new(ts(sid(), 1));
        let vb = super::super::nodes::ValNode::new(ts(sid(), 2));
        let a = CrdtNode::Val(va);
        let b = CrdtNode::Val(vb);
        assert!(!cmp_node(&a, &b));
    }

    // ── cmp_node: Vec ───────────────────────────────────────────────────

    #[test]
    fn cmp_node_vec_same() {
        let mut va = super::super::nodes::VecNode::new(ts(sid(), 1));
        va.put(0, ts(sid(), 10));
        let mut vb = super::super::nodes::VecNode::new(ts(sid(), 1));
        vb.put(0, ts(sid(), 10));
        let a = CrdtNode::Vec(va);
        let b = CrdtNode::Vec(vb);
        assert!(cmp_node(&a, &b));
    }

    #[test]
    fn cmp_node_vec_different_length() {
        let mut va = super::super::nodes::VecNode::new(ts(sid(), 1));
        va.put(0, ts(sid(), 10));
        let vb = super::super::nodes::VecNode::new(ts(sid(), 1));
        let a = CrdtNode::Vec(va);
        let b = CrdtNode::Vec(vb);
        assert!(!cmp_node(&a, &b));
    }

    #[test]
    fn cmp_node_vec_different_element_ts() {
        let mut va = super::super::nodes::VecNode::new(ts(sid(), 1));
        va.put(0, ts(sid(), 10));
        let mut vb = super::super::nodes::VecNode::new(ts(sid(), 1));
        vb.put(0, ts(sid(), 20));
        let a = CrdtNode::Vec(va);
        let b = CrdtNode::Vec(vb);
        assert!(!cmp_node(&a, &b));
    }

    #[test]
    fn cmp_node_vec_none_vs_some() {
        let mut va = super::super::nodes::VecNode::new(ts(sid(), 1));
        va.elements.push(None);
        let mut vb = super::super::nodes::VecNode::new(ts(sid(), 1));
        vb.put(0, ts(sid(), 10));
        let a = CrdtNode::Vec(va);
        let b = CrdtNode::Vec(vb);
        assert!(!cmp_node(&a, &b));
    }

    // ── cmp_node: Obj ───────────────────────────────────────────────────

    #[test]
    fn cmp_node_obj_different_id() {
        let na = ObjNode::new(ts(sid(), 1));
        let nb = ObjNode::new(ts(sid(), 2));
        let a = CrdtNode::Obj(na);
        let b = CrdtNode::Obj(nb);
        assert!(!cmp_node(&a, &b));
    }

    #[test]
    fn cmp_node_obj_different_key_count() {
        let mut na = ObjNode::new(ts(sid(), 1));
        na.put("x", ts(sid(), 10));
        let nb = ObjNode::new(ts(sid(), 1));
        let a = CrdtNode::Obj(na);
        let b = CrdtNode::Obj(nb);
        assert!(!cmp_node(&a, &b));
    }

    #[test]
    fn cmp_node_obj_different_key_names() {
        let mut na = ObjNode::new(ts(sid(), 1));
        na.put("x", ts(sid(), 10));
        let mut nb = ObjNode::new(ts(sid(), 1));
        nb.put("y", ts(sid(), 10));
        let a = CrdtNode::Obj(na);
        let b = CrdtNode::Obj(nb);
        assert!(!cmp_node(&a, &b));
    }

    // ── cmp_node: pointer identity ──────────────────────────────────────

    #[test]
    fn cmp_node_same_pointer_returns_true() {
        let a = CrdtNode::Con(ConNode::new(
            ts(sid(), 1),
            ConValue::Val(PackValue::Integer(42)),
        ));
        assert!(cmp_node(&a, &a));
    }

    // ── cmp_node: Bin ───────────────────────────────────────────────────

    #[test]
    fn cmp_node_bin_same_empty() {
        let a = CrdtNode::Bin(BinNode::new(ts(sid(), 1)));
        let b = CrdtNode::Bin(BinNode::new(ts(sid(), 1)));
        assert!(cmp_node(&a, &b));
    }

    #[test]
    fn cmp_node_bin_different_id() {
        let a = CrdtNode::Bin(BinNode::new(ts(sid(), 1)));
        let b = CrdtNode::Bin(BinNode::new(ts(sid(), 2)));
        assert!(!cmp_node(&a, &b));
    }

    #[test]
    fn cmp_node_bin_same_with_data() {
        let mut ba = BinNode::new(ts(sid(), 1));
        ba.ins(ts(sid(), 1), ts(sid(), 10), vec![1, 2, 3]);
        let mut bb = BinNode::new(ts(sid(), 1));
        bb.ins(ts(sid(), 1), ts(sid(), 10), vec![1, 2, 3]);
        let a = CrdtNode::Bin(ba);
        let b = CrdtNode::Bin(bb);
        assert!(cmp_node(&a, &b));
    }

    #[test]
    fn cmp_node_bin_different_chunk_id() {
        let mut ba = BinNode::new(ts(sid(), 1));
        ba.ins(ts(sid(), 1), ts(sid(), 10), vec![1, 2]);
        let mut bb = BinNode::new(ts(sid(), 1));
        bb.ins(ts(sid(), 1), ts(sid(), 20), vec![1, 2]);
        let a = CrdtNode::Bin(ba);
        let b = CrdtNode::Bin(bb);
        assert!(!cmp_node(&a, &b));
    }

    #[test]
    fn cmp_node_bin_different_length() {
        let mut ba = BinNode::new(ts(sid(), 1));
        ba.ins(ts(sid(), 1), ts(sid(), 10), vec![1, 2]);
        let mut bb = BinNode::new(ts(sid(), 1));
        bb.ins(ts(sid(), 1), ts(sid(), 10), vec![1, 2, 3]);
        let a = CrdtNode::Bin(ba);
        let b = CrdtNode::Bin(bb);
        assert!(!cmp_node(&a, &b));
    }

    // ── cmp_node: Arr ───────────────────────────────────────────────────

    #[test]
    fn cmp_node_arr_same_empty() {
        let a = CrdtNode::Arr(ArrNode::new(ts(sid(), 1)));
        let b = CrdtNode::Arr(ArrNode::new(ts(sid(), 1)));
        assert!(cmp_node(&a, &b));
    }

    #[test]
    fn cmp_node_arr_different_id() {
        let a = CrdtNode::Arr(ArrNode::new(ts(sid(), 1)));
        let b = CrdtNode::Arr(ArrNode::new(ts(sid(), 2)));
        assert!(!cmp_node(&a, &b));
    }

    #[test]
    fn cmp_node_arr_same_with_data() {
        let mut aa = ArrNode::new(ts(sid(), 1));
        aa.ins(ts(sid(), 1), ts(sid(), 10), vec![ts(sid(), 50)]);
        let mut ab = ArrNode::new(ts(sid(), 1));
        ab.ins(ts(sid(), 1), ts(sid(), 10), vec![ts(sid(), 50)]);
        let a = CrdtNode::Arr(aa);
        let b = CrdtNode::Arr(ab);
        assert!(cmp_node(&a, &b));
    }

    #[test]
    fn cmp_node_arr_different_chunk_id() {
        let mut aa = ArrNode::new(ts(sid(), 1));
        aa.ins(ts(sid(), 1), ts(sid(), 10), vec![ts(sid(), 50)]);
        let mut ab = ArrNode::new(ts(sid(), 1));
        ab.ins(ts(sid(), 1), ts(sid(), 20), vec![ts(sid(), 50)]);
        let a = CrdtNode::Arr(aa);
        let b = CrdtNode::Arr(ab);
        assert!(!cmp_node(&a, &b));
    }

    #[test]
    fn cmp_node_arr_different_size() {
        let mut aa = ArrNode::new(ts(sid(), 1));
        aa.ins(ts(sid(), 1), ts(sid(), 10), vec![ts(sid(), 50)]);
        let ab = ArrNode::new(ts(sid(), 1));
        let a = CrdtNode::Arr(aa);
        let b = CrdtNode::Arr(ab);
        assert!(!cmp_node(&a, &b));
    }

    // ── cmp_node: Str with content ──────────────────────────────────────

    #[test]
    fn cmp_node_str_same_with_data() {
        let mut sa = StrNode::new(ts(sid(), 1));
        sa.ins(ts(sid(), 1), ts(sid(), 10), "hello".into());
        let mut sb = StrNode::new(ts(sid(), 1));
        sb.ins(ts(sid(), 1), ts(sid(), 10), "hello".into());
        let a = CrdtNode::Str(sa);
        let b = CrdtNode::Str(sb);
        assert!(cmp_node(&a, &b));
    }

    #[test]
    fn cmp_node_str_different_chunk_id() {
        let mut sa = StrNode::new(ts(sid(), 1));
        sa.ins(ts(sid(), 1), ts(sid(), 10), "abc".into());
        let mut sb = StrNode::new(ts(sid(), 1));
        sb.ins(ts(sid(), 1), ts(sid(), 20), "abc".into());
        let a = CrdtNode::Str(sa);
        let b = CrdtNode::Str(sb);
        assert!(!cmp_node(&a, &b));
    }

    #[test]
    fn cmp_node_str_different_size() {
        let mut sa = StrNode::new(ts(sid(), 1));
        sa.ins(ts(sid(), 1), ts(sid(), 10), "ab".into());
        let mut sb = StrNode::new(ts(sid(), 1));
        sb.ins(ts(sid(), 1), ts(sid(), 10), "abc".into());
        let a = CrdtNode::Str(sa);
        let b = CrdtNode::Str(sb);
        assert!(!cmp_node(&a, &b));
    }

    #[test]
    fn cmp_node_str_different_id() {
        let a = CrdtNode::Str(StrNode::new(ts(sid(), 1)));
        let b = CrdtNode::Str(StrNode::new(ts(sid(), 2)));
        assert!(!cmp_node(&a, &b));
    }

    // ── cmp: Obj with both values missing from index ────────────────────

    #[test]
    fn cmp_obj_values_both_missing_from_index() {
        let index = BTreeMap::default();
        let mut oa = ObjNode::new(ts(sid(), 1));
        oa.put("k", ts(sid(), 10));
        let mut ob = ObjNode::new(ts(sid(), 2));
        ob.put("k", ts(sid(), 20));
        let a = CrdtNode::Obj(oa);
        let b = CrdtNode::Obj(ob);
        // Both values missing from index → (None, None) → true
        assert!(cmp(&a, &b, true, &index));
    }

    // ── cmp: Obj with multiple keys ─────────────────────────────────────

    #[test]
    fn cmp_obj_multiple_keys_same_values() {
        let mut index = BTreeMap::default();
        index.insert(
            TsKey::from(ts(sid(), 10)),
            CrdtNode::Con(ConNode::new(
                ts(sid(), 10),
                ConValue::Val(PackValue::Integer(1)),
            )),
        );
        index.insert(
            TsKey::from(ts(sid(), 20)),
            CrdtNode::Con(ConNode::new(
                ts(sid(), 20),
                ConValue::Val(PackValue::Integer(1)),
            )),
        );
        index.insert(
            TsKey::from(ts(sid(), 30)),
            CrdtNode::Con(ConNode::new(
                ts(sid(), 30),
                ConValue::Val(PackValue::Str("x".into())),
            )),
        );
        index.insert(
            TsKey::from(ts(sid(), 40)),
            CrdtNode::Con(ConNode::new(
                ts(sid(), 40),
                ConValue::Val(PackValue::Str("x".into())),
            )),
        );

        let mut oa = ObjNode::new(ts(sid(), 1));
        oa.put("a", ts(sid(), 10));
        oa.put("b", ts(sid(), 30));
        let mut ob = ObjNode::new(ts(sid(), 2));
        ob.put("a", ts(sid(), 20));
        ob.put("b", ts(sid(), 40));
        let a = CrdtNode::Obj(oa);
        let b = CrdtNode::Obj(ob);
        assert!(cmp(&a, &b, true, &index));
    }

    #[test]
    fn cmp_obj_multiple_keys_one_different() {
        let mut index = BTreeMap::default();
        index.insert(
            TsKey::from(ts(sid(), 10)),
            CrdtNode::Con(ConNode::new(
                ts(sid(), 10),
                ConValue::Val(PackValue::Integer(1)),
            )),
        );
        index.insert(
            TsKey::from(ts(sid(), 20)),
            CrdtNode::Con(ConNode::new(
                ts(sid(), 20),
                ConValue::Val(PackValue::Integer(1)),
            )),
        );
        index.insert(
            TsKey::from(ts(sid(), 30)),
            CrdtNode::Con(ConNode::new(
                ts(sid(), 30),
                ConValue::Val(PackValue::Str("x".into())),
            )),
        );
        index.insert(
            TsKey::from(ts(sid(), 40)),
            CrdtNode::Con(ConNode::new(
                ts(sid(), 40),
                ConValue::Val(PackValue::Str("y".into())),
            )),
        );

        let mut oa = ObjNode::new(ts(sid(), 1));
        oa.put("a", ts(sid(), 10));
        oa.put("b", ts(sid(), 30));
        let mut ob = ObjNode::new(ts(sid(), 2));
        ob.put("a", ts(sid(), 20));
        ob.put("b", ts(sid(), 40));
        let a = CrdtNode::Obj(oa);
        let b = CrdtNode::Obj(ob);
        assert!(!cmp(&a, &b, true, &index));
    }

    // ── cmp: Vec with different content values ──────────────────────────

    #[test]
    fn cmp_vec_same_length_different_values() {
        let mut index = BTreeMap::default();
        index.insert(
            TsKey::from(ts(sid(), 10)),
            CrdtNode::Con(ConNode::new(
                ts(sid(), 10),
                ConValue::Val(PackValue::Integer(1)),
            )),
        );
        index.insert(
            TsKey::from(ts(sid(), 20)),
            CrdtNode::Con(ConNode::new(
                ts(sid(), 20),
                ConValue::Val(PackValue::Integer(2)),
            )),
        );

        let mut va = super::super::nodes::VecNode::new(ts(sid(), 1));
        va.put(0, ts(sid(), 10));
        let mut vb = super::super::nodes::VecNode::new(ts(sid(), 2));
        vb.put(0, ts(sid(), 20));
        let a = CrdtNode::Vec(va);
        let b = CrdtNode::Vec(vb);
        assert!(!cmp(&a, &b, true, &index));
    }

    #[test]
    fn cmp_vec_no_content_ignores_values() {
        let mut index = BTreeMap::default();
        index.insert(
            TsKey::from(ts(sid(), 10)),
            CrdtNode::Con(ConNode::new(
                ts(sid(), 10),
                ConValue::Val(PackValue::Integer(1)),
            )),
        );
        index.insert(
            TsKey::from(ts(sid(), 20)),
            CrdtNode::Con(ConNode::new(
                ts(sid(), 20),
                ConValue::Val(PackValue::Integer(2)),
            )),
        );

        let mut va = super::super::nodes::VecNode::new(ts(sid(), 1));
        va.put(0, ts(sid(), 10));
        let mut vb = super::super::nodes::VecNode::new(ts(sid(), 2));
        vb.put(0, ts(sid(), 20));
        let a = CrdtNode::Vec(va);
        let b = CrdtNode::Vec(vb);
        assert!(cmp(&a, &b, false, &index));
    }

    // ── cmp: Vec element both missing from index ────────────────────────

    #[test]
    fn cmp_vec_elements_both_missing_from_index() {
        let index = BTreeMap::default();
        let mut va = super::super::nodes::VecNode::new(ts(sid(), 1));
        va.put(0, ts(sid(), 10));
        let mut vb = super::super::nodes::VecNode::new(ts(sid(), 2));
        vb.put(0, ts(sid(), 20));
        let a = CrdtNode::Vec(va);
        let b = CrdtNode::Vec(vb);
        // Both elements missing from index → (None, None) → true
        assert!(cmp(&a, &b, true, &index));
    }

    // ── cmp: Arr with same content ──────────────────────────────────────

    #[test]
    fn cmp_arr_same_content() {
        let mut index = BTreeMap::default();
        index.insert(
            TsKey::from(ts(sid(), 50)),
            CrdtNode::Con(ConNode::new(
                ts(sid(), 50),
                ConValue::Val(PackValue::Integer(42)),
            )),
        );
        index.insert(
            TsKey::from(ts(sid(), 60)),
            CrdtNode::Con(ConNode::new(
                ts(sid(), 60),
                ConValue::Val(PackValue::Integer(42)),
            )),
        );

        let mut arr_a = ArrNode::new(ts(sid(), 1));
        arr_a.ins(ts(sid(), 1), ts(sid(), 10), vec![ts(sid(), 50)]);
        let mut arr_b = ArrNode::new(ts(sid(), 2));
        arr_b.ins(ts(sid(), 2), ts(sid(), 20), vec![ts(sid(), 60)]);
        let a = CrdtNode::Arr(arr_a);
        let b = CrdtNode::Arr(arr_b);
        assert!(cmp(&a, &b, true, &index));
    }

    // ── cmp: Arr with both elements missing from index ──────────────────

    #[test]
    fn cmp_arr_elements_both_missing_from_index() {
        let index = BTreeMap::default();

        let mut arr_a = ArrNode::new(ts(sid(), 1));
        arr_a.ins(ts(sid(), 1), ts(sid(), 10), vec![ts(sid(), 50)]);
        let mut arr_b = ArrNode::new(ts(sid(), 2));
        arr_b.ins(ts(sid(), 2), ts(sid(), 20), vec![ts(sid(), 60)]);
        let a = CrdtNode::Arr(arr_a);
        let b = CrdtNode::Arr(arr_b);
        // Both elements missing → (None, None) → true
        assert!(cmp(&a, &b, true, &index));
    }

    // ── cmp: Bin different length ───────────────────────────────────────

    #[test]
    fn cmp_bin_different_length() {
        let index = BTreeMap::default();
        let mut ba = BinNode::new(ts(sid(), 1));
        ba.ins(ts(sid(), 1), ts(sid(), 10), vec![1, 2, 3]);
        let mut bb = BinNode::new(ts(sid(), 2));
        bb.ins(ts(sid(), 2), ts(sid(), 20), vec![1, 2]);
        let a = CrdtNode::Bin(ba);
        let b = CrdtNode::Bin(bb);
        assert!(!cmp(&a, &b, true, &index));
    }

    // ── cmp: Str different length ───────────────────────────────────────

    #[test]
    fn cmp_str_different_length() {
        let index = BTreeMap::default();
        let mut sa = StrNode::new(ts(sid(), 1));
        sa.ins(ts(sid(), 1), ts(sid(), 10), "hello".into());
        let mut sb = StrNode::new(ts(sid(), 2));
        sb.ins(ts(sid(), 2), ts(sid(), 20), "hi".into());
        let a = CrdtNode::Str(sa);
        let b = CrdtNode::Str(sb);
        assert!(!cmp(&a, &b, true, &index));
    }
}
