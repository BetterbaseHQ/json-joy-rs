use crate::types::Node;

use super::first::first;

/// In-order successor.
/// Mirrors upstream `util/next.ts`.
pub fn next<N: Node>(arena: &[N], mut curr: u32) -> Option<u32> {
    if let Some(r) = arena[curr as usize].r() {
        return first(arena, Some(r));
    }

    let mut p = arena[curr as usize].p();
    while let Some(pi) = p {
        if arena[pi as usize].r() == Some(curr) {
            curr = pi;
            p = arena[pi as usize].p();
        } else {
            return Some(pi);
        }
    }
    None
}
