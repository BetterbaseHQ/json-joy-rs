//! Upstream-parity parser helper (`Parser.ts`).
//!
//! Rust divergence note: upstream also has `JsonPathParser.ts`; on common
//! case-insensitive filesystems we cannot keep both `Parser.rs` and `parser.rs`
//! as distinct paths, so the full parser implementation lives in
//! `parser_impl.rs`.

/// Basic parser utility for string/position management.
#[derive(Debug, Clone, Default)]
pub struct Parser {
    pub str_: String,
    pub pos: usize,
    pub length: usize,
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self, input: &str) {
        self.str_.clear();
        self.str_.push_str(input);
        self.pos = 0;
        self.length = input.len();
    }

    pub fn eof(&self) -> bool {
        self.pos >= self.length
    }

    pub fn peek(&self, len: usize) -> String {
        if self.pos >= self.length {
            return String::new();
        }
        if len <= 1 {
            return self.str_[self.pos..]
                .chars()
                .next()
                .map(|c| c.to_string())
                .unwrap_or_default();
        }
        self.str_[self.pos..].chars().take(len).collect()
    }

    pub fn is(&self, expected: &str) -> bool {
        self.str_[self.pos..].starts_with(expected)
    }

    pub fn r#match(&self, predicate: fn(char) -> bool) -> usize {
        if self.pos >= self.length {
            return 0;
        }
        let mut len = 0usize;
        for c in self.str_[self.pos..].chars() {
            if predicate(c) {
                len += c.len_utf8();
            } else {
                break;
            }
        }
        len
    }

    pub fn skip(&mut self, count: usize) {
        self.pos = self.pos.saturating_add(count);
    }

    pub fn ws(&mut self) {
        while let Some(c) = self.str_[self.pos..].chars().next() {
            if matches!(c, ' ' | '\t' | '\n' | '\r') {
                self.pos += c.len_utf8();
            } else {
                break;
            }
            if self.pos >= self.length {
                break;
            }
        }
    }
}
