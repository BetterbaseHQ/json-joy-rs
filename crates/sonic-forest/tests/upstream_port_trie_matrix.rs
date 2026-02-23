//! Upstream: sonic-forest/src/trie/
//!
//! TrieNode construction and basic operations tests.

use sonic_forest::trie::TrieNode;
use sonic_forest::types::Node;

// ---------------------------------------------------------------------------
// TrieNode construction
// ---------------------------------------------------------------------------

#[test]
fn trie_node_new_defaults() {
    let node = TrieNode::<i32>::new("key".to_string(), Some(42));
    assert_eq!(node.k, "key");
    assert_eq!(node.v, Some(42));
    assert_eq!(node.p, None);
    assert_eq!(node.l, None);
    assert_eq!(node.r, None);
    assert_eq!(node.children, None);
}

#[test]
fn trie_node_with_no_value() {
    let node = TrieNode::<String>::new("prefix".to_string(), None);
    assert_eq!(node.k, "prefix");
    assert_eq!(node.v, None);
}

// ---------------------------------------------------------------------------
// Node trait implementation
// ---------------------------------------------------------------------------

#[test]
fn trie_node_implements_node_trait() {
    let mut node = TrieNode::<i32>::new("test".to_string(), Some(1));

    assert_eq!(node.p(), None);
    assert_eq!(node.l(), None);
    assert_eq!(node.r(), None);

    node.set_p(Some(10));
    node.set_l(Some(20));
    node.set_r(Some(30));

    assert_eq!(node.p(), Some(10));
    assert_eq!(node.l(), Some(20));
    assert_eq!(node.r(), Some(30));
}

#[test]
fn trie_node_clear_links() {
    let mut node = TrieNode::<i32>::new("test".to_string(), Some(1));
    node.set_p(Some(5));
    node.set_l(Some(6));
    node.set_r(Some(7));

    node.set_p(None);
    node.set_l(None);
    node.set_r(None);

    assert_eq!(node.p(), None);
    assert_eq!(node.l(), None);
    assert_eq!(node.r(), None);
}

// ---------------------------------------------------------------------------
// Arena-based trie node operations
// ---------------------------------------------------------------------------

#[test]
fn trie_nodes_in_arena() {
    let mut arena: Vec<TrieNode<i32>> = vec![
        TrieNode::new("root".to_string(), Some(0)),
        TrieNode::new("a".to_string(), Some(1)),
        TrieNode::new("b".to_string(), Some(2)),
    ];

    // Wire child0 as left child of root
    arena[0].children = Some(1);
    arena[1].p = Some(0);

    // Wire child1 as right sibling of child0
    arena[1].r = Some(2);
    arena[2].p = Some(0);

    assert_eq!(arena[0].children, Some(1));
    assert_eq!(arena[1].r, Some(2));
    assert_eq!(arena[1].p(), Some(0));
    assert_eq!(arena[2].p(), Some(0));
}

#[test]
fn trie_node_clone() {
    let node = TrieNode::<String>::new("key".to_string(), Some("val".to_string()));
    let cloned = node.clone();
    assert_eq!(cloned.k, "key");
    assert_eq!(cloned.v, Some("val".to_string()));
    assert_eq!(cloned.p, None);
    assert_eq!(cloned.l, None);
    assert_eq!(cloned.r, None);
    assert_eq!(cloned.children, None);
}

#[test]
fn trie_node_children_field() {
    let mut node = TrieNode::<i32>::new("parent".to_string(), None);
    assert_eq!(node.children, None);
    node.children = Some(5);
    assert_eq!(node.children, Some(5));
}

// ---------------------------------------------------------------------------
// Type flexibility
// ---------------------------------------------------------------------------

#[test]
fn trie_node_with_float_value() {
    let node = TrieNode::new("pi".to_string(), Some(2.72f64));
    assert_eq!(node.v, Some(2.72));
}

#[test]
fn trie_node_with_vec_value() {
    let node = TrieNode::new("list".to_string(), Some(vec![1, 2, 3]));
    assert_eq!(node.v, Some(vec![1, 2, 3]));
}
