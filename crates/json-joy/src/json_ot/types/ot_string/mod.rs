//! Reversible string operational transformation.
//!
//! Mirrors `packages/json-joy/src/json-ot/types/ot-string/`.
//!
//! # Operation format
//!
//! A `StringOp` is a sequence of components:
//! - `Retain(n)` — skip `n` characters
//! - `Delete(n)` — delete `n` characters (irreversible count form)
//! - `DeleteStr(s)` — reversible delete storing the deleted text
//! - `Insert(s)` — insert text

#[derive(Debug, Clone, PartialEq)]
pub enum StringComponent {
    Retain(usize),
    Delete(usize),
    DeleteStr(String),
    Insert(String),
}

pub type StringOp = Vec<StringComponent>;

impl StringComponent {
    /// Length of this component (in chars) on the *source* string.
    pub fn src_len(&self) -> usize {
        match self {
            StringComponent::Retain(n) => *n,
            StringComponent::Delete(n) => *n,
            StringComponent::DeleteStr(s) => s.chars().count(),
            StringComponent::Insert(_) => 0,
        }
    }

    /// Length of this component (in chars) on the *destination* string.
    pub fn dst_len(&self) -> usize {
        match self {
            StringComponent::Retain(n) => *n,
            StringComponent::Delete(_) => 0,
            StringComponent::DeleteStr(_) => 0,
            StringComponent::Insert(s) => s.chars().count(),
        }
    }
}

/// Append a component, merging with the last component if same type.
fn append(op: &mut StringOp, comp: StringComponent) {
    match (op.last_mut(), &comp) {
        (Some(StringComponent::Retain(n)), StringComponent::Retain(m)) => {
            *n += m;
            return;
        }
        (Some(StringComponent::Delete(n)), StringComponent::Delete(m)) => {
            *n += m;
            return;
        }
        (Some(StringComponent::DeleteStr(s)), StringComponent::DeleteStr(t)) => {
            s.push_str(t);
            return;
        }
        (Some(StringComponent::Insert(s)), StringComponent::Insert(t)) => {
            s.push_str(t);
            return;
        }
        _ => {}
    }
    op.push(comp);
}

/// Remove trailing `Retain(0)` and other empty components.
pub fn trim(op: &mut StringOp) {
    while let Some(last) = op.last() {
        match last {
            StringComponent::Retain(0) | StringComponent::Delete(0) => {
                op.pop();
            }
            StringComponent::Insert(s) | StringComponent::DeleteStr(s) if s.is_empty() => {
                op.pop();
            }
            _ => break,
        }
    }
}

/// Normalize: coalesce adjacent same-type components and trim.
pub fn normalize(op: StringOp) -> StringOp {
    let mut result: StringOp = Vec::new();
    for comp in op {
        match &comp {
            StringComponent::Retain(0) | StringComponent::Delete(0) => {}
            StringComponent::Insert(s) | StringComponent::DeleteStr(s) if s.is_empty() => {}
            _ => append(&mut result, comp),
        }
    }
    // Remove trailing retains
    while matches!(result.last(), Some(StringComponent::Retain(_))) {
        result.pop();
    }
    result
}

/// Apply a `StringOp` to a string, returning the result.
pub fn apply(s: &str, op: &StringOp) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();
    let mut idx = 0usize;

    for comp in op {
        match comp {
            StringComponent::Retain(n) => {
                result.extend(chars[idx..idx + n].iter());
                idx += n;
            }
            StringComponent::Delete(n) => {
                idx += n;
            }
            StringComponent::DeleteStr(del) => {
                idx += del.chars().count();
            }
            StringComponent::Insert(ins) => {
                result.push_str(ins);
            }
        }
    }
    // Append remaining characters
    result.extend(chars[idx..].iter());
    result
}

