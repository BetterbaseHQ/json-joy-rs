//! Irreversible string operational transformation.
//!
//! Mirrors `packages/json-joy/src/json-ot/types/ot-string-irreversible/`.
//!
//! Like `ot_string` but does not store the content of deleted text —
//! deletions are represented only by their count.

#[derive(Debug, Clone, PartialEq)]
pub enum StringIrrevComponent {
    Retain(usize),
    Delete(usize),
    Insert(String),
}

pub type StringIrrevOp = Vec<StringIrrevComponent>;

impl StringIrrevComponent {
    pub fn src_len(&self) -> usize {
        match self {
            Self::Retain(n) => *n,
            Self::Delete(n) => *n,
            Self::Insert(_) => 0,
        }
    }

    pub fn dst_len(&self) -> usize {
        match self {
            Self::Retain(n) => *n,
            Self::Delete(_) => 0,
            Self::Insert(s) => s.chars().count(),
        }
    }
}

/// Append a component, merging with the last if same type.
fn append(op: &mut StringIrrevOp, comp: StringIrrevComponent) {
    match (op.last_mut(), &comp) {
        (Some(StringIrrevComponent::Retain(n)), StringIrrevComponent::Retain(m)) => {
            *n += m;
            return;
        }
        (Some(StringIrrevComponent::Delete(n)), StringIrrevComponent::Delete(m)) => {
            *n += m;
            return;
        }
        (Some(StringIrrevComponent::Insert(s)), StringIrrevComponent::Insert(t)) => {
            s.push_str(t);
            return;
        }
        _ => {}
    }
    op.push(comp);
}

/// Remove trailing Retain components.
pub fn trim(op: &mut StringIrrevOp) {
    while matches!(op.last(), Some(StringIrrevComponent::Retain(_))) {
        op.pop();
    }
}

/// Normalize: coalesce adjacent same-type components and strip trailing retains.
pub fn normalize(op: StringIrrevOp) -> StringIrrevOp {
    let mut result: StringIrrevOp = Vec::new();
    for comp in op {
        match &comp {
            StringIrrevComponent::Retain(0) | StringIrrevComponent::Delete(0) => {}
            StringIrrevComponent::Insert(s) if s.is_empty() => {}
            _ => append(&mut result, comp),
        }
    }
    trim(&mut result);
    result
}

/// Apply a `StringIrrevOp` to a string, returning the result.
pub fn apply(s: &str, op: &StringIrrevOp) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();
    let mut idx = 0usize;

    for comp in op {
        match comp {
            StringIrrevComponent::Retain(n) => {
                result.extend(chars[idx..idx + n].iter());
                idx += n;
            }
            StringIrrevComponent::Delete(n) => {
                idx += n;
            }
            StringIrrevComponent::Insert(ins) => {
                result.push_str(ins);
            }
        }
    }
    result.extend(chars[idx..].iter());
    result
}

