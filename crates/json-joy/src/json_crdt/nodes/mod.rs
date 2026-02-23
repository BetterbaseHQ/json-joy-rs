//! JSON CRDT node types.
//!
//! Mirrors `packages/json-joy/src/json-crdt/nodes/`.
//!
//! # Node Types
//!
//! | Rust type      | TypeScript  | Semantics                         |
//! |----------------|-------------|-----------------------------------|
//! | `ConNode`      | `ConNode`   | Immutable constant value          |
//! | `ValNode`      | `ValNode`   | Last-write-wins single register   |
//! | `ObjNode`      | `ObjNode`   | LWW key→value map                 |
//! | `VecNode`      | `VecNode`   | Fixed-length LWW tuple            |
//! | `StrNode`      | `StrNode`   | RGA UTF-16 string                 |
//! | `BinNode`      | `BinNode`   | RGA binary blob                   |
//! | `ArrNode`      | `ArrNode`   | RGA array of node references      |
//! | `RootNode`     | `RootNode`  | Document root (LWW register)      |

pub mod rga;

use indexmap::IndexMap;
use json_joy_json_pack::PackValue;
use serde_json::Value;
use std::collections::BTreeMap;

use super::constants::{ORIGIN, UNDEFINED_TS};
use crate::json_crdt_patch::clock::{compare, Ts, Tss};
use crate::json_crdt_patch::operations::ConValue;
use rga::Rga;

// ── ConNode ───────────────────────────────────────────────────────────────

/// Immutable constant node.  Wraps a static value that never changes.
#[derive(Debug, Clone)]
pub struct ConNode {
    pub id: Ts,
    pub val: ConValue,
}

impl ConNode {
    pub fn new(id: Ts, val: ConValue) -> Self {
        Self { id, val }
    }

    /// Return the JSON view of this constant.
    pub fn view(&self) -> Value {
        match &self.val {
            ConValue::Ref(_) => Value::Null, // reference — caller resolves
            ConValue::Val(pv) => pack_to_json(pv),
        }
    }
}

// ── ValNode ───────────────────────────────────────────────────────────────

/// Last-write-wins single-value register.
///
/// Stores the ID of whichever node currently "wins" the register.
#[derive(Debug, Clone)]
pub struct ValNode {
    pub id: Ts,
    /// The ID of the current value node (starts at UNDEFINED).
    pub val: Ts,
}

impl ValNode {
    pub fn new(id: Ts) -> Self {
        // Use ORIGIN (sid=0, time=0) so any user timestamp wins LWW comparison.
        // Upstream TypeScript uses ORIGIN (not UNDEFINED) as ValNode's initial value.
        Self { id, val: ORIGIN }
    }

    /// Set `new_val` if it has a higher timestamp than the current.
    /// Returns the old value if it was replaced.
    pub fn set(&mut self, new_val: Ts) -> Option<Ts> {
        if compare(new_val, self.val) > 0 {
            let old = self.val;
            self.val = new_val;
            Some(old)
        } else {
            None
        }
    }

    /// View: resolve the pointed-to node from the index.
    pub fn view(&self, index: &NodeIndex) -> Value {
        match index.get(&TsKey::from(self.val)) {
            Some(node) => node.view(index),
            None => Value::Null,
        }
    }
}

// ── ObjNode ───────────────────────────────────────────────────────────────

/// Last-write-wins object (map from string keys to node IDs).
#[derive(Debug, Clone)]
pub struct ObjNode {
    pub id: Ts,
    /// key → winning node ID, in insertion order (mirrors JS Map)
    pub keys: IndexMap<String, Ts>,
}

impl ObjNode {
    pub fn new(id: Ts) -> Self {
        Self {
            id,
            keys: IndexMap::new(),
        }
    }

    /// Insert a key, keeping it only if `new_id` is newer than existing.
    /// Returns the old ID if replaced.
    pub fn put(&mut self, key: &str, new_id: Ts) -> Option<Ts> {
        match self.keys.get(key).copied() {
            Some(old) if compare(new_id, old) <= 0 => None,
            old => {
                self.keys.insert(key.to_string(), new_id);
                old
            }
        }
    }

