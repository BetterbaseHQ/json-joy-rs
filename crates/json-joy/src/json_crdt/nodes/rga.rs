//! RGA (Replicated Growable Array) implementation.
//!
//! Stores chunks in a `Vec<Chunk<T>>` (document order).
//!
//! Complexity (n = number of chunks):
//! - `find_by_id`:   O(n) linear scan
//! - `push_chunk`:   O(1) amortised
//! - `insert`:       O(n) for find + O(n) for Vec::insert
//! - `delete`:       O(k·n) per span
//! - `iter` / `iter_live`: O(n) — slice iteration

use crate::json_crdt_patch::clock::{Ts, Tss, compare};

// ── ChunkData ─────────────────────────────────────────────────────────────

/// Trait for chunk payload types that can be split at a logical item offset.
///
/// Required for partial-chunk deletion: when a deletion range covers only
/// part of a chunk, the chunk must be split before the covered part is
/// marked deleted.
pub trait ChunkData: Clone {
    /// Split `self` at logical offset `at` (number of items before the split).
    /// Modifies `self` to hold items `[0, at)` and returns items `[at, len)`.
    fn split_at_offset(&mut self, at: usize) -> Self;
}

impl ChunkData for String {
    fn split_at_offset(&mut self, at: usize) -> Self {
        let byte_pos = self.char_indices().nth(at).map(|(i, _)| i).unwrap_or(self.len());
        self.split_off(byte_pos)
    }
}

impl ChunkData for Vec<u8> {
    fn split_at_offset(&mut self, at: usize) -> Self {
        self.split_off(at)
    }
}

impl ChunkData for Vec<Ts> {
    fn split_at_offset(&mut self, at: usize) -> Self {
        self.split_off(at)
    }
}

// ── Chunk ─────────────────────────────────────────────────────────────────

/// One chunk in the RGA sequence.
///
/// A chunk represents a contiguous run of items all inserted by the same
/// operation.  Items within a chunk always carry consecutive timestamps
/// `id, id+1, id+2, ...`.
#[derive(Debug, Clone)]
pub struct Chunk<T: Clone> {
    /// Timestamp of the *first* item in this chunk.
    pub id: Ts,
    /// Number of logical items in this chunk (including deleted ones).
    pub span: u64,
    /// Whether all items in this chunk are deleted.
    pub deleted: bool,
    /// The actual content.  `None` if the chunk is a deleted tombstone.
    pub data: Option<T>,
}

impl<T: Clone> Chunk<T> {
    pub fn new(id: Ts, span: u64, data: T) -> Self {
        Self { id, span, deleted: false, data: Some(data) }
    }

    pub fn new_deleted(id: Ts, span: u64) -> Self {
        Self { id, span, deleted: true, data: None }
    }

    pub fn len(&self) -> u64 {
        if self.deleted { 0 } else { self.span }
    }
}

// ── Rga ───────────────────────────────────────────────────────────────────

/// RGA sequence backed by a `Vec<Chunk<T>>`.
///
/// `chunks` stores chunks in document order.
#[derive(Debug, Clone, Default)]
pub struct Rga<T: Clone> {
    chunks: Vec<Chunk<T>>,
}

impl<T: Clone + ChunkData> Rga<T> {
    pub fn new() -> Self {
        Self { chunks: Vec::new() }
    }

    // ── Public accessors ──────────────────────────────────────────────────

    /// Total chunk count (including deleted tombstones).
    pub fn chunk_count(&self) -> usize { self.chunks.len() }

    /// Reference to the chunk at vec index `idx`.
    pub fn slot(&self, idx: usize) -> &Chunk<T> { &self.chunks[idx] }

    /// Mutable reference to the chunk at vec index `idx`.
    pub fn slot_mut(&mut self, idx: usize) -> &mut Chunk<T> { &mut self.chunks[idx] }

    /// Last chunk in document order.
    pub fn last_chunk(&self) -> Option<&Chunk<T>> { self.chunks.last() }

    // ── ID lookup ─────────────────────────────────────────────────────────