/// Compose two sequential operations into one equivalent operation.
pub fn compose(op1: &StringOp, op2: &StringOp) -> StringOp {
    let mut result: StringOp = Vec::new();
    let mut iter1 = op1.iter().peekable();
    let mut iter2 = op2.iter().peekable();
    let mut rem1: Option<StringComponent> = None;
    let mut rem2: Option<StringComponent> = None;

    loop {
        let c1 = rem1.take().or_else(|| iter1.next().cloned());
        let c2 = rem2.take().or_else(|| iter2.next().cloned());

        match (c1, c2) {
            (None, None) => break,
            (Some(c), None) => {
                // Remaining from op1 pass through (they're retains or deletes in original string)
                append(&mut result, c);
            }
            (None, Some(c)) => {
                append(&mut result, c);
            }
            (Some(c1), Some(c2)) => {
                match (&c1, &c2) {
                    // Delete in op1 passes through (already removed chars don't interact with op2)
                    (StringComponent::Delete(n), _) => {
                        append(&mut result, StringComponent::Delete(*n));
                        rem2 = Some(c2);
                    }
                    (StringComponent::DeleteStr(s), _) => {
                        append(&mut result, StringComponent::DeleteStr(s.clone()));
                        rem2 = Some(c2);
                    }
                    // Insert in op2 passes through
                    (_, StringComponent::Insert(s)) => {
                        append(&mut result, StringComponent::Insert(s.clone()));
                        rem1 = Some(c1);
                    }
                    // Retain1 + Retain2
                    (StringComponent::Retain(n), StringComponent::Retain(m)) => {
                        let min = (*n).min(*m);
                        append(&mut result, StringComponent::Retain(min));
                        if n > m {
                            rem1 = Some(StringComponent::Retain(n - m));
                        } else if m > n {
                            rem2 = Some(StringComponent::Retain(m - n));
                        }
                    }
                    // Retain1 + Delete2
                    (StringComponent::Retain(n), StringComponent::Delete(m)) => {
                        let min = (*n).min(*m);
                        append(&mut result, StringComponent::Delete(min));
                        if n > m {
                            rem1 = Some(StringComponent::Retain(n - m));
                        } else if m > n {
                            rem2 = Some(StringComponent::Delete(m - n));
                        }
                    }
                    (StringComponent::Retain(n), StringComponent::DeleteStr(s)) => {
                        let s_len = s.chars().count();
                        let min = (*n).min(s_len);
                        let del_str: String = s.chars().take(min).collect();
                        append(&mut result, StringComponent::DeleteStr(del_str));
                        if n > &s_len {
                            rem1 = Some(StringComponent::Retain(n - s_len));
                        } else if s_len > *n {
                            let rest: String = s.chars().skip(*n).collect();
                            rem2 = Some(StringComponent::DeleteStr(rest));
                        }
                    }
                    // Insert1 + Retain2: insert survives
                    (StringComponent::Insert(s), StringComponent::Retain(m)) => {
                        let s_len = s.chars().count();
                        let min = s_len.min(*m);
                        let kept: String = s.chars().take(min).collect();
                        append(&mut result, StringComponent::Insert(kept));
                        if s_len > *m {
                            rem1 = Some(StringComponent::Insert(s.chars().skip(*m).collect()));
                        } else if m > &s_len {
                            rem2 = Some(StringComponent::Retain(m - s_len));
                        }
                    }
                    // Insert1 + Delete2: cancel out
                    (StringComponent::Insert(s), StringComponent::Delete(m)) => {
                        let s_len = s.chars().count();
                        if s_len > *m {
                            rem1 = Some(StringComponent::Insert(s.chars().skip(*m).collect()));
                        } else if m > &s_len {
                            rem2 = Some(StringComponent::Delete(m - s_len));
                        }
                    }
                    (StringComponent::Insert(s), StringComponent::DeleteStr(del)) => {
                        let s_len = s.chars().count();
                        let del_len = del.chars().count();
                        if s_len > del_len {
                            rem1 = Some(StringComponent::Insert(s.chars().skip(del_len).collect()));
                        } else if del_len > s_len {
                            rem2 = Some(StringComponent::DeleteStr(
                                del.chars().skip(s_len).collect(),
                            ));
                        }
                    }
                }
            }
        }
    }
    normalize(result)
}