    /// View: build a JSON object by resolving each value from the index.
    ///
    /// Keys are iterated in insertion order, matching upstream TypeScript Map semantics.
    pub fn view(&self, index: &NodeIndex) -> Value {
        let mut map = serde_json::Map::new();
        for (key, &id) in &self.keys {
            let val = match index.get(&TsKey::from(id)) {
                // Upstream omits object keys whose winning value is `con(undefined)`.
                Some(CrdtNode::Con(con)) => match &con.val {
                    ConValue::Val(PackValue::Undefined) => continue,
                    _ => con.view(),
                },
                Some(node) => node.view(index),
                // Missing node references are omitted from object views.
                None => continue,
            };
            map.insert(key.clone(), val);
        }
        Value::Object(map)
    }
}

// ── VecNode ───────────────────────────────────────────────────────────────

/// Fixed-length LWW tuple (vector).
#[derive(Debug, Clone)]
pub struct VecNode {
    pub id: Ts,
    /// Indexed by position → node ID (None = unset).
    pub elements: Vec<Option<Ts>>,
}

impl VecNode {
    pub fn new(id: Ts) -> Self {
        Self {
            id,
            elements: Vec::new(),
        }
    }

    /// Set element at `index`, keeping it only if `new_id` is newer.
    /// Returns old ID if replaced.
    pub fn put(&mut self, index: usize, new_id: Ts) -> Option<Ts> {
        if index >= self.elements.len() {
            self.elements.resize(index + 1, None);
        }
        match self.elements[index] {
            Some(old) if compare(new_id, old) <= 0 => None,
            old => {
                self.elements[index] = Some(new_id);
                old // old: Option<Ts>, already the right type
            }
        }
    }

    /// View: build a JSON array by resolving each element.
    pub fn view(&self, index: &NodeIndex) -> Value {
        let items: Vec<Value> = self
            .elements
            .iter()
            .map(|e| match e {
                Some(id) => match index.get(&TsKey::from(*id)) {
                    Some(node) => node.view(index),
                    None => Value::Null,
                },
                None => Value::Null,
            })
            .collect();
        Value::Array(items)
    }
}

// ── StrNode ───────────────────────────────────────────────────────────────

/// RGA string node (UTF-16 chunks, as in the upstream).
#[derive(Debug, Clone)]
pub struct StrNode {
    pub id: Ts,
    pub rga: Rga<String>,
}

impl StrNode {
    pub fn new(id: Ts) -> Self {
        Self {
            id,
            rga: Rga::new(),
        }
    }

    pub fn ins(&mut self, after: Ts, id: Ts, data: String) {
        // Upstream uses JS `string.length` (UTF-16 code units) for span.
        let span = data.encode_utf16().count() as u64;
        self.rga.insert(after, id, span, data);
    }

    pub fn delete(&mut self, spans: &[Tss]) {
        self.rga.delete(spans);
    }

    pub fn view(&self) -> Value {
        let s: String = self
            .rga
            .iter_live()
            .filter_map(|c| c.data.as_deref())
            .collect();
        Value::String(s)
    }

    /// Return the string content as a plain `String`.
    pub fn view_str(&self) -> String {
        self.rga
            .iter_live()
            .filter_map(|c| c.data.as_deref())
            .collect()
    }

    /// Number of live UTF-16 code units in this string.
    ///
    /// Matches upstream `StrNode.length()` which uses JS `string.length`
    /// (UTF-16 code units).
    pub fn size(&self) -> usize {
        self.rga.iter_live().map(|c| c.span as usize).sum()
    }

    /// Find the chunk-ID timestamp of the character at live position `pos`.
    ///
    /// Returns `None` if `pos >= self.size()`.
    pub fn find(&self, pos: usize) -> Option<Ts> {
        let mut count = 0usize;
        for chunk in self.rga.iter_live() {
            let chunk_len = chunk.span as usize;
            if pos < count + chunk_len {
                let offset = pos - count;
                return Some(Ts::new(chunk.id.sid, chunk.id.time + offset as u64));
            }
            count += chunk_len;
        }
        None
    }

