use crate::types::Node;

/// Lightweight debug printer for arena-backed trees.
///
/// Divergence from upstream `util/print.ts`: upstream prints JS object
/// constructors and key/value payloads; this Rust variant prints node indices.
pub fn print<N: Node>(arena: &[N], node: Option<u32>, tab: &str) -> String {
    match node {
        None => "∅".to_string(),
        Some(i) => {
            let l = arena[i as usize].l();
            let r = arena[i as usize].r();
            let left = print(arena, l, &format!("{}  ", tab));
            let right = print(arena, r, &format!("{}  ", tab));
            format!("Node({i})\n{tab}L={left}\n{tab}R={right}")
        }
    }
}
