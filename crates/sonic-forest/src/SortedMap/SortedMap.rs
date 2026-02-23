use super::constants::IteratorType;
use super::sorted_map_iterator::OrderedMapIterator;
use super::util::throw_iterator_access_error;
use crate::red_black::{insert, insert_left, insert_right, remove, RbNode};
use crate::util::{first, last, next, prev};

fn default_comparator<K: PartialOrd>(a: &K, b: &K) -> i32 {
    if a == b {
        0
    } else if a < b {
        -1
    } else {
        1
    }
}

/// Mirrors upstream `SortedMap/SortedMap.ts` public API shape.
///
/// Rust divergence:
/// - Uses arena indices (`u32`) instead of object references for root/min/max.
/// - Keeps iterator API shape, but iterator state is position-based.
pub struct SortedMap<K, V, C = fn(&K, &K) -> i32>
where
    C: Fn(&K, &K) -> i32,
{
    pub enable_index: bool,
    pub min: Option<u32>,
    pub root: Option<u32>,
    pub max: Option<u32>,
    pub comparator: C,
    arena: Vec<RbNode<K, V>>,
    _length: usize,
    /// When `true`, subtree sizes need recomputation before rank queries.
    sizes_dirty: bool,
}

impl<K, V> SortedMap<K, V, fn(&K, &K) -> i32>
where
    K: PartialOrd,
{
    pub fn new() -> Self {
        Self::with_comparator(default_comparator::<K>, false)
    }
}

impl<K, V> Default for SortedMap<K, V, fn(&K, &K) -> i32>
where
    K: PartialOrd,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V, C> SortedMap<K, V, C>
