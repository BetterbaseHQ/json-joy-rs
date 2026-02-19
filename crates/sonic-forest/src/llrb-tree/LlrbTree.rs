use crate::red_black::types::RbNodeLike;
use crate::types::{KvNode, Node};
use crate::util::{first, last, next};

use super::util::{assert_llrb_tree, delete_node};

/// Left-Leaning Red-Black (LLRB) tree node implementation.
#[derive(Clone, Debug)]
pub struct LlrbNode<K, V> {
    pub p: Option<u32>,
    pub l: Option<u32>,
    pub r: Option<u32>,
    pub k: K,
    pub v: V,
    /// `false` = red, `true` = black.
    pub b: bool,
}

impl<K, V> LlrbNode<K, V> {
    fn new(k: K, v: V, b: bool) -> Self {
        Self {
            p: None,
            l: None,
            r: None,
            k,
            v,
            b,
        }
    }
}

impl<K, V> Node for LlrbNode<K, V> {
    fn p(&self) -> Option<u32> {
        self.p
    }

    fn l(&self) -> Option<u32> {
        self.l
    }

    fn r(&self) -> Option<u32> {
        self.r
    }

    fn set_p(&mut self, v: Option<u32>) {
        self.p = v;
    }

    fn set_l(&mut self, v: Option<u32>) {
        self.l = v;
    }

    fn set_r(&mut self, v: Option<u32>) {
        self.r = v;
    }
}

impl<K, V> KvNode<K, V> for LlrbNode<K, V> {
    fn key(&self) -> &K {
        &self.k
    }

    fn value(&self) -> &V {
        &self.v
    }

    fn value_mut(&mut self) -> &mut V {
        &mut self.v
    }

    fn set_key(&mut self, key: K) {
        self.k = key;
    }

    fn set_value(&mut self, value: V) {
        self.v = value;
    }
}

impl<K, V> RbNodeLike<K, V> for LlrbNode<K, V> {
    fn is_black(&self) -> bool {
        self.b
    }

    fn set_black(&mut self, black: bool) {
        self.b = black;
    }
}

fn default_comparator<K: PartialOrd>(a: &K, b: &K) -> i32 {
    if a == b {
        0
    } else if a < b {
        -1
    } else {
        1
    }
}

/// Left-Leaning Red-Black (LLRB) tree implementation.
pub struct LlrbTree<K, V, C = fn(&K, &K) -> i32>
where
    C: Fn(&K, &K) -> i32,
{
    pub min: Option<u32>,
    pub root: Option<u32>,
    pub max: Option<u32>,
    _size: usize,
    pub comparator: C,
    arena: Vec<LlrbNode<K, V>>,
}

impl<K, V> LlrbTree<K, V, fn(&K, &K) -> i32>
where
    K: PartialOrd,
{
    pub fn new() -> Self {
        Self::with_comparator(default_comparator::<K>)
    }
}

impl<K, V> Default for LlrbTree<K, V, fn(&K, &K) -> i32>
where
    K: PartialOrd,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V, C> LlrbTree<K, V, C>