    /// Return the timestamp spans covering live positions `[pos, pos + len)`.
    pub fn find_interval(&self, pos: usize, len: usize) -> Vec<Tss> {
        let mut result = Vec::new();
        let mut count = 0usize;
        let end = pos + len;
        for chunk in self.rga.iter_live() {
            let chunk_len = chunk.span as usize;
            let chunk_start = count;
            let chunk_end = count + chunk_len;
            if chunk_end > pos && chunk_start < end {
                let local_start = pos.saturating_sub(chunk_start);
                let local_end = (end - chunk_start).min(chunk_len);
                result.push(Tss::new(
                    chunk.id.sid,
                    chunk.id.time + local_start as u64,
                    (local_end - local_start) as u64,
                ));
            }
            count = chunk_end;
        }
        result
    }
}

// ── BinNode ───────────────────────────────────────────────────────────────

/// RGA binary node.
#[derive(Debug, Clone)]
pub struct BinNode {
    pub id: Ts,
    pub rga: Rga<Vec<u8>>,
}

impl BinNode {
    pub fn new(id: Ts) -> Self {
        Self {
            id,
            rga: Rga::new(),
        }
    }

    pub fn ins(&mut self, after: Ts, id: Ts, data: Vec<u8>) {
        let span = data.len() as u64;
        self.rga.insert(after, id, span, data);
    }

    pub fn delete(&mut self, spans: &[Tss]) {
        self.rga.delete(spans);
    }

    pub fn view(&self) -> Vec<u8> {
        self.rga
            .iter_live()
            .flat_map(|c| c.data.as_deref().unwrap_or(&[]))
            .copied()
            .collect()
    }

    /// View as a JSON array of byte values.
    pub fn view_json(&self) -> Value {
        let bytes = self.view();
        Value::Array(bytes.into_iter().map(|b| Value::Number(b.into())).collect())
    }
}

// ── ArrNode ───────────────────────────────────────────────────────────────

/// RGA array of node-ID references.
#[derive(Debug, Clone)]
pub struct ArrNode {
    pub id: Ts,
    pub rga: Rga<Vec<Ts>>,
}

impl ArrNode {
    pub fn new(id: Ts) -> Self {
        Self {
            id,
            rga: Rga::new(),
        }
    }

    /// Insert node IDs after `after`.
    pub fn ins(&mut self, after: Ts, id: Ts, data: Vec<Ts>) {
        let span = data.len() as u64;
        self.rga.insert(after, id, span, data);
    }

    /// Get the node ID at the given absolute position.
    pub fn get_by_id(&self, target: Ts) -> Option<Ts> {
        let idx = self.rga.find_by_id(target)?;
        let chunk = self.rga.slot(idx);
        if let Some(data) = &chunk.data {
            let offset = (target.time - chunk.id.time) as usize;
            return data.get(offset).copied();
        }
        None
    }

    /// Update (replace) an existing element at the slot identified by `ref_id`.
    ///
    /// Mirrors `ArrNode.upd` in the upstream TypeScript.
    /// Only replaces the current value if `val` has a higher timestamp.
    pub fn upd(&mut self, ref_id: Ts, val: Ts) -> Option<Ts> {
        let idx = self.rga.find_by_id(ref_id)?;
        let chunk = self.rga.slot_mut(idx);
        if let Some(data) = &mut chunk.data {
            let offset = (ref_id.time - chunk.id.time) as usize;
            if let Some(current) = data.get(offset).copied() {
                use crate::json_crdt_patch::clock::compare;
                if compare(current, val) >= 0 {
                    return None; // existing is same or newer
                }
                let old = data[offset];
                data[offset] = val;
                return Some(old);
            }
        }
        None
    }

    pub fn delete(&mut self, spans: &[Tss]) {
        self.rga.delete(spans);
    }

    /// Number of live elements in this array.
    pub fn size(&self) -> usize {
        self.rga
            .iter_live()
            .filter_map(|c| c.data.as_ref())
            .map(|v| v.len())
            .sum()
    }

    /// Return the slot-ID timestamp of the element at live position `pos`.
    ///
    /// The slot-ID is the timestamp assigned to the **slot** in the RGA
    /// (not the data node stored at that slot).
    pub fn find(&self, pos: usize) -> Option<Ts> {
        let mut count = 0usize;
        for chunk in self.rga.iter_live() {
            if let Some(data) = &chunk.data {
                let chunk_len = data.len();
                if pos < count + chunk_len {
                    let offset = pos - count;
                    return Some(Ts::new(chunk.id.sid, chunk.id.time + offset as u64));
                }
                count += chunk_len;
            }
        }
        None
    }

