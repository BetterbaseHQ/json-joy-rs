#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IteratorType {
    Normal,
    Reverse,
}

impl IteratorType {
    pub const NORMAL: Self = Self::Normal;
    pub const REVERSE: Self = Self::Reverse;
}