where
    C: Fn(&K, &K) -> i32,
{
    pub fn with_comparator(comparator: C, enable_index: bool) -> Self {
        Self {
            enable_index,
            min: None,
            root: None,
            max: None,
            comparator,
            arena: Vec::new(),
            _length: 0,
            sizes_dirty: false,
        }
    }

    #[inline]
    fn compare(&self, a: &K, b: &K) -> i32 {
        (self.comparator)(a, b)
    }

    fn find_node(&self, key: &K) -> Option<u32> {
        let mut curr = self.root;
        while let Some(i) = curr {
            let cmp = self.compare(key, &self.arena[i as usize].k);
            if cmp == 0 {
                return Some(i);
            }
            curr = if cmp < 0 {
                self.arena[i as usize].l
            } else {
                self.arena[i as usize].r
            };
        }
        None
    }

    fn nth_index(&self, pos: usize) -> Option<u32> {
        if pos >= self._length {
            return None;
        }
        let mut curr = self.min?;
        for _ in 0..pos {
            curr = next(&self.arena, curr)?;
        }
        Some(curr)
    }

    /// Compute the 0-based rank (sequential position) of `node` in O(log n)
    /// using subtree sizes. Calls `ensure_sizes()` first if sizes are dirty.
    fn index_of(&mut self, node: u32) -> usize {
        self.ensure_sizes();
        Self::rank_of(&self.arena, node)
    }

    /// Compute rank of `node` using `_size` fields. O(log n).
    fn rank_of(arena: &[RbNode<K, V>], mut node: u32) -> usize {
        let mut rank = arena[node as usize]
            .l
            .map(|l| arena[l as usize]._size as usize)
            .unwrap_or(0);
        while let Some(parent) = arena[node as usize].p {
            if arena[parent as usize].r == Some(node) {
                rank += 1; // count parent itself
                rank += arena[parent as usize]
                    .l
                    .map(|l| arena[l as usize]._size as usize)
                    .unwrap_or(0);
            }
            node = parent;
        }
        rank
    }

    /// Recompute subtree sizes via post-order DFS if the dirty flag is set.
    fn ensure_sizes(&mut self) {
        if !self.sizes_dirty {
            return;
        }
        if let Some(root) = self.root {
            Self::recompute_subtree_sizes(&mut self.arena, root);
        }
        self.sizes_dirty = false;
    }

    /// Post-order DFS to set correct `_size` on every node in the subtree.
    fn recompute_subtree_sizes(arena: &mut [RbNode<K, V>], node: u32) -> u32 {
        let l = arena[node as usize].l;
        let r = arena[node as usize].r;
        let ls = l
            .map(|l| Self::recompute_subtree_sizes(arena, l))
            .unwrap_or(0);
        let rs = r
            .map(|r| Self::recompute_subtree_sizes(arena, r))
            .unwrap_or(0);
        let size = 1 + ls + rs;
        arena[node as usize]._size = size;
        size
    }

    fn lower_bound_node(&self, key: &K) -> Option<u32> {
        let mut curr = self.root;
        let mut res = None;
        while let Some(i) = curr {
            let cmp = self.compare(&self.arena[i as usize].k, key);
            if cmp < 0 {
                curr = self.arena[i as usize].r;
            } else if cmp > 0 {
                res = Some(i);
                curr = self.arena[i as usize].l;
            } else {
                return Some(i);
            }
        }
        res
    }

    fn upper_bound_node(&self, key: &K) -> Option<u32> {
        let mut curr = self.root;
        let mut res = None;
        while let Some(i) = curr {
            let cmp = self.compare(&self.arena[i as usize].k, key);
            if cmp <= 0 {
                curr = self.arena[i as usize].r;
            } else {
                res = Some(i);
                curr = self.arena[i as usize].l;
            }
        }
        res
    }

    fn reverse_lower_bound_node(&self, key: &K) -> Option<u32> {
        let mut curr = self.root;
        let mut res = None;
        while let Some(i) = curr {
            let cmp = self.compare(&self.arena[i as usize].k, key);
            if cmp < 0 {
                res = Some(i);
                curr = self.arena[i as usize].r;
            } else if cmp > 0 {
                curr = self.arena[i as usize].l;
            } else {
                return Some(i);
            }
        }
        res
    }

    fn reverse_upper_bound_node(&self, key: &K) -> Option<u32> {
        let mut curr = self.root;
        let mut res = None;
        while let Some(i) = curr {
            let cmp = self.compare(&self.arena[i as usize].k, key);
            if cmp < 0 {
                res = Some(i);
                curr = self.arena[i as usize].r;
            } else {
                curr = self.arena[i as usize].l;
            }
        }
        res
    }

    fn remove_node(&mut self, node: u32) {
        if self.max == Some(node) {
            self.max = prev(&self.arena, node);
        }
        if self.min == Some(node) {
            self.min = next(&self.arena, node);
        }

        self.root = remove(&mut self.arena, self.root, node);
        if self._length > 0 {
            self._length -= 1;
        }
        self.sizes_dirty = true;

        if self.root.is_none() {
            self.min = None;
            self.max = None;
            self._length = 0;
            self.sizes_dirty = false;
        } else {
            if self.min.is_none() {
                self.min = first(&self.arena, self.root);
            }
            if self.max.is_none() {
                self.max = last(&self.arena, self.root);
            }
        }
    }

    pub fn length(&self) -> usize {
        self._length
    }

    pub fn empty(&self) -> bool {
        self._length == 0
    }

    pub fn set_element(&mut self, key: K, value: V, _hint: Option<&OrderedMapIterator>) -> usize {
        if self.root.is_none() {
            self.arena.push(RbNode::new(key, value));
            let idx = (self.arena.len() - 1) as u32;
            self.root = insert(&mut self.arena, None, idx, &self.comparator);
            self.min = self.root;
            self.max = self.root;
            self._length = 1;
            // Single node: _size = 1, already correct.
            return self._length;
        }

        let root = self.root.expect("root exists");

        let max = self.max.expect("max exists");
        let max_cmp = self.compare(&key, &self.arena[max as usize].k);
        if max_cmp == 0 {
            self.arena[max as usize].v = value;
            return self._length;
        }
        if max_cmp > 0 {
            self.arena.push(RbNode::new(key, value));
            let idx = (self.arena.len() - 1) as u32;
            self.root = insert_right(&mut self.arena, Some(root), idx, max);
            self.max = Some(idx);
            self._length += 1;
            self.sizes_dirty = true;
            return self._length;
        }

        let min = self.min.expect("min exists");
        let min_cmp = self.compare(&key, &self.arena[min as usize].k);
        if min_cmp == 0 {
            self.arena[min as usize].v = value;
            return self._length;
        }
        if min_cmp < 0 {
            self.arena.push(RbNode::new(key, value));
            let idx = (self.arena.len() - 1) as u32;
            self.root = insert_left(&mut self.arena, Some(root), idx, min);
            self.min = Some(idx);
            self._length += 1;
            self.sizes_dirty = true;
            return self._length;
        }

        let mut curr = root;
        loop {
            let cmp = self.compare(&key, &self.arena[curr as usize].k);
            if cmp == 0 {
                self.arena[curr as usize].v = value;
                return self._length;
            }
            if cmp > 0 {
                match self.arena[curr as usize].r {
                    Some(next) => curr = next,
                    None => {
                        self.arena.push(RbNode::new(key, value));
                        let idx = (self.arena.len() - 1) as u32;
                        self.root = insert_right(&mut self.arena, self.root, idx, curr);
                        self._length += 1;
                        self.sizes_dirty = true;
                        return self._length;
                    }
                }
            } else {
                match self.arena[curr as usize].l {
                    Some(next) => curr = next,
                    None => {
                        self.arena.push(RbNode::new(key, value));
                        let idx = (self.arena.len() - 1) as u32;
                        self.root = insert_left(&mut self.arena, self.root, idx, curr);
                        self._length += 1;
                        self.sizes_dirty = true;
                        return self._length;
                    }
                }
            }
        }
    }

    #[allow(non_snake_case)]
    pub fn setElement(&mut self, key: K, value: V, hint: Option<&OrderedMapIterator>) -> usize {
        self.set_element(key, value, hint)
    }

    pub fn erase_element_by_key(&mut self, key: &K) -> bool {
        let Some(node) = self.find_node(key) else {
            return false;
        };
        self.remove_node(node);
        true
    }

    #[allow(non_snake_case)]
    pub fn eraseElementByKey(&mut self, key: &K) -> bool {
        self.erase_element_by_key(key)
    }

    pub fn get_element_by_key(&self, key: &K) -> Option<&V> {
        self.find_node(key).map(|idx| &self.arena[idx as usize].v)
    }

    #[allow(non_snake_case)]
    pub fn getElementByKey(&self, key: &K) -> Option<&V> {
        self.get_element_by_key(key)
    }

    pub fn update_key_by_iterator(&mut self, iter: &OrderedMapIterator, key: K) -> bool {
        let Some(pos) = iter.position() else {
            throw_iterator_access_error();
        };
        if pos >= self._length {
            throw_iterator_access_error();
        }

        let idx = if let Some(node) = iter.node {
            node
        } else {
            let Some(idx) = self.nth_index(pos) else {
                throw_iterator_access_error();
            };
            idx
        };

        if self._length == 1 {
            self.arena[idx as usize].k = key;
            return true;
        }

        if pos == 0 {
            let next_idx = next(&self.arena, idx).expect("next exists for first node");
            if self.compare(&self.arena[next_idx as usize].k, &key) > 0 {
                self.arena[idx as usize].k = key;
                return true;
            }
            return false;
        }

        if pos == self._length - 1 {
            let prev_idx = prev(&self.arena, idx).expect("prev exists for last node");
            if self.compare(&self.arena[prev_idx as usize].k, &key) < 0 {
                self.arena[idx as usize].k = key;
                return true;
            }
            return false;
        }

        let prev_idx = prev(&self.arena, idx).expect("prev exists");
        let next_idx = next(&self.arena, idx).expect("next exists");
        let pre_ok = self.compare(&self.arena[prev_idx as usize].k, &key) < 0;
        let next_ok = self.compare(&self.arena[next_idx as usize].k, &key) > 0;
        if pre_ok && next_ok {
            self.arena[idx as usize].k = key;
            true
        } else {
            false
        }
    }

    #[allow(non_snake_case)]
    pub fn updateKeyByIterator(&mut self, iter: &OrderedMapIterator, key: K) -> bool {
        self.update_key_by_iterator(iter, key)
    }

    pub fn erase_element_by_iterator(&mut self, iter: OrderedMapIterator) -> OrderedMapIterator {
        let Some(pos) = iter.position() else {
            throw_iterator_access_error();
        };
        if pos >= self._length {
            throw_iterator_access_error();
        }

        let node = if let Some(n) = iter.node {
            n
        } else {
            let Some(n) = self.nth_index(pos) else {
                throw_iterator_access_error();
            };
            n
        };

        let mut out = iter.copy();
        let old_len = self._length;

        match out.iterator_type {
            IteratorType::Normal => {
                if pos + 1 >= old_len {
                    out.set_position(old_len);
                } else {
                    out.set_position(pos);
                }
            }
            IteratorType::Reverse => {
                if pos == 0 {
                    out.set_position(old_len);
                } else {
                    out.set_position(pos - 1);
                }
            }
        }

        self.remove_node(node);
        out.sync_len(self._length);
        out
    }

    #[allow(non_snake_case)]
    pub fn eraseElementByIterator(&mut self, iter: OrderedMapIterator) -> OrderedMapIterator {
        self.erase_element_by_iterator(iter)
    }

    pub fn erase_element_by_pos(&self, _pos: usize) -> ! {
        panic!("Method not implemented.")
    }

    #[allow(non_snake_case)]
    pub fn eraseElementByPos(&self, pos: usize) -> ! {
        self.erase_element_by_pos(pos)
    }

    pub fn get_height(&self) -> usize {
        fn height<K, V>(arena: &[RbNode<K, V>], root: Option<u32>) -> usize {
            let Some(i) = root else {
                return 0;
            };
            let n = &arena[i as usize];
            1 + height(arena, n.l).max(height(arena, n.r))
        }
        height(&self.arena, self.root)
    }

    #[allow(non_snake_case)]
    pub fn getHeight(&self) -> usize {
        self.get_height()
    }

    pub fn begin(&self) -> OrderedMapIterator {
        let pos = if self._length == 0 { self._length } else { 0 };
        OrderedMapIterator::new(pos, self._length, IteratorType::NORMAL)
    }

    pub fn end(&self) -> OrderedMapIterator {
        OrderedMapIterator::new(self._length, self._length, IteratorType::NORMAL)
    }

    pub fn r_begin(&self) -> OrderedMapIterator {
        let pos = if self._length == 0 {
            self._length
        } else {
            self._length - 1
        };
        OrderedMapIterator::new(pos, self._length, IteratorType::REVERSE)
    }

    #[allow(non_snake_case)]
    pub fn rBegin(&self) -> OrderedMapIterator {
        self.r_begin()
    }

    pub fn r_end(&self) -> OrderedMapIterator {
        OrderedMapIterator::new(self._length, self._length, IteratorType::REVERSE)
    }

    #[allow(non_snake_case)]
    pub fn rEnd(&self) -> OrderedMapIterator {
        self.r_end()
    }

    pub fn front(&self) -> Option<(&K, &V)> {
        self.min.map(|i| {
            let n = &self.arena[i as usize];
            (&n.k, &n.v)
        })
    }

    pub fn back(&self) -> Option<(&K, &V)> {
        self.max.map(|i| {
            let n = &self.arena[i as usize];
            (&n.k, &n.v)
        })
    }

    pub fn lower_bound(&mut self, key: &K) -> OrderedMapIterator {
        let node = self.lower_bound_node(key);
        let pos = node.map_or(self._length, |i| self.index_of(i));
        OrderedMapIterator::with_node(pos, self._length, IteratorType::NORMAL, node)
    }

    #[allow(non_snake_case)]
    pub fn lowerBound(&mut self, key: &K) -> OrderedMapIterator {
        self.lower_bound(key)
    }

    pub fn upper_bound(&mut self, key: &K) -> OrderedMapIterator {
        let node = self.upper_bound_node(key);
        let pos = node.map_or(self._length, |i| self.index_of(i));
        OrderedMapIterator::with_node(pos, self._length, IteratorType::NORMAL, node)
    }

    #[allow(non_snake_case)]
    pub fn upperBound(&mut self, key: &K) -> OrderedMapIterator {
        self.upper_bound(key)
    }

    pub fn reverse_lower_bound(&mut self, key: &K) -> OrderedMapIterator {
        let node = self.reverse_lower_bound_node(key);
        let pos = node.map_or(self._length, |i| self.index_of(i));
        OrderedMapIterator::with_node(pos, self._length, IteratorType::NORMAL, node)
    }

    #[allow(non_snake_case)]
    pub fn reverseLowerBound(&mut self, key: &K) -> OrderedMapIterator {
        self.reverse_lower_bound(key)
    }

    pub fn reverse_upper_bound(&mut self, key: &K) -> OrderedMapIterator {
        let node = self.reverse_upper_bound_node(key);
        let pos = node.map_or(self._length, |i| self.index_of(i));
        OrderedMapIterator::with_node(pos, self._length, IteratorType::NORMAL, node)
    }

    #[allow(non_snake_case)]
    pub fn reverseUpperBound(&mut self, key: &K) -> OrderedMapIterator {
        self.reverse_upper_bound(key)
    }

    pub fn get_element_by_pos(&self, _pos: usize) -> ! {
        panic!("Method not implemented.")
    }

    #[allow(non_snake_case)]
    pub fn getElementByPos(&self, pos: usize) -> ! {
        self.get_element_by_pos(pos)
    }

    pub fn clear(&mut self) {
        self.arena.clear();
        self._length = 0;
        self.min = None;
        self.root = None;
        self.max = None;
        self.sizes_dirty = false;
    }

    pub fn size(&self) -> usize {
        self._length
    }

    pub fn is_empty(&self) -> bool {
        self.min.is_none()
    }

    #[allow(non_snake_case)]
    pub fn isEmpty(&self) -> bool {
        self.is_empty()
    }

    pub fn to_string(&self, _tab: &str) -> String {
        format!("SortedMap(len={})", self._length)
    }

    // SonicMap API stubs (mirrors upstream unimplemented methods).
    pub fn set(&mut self, _k: K, _v: V) -> ! {
        panic!("Method not implemented.")
    }

    pub fn find(&self, _k: &K) -> ! {
        panic!("Method not implemented.")
    }

    pub fn get(&self, _k: &K) -> ! {
        panic!("Method not implemented.")
    }

    pub fn del(&mut self, _k: &K) -> ! {
        panic!("Method not implemented.")
    }

    pub fn has(&self, _k: &K) -> bool {
        panic!("Method not implemented.")
    }

    pub fn get_or_next_lower(&self, _k: &K) -> ! {
        panic!("Method not implemented.")
    }

    #[allow(non_snake_case)]
    pub fn getOrNextLower(&self, k: &K) -> ! {
        self.get_or_next_lower(k)
    }

    pub fn for_each<F: FnMut()>(&self, _f: F) -> ! {
        panic!("Method not implemented.")
    }

    pub fn first(&self) -> ! {
        panic!("Method not implemented.")
    }

    pub fn last(&self) -> ! {
        panic!("Method not implemented.")
    }

    pub fn iterator0(&self) -> ! {
        panic!("Method not implemented.")
    }

    pub fn iterator(&self) -> ! {
        panic!("Method not implemented.")
    }

    pub fn entries(&self) -> ! {
        panic!("Method not implemented.")
    }
}