    /// Return the data-node timestamps of live elements from position `pos` for `len` items.
    pub fn find_data_at(&self, pos: usize, len: usize) -> Vec<Ts> {
        let mut result = Vec::new();
        let mut count = 0usize;
        let end = pos + len;
        for chunk in self.rga.iter_live() {
            if let Some(data) = &chunk.data {
                let chunk_len = data.len();
                let chunk_start = count;
                let chunk_end = count + chunk_len;
                if chunk_end > pos && chunk_start < end {
                    let local_start = pos.saturating_sub(chunk_start);
                    let local_end = (end - chunk_start).min(chunk_len);
                    result.extend_from_slice(&data[local_start..local_end]);
                }
                count = chunk_end;
            }
        }
        result
    }

    /// Return the slot-ID spans covering live positions `[pos, pos + len)`.
    pub fn find_interval(&self, pos: usize, len: usize) -> Vec<Tss> {
        let mut result = Vec::new();
        let mut count = 0usize;
        let end = pos + len;
        for chunk in self.rga.iter_live() {
            if let Some(data) = &chunk.data {
                let chunk_len = data.len();
                let chunk_start = count;
                let chunk_end = count + chunk_len;
                if chunk_end > pos && chunk_start < end {
                    let local_start = pos.saturating_sub(chunk_start);
                    let local_end = (end - chunk_start).min(chunk_len);
                    result.push(Tss::new(
                        chunk.id.sid,
                        chunk.id.time + local_start as u64,
                        (local_end - local_start) as u64,
                    ));
                }
                count = chunk_end;
            }
        }
        result
    }

    /// Get the data-node timestamp (what the slot points to) at live position `pos`.
    pub fn get_data_ts(&self, pos: usize) -> Option<Ts> {
        let mut count = 0usize;
        for chunk in self.rga.iter_live() {
            if let Some(data) = &chunk.data {
                let chunk_len = data.len();
                if pos < count + chunk_len {
                    return Some(data[pos - count]);
                }
                count += chunk_len;
            }
        }
        None
    }

    /// View: resolve all non-deleted element IDs from the index.
    pub fn view(&self, index: &NodeIndex) -> Value {
        let mut items = Vec::new();
        for chunk in self.rga.iter_live() {
            if let Some(ids) = &chunk.data {
                for id in ids {
                    let val = match index.get(&TsKey::from(*id)) {
                        Some(node) => node.view(index),
                        None => Value::Null,
                    };
                    items.push(val);
                }
            }
        }
        Value::Array(items)
    }
}

// ── RootNode ──────────────────────────────────────────────────────────────

/// Document root — a LWW register pointing to the root JSON node.
#[derive(Debug, Clone)]
pub struct RootNode {
    pub val: Ts,
}

impl RootNode {
    pub fn new() -> Self {
        Self { val: UNDEFINED_TS }
    }

    pub fn set(&mut self, new_val: Ts) -> Option<Ts> {
        if compare(new_val, self.val) > 0 {
            let old = self.val;
            self.val = new_val;
            Some(old)
        } else {
            None
        }
    }

    pub fn view(&self, index: &NodeIndex) -> Value {
        match index.get(&TsKey::from(self.val)) {
            Some(node) => node.view(index),
            None => Value::Null,
        }
    }
}

impl Default for RootNode {
    fn default() -> Self {
        Self::new()
    }
}

// ── CrdtNode enum ─────────────────────────────────────────────────────────

/// All possible CRDT node types.
#[derive(Debug, Clone)]
pub enum CrdtNode {
    Con(ConNode),
    Val(ValNode),
    Obj(ObjNode),
    Vec(VecNode),
    Str(StrNode),
    Bin(BinNode),
    Arr(ArrNode),
}

impl CrdtNode {
    pub fn id(&self) -> Ts {
        match self {
            Self::Con(n) => n.id,
            Self::Val(n) => n.id,
            Self::Obj(n) => n.id,
            Self::Vec(n) => n.id,
            Self::Str(n) => n.id,
            Self::Bin(n) => n.id,
            Self::Arr(n) => n.id,
        }
    }

