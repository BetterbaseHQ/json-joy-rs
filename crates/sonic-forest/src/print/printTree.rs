use super::types::PrintChild;

/// Mirrors `tree-dump/lib/printTree`.
pub fn print_tree(tab: Option<&str>, children: &[Option<&PrintChild>]) -> String {
    let tab = tab.unwrap_or("");
    let mut out = String::new();

    let mut last: isize = children.len() as isize - 1;
    while last >= 0 && children[last as usize].is_none() {
        last -= 1;
    }

    if last < 0 {
        return out;
    }

    for i in 0..=last as usize {
        let Some(child_fn) = children[i] else {
            continue;
        };

        let is_last = i == last as usize;
        let child_tab = format!("{tab}{}  ", if is_last { " " } else { "│" });
        let child = child_fn(&child_tab);
        let branch = if child.is_empty() {
            "│"
        } else if is_last {
            "└─"
        } else {
            "├─"
        };

        out.push('\n');
        out.push_str(tab);
        out.push_str(branch);
        if !child.is_empty() {
            out.push(' ');
            out.push_str(&child);
        }
    }

    out
}

#[allow(non_snake_case)]
pub fn printTree(tab: Option<&str>, children: &[Option<&PrintChild>]) -> String {
    print_tree(tab, children)
}