where
    C: Fn(&K, &K) -> i32,
{
    pub fn with_comparator(comparator: C) -> Self {
        Self {
            min: None,
            root: None,
            max: None,
            _size: 0,
            comparator,
            arena: Vec::new(),
        }
    }

    fn push_node(&mut self, key: K, value: V, black: bool) -> u32 {
        self.arena.push(LlrbNode::new(key, value, black));
        (self.arena.len() - 1) as u32
    }

    pub fn set(&mut self, key: K, value: V) -> u32 {
        let Some(root) = self.root else {
            let node = self.push_node(key, value, true);
            self.min = Some(node);
            self.max = Some(node);
            self.root = Some(node);
            self._size = 1;
            return node;
        };

        let mut p = root;

        if let Some(min) = self.min {
            let cmp = (self.comparator)(&key, &self.arena[min as usize].k);
            if cmp < 0 {
                let node = self.push_node(key, value, false);
                self.min = Some(node);
                self.arena[node as usize].p = Some(min);
                self.arena[min as usize].l = Some(node);
                self._size += 1;
                if !self.arena[min as usize].b {
                    self._fix_rrb(node, min);
                }
                return node;
            }
        }

        if let Some(max) = self.max {
            let cmp = (self.comparator)(&key, &self.arena[max as usize].k);
            if cmp > 0 {
                let node = self.push_node(key, value, false);
                self.max = Some(node);
                self.arena[node as usize].p = Some(max);
                self.arena[max as usize].r = Some(node);
                self._size += 1;
                self._fix(node);
                return node;
            }
        }

        loop {
            let cmp = (self.comparator)(&key, &self.arena[p as usize].k);
            if cmp < 0 {
                match self.arena[p as usize].l {
                    Some(l) => p = l,
                    None => {
                        let n = self.push_node(key, value, false);
                        self.arena[n as usize].p = Some(p);
                        self.arena[p as usize].l = Some(n);
                        self._size += 1;
                        if !self.arena[p as usize].b {
                            self._fix_rrb(n, p);
                        }
                        return n;
                    }
                }
            } else if cmp > 0 {
                match self.arena[p as usize].r {
                    Some(r) => p = r,
                    None => {
                        let n = self.push_node(key, value, false);
                        self.arena[n as usize].p = Some(p);
                        self.arena[p as usize].r = Some(n);
                        self._size += 1;
                        self._fix(n);
                        return n;
                    }
                }
            } else {
                self.arena[p as usize].v = value;
                return p;
            }
        }
    }

    fn _fix_rrb(&mut self, n: u32, p: u32) {
        let g = self.arena[p as usize].p.expect("parent of parent exists");
        let s = self.arena[p as usize].r;
        let gp = self.arena[g as usize].p;

        self.arena[p as usize].p = gp;
        self.arena[p as usize].r = Some(g);
        self.arena[g as usize].p = Some(p);
        self.arena[g as usize].l = s;
        if let Some(s) = s {
            self.arena[s as usize].p = Some(g);
        }

        self.arena[n as usize].b = true;
        if let Some(gp) = gp {
            if self.arena[gp as usize].l == Some(g) {
                self.arena[gp as usize].l = Some(p);
            } else {
                self.arena[gp as usize].r = Some(p);
            }
        } else {
            self.root = Some(p);
        }

        self._fix(p);
    }

    fn _fix_brr(&mut self, n: u32, p: u32) {
        let g = self.arena[p as usize].p.expect("parent of parent exists");
        let nl = self.arena[n as usize].l;

        self.arena[g as usize].l = Some(n);
        self.arena[n as usize].p = Some(g);
        self.arena[n as usize].l = Some(p);
        self.arena[p as usize].p = Some(n);
        self.arena[p as usize].r = nl;
        if let Some(nl) = nl {
            self.arena[nl as usize].p = Some(p);
        }

        self._fix_rrb(p, n);
    }

    fn _fix_bbr(&mut self, n: u32, p: u32) {
        let g = self.arena[p as usize].p;
        let nl = self.arena[n as usize].l;

        self.arena[n as usize].p = g;
        self.arena[p as usize].p = Some(n);
        self.arena[n as usize].l = Some(p);
        self.arena[p as usize].r = nl;
        if let Some(g) = g {
            if self.arena[g as usize].l == Some(p) {
                self.arena[g as usize].l = Some(n);
            } else {
                self.arena[g as usize].r = Some(n);
            }
        } else {
            self.root = Some(n);
        }

        if let Some(nl) = nl {
            self.arena[nl as usize].p = Some(p);
        }

        self.arena[p as usize].b = false;
        self.arena[n as usize].b = true;
    }

    fn _fix(&mut self, n: u32) {
        let Some(p) = self.arena[n as usize].p else {
            self.arena[n as usize].b = true;
            return;
        };

        if self.arena[p as usize].l == Some(n) {
            if self.arena[p as usize].b {
                return;
            }
            self._fix_rrb(n, p);
            return;
        }

        let s = self.arena[p as usize].l;
        let sibling_is_black = s.map(|i| self.arena[i as usize].b).unwrap_or(true);
        if sibling_is_black {
            if self.arena[p as usize].b {
                self._fix_bbr(n, p);
            } else {
                self._fix_brr(n, p);
            }
        } else {
            self.arena[p as usize].b = false;
            if let Some(s) = s {
                self.arena[s as usize].b = true;
            }
            self.arena[n as usize].b = true;
            self._fix(p);
        }
    }

    fn update_min_max(&mut self) {
        let Some(root) = self.root else {
            self.min = None;
            self.max = None;
            return;
        };

        let mut curr = root;
        while let Some(l) = self.arena[curr as usize].l {
            curr = l;
        }
        self.min = Some(curr);

        curr = root;
        while let Some(r) = self.arena[curr as usize].r {
            curr = r;
        }
        self.max = Some(curr);
    }

    pub fn find(&self, key: &K) -> Option<u32> {
        let mut curr = self.root;
        while let Some(i) = curr {
            let cmp = (self.comparator)(key, &self.arena[i as usize].k);
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

    pub fn get(&self, key: &K) -> Option<&V> {
        self.find(key).map(|i| &self.arena[i as usize].v)
    }

    pub fn del(&mut self, key: &K) -> bool {
        if self.find(key).is_none() {
            return false;
        }

        self.root = delete_node(&mut self.arena, self.root, key, &self.comparator);
        if let Some(root) = self.root {
            self.arena[root as usize].b = true;
            self.arena[root as usize].p = None;
        }
        self._size -= 1;
        self.update_min_max();

        true
    }

    pub fn clear(&mut self) {
        // Intentionally mirrors upstream `LlrbTree.clear()` behavior, which only
        // resets `root` and leaves `min`, `max`, and `_size` untouched.
        self.root = None;
    }

    pub fn has(&self, key: &K) -> bool {
        self.find(key).is_some()
    }

    pub fn size(&self) -> usize {
        self._size
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn get_or_next_lower(&self, _key: &K) -> Option<u32> {
        panic!("Method not implemented.");
    }

    pub fn for_each<F: FnMut(&LlrbNode<K, V>)>(&self, _f: F) {
        panic!("Method not implemented.");
    }

    pub fn first(&self) -> Option<u32> {
        first(&self.arena, self.root)
    }

    pub fn last(&self) -> Option<u32> {
        last(&self.arena, self.root)
    }

    pub fn next(&self, curr: u32) -> Option<u32> {
        next(&self.arena, curr)
    }

    pub fn iterator0(&self) -> impl FnMut() -> Option<u32> + '_ {
        panic!("Method not implemented.");
        #[allow(unreachable_code)]
        move || None
    }

    pub fn iterator(&self) -> std::iter::Empty<u32> {
        panic!("Method not implemented.");
    }

    pub fn entries(&self) -> LlrbEntries<'_, K, V, C> {
        LlrbEntries {
            _tree: self,
            exhausted: false,
        }
    }

    pub fn root_index(&self) -> Option<u32> {
        self.root
    }

    pub fn min_index(&self) -> Option<u32> {
        self.min
    }

    pub fn max_index(&self) -> Option<u32> {
        self.max
    }

    pub fn key(&self, idx: u32) -> &K {
        &self.arena[idx as usize].k
    }

    pub fn value(&self, idx: u32) -> &V {
        &self.arena[idx as usize].v
    }

    pub fn value_mut_by_index(&mut self, idx: u32) -> &mut V {
        &mut self.arena[idx as usize].v
    }

    pub fn assert_valid(&self) -> Result<(), String> {
        assert_llrb_tree(&self.arena, self.root, &self.comparator)
    }

    pub fn arena(&self) -> &[LlrbNode<K, V>] {
        &self.arena
    }
}

pub struct LlrbEntries<'a, K, V, C>
where
    C: Fn(&K, &K) -> i32,
{
    _tree: &'a LlrbTree<K, V, C>,
    exhausted: bool,
}

impl<'a, K, V, C> Iterator for LlrbEntries<'a, K, V, C>
where
    C: Fn(&K, &K) -> i32,
{
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }
        self.exhausted = true;
        panic!("Method not implemented.");
    }
}
