/// Mirrors upstream `tree-dump/lib/types` export shape.
pub trait Printable {
    fn to_string_with_tab(&self, tab: Option<&str>) -> String;
}

/// Child printer callback used by `print_tree` and `print_binary`.
pub type PrintChild = dyn Fn(&str) -> String;
