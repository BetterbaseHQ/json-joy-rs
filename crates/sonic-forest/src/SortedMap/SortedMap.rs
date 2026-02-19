use super::constants::IteratorType;
use super::sorted_map_iterator::OrderedMapIterator;
use super::util::throw_iterator_access_error;

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
/// Rust divergence: uses an ordered-vector backend instead of mutable
/// node-pointer red-black internals. API behavior for supported operations
/// is preserved for parity tests.
pub struct SortedMap<K, V, C = fn(&K, &K) -> i32>
where
    C: Fn(&K, &K) -> i32,
{
    pub enable_index: bool,
    pub min: Option<usize>,
    pub root: Option<usize>,
    pub max: Option<usize>,
    pub comparator: C,
    entries: Vec<(K, V)>,
    _length: usize,
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
            entries: Vec::new(),
            _length: 0,
        }
    }

    fn update_markers(&mut self) {
        if self._length == 0 {
            self.min = None;
            self.root = None;
            self.max = None;
            return;
        }
        self.min = Some(0);
        self.max = Some(self._length - 1);
        self.root = Some(self._length / 2);
    }

    fn compare(&self, a: &K, b: &K) -> i32 {
        (self.comparator)(a, b)
    }

    fn lower_bound_idx(&self, key: &K) -> usize {
        let mut lo = 0usize;
        let mut hi = self._length;
        while lo < hi {
            let mid = (lo + hi) / 2;
            if self.compare(&self.entries[mid].0, key) < 0 {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        lo
    }

    fn upper_bound_idx(&self, key: &K) -> usize {
        let mut lo = 0usize;
        let mut hi = self._length;
        while lo < hi {
            let mid = (lo + hi) / 2;
            if self.compare(&self.entries[mid].0, key) <= 0 {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        lo
    }

    fn find_idx(&self, key: &K) -> Option<usize> {
        let idx = self.lower_bound_idx(key);
        if idx < self._length && self.compare(&self.entries[idx].0, key) == 0 {
            Some(idx)
        } else {
            None
        }
    }

    pub fn length(&self) -> usize {
        self._length
    }

    pub fn empty(&self) -> bool {
        self._length == 0
    }

    pub fn set_element(&mut self, key: K, value: V, _hint: Option<&OrderedMapIterator>) -> usize {
        if let Some(idx) = self.find_idx(&key) {
            self.entries[idx].1 = value;
            return self._length;
        }
        let idx = self.lower_bound_idx(&key);
        self.entries.insert(idx, (key, value));
        self._length += 1;
        self.update_markers();
        self._length
    }

    #[allow(non_snake_case)]
    pub fn setElement(&mut self, key: K, value: V, hint: Option<&OrderedMapIterator>) -> usize {
        self.set_element(key, value, hint)
    }

    pub fn erase_element_by_key(&mut self, key: &K) -> bool {
        let Some(idx) = self.find_idx(key) else {
            return false;
        };
        self.entries.remove(idx);
        self._length -= 1;
        self.update_markers();
        true
    }

    #[allow(non_snake_case)]
    pub fn eraseElementByKey(&mut self, key: &K) -> bool {
        self.erase_element_by_key(key)
    }

    pub fn get_element_by_key(&self, key: &K) -> Option<&V> {
        self.find_idx(key).map(|idx| &self.entries[idx].1)
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

        if self._length == 1 {
            self.entries[0].0 = key;
            return true;
        }

        if pos == 0 {
            if self.compare(&self.entries[1].0, &key) > 0 {
                self.entries[0].0 = key;
                return true;
            }
            return false;
        }

        if pos == self._length - 1 {
            if self.compare(&self.entries[self._length - 2].0, &key) < 0 {
                self.entries[pos].0 = key;
                return true;
            }
            return false;
        }

        let pre_ok = self.compare(&self.entries[pos - 1].0, &key) < 0;
        let next_ok = self.compare(&self.entries[pos + 1].0, &key) > 0;
        if pre_ok && next_ok {
            self.entries[pos].0 = key;
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

        self.entries.remove(pos);
        self._length -= 1;
        self.update_markers();
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
        if self._length == 0 {
            0
        } else {
            ((self._length as f64 + 1.0).log2().ceil()) as usize
        }
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
        self.entries.first().map(|(k, v)| (k, v))
    }

    pub fn back(&self) -> Option<(&K, &V)> {
        self.entries.last().map(|(k, v)| (k, v))
    }

    pub fn lower_bound(&self, key: &K) -> OrderedMapIterator {
        let idx = self.lower_bound_idx(key);
        OrderedMapIterator::new(idx, self._length, IteratorType::NORMAL)
    }

    #[allow(non_snake_case)]
    pub fn lowerBound(&self, key: &K) -> OrderedMapIterator {
        self.lower_bound(key)
    }

    pub fn upper_bound(&self, key: &K) -> OrderedMapIterator {
        let idx = self.upper_bound_idx(key);
        OrderedMapIterator::new(idx, self._length, IteratorType::NORMAL)
    }

    #[allow(non_snake_case)]
    pub fn upperBound(&self, key: &K) -> OrderedMapIterator {
        self.upper_bound(key)
    }

    pub fn reverse_lower_bound(&self, key: &K) -> OrderedMapIterator {
        let idx = self.upper_bound_idx(key);
        let pos = if idx == 0 { self._length } else { idx - 1 };
        OrderedMapIterator::new(pos, self._length, IteratorType::NORMAL)
    }

    #[allow(non_snake_case)]
    pub fn reverseLowerBound(&self, key: &K) -> OrderedMapIterator {
        self.reverse_lower_bound(key)
    }

    pub fn reverse_upper_bound(&self, key: &K) -> OrderedMapIterator {
        let idx = self.lower_bound_idx(key);
        let pos = if idx == 0 { self._length } else { idx - 1 };
        OrderedMapIterator::new(pos, self._length, IteratorType::NORMAL)
    }

    #[allow(non_snake_case)]
    pub fn reverseUpperBound(&self, key: &K) -> OrderedMapIterator {
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
        self.entries.clear();
        self._length = 0;
        self.update_markers();
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