    pub fn view(&self, index: &NodeIndex) -> Value {
        match self {
            Self::Con(n) => n.view(),
            Self::Val(n) => n.view(index),
            Self::Obj(n) => n.view(index),
            Self::Vec(n) => n.view(index),
            Self::Str(n) => n.view(),
            Self::Bin(n) => n.view_json(),
            Self::Arr(n) => n.view(index),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Con(_) => "con",
            Self::Val(_) => "val",
            Self::Obj(_) => "obj",
            Self::Vec(_) => "vec",
            Self::Str(_) => "str",
            Self::Bin(_) => "bin",
            Self::Arr(_) => "arr",
        }
    }

    /// Collect the IDs of all immediate child nodes.
    ///
    /// Mirrors the `children(callback)` method on each upstream node type.
    /// - `ConNode`, `StrNode`, `BinNode` have no children.
    /// - `ValNode` has one child: the pointed-to node.
    /// - `ObjNode` children are all values in its keys map.
    /// - `VecNode` children are all non-None elements.
    /// - `ArrNode` children are all data-node timestamps across all chunks.
    pub fn child_ids(&self) -> Vec<Ts> {
        match self {
            Self::Con(_) | Self::Str(_) | Self::Bin(_) => Vec::new(),
            Self::Val(n) => {
                if n.val.sid == 0 && n.val.time == 0 {
                    Vec::new() // ORIGIN — no child
                } else {
                    vec![n.val]
                }
            }
            Self::Obj(n) => n.keys.values().copied().collect(),
            Self::Vec(n) => n.elements.iter().filter_map(|e| *e).collect(),
            Self::Arr(n) => {
                let mut ids = Vec::new();
                // Walk ALL chunks (including tombstoned), matching upstream
                // ArrNode.children() which iterates first()→next().
                for chunk in n.rga.iter() {
                    if let Some(data) = &chunk.data {
                        ids.extend_from_slice(data);
                    }
                }
                ids
            }
        }
    }
}

// ── NodeIndex ─────────────────────────────────────────────────────────────

/// Map from timestamp ID to CRDT node. Uses `BTreeMap` for deterministic
/// iteration order, matching upstream's `AvlMap` with `clock.compare`
/// (time-first, then sid).
pub type NodeIndex = BTreeMap<TsKey, CrdtNode>;

/// Ordered key for Ts. Compares time first, then sid — matching upstream
/// `clock.compare` used by the `AvlMap`-backed node index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TsKey {
    pub sid: u64,
    pub time: u64,
}

impl PartialOrd for TsKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TsKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time
            .cmp(&other.time)
            .then_with(|| self.sid.cmp(&other.sid))
    }
}

impl From<Ts> for TsKey {
    fn from(ts: Ts) -> Self {
        Self {
            sid: ts.sid,
            time: ts.time,
        }
    }
}

/// Convenience trait to look up nodes using `&Ts`.
pub trait IndexExt {
    fn get(&self, ts: &Ts) -> Option<&CrdtNode>;
    fn get_mut_ts(&mut self, ts: &Ts) -> Option<&mut CrdtNode>;
    fn insert_node(&mut self, ts: Ts, node: CrdtNode);
    fn remove_node(&mut self, ts: &Ts) -> Option<CrdtNode>;
    fn contains_ts(&self, ts: &Ts) -> bool;
}

impl IndexExt for NodeIndex {
    fn get(&self, ts: &Ts) -> Option<&CrdtNode> {
        self.get(&TsKey::from(*ts))
    }

    fn get_mut_ts(&mut self, ts: &Ts) -> Option<&mut CrdtNode> {
        self.get_mut(&TsKey::from(*ts))
    }

    fn insert_node(&mut self, ts: Ts, node: CrdtNode) {
        self.insert(TsKey::from(ts), node);
    }

    fn remove_node(&mut self, ts: &Ts) -> Option<CrdtNode> {
        self.remove(&TsKey::from(*ts))
    }

    fn contains_ts(&self, ts: &Ts) -> bool {
        self.contains_key(&TsKey::from(*ts))
    }
}

// ── Helper: PackValue → serde_json::Value ─────────────────────────────────