/// Compose two sequential operations into one equivalent operation.
pub fn compose(op1: &StringIrrevOp, op2: &StringIrrevOp) -> StringIrrevOp {
    let mut result: StringIrrevOp = Vec::new();
    let mut iter1 = op1.iter().peekable();
    let mut iter2 = op2.iter().peekable();
    let mut rem1: Option<StringIrrevComponent> = None;
    let mut rem2: Option<StringIrrevComponent> = None;

    loop {
        let c1 = rem1.take().or_else(|| iter1.next().cloned());
        let c2 = rem2.take().or_else(|| iter2.next().cloned());

        match (c1, c2) {
            (None, None) => break,
            (Some(c), None) => {
                append(&mut result, c);
            }
            (None, Some(c)) => {
                append(&mut result, c);
            }
            (Some(c1), Some(c2)) => {
                match (&c1, &c2) {
                    // Delete from op1 passes through unchanged
                    (StringIrrevComponent::Delete(n), _) => {
                        append(&mut result, StringIrrevComponent::Delete(*n));
                        rem2 = Some(c2);
                    }
                    // Insert from op2 passes through unchanged
                    (_, StringIrrevComponent::Insert(s)) => {
                        append(&mut result, StringIrrevComponent::Insert(s.clone()));
                        rem1 = Some(c1);
                    }
                    // Retain op1 + Retain op2
                    (StringIrrevComponent::Retain(n), StringIrrevComponent::Retain(m)) => {
                        let min = (*n).min(*m);
                        append(&mut result, StringIrrevComponent::Retain(min));
                        if n > m {
                            rem1 = Some(StringIrrevComponent::Retain(n - m));
                        } else if m > n {
                            rem2 = Some(StringIrrevComponent::Retain(m - n));
                        }
                    }
                    // Retain op1 + Delete op2: retain becomes delete
                    (StringIrrevComponent::Retain(n), StringIrrevComponent::Delete(m)) => {
                        let min = (*n).min(*m);
                        append(&mut result, StringIrrevComponent::Delete(min));
                        if n > m {
                            rem1 = Some(StringIrrevComponent::Retain(n - m));
                        } else if m > n {
                            rem2 = Some(StringIrrevComponent::Delete(m - n));
                        }
                    }
                    // Insert op1 + Retain op2: keep the inserted portion
                    (StringIrrevComponent::Insert(s), StringIrrevComponent::Retain(m)) => {
                        let s_len = s.chars().count();
                        let kept: String = s.chars().take(*m).collect();
                        append(&mut result, StringIrrevComponent::Insert(kept));
                        if s_len > *m {
                            rem1 = Some(StringIrrevComponent::Insert(s.chars().skip(*m).collect()));
                        } else if *m > s_len {
                            rem2 = Some(StringIrrevComponent::Retain(m - s_len));
                        }
                    }
                    // Insert op1 + Delete op2: they cancel out
                    (StringIrrevComponent::Insert(s), StringIrrevComponent::Delete(m)) => {
                        let s_len = s.chars().count();
                        if s_len > *m {
                            rem1 = Some(StringIrrevComponent::Insert(s.chars().skip(*m).collect()));
                        } else if *m > s_len {
                            rem2 = Some(StringIrrevComponent::Delete(m - s_len));
                        }
                    }
                }
            }
        }
    }
    normalize(result)
}

