//! Upstream: sonic-forest/src/splay/
//!
//! Splay tree rotation and splay-to-root operation tests.

use sonic_forest::splay::{l_splay, ll_splay, lr_splay, r_splay, rl_splay, rr_splay, splay};
use sonic_forest::types::Node;
use sonic_forest::util::{first, insert, insert_left, insert_right, next, size};

#[derive(Clone, Debug)]
struct SplayNode {
    p: Option<u32>,
    l: Option<u32>,
    r: Option<u32>,
    k: i32,
}

impl SplayNode {
    fn new(k: i32) -> Self {
        Self {
            p: None,
            l: None,
            r: None,
            k,
        }
    }
}

impl Node for SplayNode {
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

fn cmp(a: &i32, b: &i32) -> i32 {
    a.cmp(b) as i32
}
fn key_of(n: &SplayNode) -> &i32 {
    &n.k
}

fn inorder(arena: &[SplayNode], root: Option<u32>) -> Vec<i32> {
    let mut out = Vec::new();
    let mut cur = first(arena, root);
    while let Some(i) = cur {
        out.push(arena[i as usize].k);
        cur = next(arena, i);
    }
    out
}

fn assert_parent_links(arena: &[SplayNode], root: Option<u32>) {
    if let Some(r) = root {
        assert!(arena[r as usize].p.is_none(), "root must have no parent");
        check_subtree(arena, r);
    }
}

fn check_subtree(arena: &[SplayNode], idx: u32) {
    let node = &arena[idx as usize];
    if let Some(l) = node.l {
        assert_eq!(arena[l as usize].p, Some(idx));
        check_subtree(arena, l);
    }
    if let Some(r) = node.r {
        assert_eq!(arena[r as usize].p, Some(idx));
        check_subtree(arena, r);
    }
}

// ---------------------------------------------------------------------------
// Single rotations
// ---------------------------------------------------------------------------

#[test]
fn r_splay_promotes_left_child() {
    //   1
    //  /
    // 0
    let mut arena = vec![SplayNode::new(0), SplayNode::new(1)];
    arena[1].l = Some(0);
    arena[0].p = Some(1);

    r_splay(&mut arena, 0, 1);

    // After: 0 is root, 1 is right child
    assert!(arena[0].p.is_none());
    assert_eq!(arena[0].r, Some(1));
    assert_eq!(arena[1].p, Some(0));
    assert_parent_links(&arena, Some(0));
}

#[test]
fn l_splay_promotes_right_child() {
    // 0
    //  \
    //   1
    let mut arena = vec![SplayNode::new(0), SplayNode::new(1)];
    arena[0].r = Some(1);
    arena[1].p = Some(0);

    l_splay(&mut arena, 1, 0);

    assert!(arena[1].p.is_none());
    assert_eq!(arena[1].l, Some(0));
    assert_eq!(arena[0].p, Some(1));
    assert_parent_links(&arena, Some(1));
}

// ---------------------------------------------------------------------------
// Double rotations
// ---------------------------------------------------------------------------

#[test]
fn ll_splay_zig_zig_left() {
    //     2
    //    /
    //   1
    //  /
    // 0
    let mut arena = vec![SplayNode::new(0), SplayNode::new(1), SplayNode::new(2)];
    arena[2].l = Some(1);
    arena[1].p = Some(2);
    arena[1].l = Some(0);
    arena[0].p = Some(1);

    let root = ll_splay(&mut arena, Some(2), 0, 1, 2);
    assert_eq!(root, Some(0));
    assert_parent_links(&arena, root);
    assert_eq!(inorder(&arena, root), vec![0, 1, 2]);
}

#[test]
fn rr_splay_zig_zig_right() {
    // 0
    //  \
    //   1
    //    \
    //     2
    let mut arena = vec![SplayNode::new(0), SplayNode::new(1), SplayNode::new(2)];
    arena[0].r = Some(1);
    arena[1].p = Some(0);
    arena[1].r = Some(2);
    arena[2].p = Some(1);

    let root = rr_splay(&mut arena, Some(0), 2, 1, 0);
    assert_eq!(root, Some(2));
    assert_parent_links(&arena, root);
    assert_eq!(inorder(&arena, root), vec![0, 1, 2]);
}

#[test]
fn lr_splay_zig_zag() {
    //   2
    //  /
    // 0
    //  \
    //   1
    let mut arena = vec![SplayNode::new(0), SplayNode::new(1), SplayNode::new(2)];
    arena[2].l = Some(0);
    arena[0].p = Some(2);
    arena[0].r = Some(1);
    arena[1].p = Some(0);

    let root = lr_splay(&mut arena, Some(2), 1, 0, 2);
    assert_eq!(root, Some(1));
    assert_parent_links(&arena, root);
    assert_eq!(inorder(&arena, root), vec![0, 1, 2]);
}

#[test]
fn rl_splay_zig_zag() {
    // 0
    //  \
    //   2
    //  /
    // 1
    let mut arena = vec![SplayNode::new(0), SplayNode::new(1), SplayNode::new(2)];
    arena[0].r = Some(2);
    arena[2].p = Some(0);
    arena[2].l = Some(1);
    arena[1].p = Some(2);

    let root = rl_splay(&mut arena, Some(0), 1, 2, 0);
    assert_eq!(root, Some(1));
    assert_parent_links(&arena, root);
    assert_eq!(inorder(&arena, root), vec![0, 1, 2]);
}

// ---------------------------------------------------------------------------
// Full splay-to-root
// ---------------------------------------------------------------------------

#[test]
fn splay_leaf_to_root() {
    let mut arena = Vec::new();
    let mut root = None;
    for k in [10, 5, 20, 3, 7, 15, 25] {
        arena.push(SplayNode::new(k));
        let idx = (arena.len() - 1) as u32;
        root = insert(&mut arena, root, idx, key_of, cmp);
    }

    // Splay the node with key=3 (index 3) to root
    root = splay(&mut arena, root, 3, 5);
    assert_parent_links(&arena, root);
    assert_eq!(size(&arena, root), 7);
    assert_eq!(inorder(&arena, root), vec![3, 5, 7, 10, 15, 20, 25]);
}

#[test]
fn splay_root_is_noop() {
    let mut arena = vec![SplayNode::new(10), SplayNode::new(5), SplayNode::new(20)];
    let root = Some(0);
    insert_left(&mut arena, 1, 0);
    insert_right(&mut arena, 2, 0);

    let new_root = splay(&mut arena, root, 0, 5);
    assert_eq!(new_root, root);
    assert_parent_links(&arena, new_root);
}

#[test]
fn splay_preserves_size() {
    let mut arena = Vec::new();
    let mut root = None;
    let keys = [50, 25, 75, 12, 37, 62, 87, 6, 18, 31, 43];
    for k in keys {
        arena.push(SplayNode::new(k));
        let idx = (arena.len() - 1) as u32;
        root = insert(&mut arena, root, idx, key_of, cmp);
    }
    let original_size = size(&arena, root);

    // Splay various nodes
    for idx in 0..arena.len() as u32 {
        root = splay(&mut arena, root, idx, 5);
        assert_parent_links(&arena, root);
        assert_eq!(size(&arena, root), original_size);
    }
}

#[test]
fn splay_preserves_inorder() {
    let mut arena = Vec::new();
    let mut root = None;
    for k in [4, 2, 6, 1, 3, 5, 7] {
        arena.push(SplayNode::new(k));
        let idx = (arena.len() - 1) as u32;
        root = insert(&mut arena, root, idx, key_of, cmp);
    }
    let expected = inorder(&arena, root);

    // Splay each node and verify inorder is unchanged
    for idx in 0..arena.len() as u32 {
        root = splay(&mut arena, root, idx, 5);
        assert_eq!(inorder(&arena, root), expected);
        assert_parent_links(&arena, root);
    }
}