pub fn pack_to_json(pv: &PackValue) -> Value {
    Value::from(pv.clone())
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json_crdt::constants::ORIGIN;
    use crate::json_crdt_patch::clock::ts;

    fn sid() -> u64 {
        42
    }

    // ── UTF-16 counting tests ───────────────────────────────────────────

    #[test]
    fn str_size_ascii() {
        let mut s = StrNode::new(ts(sid(), 1));
        s.ins(ORIGIN, ts(sid(), 2), "hello".into());
        // "hello" = 5 UTF-16 code units
        assert_eq!(s.size(), 5);
    }

    #[test]
    fn str_size_emoji_surrogate_pair() {
        let mut s = StrNode::new(ts(sid(), 1));
        // U+1F600 (😀) is a supplementary plane character → 2 UTF-16 code units
        s.ins(ORIGIN, ts(sid(), 2), "😀".into());
        assert_eq!(s.size(), 2, "emoji should count as 2 UTF-16 code units");
    }

    #[test]
    fn str_size_mixed_bmp_and_supplementary() {
        let mut s = StrNode::new(ts(sid(), 1));
        // "a😀b" = 1 + 2 + 1 = 4 UTF-16 code units
        s.ins(ORIGIN, ts(sid(), 2), "a😀b".into());
        assert_eq!(s.size(), 4);
    }

    #[test]
    fn str_size_multiple_supplementary_chars() {
        let mut s = StrNode::new(ts(sid(), 1));
        // "🎉🎊" = 2 + 2 = 4 UTF-16 code units
        s.ins(ORIGIN, ts(sid(), 2), "🎉🎊".into());
        assert_eq!(s.size(), 4);
    }

    #[test]
    fn str_size_cjk_bmp_char() {
        let mut s = StrNode::new(ts(sid(), 1));
        // CJK character U+4E16 (世) is in BMP → 1 UTF-16 code unit
        s.ins(ORIGIN, ts(sid(), 2), "世".into());
        assert_eq!(s.size(), 1, "BMP CJK char should be 1 UTF-16 code unit");
    }

    #[test]
    fn str_size_empty() {
        let s = StrNode::new(ts(sid(), 1));
        assert_eq!(s.size(), 0);
    }

    #[test]
    fn str_find_accounts_for_utf16_spans() {
        let mut s = StrNode::new(ts(sid(), 1));
        // "😀b" — emoji takes positions 0,1; 'b' is at position 2
        s.ins(ORIGIN, ts(sid(), 2), "😀b".into());
        // Position 2 should find the character 'b' (offset 2 within the chunk)
        let found = s.find(2);
        assert!(found.is_some(), "should find char at UTF-16 position 2");
        let found_ts = found.unwrap();
        assert_eq!(found_ts.time, ts(sid(), 2).time + 2);
    }

    // ── NodeIndex TsKey ordering tests ──────────────────────────────────

    #[test]
    fn tskey_orders_by_time_first() {
        let a = TsKey { sid: 100, time: 1 };
        let b = TsKey { sid: 1, time: 2 };
        // b has higher time so a < b regardless of sid
        assert!(a < b);
    }

    #[test]
    fn tskey_orders_by_sid_on_time_tie() {
        let a = TsKey { sid: 1, time: 5 };
        let b = TsKey { sid: 2, time: 5 };
        assert!(a < b);
    }

    #[test]
    fn node_index_iterates_in_time_first_order() {
        let mut index = NodeIndex::new();
        let nodes = [
            (ts(100, 3), "con3"),
            (ts(1, 1), "con1"),
            (ts(50, 2), "con2"),
        ];
        for (id, _label) in &nodes {
            use crate::json_crdt_patch::operations::ConValue;
            index.insert_node(
                *id,
                CrdtNode::Con(ConNode::new(*id, ConValue::Val(PackValue::Null))),
            );
        }
        let times: Vec<u64> = index.keys().map(|k| k.time).collect();
        assert_eq!(
            times,
            vec![1, 2, 3],
            "NodeIndex should iterate time-ascending"
        );
    }

    #[test]
    fn node_index_iterates_sid_ascending_on_time_tie() {
        let mut index = NodeIndex::new();
        let ids = [ts(30, 5), ts(10, 5), ts(20, 5)];
        for id in &ids {
            use crate::json_crdt_patch::operations::ConValue;
            index.insert_node(
                *id,
                CrdtNode::Con(ConNode::new(*id, ConValue::Val(PackValue::Null))),
            );
        }
        let sids: Vec<u64> = index.keys().map(|k| k.sid).collect();
        assert_eq!(
            sids,
            vec![10, 20, 30],
            "same-time entries should order by sid"
        );
    }
}