    /// Find the vec index of the chunk whose ID range contains `ts`.
    ///
    /// Uses a simple linear scan.
    pub fn find_by_id(&self, ts: Ts) -> Option<usize> {
        for (i, chunk) in self.chunks.iter().enumerate() {
            if chunk.id.sid == ts.sid
                && chunk.id.time <= ts.time
                && chunk.id.time + chunk.span > ts.time
            {
                return Some(i);
            }
        }
        None
    }

    // ── Chunk splitting ───────────────────────────────────────────────────

    /// Split the chunk at `chunk_idx` at logical offset `at_offset`.
    ///
    /// After the call:
    /// - `chunks[chunk_idx]` holds items `[0, at_offset)`.
    /// - `chunks[chunk_idx + 1]` holds items `[at_offset, original_span)`.
    fn split_chunk_at(&mut self, chunk_idx: usize, at_offset: usize) {
        if at_offset == 0 { return; }
        let span = self.chunks[chunk_idx].span;
        if at_offset as u64 >= span { return; }

        let chunk = &mut self.chunks[chunk_idx];
        let id      = chunk.id;
        let deleted = chunk.deleted;
        let right_data = chunk.data.as_mut().map(|d| d.split_at_offset(at_offset));
        chunk.span = at_offset as u64;

        let right_start = id.time + at_offset as u64;
        let right_span  = span - at_offset as u64;
        let right_chunk = match (deleted, right_data) {
            (true, _) | (_, None) => Chunk::new_deleted(Ts::new(id.sid, right_start), right_span),
            (false, Some(data))   => Chunk::new(Ts::new(id.sid, right_start), right_span, data),
        };

        self.chunks.insert(chunk_idx + 1, right_chunk);
    }

    // ── Insert ────────────────────────────────────────────────────────────

    /// Insert `data` with timestamp `id` (span = data length) after the
    /// specific item identified by `after`.  If `after` is the ORIGIN
    /// sentinel `(0, 0)`, insert at the beginning.
    ///
    /// When `after` falls in the middle of a multi-item chunk the chunk is
    /// split so the insertion lands immediately after the targeted item.
    ///
    /// Concurrent inserts at the same position are ordered by
    /// `compare(id, existing)`.
    pub fn insert(&mut self, after: Ts, id: Ts, span: u64, data: T) {
        // Step 1: find the insertion point (right after the `after` item).
        let insert_pos = if after.sid == 0 && after.time == 0 {
            0 // ORIGIN → prepend
        } else {
            match self.find_by_id(after) {
                Some(idx) => {
                    // Split if `after` is not the last item in its chunk.
                    let chunk_last = self.chunks[idx].id.time + self.chunks[idx].span - 1;
                    if after.time < chunk_last {
                        let split_at = (after.time - self.chunks[idx].id.time + 1) as usize;
                        self.split_chunk_at(idx, split_at);
                    }
                    idx + 1
                }
                None => self.chunks.len(),
            }
        };

        // Step 2: skip past concurrent higher-priority inserts.
        let mut pos = insert_pos;
        while pos < self.chunks.len() {
            if compare(self.chunks[pos].id, id) > 0 {
                pos += 1;
            } else {
                break;
            }
        }

        // Step 3: insert the new chunk.
        let new_chunk = Chunk::new(id, span, data);
        if pos == self.chunks.len() {
            self.chunks.push(new_chunk);
        } else {
            self.chunks.insert(pos, new_chunk);
        }
    }

    // ── Append (for codec decode) ─────────────────────────────────────────

    /// Append a pre-built chunk at the document-order tail.
    ///
    /// Used by codec decoders that push chunks in their encoded (document)
    /// order.  O(1) amortised.
    pub fn push_chunk(&mut self, chunk: Chunk<T>) {
        self.chunks.push(chunk);
    }

    // ── Deletion ─────────────────────────────────────────────────────────

