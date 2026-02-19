use sonic_forest::print::{print_binary, print_tree, PrintChild};

#[test]
fn print_tree_matrix() {
    let a = |tab: &str| format!("A({tab})");
    let c = |tab: &str| format!("C({tab})");
    let children: [Option<&PrintChild>; 4] = [Some(&a), None, Some(&c), None];

    let out = print_tree(Some(""), &children);
    assert_eq!(out, "\n├─ A(│  )\n└─ C(   )");
}

#[test]
fn print_tree_empty_child_matrix() {
    let empty = |_tab: &str| "".to_string();
    let leaf = |_tab: &str| "leaf".to_string();
    let children: [Option<&PrintChild>; 2] = [Some(&empty), Some(&leaf)];

    let out = print_tree(None, &children);
    assert_eq!(out, "\n│\n└─ leaf");
}

#[test]
fn print_binary_matrix() {
    let left = |tab: &str| format!("L({tab})");
    let right = |tab: &str| format!("R({tab})");

    let out = print_binary(Some("--"), [Some(&left), Some(&right)]);
    assert_eq!(out, "\n--← L(--  )\n--→ R(--  )");
}