/// Transform `op` against `against`, assuming `left_wins` for concurrent inserts.
pub fn transform(op: &StringIrrevOp, against: &StringIrrevOp, left_wins: bool) -> StringIrrevOp {
    let mut result: StringIrrevOp = Vec::new();
    let mut op_iter = op.iter().cloned().peekable();
    let mut ag_iter = against.iter().cloned().peekable();
    let mut rem_op: Option<StringIrrevComponent> = None;
    let mut rem_ag: Option<StringIrrevComponent> = None;

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
                    // Against inserts: add a retain to skip over the inserted chars
                    (_, StringIrrevComponent::Insert(s)) => {
                        let n = s.chars().count();
                        if left_wins {
                            rem_op = Some(o);
                            append(&mut result, StringIrrevComponent::Retain(n));
                        } else {
                            append(&mut result, StringIrrevComponent::Retain(n));
                            rem_op = Some(o);
                        }
                    }
                    // Op inserts: pass through
                    (StringIrrevComponent::Insert(s), _) => {
                        append(&mut result, StringIrrevComponent::Insert(s.clone()));
                        rem_ag = Some(a);
                    }
                    // Retain vs retain
                    (StringIrrevComponent::Retain(n), StringIrrevComponent::Retain(m)) => {
                        let min = (*n).min(*m);
                        append(&mut result, StringIrrevComponent::Retain(min));
                        if n > m {
                            rem_op = Some(StringIrrevComponent::Retain(n - m));
                        } else if m > n {
                            rem_ag = Some(StringIrrevComponent::Retain(m - n));
                        }
                    }
                    // Retain vs delete: the chars we wanted to retain are gone
                    (StringIrrevComponent::Retain(n), StringIrrevComponent::Delete(m)) => {
                        if n > m {
                            rem_op = Some(StringIrrevComponent::Retain(n - m));
                        } else if m > n {
                            rem_ag = Some(StringIrrevComponent::Delete(m - n));
                        }
                    }
                    // Delete vs retain: delete passes through
                    (StringIrrevComponent::Delete(n), StringIrrevComponent::Retain(m)) => {
                        let min = (*n).min(*m);
                        append(&mut result, StringIrrevComponent::Delete(min));
                        if n > m {
                            rem_op = Some(StringIrrevComponent::Delete(n - m));
                        } else if m > n {
                            rem_ag = Some(StringIrrevComponent::Retain(m - n));
                        }
                    }
                    // Delete vs delete: both deleting same region — op delete is redundant
                    (StringIrrevComponent::Delete(n), StringIrrevComponent::Delete(m)) => {
                        if n > m {
                            rem_op = Some(StringIrrevComponent::Delete(n - m));
                        } else if m > n {
                            rem_ag = Some(StringIrrevComponent::Delete(m - n));
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
    fn apply_insert() {
        let op = vec![StringIrrevComponent::Insert("hello".to_string())];
        assert_eq!(apply("", &op), "hello");
    }

    #[test]
    fn apply_delete() {
        let op = vec![StringIrrevComponent::Delete(3)];
        assert_eq!(apply("hello", &op), "lo");
    }

    #[test]
    fn apply_retain_then_insert() {
        let op = vec![
            StringIrrevComponent::Retain(5),
            StringIrrevComponent::Insert(" world".to_string()),
        ];
        assert_eq!(apply("hello", &op), "hello world");
    }

    #[test]
    fn compose_insert_then_delete() {
        let op1 = vec![StringIrrevComponent::Insert("X".to_string())];
        let op2 = vec![StringIrrevComponent::Delete(1)];
        let composed = compose(&op1, &op2);
        assert!(composed.is_empty());
    }

    #[test]
    fn transform_concurrent_inserts() {
        let op = vec![StringIrrevComponent::Insert("A".to_string())];
        let against = vec![StringIrrevComponent::Insert("B".to_string())];
        let t = transform(&op, &against, true);
        let result = apply("hello", &t);
        assert!(result.contains('A'));
    }

    // ── StringIrrevComponent src_len / dst_len ──────────────────────────

    #[test]
    fn src_len_retain() {
        assert_eq!(StringIrrevComponent::Retain(5).src_len(), 5);
    }

    #[test]
    fn src_len_delete() {
        assert_eq!(StringIrrevComponent::Delete(3).src_len(), 3);
    }

    #[test]
    fn src_len_insert() {
        assert_eq!(StringIrrevComponent::Insert("abc".to_string()).src_len(), 0);
    }

    #[test]
    fn dst_len_retain() {
        assert_eq!(StringIrrevComponent::Retain(5).dst_len(), 5);
    }

    #[test]
    fn dst_len_delete() {
        assert_eq!(StringIrrevComponent::Delete(3).dst_len(), 0);
    }

    #[test]
    fn dst_len_insert() {
        assert_eq!(StringIrrevComponent::Insert("abc".to_string()).dst_len(), 3);
    }

    #[test]
    fn dst_len_unicode() {
        assert_eq!(
            StringIrrevComponent::Insert("日本語".to_string()).dst_len(),
            3
        );
    }

    // ── apply edge cases ────────────────────────────────────────────────

    #[test]
    fn apply_empty_op() {
        assert_eq!(apply("hello", &vec![]), "hello");
    }

    #[test]
    fn apply_empty_string() {
        let op = vec![StringIrrevComponent::Insert("abc".to_string())];
        assert_eq!(apply("", &op), "abc");
    }

    #[test]
    fn apply_retain_only() {
        let op = vec![StringIrrevComponent::Retain(5)];
        assert_eq!(apply("hello world", &op), "hello world");
    }

    #[test]
    fn apply_complex_sequence() {
        let op = vec![
            StringIrrevComponent::Retain(1),
            StringIrrevComponent::Delete(4),
            StringIrrevComponent::Insert("X".to_string()),
            StringIrrevComponent::Delete(2),
        ];
        assert_eq!(apply("hello world", &op), "hXorld");
    }

    // ── trim ────────────────────────────────────────────────────────────

    #[test]
    fn trim_removes_trailing_retains() {
        let mut op = vec![
            StringIrrevComponent::Insert("a".to_string()),
            StringIrrevComponent::Retain(5),
        ];
        trim(&mut op);
        assert_eq!(op, vec![StringIrrevComponent::Insert("a".to_string())]);
    }

    #[test]
    fn trim_does_not_remove_non_retain() {
        let mut op = vec![
            StringIrrevComponent::Insert("a".to_string()),
            StringIrrevComponent::Delete(2),
        ];
        trim(&mut op);
        assert_eq!(
            op,
            vec![
                StringIrrevComponent::Insert("a".to_string()),
                StringIrrevComponent::Delete(2)
            ]
        );
    }

    // ── normalize ───────────────────────────────────────────────────────

    #[test]
    fn normalize_removes_zero_and_empty() {
        let op = vec![
            StringIrrevComponent::Retain(0),
            StringIrrevComponent::Delete(0),
            StringIrrevComponent::Insert(String::new()),
            StringIrrevComponent::Insert("a".to_string()),
        ];
        assert_eq!(
            normalize(op),
            vec![StringIrrevComponent::Insert("a".to_string())]
        );
    }

    #[test]
    fn normalize_coalesces_adjacent() {
        let op = vec![
            StringIrrevComponent::Delete(2),
            StringIrrevComponent::Delete(3),
            StringIrrevComponent::Insert("x".to_string()),
        ];
        assert_eq!(
            normalize(op),
            vec![
                StringIrrevComponent::Delete(5),
                StringIrrevComponent::Insert("x".to_string()),
            ]
        );
    }

    #[test]
    fn normalize_coalesces_adjacent_inserts() {
        let op = vec![
            StringIrrevComponent::Insert("a".to_string()),
            StringIrrevComponent::Insert("b".to_string()),
        ];
        assert_eq!(
            normalize(op),
            vec![StringIrrevComponent::Insert("ab".to_string())]
        );
    }

    #[test]
    fn normalize_strips_trailing_retain() {
        let op = vec![
            StringIrrevComponent::Insert("a".to_string()),
            StringIrrevComponent::Retain(5),
        ];
        assert_eq!(
            normalize(op),
            vec![StringIrrevComponent::Insert("a".to_string())]
        );
    }

    // ── compose ─────────────────────────────────────────────────────────

    #[test]
    fn compose_identity() {
        let op1: StringIrrevOp = vec![];
        let op2: StringIrrevOp = vec![];
        assert!(compose(&op1, &op2).is_empty());
    }

    #[test]
    fn compose_retain_retain() {
        let op1 = vec![StringIrrevComponent::Retain(5)];
        let op2 = vec![StringIrrevComponent::Retain(5)];
        assert!(compose(&op1, &op2).is_empty());
    }

    #[test]
    fn compose_retain_delete() {
        let op1 = vec![StringIrrevComponent::Retain(5)];
        let op2 = vec![StringIrrevComponent::Delete(3)];
        let composed = compose(&op1, &op2);
        assert_eq!(composed, vec![StringIrrevComponent::Delete(3)]);
    }

    #[test]
    fn compose_insert_retain() {
        let op1 = vec![StringIrrevComponent::Insert("abc".to_string())];
        let op2 = vec![StringIrrevComponent::Retain(3)];
        let composed = compose(&op1, &op2);
        assert_eq!(
            composed,
            vec![StringIrrevComponent::Insert("abc".to_string())]
        );
    }

    #[test]
    fn compose_insert_partial_delete() {
        let op1 = vec![StringIrrevComponent::Insert("abcde".to_string())];
        let op2 = vec![StringIrrevComponent::Delete(3)];
        let composed = compose(&op1, &op2);
        assert_eq!(
            composed,
            vec![StringIrrevComponent::Insert("de".to_string())]
        );
    }

    #[test]
    fn compose_insert_partial_retain() {
        let op1 = vec![StringIrrevComponent::Insert("abcde".to_string())];
        let op2 = vec![
            StringIrrevComponent::Retain(3),
            StringIrrevComponent::Delete(2),
        ];
        let composed = compose(&op1, &op2);
        assert_eq!(
            composed,
            vec![StringIrrevComponent::Insert("abc".to_string())]
        );
    }

    #[test]
    fn compose_delete_passes_through() {
        let op1 = vec![StringIrrevComponent::Delete(3)];
        let op2 = vec![StringIrrevComponent::Insert("X".to_string())];
        let composed = compose(&op1, &op2);
        let s = "abcdef";
        let sequential = apply(&apply(s, &op1), &op2);
        let direct = apply(s, &composed);
        assert_eq!(sequential, direct);
    }

    #[test]
    fn compose_verifies_apply_equivalence() {
        let s = "hello world";
        let op1 = vec![
            StringIrrevComponent::Retain(5),
            StringIrrevComponent::Delete(1),
            StringIrrevComponent::Insert("-".to_string()),
        ];
        let op2 = vec![
            StringIrrevComponent::Retain(6),
            StringIrrevComponent::Insert("!".to_string()),
        ];
        let sequential = apply(&apply(s, &op1), &op2);
        let composed = compose(&op1, &op2);
        let direct = apply(s, &composed);
        assert_eq!(sequential, direct);
    }

    // ── transform ───────────────────────────────────────────────────────

    #[test]
    fn transform_identity() {
        let op: StringIrrevOp = vec![];
        let against: StringIrrevOp = vec![];
        assert!(transform(&op, &against, true).is_empty());
    }

    #[test]
    fn transform_insert_right_wins() {
        let op = vec![StringIrrevComponent::Insert("A".to_string())];
        let against = vec![StringIrrevComponent::Insert("B".to_string())];
        let t = transform(&op, &against, false);
        let result = apply("B", &t);
        assert!(result.contains('A'));
    }

    #[test]
    fn transform_retain_vs_retain() {
        let op = vec![StringIrrevComponent::Retain(5)];
        let against = vec![StringIrrevComponent::Retain(5)];
        let t = transform(&op, &against, true);
        assert!(t.is_empty());
    }

    #[test]
    fn transform_retain_vs_delete() {
        let op = vec![StringIrrevComponent::Retain(5)];
        let against = vec![StringIrrevComponent::Delete(3)];
        let t = transform(&op, &against, true);
        assert!(t.is_empty());
    }

    #[test]
    fn transform_retain_vs_delete_partial() {
        let op = vec![StringIrrevComponent::Retain(3)];
        let against = vec![StringIrrevComponent::Delete(5)];
        let t = transform(&op, &against, true);
        assert!(t.is_empty());
    }

    #[test]
    fn transform_delete_vs_retain() {
        let op = vec![StringIrrevComponent::Delete(3)];
        let against = vec![StringIrrevComponent::Retain(5)];
        let t = transform(&op, &against, true);
        assert_eq!(t, vec![StringIrrevComponent::Delete(3)]);
    }

    #[test]
    fn transform_delete_vs_delete() {
        let op = vec![StringIrrevComponent::Delete(3)];
        let against = vec![StringIrrevComponent::Delete(3)];
        let t = transform(&op, &against, true);
        assert!(t.is_empty());
    }

    #[test]
    fn transform_delete_partial_vs_delete() {
        let op = vec![StringIrrevComponent::Delete(5)];
        let against = vec![StringIrrevComponent::Delete(3)];
        let t = transform(&op, &against, true);
        assert_eq!(t, vec![StringIrrevComponent::Delete(2)]);
    }

    #[test]
    fn transform_op_remaining_after_against_exhausted() {
        let op = vec![
            StringIrrevComponent::Retain(2),
            StringIrrevComponent::Insert("X".to_string()),
        ];
        let against: StringIrrevOp = vec![];
        let t = transform(&op, &against, true);
        assert_eq!(
            t,
            vec![
                StringIrrevComponent::Retain(2),
                StringIrrevComponent::Insert("X".to_string()),
            ]
        );
    }

    #[test]
    fn transform_convergence() {
        let s = "hello";
        let op_a = vec![
            StringIrrevComponent::Retain(5),
            StringIrrevComponent::Insert(" world".to_string()),
        ];
        let op_b = vec![
            StringIrrevComponent::Delete(1),
            StringIrrevComponent::Insert("H".to_string()),
        ];
        let t_a = transform(&op_a, &op_b, true);
        let t_b = transform(&op_b, &op_a, false);
        let result_a = apply(&apply(s, &op_b), &t_a);
        let result_b = apply(&apply(s, &op_a), &t_b);
        assert_eq!(result_a, result_b);
    }
}
