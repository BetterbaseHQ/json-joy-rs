use crate::types::Node;

/// Leftmost node in the tree.
/// Mirrors upstream `util/first.ts`.
pub fn first<N: Node>(arena: &[N], root: Option<u32>) -> Option<u32> {
    let mut curr = root;
    while let Some(idx) = curr {
        match arena[idx as usize].l() {
            Some(l) => curr = Some(l),
            None => return Some(idx),
        }
    }
    curr
}