    /// Delete all items covered by the given timestamp spans.
    ///
    /// Chunks that are only partially covered are split at the deletion
    /// boundaries so that only the targeted items are removed.
    pub fn delete(&mut self, spans: &[Tss]) {
        for tss in spans {
            let del_start = tss.time;
            let del_end   = tss.time + tss.span; // exclusive upper bound
            let sid       = tss.sid;

            let mut i = 0;
            while i < self.chunks.len() {
                let chunk = &self.chunks[i];

                // Skip chunks from a different session.
                if chunk.id.sid != sid { i += 1; continue; }

                let chunk_start = chunk.id.time;
                let chunk_end   = chunk.id.time + chunk.span;

                // No overlap.
                if chunk_start >= del_end || chunk_end <= del_start { i += 1; continue; }

                let overlap_start = del_start.max(chunk_start);
                let overlap_end   = del_end.min(chunk_end);

                // Split off prefix that precedes the deletion (if any).
                if overlap_start > chunk_start {
                    let prefix_len = (overlap_start - chunk_start) as usize;
                    self.split_chunk_at(i, prefix_len);
                    i += 1; // advance to the right half (starts at overlap_start)
                }

                // Split off suffix that follows the deletion (if any).
                let chunk_end2 = self.chunks[i].id.time + self.chunks[i].span;
                if overlap_end < chunk_end2 {
                    let del_len = (overlap_end - self.chunks[i].id.time) as usize;
                    self.split_chunk_at(i, del_len);
                    // chunks[i] now covers exactly [overlap_start, overlap_end)
                }

                // Mark the targeted chunk as deleted.
                let chunk = &mut self.chunks[i];
                chunk.deleted = true;
                chunk.data    = None;

                i += 1;
            }
        }
    }

    // ── Iteration ─────────────────────────────────────────────────────────

    /// Iterate all chunks in document order.
    pub fn iter(&self) -> impl Iterator<Item = &Chunk<T>> {
        self.chunks.iter()
    }

