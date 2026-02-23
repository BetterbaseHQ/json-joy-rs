use super::constants::IteratorType;
use super::util::throw_iterator_access_error;

#[derive(Clone, Debug)]
pub struct OrderedMapIterator {
    pos: usize,
    len: usize,
    pub iterator_type: IteratorType,
    /// Arena index of the tree node this iterator points to. Set by bound
    /// methods and used by `update_key_by_iterator`/`erase_element_by_iterator`
    /// to avoid O(n) `nth_index` lookups.
    pub(crate) node: Option<u32>,
}

impl OrderedMapIterator {
    pub fn new(pos: usize, len: usize, iterator_type: IteratorType) -> Self {
        Self {
            pos,
            len,
            iterator_type,
            node: None,
        }
    }

    pub fn with_node(
        pos: usize,
        len: usize,
        iterator_type: IteratorType,
        node: Option<u32>,
    ) -> Self {
        Self {
            pos,
            len,
            iterator_type,
            node,
        }
    }

    pub fn pre(&mut self) -> &mut Self {
        match self.iterator_type {
            IteratorType::Normal => {
                if self.len == 0 || self.pos == 0 {
                    throw_iterator_access_error();
                }
                self.pos -= 1;
            }
            IteratorType::Reverse => {
                if self.len == 0 {
                    throw_iterator_access_error();
                }
                if self.pos == self.len - 1 {
                    throw_iterator_access_error();
                }
                if self.pos == self.len {
                    self.pos = 0;
                } else {
                    self.pos += 1;
                }
            }
        }
        self.node = None; // position changed; cached node is stale
        self
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> &mut Self {
        match self.iterator_type {
            IteratorType::Normal => {
                if self.pos == self.len {
                    throw_iterator_access_error();
                }
                self.pos += 1;
            }
            IteratorType::Reverse => {
                if self.pos == self.len {
                    throw_iterator_access_error();
                }
                if self.pos == 0 {
                    self.pos = self.len;
                } else {
                    self.pos -= 1;
                }
            }
        }
        self.node = None; // position changed; cached node is stale
        self
    }

    pub fn index(&self) -> usize {
        if self.pos == self.len {
            self.len.saturating_sub(1)
        } else {
            self.pos
        }
    }

    pub fn is_accessible(&self) -> bool {
        self.pos != self.len
    }

    pub fn copy(&self) -> Self {
        self.clone()
    }

    pub fn equals(&self, other: &Self) -> bool {
        self.pos == other.pos && self.iterator_type == other.iterator_type
    }

    pub(crate) fn position(&self) -> Option<usize> {
        if self.is_accessible() {
            Some(self.pos)
        } else {
            None
        }
    }

    pub(crate) fn sync_len(&mut self, len: usize) {
        self.len = len;
        if self.pos > len {
            self.pos = len;
        }
    }

    pub(crate) fn set_position(&mut self, pos: usize) {
        self.pos = pos;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- new / with_node ---

    #[test]
    fn test_new() {
        let it = OrderedMapIterator::new(0, 5, IteratorType::Normal);
        assert_eq!(it.index(), 0);
        assert!(it.is_accessible());
        assert!(it.node.is_none());
    }

    #[test]
    fn test_with_node() {
        let it = OrderedMapIterator::with_node(2, 5, IteratorType::Normal, Some(42));
        assert_eq!(it.index(), 2);
        assert_eq!(it.node, Some(42));
    }

    // --- is_accessible ---

    #[test]
    fn test_is_accessible_at_end() {
        let it = OrderedMapIterator::new(5, 5, IteratorType::Normal);
        assert!(!it.is_accessible());
    }

    #[test]
    fn test_is_accessible_before_end() {
        let it = OrderedMapIterator::new(4, 5, IteratorType::Normal);
        assert!(it.is_accessible());
    }

    // --- index ---

    #[test]
    fn test_index_at_end_returns_last() {
        let it = OrderedMapIterator::new(5, 5, IteratorType::Normal);
        assert_eq!(it.index(), 4); // saturating_sub(1)
    }

    #[test]
    fn test_index_at_end_empty_list() {
        let it = OrderedMapIterator::new(0, 0, IteratorType::Normal);
        assert_eq!(it.index(), 0); // 0.saturating_sub(1) = 0
    }

    // --- next (Normal) ---

    #[test]
    fn test_next_normal_advances() {
        let mut it = OrderedMapIterator::new(0, 3, IteratorType::Normal);
        it.next();
        assert_eq!(it.index(), 1);
        it.next();
        assert_eq!(it.index(), 2);
    }

    #[test]
    fn test_next_normal_to_end() {
        let mut it = OrderedMapIterator::new(2, 3, IteratorType::Normal);
        it.next();
        assert!(!it.is_accessible());
    }

    #[test]
    #[should_panic(expected = "Iterator access denied!")]
    fn test_next_normal_past_end_panics() {
        let mut it = OrderedMapIterator::new(3, 3, IteratorType::Normal);
        it.next();
    }

    #[test]
    fn test_next_clears_node_cache() {
        let mut it = OrderedMapIterator::with_node(0, 3, IteratorType::Normal, Some(10));
        it.next();
        assert!(it.node.is_none());
    }

    // --- pre (Normal) ---

    #[test]
    fn test_pre_normal_decrements() {
        let mut it = OrderedMapIterator::new(2, 3, IteratorType::Normal);
        it.pre();
        assert_eq!(it.index(), 1);
    }

    #[test]
    #[should_panic(expected = "Iterator access denied!")]
    fn test_pre_normal_at_zero_panics() {
        let mut it = OrderedMapIterator::new(0, 3, IteratorType::Normal);
        it.pre();
    }

    #[test]
    #[should_panic(expected = "Iterator access denied!")]
    fn test_pre_normal_empty_panics() {
        let mut it = OrderedMapIterator::new(0, 0, IteratorType::Normal);
        it.pre();
    }

    #[test]
    fn test_pre_clears_node_cache() {
        let mut it = OrderedMapIterator::with_node(1, 3, IteratorType::Normal, Some(5));
        it.pre();
        assert!(it.node.is_none());
    }

    // --- next (Reverse) ---

    #[test]
    fn test_next_reverse_decrements() {
        let mut it = OrderedMapIterator::new(2, 3, IteratorType::Reverse);
        it.next();
        assert_eq!(it.position(), Some(1));
    }

    #[test]
    fn test_next_reverse_from_zero_wraps_to_end() {
        let mut it = OrderedMapIterator::new(0, 3, IteratorType::Reverse);
        it.next();
        assert!(!it.is_accessible()); // pos == len
    }

    #[test]
    #[should_panic(expected = "Iterator access denied!")]
    fn test_next_reverse_past_end_panics() {
        let mut it = OrderedMapIterator::new(3, 3, IteratorType::Reverse);
        it.next();
    }

    // --- pre (Reverse) ---

    #[test]
    fn test_pre_reverse_increments() {
        let mut it = OrderedMapIterator::new(0, 3, IteratorType::Reverse);
        it.pre();
        assert_eq!(it.position(), Some(1));
    }

    #[test]
    fn test_pre_reverse_from_end_wraps_to_zero() {
        let mut it = OrderedMapIterator::new(3, 3, IteratorType::Reverse);
        // pos == len => wraps to 0
        it.pre();
        assert_eq!(it.position(), Some(0));
    }

    #[test]
    #[should_panic(expected = "Iterator access denied!")]
    fn test_pre_reverse_at_last_panics() {
        // pos == len - 1 => panics
        let mut it = OrderedMapIterator::new(2, 3, IteratorType::Reverse);
        it.pre();
    }

    #[test]
    #[should_panic(expected = "Iterator access denied!")]
    fn test_pre_reverse_empty_panics() {
        let mut it = OrderedMapIterator::new(0, 0, IteratorType::Reverse);
        it.pre();
    }

    // --- copy / equals ---

    #[test]
    fn test_copy() {
        let it = OrderedMapIterator::new(2, 5, IteratorType::Normal);
        let copy = it.copy();
        assert!(it.equals(&copy));
    }

    #[test]
    fn test_equals_same() {
        let a = OrderedMapIterator::new(1, 5, IteratorType::Normal);
        let b = OrderedMapIterator::new(1, 5, IteratorType::Normal);
        assert!(a.equals(&b));
    }

    #[test]
    fn test_equals_different_pos() {
        let a = OrderedMapIterator::new(1, 5, IteratorType::Normal);
        let b = OrderedMapIterator::new(2, 5, IteratorType::Normal);
        assert!(!a.equals(&b));
    }

    #[test]
    fn test_equals_different_type() {
        let a = OrderedMapIterator::new(1, 5, IteratorType::Normal);
        let b = OrderedMapIterator::new(1, 5, IteratorType::Reverse);
        assert!(!a.equals(&b));
    }

    // --- position ---

    #[test]
    fn test_position_accessible() {
        let it = OrderedMapIterator::new(2, 5, IteratorType::Normal);
        assert_eq!(it.position(), Some(2));
    }

    #[test]
    fn test_position_not_accessible() {
        let it = OrderedMapIterator::new(5, 5, IteratorType::Normal);
        assert_eq!(it.position(), None);
    }

    // --- sync_len ---

    #[test]
    fn test_sync_len_shrinks_pos() {
        let mut it = OrderedMapIterator::new(5, 10, IteratorType::Normal);
        it.sync_len(3);
        // pos clamped to len=3, so not accessible
        assert!(!it.is_accessible());
    }

    #[test]
    fn test_sync_len_keeps_pos_if_within() {
        let mut it = OrderedMapIterator::new(2, 10, IteratorType::Normal);
        it.sync_len(5);
        assert_eq!(it.position(), Some(2));
    }

    // --- set_position ---

    #[test]
    fn test_set_position() {
        let mut it = OrderedMapIterator::new(0, 5, IteratorType::Normal);
        it.set_position(3);
        assert_eq!(it.index(), 3);
    }
}
