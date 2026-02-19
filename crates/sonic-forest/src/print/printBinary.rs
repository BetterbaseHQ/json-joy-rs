use super::types::PrintChild;

/// Mirrors `tree-dump/lib/printBinary`.
pub fn print_binary(tab: Option<&str>, children: [Option<&PrintChild>; 2]) -> String {
    let tab = tab.unwrap_or("");
    let [left, right] = children;

    let mut out = String::new();
    if let Some(left) = left {
        let left_tab = format!("{tab}  ");
        out.push('\n');
        out.push_str(tab);
        out.push_str("← ");
        out.push_str(&left(&left_tab));
    }
    if let Some(right) = right {
        let right_tab = format!("{tab}  ");
        out.push('\n');
        out.push_str(tab);
        out.push_str("→ ");
        out.push_str(&right(&right_tab));
    }

    out
}

#[allow(non_snake_case)]
pub fn printBinary(tab: Option<&str>, children: [Option<&PrintChild>; 2]) -> String {
    print_binary(tab, children)
}