    /// Iterate live (non-deleted) chunks.
    pub fn iter_live(&self) -> impl Iterator<Item = &Chunk<T>> {
        self.chunks.iter().filter(|c| !c.deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json_crdt_patch::clock::{ts, tss};

    fn origin() -> Ts { ts(0, 0) }
    fn sid() -> u64 { 1 }

    #[test]
    fn insert_single_chunk() {
        let mut rga: Rga<String> = Rga::new();
        rga.insert(origin(), ts(sid(), 1), 5, "hello".to_string());
        assert_eq!(rga.chunk_count(), 1);
        assert_eq!(rga.iter().next().unwrap().data.as_deref(), Some("hello"));
    }

    #[test]
    fn view_after_insert() {
        let mut rga: Rga<String> = Rga::new();
        rga.insert(origin(), ts(sid(), 1), 5, "hello".to_string());
        let s: String = rga.iter_live().filter_map(|c| c.data.as_deref()).collect();
        assert_eq!(s, "hello");
    }

    #[test]
    fn partial_delete_middle() {
        let mut rga: Rga<String> = Rga::new();
        // Insert "hello" at ts(1,1), span=5 → items at times 1,2,3,4,5
        rga.insert(origin(), ts(1, 1), 5, "hello".to_string());
        // Delete 'e','l','l' = tss(1, 2, 3) → times 2,3,4
        rga.delete(&[tss(1, 2, 3)]);

        let s: String = rga.iter_live().filter_map(|c| c.data.as_deref()).collect();
        assert_eq!(s, "ho");
    }

    #[test]
    fn partial_delete_prefix() {
        let mut rga: Rga<String> = Rga::new();
        rga.insert(origin(), ts(1, 1), 5, "hello".to_string());
        // Delete 'h','e' = tss(1, 1, 2)
        rga.delete(&[tss(1, 1, 2)]);
        let s: String = rga.iter_live().filter_map(|c| c.data.as_deref()).collect();
        assert_eq!(s, "llo");
    }

    #[test]
    fn partial_delete_suffix() {
        let mut rga: Rga<String> = Rga::new();
        rga.insert(origin(), ts(1, 1), 5, "hello".to_string());
        // Delete 'l','l','o' = tss(1, 3, 3)
        rga.delete(&[tss(1, 3, 3)]);
        let s: String = rga.iter_live().filter_map(|c| c.data.as_deref()).collect();
        assert_eq!(s, "he");
    }

    #[test]
    fn delete_full_chunk() {
        let mut rga: Rga<String> = Rga::new();
        rga.insert(origin(), ts(1, 1), 5, "hello".to_string());
        rga.delete(&[tss(1, 1, 5)]);
        let s: String = rga.iter_live().filter_map(|c| c.data.as_deref()).collect();
        assert_eq!(s, "");
    }

    #[test]
    fn two_chunk_delete_spanning_boundary() {
        let mut rga: Rga<String> = Rga::new();
        // "he" at ts(1,1), "llo" at ts(1,3) inserted after chunk 1
        rga.insert(origin(),   ts(1, 1), 2, "he".to_string());
        rga.insert(ts(1, 2),   ts(1, 3), 3, "llo".to_string());
        // Delete 'e','l' spanning both chunks = tss(1, 2, 2)
        rga.delete(&[tss(1, 2, 2)]);
        let s: String = rga.iter_live().filter_map(|c| c.data.as_deref()).collect();
        assert_eq!(s, "hlo");
    }

    /// Convergence test: two peers apply the same concurrent inserts at the same
    /// position in different orders and must produce identical final views.
    #[test]
    fn concurrent_inserts_converge_regardless_of_application_order() {
        let build = |order: &[(u64, u64)]| -> String {
            let mut rga: Rga<String> = Rga::new();
            rga.insert(origin(), ts(1, 1), 1, "A".to_string());
            for &(sid, time) in order {
                rga.insert(ts(1, 1), ts(sid, time), 1, sid.to_string());
            }
            rga.iter_live().filter_map(|c| c.data.as_deref()).collect()
        };

        let view_a = build(&[(2, 1), (3, 1)]);
        let view_b = build(&[(3, 1), (2, 1)]);
        assert_eq!(view_a, view_b, "concurrent inserts must converge");
        let pos3 = view_a.find('3').unwrap();
        let pos2 = view_a.find('2').unwrap();
        assert!(pos3 < pos2, "higher-priority (sid=3) chunk should precede sid=2 chunk");
    }

    #[test]
    fn split_at_offset_multibyte_chars() {
        let mut rga: Rga<String> = Rga::new();
        rga.insert(origin(), ts(1, 1), 3, "A😀B".to_string());
        rga.delete(&[tss(1, 2, 1)]);
        let s: String = rga.iter_live().filter_map(|c| c.data.as_deref()).collect();
        assert_eq!(s, "AB");
    }

    #[test]
    fn insert_after_mid_chunk_character_with_higher_priority() {
        let mut rga: Rga<String> = Rga::new();
        rga.insert(origin(), ts(1, 1), 5, "hello".to_string());
        rga.insert(ts(1, 3), ts(2, 1000), 1, "X".to_string());
        let s: String = rga.iter_live().filter_map(|c| c.data.as_deref()).collect();
        assert_eq!(s, "helXlo");
    }

    #[test]
    fn push_chunk_builds_sequence_in_order() {
        let mut rga: Rga<String> = Rga::new();
        rga.push_chunk(Chunk::new(ts(1, 1), 5, "hello".to_string()));
        rga.push_chunk(Chunk::new(ts(1, 6), 1, " ".to_string()));
        rga.push_chunk(Chunk::new(ts(1, 7), 5, "world".to_string()));
        let s: String = rga.iter_live().filter_map(|c| c.data.as_deref()).collect();
        assert_eq!(s, "hello world");
        assert_eq!(rga.chunk_count(), 3);
    }

    #[test]
    fn find_by_id_locates_mid_chunk_item() {
        let mut rga: Rga<String> = Rga::new();
        rga.insert(origin(), ts(1, 1), 5, "hello".to_string());
        assert!(rga.find_by_id(ts(1, 3)).is_some());
        assert!(rga.find_by_id(ts(2, 1)).is_none());
    }
}