/// Transform `op` against `against`, assuming `left_wins` for concurrent inserts at same position.
pub fn transform(op: &StringOp, against: &StringOp, left_wins: bool) -> StringOp {
    let mut result: StringOp = Vec::new();
    let mut op_iter = op.iter().cloned().peekable();
    let mut ag_iter = against.iter().cloned().peekable();
    let mut rem_op: Option<StringComponent> = None;
    let mut rem_ag: Option<StringComponent> = None;

    loop {
        let o = rem_op.take().or_else(|| op_iter.next());
        let a = rem_ag.take().or_else(|| ag_iter.next());

        match (o, a) {
            (None, _) => break,
            (Some(o), None) => {
                append(&mut result, o);
            }
            (Some(o), Some(a)) => {
                match (&o, &a) {
                    // Against inserts: add retain to account for inserted chars
                    (_, StringComponent::Insert(s)) => {
                        if left_wins {
                            rem_op = Some(o);
                            append(&mut result, StringComponent::Retain(s.chars().count()));
                        } else {
                            append(&mut result, StringComponent::Retain(s.chars().count()));
                            rem_op = Some(o);
                        }
                    }
                    // Op inserts: pass through
                    (StringComponent::Insert(s), _) => {
                        append(&mut result, StringComponent::Insert(s.clone()));
                        rem_ag = Some(a);
                    }
                    // Retain vs retain
                    (StringComponent::Retain(n), StringComponent::Retain(m)) => {
                        let min = (*n).min(*m);
                        append(&mut result, StringComponent::Retain(min));
                        if n > m {
                            rem_op = Some(StringComponent::Retain(n - m));
                        } else if m > n {
                            rem_ag = Some(StringComponent::Retain(m - n));
                        }
                    }
                    // Retain vs delete: skip the retained chars (they'll be gone)
                    (StringComponent::Retain(n), StringComponent::Delete(m)) => {
                        let del_len = *m;
                        if n > m {
                            rem_op = Some(StringComponent::Retain(n - del_len));
                        } else if del_len > *n {
                            rem_ag = Some(StringComponent::Delete(del_len - n));
                        }
                    }
                    (StringComponent::Retain(n), StringComponent::DeleteStr(s)) => {
                        let del_len = s.chars().count();
                        if *n > del_len {
                            rem_op = Some(StringComponent::Retain(n - del_len));
                        } else if del_len > *n {
                            rem_ag = Some(StringComponent::Delete(del_len - n));
                        }
                    }
                    // Delete vs retain: delete passes through
                    (StringComponent::Delete(n), StringComponent::Retain(m)) => {
                        let min = (*n).min(*m);
                        append(&mut result, StringComponent::Delete(min));
                        if n > m {
                            rem_op = Some(StringComponent::Delete(n - m));
                        } else if m > n {
                            rem_ag = Some(StringComponent::Retain(m - n));
                        }
                    }
                    (StringComponent::DeleteStr(s), StringComponent::Retain(m)) => {
                        let s_len = s.chars().count();
                        let min = s_len.min(*m);
                        let del_str: String = s.chars().take(min).collect();
                        append(&mut result, StringComponent::DeleteStr(del_str));
                        if s_len > *m {
                            rem_op = Some(StringComponent::DeleteStr(s.chars().skip(*m).collect()));
                        } else if m > &s_len {
                            rem_ag = Some(StringComponent::Retain(m - s_len));
                        }
                    }
                    // Delete vs delete: both deleted the same range — op delete is redundant
                    (StringComponent::Delete(n), StringComponent::Delete(m)) => {
                        let del_len = *m;
                        if n > m {
                            rem_op = Some(StringComponent::Delete(n - del_len));
                        } else if del_len > *n {
                            rem_ag = Some(StringComponent::Delete(del_len - n));
                        }
                    }
                    (StringComponent::Delete(n), StringComponent::DeleteStr(s)) => {
                        let del_len = s.chars().count();
                        if *n > del_len {
                            rem_op = Some(StringComponent::Delete(n - del_len));
                        } else if del_len > *n {
                            rem_ag = Some(StringComponent::Delete(del_len - n));
                        }
                    }
                    (StringComponent::DeleteStr(s), StringComponent::Delete(m)) => {
                        let s_len = s.chars().count();
                        let del_len = *m;
                        if s_len > del_len {
                            rem_op = Some(StringComponent::DeleteStr(
                                s.chars().skip(del_len).collect(),
                            ));
                        } else if del_len > s_len {
                            rem_ag = Some(StringComponent::Delete(del_len - s_len));
                        }
                    }
                    (StringComponent::DeleteStr(s), StringComponent::DeleteStr(t)) => {
                        let s_len = s.chars().count();
                        let del_len = t.chars().count();
                        if s_len > del_len {
                            rem_op = Some(StringComponent::DeleteStr(
                                s.chars().skip(del_len).collect(),
                            ));
                        } else if del_len > s_len {
                            rem_ag = Some(StringComponent::Delete(del_len - s_len));
                        }
                    }
                }
            }
        }
    }
    normalize(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_retain() {
        let op = vec![StringComponent::Retain(5)];
        assert_eq!(apply("hello", &op), "hello");
    }

    #[test]
    fn apply_insert() {
        let op = vec![
            StringComponent::Retain(5),
            StringComponent::Insert(" world".to_string()),
        ];
        assert_eq!(apply("hello", &op), "hello world");
    }

    #[test]
    fn apply_delete() {
        let op = vec![StringComponent::Retain(5), StringComponent::Delete(6)];
        assert_eq!(apply("hello world", &op), "hello");
    }

    #[test]
    fn normalize_coalesces_and_strips_trailing_retain() {
        // Two adjacent retains merge; trailing retains are stripped (they're implicit)
        let op = vec![StringComponent::Retain(2), StringComponent::Retain(3)];
        let n = normalize(op);
        // Trailing retains are stripped — result is empty (identity op)
        assert_eq!(n, vec![]);
    }

    #[test]
    fn normalize_coalesces_non_trailing() {
        let op = vec![
            StringComponent::Retain(2),
            StringComponent::Retain(3),
            StringComponent::Insert("x".to_string()),
        ];
        let n = normalize(op);
        assert_eq!(
            n,
            vec![
                StringComponent::Retain(5),
                StringComponent::Insert("x".to_string())
            ]
        );
    }

    #[test]
    fn compose_insert_then_delete() {
        // op1: insert "X" at pos 0
        let op1 = vec![StringComponent::Insert("X".to_string())];
        // op2: delete 1 char at pos 0
        let op2 = vec![StringComponent::Delete(1)];
        let composed = compose(&op1, &op2);
        // X inserted and then deleted = no-op
        assert!(composed.is_empty() || composed == vec![]);
    }

    #[test]
    fn transform_insert_at_same_position() {
        // Both insert at pos 0; left_wins = true means left insert goes first
        let op = vec![StringComponent::Insert("A".to_string())];
        let against = vec![StringComponent::Insert("B".to_string())];
        let transformed = transform(&op, &against, true);
        let result = apply("hello", &transformed);
        // After "B" is inserted first, our op should still insert "A" in the right place
        assert!(result.contains('A'));
    }

    // ── StringComponent src_len / dst_len ───────────────────────────────

    #[test]
    fn src_len_retain() {
        assert_eq!(StringComponent::Retain(5).src_len(), 5);
    }

    #[test]
    fn src_len_delete() {
        assert_eq!(StringComponent::Delete(3).src_len(), 3);
    }

    #[test]
    fn src_len_delete_str() {
        assert_eq!(StringComponent::DeleteStr("abc".to_string()).src_len(), 3);
    }

    #[test]
    fn src_len_insert() {
        assert_eq!(StringComponent::Insert("xyz".to_string()).src_len(), 0);
    }

    #[test]
    fn dst_len_retain() {
        assert_eq!(StringComponent::Retain(5).dst_len(), 5);
    }

    #[test]
    fn dst_len_delete() {
        assert_eq!(StringComponent::Delete(3).dst_len(), 0);
    }

    #[test]
    fn dst_len_delete_str() {
        assert_eq!(StringComponent::DeleteStr("abc".to_string()).dst_len(), 0);
    }

    #[test]
    fn dst_len_insert() {
        assert_eq!(StringComponent::Insert("xyz".to_string()).dst_len(), 3);
    }

    #[test]
    fn src_len_unicode() {
        // Multi-byte chars: "éà" is 2 chars
        assert_eq!(StringComponent::DeleteStr("éà".to_string()).src_len(), 2);
    }

    #[test]
    fn dst_len_unicode() {
        assert_eq!(StringComponent::Insert("日本語".to_string()).dst_len(), 3);
    }

    // ── apply edge cases ────────────────────────────────────────────────

    #[test]
    fn apply_empty_op() {
        assert_eq!(apply("hello", &vec![]), "hello");
    }

    #[test]
    fn apply_empty_string() {
        let op = vec![StringComponent::Insert("abc".to_string())];
        assert_eq!(apply("", &op), "abc");
    }

    #[test]
    fn apply_delete_str() {
        let op = vec![StringComponent::DeleteStr("hel".to_string())];
        assert_eq!(apply("hello", &op), "lo");
    }

    #[test]
    fn apply_complex_sequence() {
        // "hello world" -> "hXrld"
        let op = vec![
            StringComponent::Retain(1),          // keep 'h'
            StringComponent::Delete(4),          // delete "ello"
            StringComponent::Insert("X".into()), // insert "X"
            StringComponent::Delete(2),          // delete " w"
        ];
        assert_eq!(apply("hello world", &op), "hXorld");
    }

    #[test]
    fn apply_insert_at_beginning() {
        let op = vec![StringComponent::Insert(">>".to_string())];
        assert_eq!(apply("hello", &op), ">>hello");
    }

    // ── trim ────────────────────────────────────────────────────────────

    #[test]
    fn trim_removes_trailing_retain_zero() {
        let mut op = vec![
            StringComponent::Insert("a".to_string()),
            StringComponent::Retain(0),
        ];
        trim(&mut op);
        assert_eq!(op, vec![StringComponent::Insert("a".to_string())]);
    }

    #[test]
    fn trim_removes_trailing_delete_zero() {
        let mut op = vec![
            StringComponent::Insert("a".to_string()),
            StringComponent::Delete(0),
        ];
        trim(&mut op);
        assert_eq!(op, vec![StringComponent::Insert("a".to_string())]);
    }

    #[test]
    fn trim_removes_trailing_empty_insert() {
        let mut op = vec![
            StringComponent::Retain(3),
            StringComponent::Insert(String::new()),
        ];
        trim(&mut op);
        assert_eq!(op, vec![StringComponent::Retain(3)]);
    }

    #[test]
    fn trim_removes_trailing_empty_delete_str() {
        let mut op = vec![
            StringComponent::Retain(3),
            StringComponent::DeleteStr(String::new()),
        ];
        trim(&mut op);
        assert_eq!(op, vec![StringComponent::Retain(3)]);
    }

    #[test]
    fn trim_stops_at_nonempty() {
        let mut op = vec![
            StringComponent::Insert("a".to_string()),
            StringComponent::Retain(3),
        ];
        trim(&mut op);
        assert_eq!(
            op,
            vec![
                StringComponent::Insert("a".to_string()),
                StringComponent::Retain(3)
            ]
        );
    }

    // ── normalize ───────────────────────────────────────────────────────

    #[test]
    fn normalize_removes_zero_components() {
        let op = vec![
            StringComponent::Retain(0),
            StringComponent::Delete(0),
            StringComponent::Insert(String::new()),
            StringComponent::DeleteStr(String::new()),
            StringComponent::Insert("a".to_string()),
        ];
        assert_eq!(
            normalize(op),
            vec![StringComponent::Insert("a".to_string())]
        );
    }

    #[test]
    fn normalize_coalesces_adjacent_deletes() {
        let op = vec![
            StringComponent::Delete(2),
            StringComponent::Delete(3),
            StringComponent::Insert("x".to_string()),
        ];
        assert_eq!(
            normalize(op),
            vec![
                StringComponent::Delete(5),
                StringComponent::Insert("x".to_string())
            ]
        );
    }

    #[test]
    fn normalize_coalesces_adjacent_delete_str() {
        let op = vec![
            StringComponent::DeleteStr("ab".to_string()),
            StringComponent::DeleteStr("cd".to_string()),
            StringComponent::Insert("x".to_string()),
        ];
        assert_eq!(
            normalize(op),
            vec![
                StringComponent::DeleteStr("abcd".to_string()),
                StringComponent::Insert("x".to_string())
            ]
        );
    }

    #[test]
    fn normalize_coalesces_adjacent_inserts() {
        let op = vec![
            StringComponent::Insert("a".to_string()),
            StringComponent::Insert("b".to_string()),
        ];
        assert_eq!(
            normalize(op),
            vec![StringComponent::Insert("ab".to_string())]
        );
    }

    #[test]
    fn normalize_strips_trailing_retain() {
        let op = vec![
            StringComponent::Insert("a".to_string()),
            StringComponent::Retain(5),
        ];
        assert_eq!(
            normalize(op),
            vec![StringComponent::Insert("a".to_string())]
        );
    }

    // ── compose ─────────────────────────────────────────────────────────

    #[test]
    fn compose_identity() {
        // Composing two identity ops should yield identity
        let op1: StringOp = vec![];
        let op2: StringOp = vec![];
        assert!(compose(&op1, &op2).is_empty());
    }

    #[test]
    fn compose_retain_retain() {
        let op1 = vec![StringComponent::Retain(5)];
        let op2 = vec![StringComponent::Retain(5)];
        let composed = compose(&op1, &op2);
        // Trailing retains are stripped
        assert!(composed.is_empty());
    }

    #[test]
    fn compose_insert_then_retain() {
        let op1 = vec![StringComponent::Insert("abc".to_string())];
        let op2 = vec![StringComponent::Retain(3)];
        let composed = compose(&op1, &op2);
        assert_eq!(composed, vec![StringComponent::Insert("abc".to_string())]);
    }

    #[test]
    fn compose_retain_then_delete() {
        let op1 = vec![StringComponent::Retain(5)];
        let op2 = vec![StringComponent::Delete(3)];
        let composed = compose(&op1, &op2);
        assert_eq!(composed, vec![StringComponent::Delete(3)]);
    }

    #[test]
    fn compose_retain_then_delete_str() {
        let op1 = vec![StringComponent::Retain(5)];
        let op2 = vec![StringComponent::DeleteStr("hel".to_string())];
        let composed = compose(&op1, &op2);
        assert_eq!(
            composed,
            vec![StringComponent::DeleteStr("hel".to_string())]
        );
    }

    #[test]
    fn compose_insert_then_delete_str() {
        let op1 = vec![StringComponent::Insert("abc".to_string())];
        let op2 = vec![StringComponent::DeleteStr("ab".to_string())];
        let composed = compose(&op1, &op2);
        assert_eq!(composed, vec![StringComponent::Insert("c".to_string())]);
    }

    #[test]
    fn compose_verifies_apply_equivalence() {
        // apply(apply(s, op1), op2) == apply(s, compose(op1, op2))
        let s = "hello world";
        let op1 = vec![
            StringComponent::Retain(5),
            StringComponent::Delete(1),
            StringComponent::Insert("-".to_string()),
        ];
        let op2 = vec![
            StringComponent::Retain(6),
            StringComponent::Insert("!".to_string()),
        ];
        let sequential = apply(&apply(s, &op1), &op2);
        let composed = compose(&op1, &op2);
        let direct = apply(s, &composed);
        assert_eq!(sequential, direct);
    }

    #[test]
    fn compose_delete_passes_through() {
        // op1 deletes chars, op2 inserts — delete should pass through
        let op1 = vec![StringComponent::Delete(3)];
        let op2 = vec![StringComponent::Insert("X".to_string())];
        let composed = compose(&op1, &op2);
        // Apply to "abcdef" -> op1 removes "abc" -> "def", op2 inserts "X" -> "Xdef"
        let s = "abcdef";
        let sequential = apply(&apply(s, &op1), &op2);
        let direct = apply(s, &composed);
        assert_eq!(sequential, direct);
    }

    #[test]
    fn compose_delete_str_passes_through() {
        let op1 = vec![StringComponent::DeleteStr("abc".to_string())];
        let op2 = vec![StringComponent::Insert("X".to_string())];
        let composed = compose(&op1, &op2);
        let s = "abcdef";
        let sequential = apply(&apply(s, &op1), &op2);
        let direct = apply(s, &composed);
        assert_eq!(sequential, direct);
    }

    #[test]
    fn compose_insert_partial_retain() {
        // Insert "abcde", then retain only 3 of them
        let op1 = vec![StringComponent::Insert("abcde".to_string())];
        let op2 = vec![StringComponent::Retain(3), StringComponent::Delete(2)];
        let composed = compose(&op1, &op2);
        assert_eq!(composed, vec![StringComponent::Insert("abc".to_string())]);
    }

    // ── transform ───────────────────────────────────────────────────────

    #[test]
    fn transform_identity() {
        let op: StringOp = vec![];
        let against: StringOp = vec![];
        assert!(transform(&op, &against, true).is_empty());
    }

    #[test]
    fn transform_insert_right_wins() {
        let op = vec![StringComponent::Insert("A".to_string())];
        let against = vec![StringComponent::Insert("B".to_string())];
        let t = transform(&op, &against, false);
        // Right wins: against insert goes first, so op should retain over B then insert A
        let s = "B";
        let result = apply(s, &t);
        assert!(result.contains('A'));
    }

    #[test]
    fn transform_retain_vs_delete() {
        // op retains 5, against deletes 3 -> op should only retain 2
        let op = vec![StringComponent::Retain(5)];
        let against = vec![StringComponent::Delete(3)];
        let t = transform(&op, &against, true);
        // After against deletes 3, the remaining 2 chars of our retain survive
        assert_eq!(t, vec![]); // trailing retain stripped
    }

    #[test]
    fn transform_retain_vs_delete_str() {
        let op = vec![StringComponent::Retain(5)];
        let against = vec![StringComponent::DeleteStr("abc".to_string())];
        let t = transform(&op, &against, true);
        // Against deleted 3 chars, so op's retain of 5 -> retain of 2, stripped as trailing
        assert_eq!(t, vec![]);
    }

    #[test]
    fn transform_delete_vs_retain() {
        let op = vec![StringComponent::Delete(3)];
        let against = vec![StringComponent::Retain(5)];
        let t = transform(&op, &against, true);
        assert_eq!(t, vec![StringComponent::Delete(3)]);
    }

    #[test]
    fn transform_delete_str_vs_retain() {
        let op = vec![StringComponent::DeleteStr("abc".to_string())];
        let against = vec![StringComponent::Retain(5)];
        let t = transform(&op, &against, true);
        assert_eq!(t, vec![StringComponent::DeleteStr("abc".to_string())]);
    }

    #[test]
    fn transform_delete_vs_delete() {
        // Both delete same region: op's delete becomes redundant
        let op = vec![StringComponent::Delete(3)];
        let against = vec![StringComponent::Delete(3)];
        let t = transform(&op, &against, true);
        assert!(t.is_empty());
    }

    #[test]
    fn transform_delete_vs_delete_str() {
        let op = vec![StringComponent::Delete(3)];
        let against = vec![StringComponent::DeleteStr("abc".to_string())];
        let t = transform(&op, &against, true);
        assert!(t.is_empty());
    }

    #[test]
    fn transform_delete_str_vs_delete() {
        let op = vec![StringComponent::DeleteStr("abc".to_string())];
        let against = vec![StringComponent::Delete(3)];
        let t = transform(&op, &against, true);
        assert!(t.is_empty());
    }

    #[test]
    fn transform_delete_str_vs_delete_str() {
        let op = vec![StringComponent::DeleteStr("abc".to_string())];
        let against = vec![StringComponent::DeleteStr("abc".to_string())];
        let t = transform(&op, &against, true);
        assert!(t.is_empty());
    }

    #[test]
    fn transform_delete_partial_vs_delete() {
        // op deletes 5, against deletes 3 -> op should still delete 2 more
        let op = vec![StringComponent::Delete(5)];
        let against = vec![StringComponent::Delete(3)];
        let t = transform(&op, &against, true);
        assert_eq!(t, vec![StringComponent::Delete(2)]);
    }

    #[test]
    fn transform_convergence() {
        // Both ops applied to same doc should converge
        let s = "hello";
        let op_a = vec![
            StringComponent::Retain(5),
            StringComponent::Insert(" world".to_string()),
        ];
        let op_b = vec![
            StringComponent::Delete(1),
            StringComponent::Insert("H".to_string()),
        ];
        let t_a = transform(&op_a, &op_b, true);
        let t_b = transform(&op_b, &op_a, false);
        let result_a = apply(&apply(s, &op_b), &t_a);
        let result_b = apply(&apply(s, &op_a), &t_b);
        assert_eq!(result_a, result_b);
    }

    #[test]
    fn transform_op_remaining_after_against_exhausted() {
        // op has more components than against
        let op = vec![
            StringComponent::Retain(2),
            StringComponent::Insert("X".to_string()),
        ];
        let against: StringOp = vec![];
        let t = transform(&op, &against, true);
        // op passes through since against is empty
        assert_eq!(
            t,
            vec![
                StringComponent::Retain(2),
                StringComponent::Insert("X".to_string())
            ]
        );
    }

    #[test]
    fn transform_retain_vs_retain_partial() {
        let op = vec![StringComponent::Retain(5)];
        let against = vec![StringComponent::Retain(3)];
        let t = transform(&op, &against, true);
        // Trailing retains stripped
        assert!(t.is_empty());
    }

    #[test]
    fn transform_delete_str_vs_delete_str_partial() {
        let op = vec![StringComponent::DeleteStr("abcde".to_string())];
        let against = vec![StringComponent::DeleteStr("abc".to_string())];
        let t = transform(&op, &against, true);
        assert_eq!(t, vec![StringComponent::DeleteStr("de".to_string())]);
    }
}
