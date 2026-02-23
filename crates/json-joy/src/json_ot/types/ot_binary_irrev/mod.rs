//! Irreversible binary operational transformation.
//!
//! Mirrors `packages/json-joy/src/json-ot/types/ot-binary-irreversible/`.
//!
//! Operates on `Vec<u8>` documents. Components are:
//! - `Retain(n)` — keep n bytes
//! - `Delete(n)` — skip n bytes
//! - `Insert(bytes)` — insert bytes

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryComponent {
    Retain(usize),
    Delete(usize),
    Insert(Vec<u8>),
}

pub type BinaryOp = Vec<BinaryComponent>;

impl BinaryComponent {
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
            Self::Insert(b) => b.len(),
        }
    }
}

/// Append a component, merging with the last if same type.
fn append(op: &mut BinaryOp, comp: BinaryComponent) {
    match (op.last_mut(), &comp) {
        (Some(BinaryComponent::Retain(n)), BinaryComponent::Retain(m)) => {
            *n += m;
            return;
        }
        (Some(BinaryComponent::Delete(n)), BinaryComponent::Delete(m)) => {
            *n += m;
            return;
        }
        (Some(BinaryComponent::Insert(s)), BinaryComponent::Insert(t)) => {
            s.extend_from_slice(t);
            return;
        }
        _ => {}
    }
    op.push(comp);
}

/// Remove trailing Retain components.
pub fn trim(op: &mut BinaryOp) {
    while matches!(op.last(), Some(BinaryComponent::Retain(_))) {
        op.pop();
    }
}

/// Normalize: coalesce adjacent same-type components and strip trailing retains.
pub fn normalize(op: BinaryOp) -> BinaryOp {
    let mut result: BinaryOp = Vec::new();
    for comp in op {
        match &comp {
            BinaryComponent::Retain(0) | BinaryComponent::Delete(0) => {}
            BinaryComponent::Insert(b) if b.is_empty() => {}
            _ => append(&mut result, comp),
        }
    }
    trim(&mut result);
    result
}

/// Apply a `BinaryOp` to a byte slice, returning the result.
pub fn apply(data: &[u8], op: &BinaryOp) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    let mut idx = 0usize;

    for comp in op {
        match comp {
            BinaryComponent::Retain(n) => {
                result.extend_from_slice(&data[idx..idx + n]);
                idx += n;
            }
            BinaryComponent::Delete(n) => {
                idx += n;
            }
            BinaryComponent::Insert(bytes) => {
                result.extend_from_slice(bytes);
            }
        }
    }
    result.extend_from_slice(&data[idx..]);
    result
}

/// Compose two sequential binary operations into one equivalent operation.
pub fn compose(op1: &BinaryOp, op2: &BinaryOp) -> BinaryOp {
    let mut result: BinaryOp = Vec::new();
    let mut iter1 = op1.iter().peekable();
    let mut iter2 = op2.iter().peekable();
    let mut rem1: Option<BinaryComponent> = None;
    let mut rem2: Option<BinaryComponent> = None;

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
            (Some(c1), Some(c2)) => match (&c1, &c2) {
                (BinaryComponent::Delete(n), _) => {
                    append(&mut result, BinaryComponent::Delete(*n));
                    rem2 = Some(c2);
                }
                (_, BinaryComponent::Insert(b)) => {
                    append(&mut result, BinaryComponent::Insert(b.clone()));
                    rem1 = Some(c1);
                }
                (BinaryComponent::Retain(n), BinaryComponent::Retain(m)) => {
                    let min = (*n).min(*m);
                    append(&mut result, BinaryComponent::Retain(min));
                    if n > m {
                        rem1 = Some(BinaryComponent::Retain(n - m));
                    } else if m > n {
                        rem2 = Some(BinaryComponent::Retain(m - n));
                    }
                }
                (BinaryComponent::Retain(n), BinaryComponent::Delete(m)) => {
                    let min = (*n).min(*m);
                    append(&mut result, BinaryComponent::Delete(min));
                    if n > m {
                        rem1 = Some(BinaryComponent::Retain(n - m));
                    } else if m > n {
                        rem2 = Some(BinaryComponent::Delete(m - n));
                    }
                }
                (BinaryComponent::Insert(b), BinaryComponent::Retain(m)) => {
                    let b_len = b.len();
                    let kept = b[..(*m).min(b_len)].to_vec();
                    append(&mut result, BinaryComponent::Insert(kept));
                    if b_len > *m {
                        rem1 = Some(BinaryComponent::Insert(b[*m..].to_vec()));
                    } else if *m > b_len {
                        rem2 = Some(BinaryComponent::Retain(m - b_len));
                    }
                }
                (BinaryComponent::Insert(b), BinaryComponent::Delete(m)) => {
                    let b_len = b.len();
                    if b_len > *m {
                        rem1 = Some(BinaryComponent::Insert(b[*m..].to_vec()));
                    } else if *m > b_len {
                        rem2 = Some(BinaryComponent::Delete(m - b_len));
                    }
                }
            },
        }
    }
    normalize(result)
}

/// Transform `op` against `against`.
pub fn transform(op: &BinaryOp, against: &BinaryOp, left_wins: bool) -> BinaryOp {
    let mut result: BinaryOp = Vec::new();
    let mut op_iter = op.iter().cloned().peekable();
    let mut ag_iter = against.iter().cloned().peekable();
    let mut rem_op: Option<BinaryComponent> = None;
    let mut rem_ag: Option<BinaryComponent> = None;

    loop {
        let o = rem_op.take().or_else(|| op_iter.next());
        let a = rem_ag.take().or_else(|| ag_iter.next());

        match (o, a) {
            (None, _) => break,
            (Some(o), None) => {
                append(&mut result, o);
            }
            (Some(o), Some(a)) => match (&o, &a) {
                (_, BinaryComponent::Insert(b)) => {
                    let n = b.len();
                    if left_wins {
                        rem_op = Some(o);
                        append(&mut result, BinaryComponent::Retain(n));
                    } else {
                        append(&mut result, BinaryComponent::Retain(n));
                        rem_op = Some(o);
                    }
                }
                (BinaryComponent::Insert(b), _) => {
                    append(&mut result, BinaryComponent::Insert(b.clone()));
                    rem_ag = Some(a);
                }
                (BinaryComponent::Retain(n), BinaryComponent::Retain(m)) => {
                    let min = (*n).min(*m);
                    append(&mut result, BinaryComponent::Retain(min));
                    if n > m {
                        rem_op = Some(BinaryComponent::Retain(n - m));
                    } else if m > n {
                        rem_ag = Some(BinaryComponent::Retain(m - n));
                    }
                }
                (BinaryComponent::Retain(n), BinaryComponent::Delete(m)) => {
                    if n > m {
                        rem_op = Some(BinaryComponent::Retain(n - m));
                    } else if m > n {
                        rem_ag = Some(BinaryComponent::Delete(m - n));
                    }
                }
                (BinaryComponent::Delete(n), BinaryComponent::Retain(m)) => {
                    let min = (*n).min(*m);
                    append(&mut result, BinaryComponent::Delete(min));
                    if n > m {
                        rem_op = Some(BinaryComponent::Delete(n - m));
                    } else if m > n {
                        rem_ag = Some(BinaryComponent::Retain(m - n));
                    }
                }
                (BinaryComponent::Delete(n), BinaryComponent::Delete(m)) => {
                    if n > m {
                        rem_op = Some(BinaryComponent::Delete(n - m));
                    } else if m > n {
                        rem_ag = Some(BinaryComponent::Delete(m - n));
                    }
                }
            },
        }
    }
    normalize(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_insert_bytes() {
        let op = vec![BinaryComponent::Insert(vec![1, 2, 3])];
        assert_eq!(apply(&[], &op), vec![1, 2, 3]);
    }

    #[test]
    fn apply_delete_bytes() {
        let op = vec![BinaryComponent::Delete(2)];
        assert_eq!(apply(&[1, 2, 3], &op), vec![3]);
    }

    #[test]
    fn apply_retain_then_insert() {
        let op = vec![
            BinaryComponent::Retain(2),
            BinaryComponent::Insert(vec![99]),
        ];
        assert_eq!(apply(&[1, 2, 3], &op), vec![1, 2, 99, 3]);
    }

    #[test]
    fn compose_insert_then_delete() {
        let op1 = vec![BinaryComponent::Insert(vec![10])];
        let op2 = vec![BinaryComponent::Delete(1)];
        let composed = compose(&op1, &op2);
        assert!(composed.is_empty());
    }

    // ── BinaryComponent src_len / dst_len ───────────────────────────────

    #[test]
    fn src_len_retain() {
        assert_eq!(BinaryComponent::Retain(5).src_len(), 5);
    }

    #[test]
    fn src_len_delete() {
        assert_eq!(BinaryComponent::Delete(3).src_len(), 3);
    }

    #[test]
    fn src_len_insert() {
        assert_eq!(BinaryComponent::Insert(vec![1, 2, 3]).src_len(), 0);
    }

    #[test]
    fn dst_len_retain() {
        assert_eq!(BinaryComponent::Retain(5).dst_len(), 5);
    }

    #[test]
    fn dst_len_delete() {
        assert_eq!(BinaryComponent::Delete(3).dst_len(), 0);
    }

    #[test]
    fn dst_len_insert() {
        assert_eq!(BinaryComponent::Insert(vec![1, 2, 3]).dst_len(), 3);
    }

    // ── apply edge cases ────────────────────────────────────────────────

    #[test]
    fn apply_empty_op() {
        assert_eq!(apply(&[1, 2, 3], &vec![]), vec![1, 2, 3]);
    }

    #[test]
    fn apply_empty_data() {
        let op = vec![BinaryComponent::Insert(vec![1, 2])];
        assert_eq!(apply(&[], &op), vec![1, 2]);
    }

    #[test]
    fn apply_retain_only() {
        let op = vec![BinaryComponent::Retain(3)];
        assert_eq!(apply(&[1, 2, 3, 4], &op), vec![1, 2, 3, 4]);
    }

    #[test]
    fn apply_complex_sequence() {
        let op = vec![
            BinaryComponent::Retain(1),
            BinaryComponent::Delete(2),
            BinaryComponent::Insert(vec![99]),
        ];
        assert_eq!(apply(&[1, 2, 3, 4, 5], &op), vec![1, 99, 4, 5]);
    }

    // ── trim ────────────────────────────────────────────────────────────

    #[test]
    fn trim_removes_trailing_retains() {
        let mut op = vec![BinaryComponent::Insert(vec![1]), BinaryComponent::Retain(5)];
        trim(&mut op);
        assert_eq!(op, vec![BinaryComponent::Insert(vec![1])]);
    }

    #[test]
    fn trim_does_not_remove_non_retain() {
        let mut op = vec![BinaryComponent::Insert(vec![1]), BinaryComponent::Delete(2)];
        trim(&mut op);
        assert_eq!(
            op,
            vec![BinaryComponent::Insert(vec![1]), BinaryComponent::Delete(2)]
        );
    }

    #[test]
    fn trim_removes_multiple_trailing_retains() {
        let mut op = vec![
            BinaryComponent::Delete(1),
            BinaryComponent::Retain(2),
            BinaryComponent::Retain(3),
        ];
        trim(&mut op);
        assert_eq!(op, vec![BinaryComponent::Delete(1)]);
    }

    // ── normalize ───────────────────────────────────────────────────────

    #[test]
    fn normalize_removes_zero_components() {
        let op = vec![
            BinaryComponent::Retain(0),
            BinaryComponent::Delete(0),
            BinaryComponent::Insert(vec![]),
            BinaryComponent::Insert(vec![1]),
        ];
        assert_eq!(normalize(op), vec![BinaryComponent::Insert(vec![1])]);
    }

    #[test]
    fn normalize_coalesces_adjacent_retains() {
        let op = vec![
            BinaryComponent::Retain(2),
            BinaryComponent::Retain(3),
            BinaryComponent::Delete(1),
        ];
        assert_eq!(
            normalize(op),
            vec![BinaryComponent::Retain(5), BinaryComponent::Delete(1)]
        );
    }

    #[test]
    fn normalize_coalesces_adjacent_deletes() {
        let op = vec![
            BinaryComponent::Delete(2),
            BinaryComponent::Delete(3),
            BinaryComponent::Insert(vec![1]),
        ];
        assert_eq!(
            normalize(op),
            vec![BinaryComponent::Delete(5), BinaryComponent::Insert(vec![1])]
        );
    }

    #[test]
    fn normalize_coalesces_adjacent_inserts() {
        let op = vec![
            BinaryComponent::Insert(vec![1, 2]),
            BinaryComponent::Insert(vec![3]),
        ];
        assert_eq!(normalize(op), vec![BinaryComponent::Insert(vec![1, 2, 3])]);
    }

    #[test]
    fn normalize_strips_trailing_retain() {
        let op = vec![BinaryComponent::Insert(vec![1]), BinaryComponent::Retain(5)];
        assert_eq!(normalize(op), vec![BinaryComponent::Insert(vec![1])]);
    }

    // ── compose ─────────────────────────────────────────────────────────

    #[test]
    fn compose_identity() {
        let op1: BinaryOp = vec![];
        let op2: BinaryOp = vec![];
        assert!(compose(&op1, &op2).is_empty());
    }

    #[test]
    fn compose_retain_retain() {
        let op1 = vec![BinaryComponent::Retain(3)];
        let op2 = vec![BinaryComponent::Retain(3)];
        assert!(compose(&op1, &op2).is_empty());
    }

    #[test]
    fn compose_retain_delete() {
        let op1 = vec![BinaryComponent::Retain(5)];
        let op2 = vec![BinaryComponent::Delete(3)];
        let composed = compose(&op1, &op2);
        assert_eq!(composed, vec![BinaryComponent::Delete(3)]);
    }

    #[test]
    fn compose_insert_retain() {
        let op1 = vec![BinaryComponent::Insert(vec![1, 2, 3])];
        let op2 = vec![BinaryComponent::Retain(3)];
        let composed = compose(&op1, &op2);
        assert_eq!(composed, vec![BinaryComponent::Insert(vec![1, 2, 3])]);
    }

    #[test]
    fn compose_insert_partial_retain() {
        let op1 = vec![BinaryComponent::Insert(vec![1, 2, 3, 4, 5])];
        let op2 = vec![BinaryComponent::Retain(3), BinaryComponent::Delete(2)];
        let composed = compose(&op1, &op2);
        assert_eq!(composed, vec![BinaryComponent::Insert(vec![1, 2, 3])]);
    }

    #[test]
    fn compose_insert_partial_delete() {
        let op1 = vec![BinaryComponent::Insert(vec![1, 2, 3, 4, 5])];
        let op2 = vec![BinaryComponent::Delete(3)];
        let composed = compose(&op1, &op2);
        assert_eq!(composed, vec![BinaryComponent::Insert(vec![4, 5])]);
    }

    #[test]
    fn compose_delete_passes_through() {
        let op1 = vec![BinaryComponent::Delete(3)];
        let op2 = vec![BinaryComponent::Insert(vec![99])];
        let composed = compose(&op1, &op2);
        let data = &[1, 2, 3, 4, 5];
        let sequential = apply(&apply(data, &op1), &op2);
        let direct = apply(data, &composed);
        assert_eq!(sequential, direct);
    }

    #[test]
    fn compose_verifies_apply_equivalence() {
        let data = &[1, 2, 3, 4, 5];
        let op1 = vec![
            BinaryComponent::Retain(2),
            BinaryComponent::Delete(1),
            BinaryComponent::Insert(vec![99]),
        ];
        let op2 = vec![
            BinaryComponent::Retain(3),
            BinaryComponent::Insert(vec![88]),
        ];
        let sequential = apply(&apply(data, &op1), &op2);
        let composed = compose(&op1, &op2);
        let direct = apply(data, &composed);
        assert_eq!(sequential, direct);
    }

    // ── transform ───────────────────────────────────────────────────────

    #[test]
    fn transform_identity() {
        let op: BinaryOp = vec![];
        let against: BinaryOp = vec![];
        assert!(transform(&op, &against, true).is_empty());
    }

    #[test]
    fn transform_insert_left_wins() {
        let op = vec![BinaryComponent::Insert(vec![1])];
        let against = vec![BinaryComponent::Insert(vec![2])];
        let t = transform(&op, &against, true);
        // Left wins: retain over against's insert comes first, then our insert
        assert_eq!(
            t,
            vec![BinaryComponent::Retain(1), BinaryComponent::Insert(vec![1]),]
        );
    }

    #[test]
    fn transform_insert_right_wins() {
        let op = vec![BinaryComponent::Insert(vec![1])];
        let against = vec![BinaryComponent::Insert(vec![2])];
        let t = transform(&op, &against, false);
        // Right wins: retain over against's insert, then insert
        assert_eq!(
            t,
            vec![BinaryComponent::Retain(1), BinaryComponent::Insert(vec![1]),]
        );
    }

    #[test]
    fn transform_retain_vs_retain() {
        let op = vec![BinaryComponent::Retain(5)];
        let against = vec![BinaryComponent::Retain(5)];
        let t = transform(&op, &against, true);
        assert!(t.is_empty()); // trailing retains stripped
    }

    #[test]
    fn transform_retain_vs_delete() {
        let op = vec![BinaryComponent::Retain(5)];
        let against = vec![BinaryComponent::Delete(3)];
        let t = transform(&op, &against, true);
        // Against deleted 3, so our retain of 5 -> retain of 2 (trailing, stripped)
        assert!(t.is_empty());
    }

    #[test]
    fn transform_delete_vs_retain() {
        let op = vec![BinaryComponent::Delete(3)];
        let against = vec![BinaryComponent::Retain(5)];
        let t = transform(&op, &against, true);
        assert_eq!(t, vec![BinaryComponent::Delete(3)]);
    }

    #[test]
    fn transform_delete_vs_delete() {
        let op = vec![BinaryComponent::Delete(3)];
        let against = vec![BinaryComponent::Delete(3)];
        let t = transform(&op, &against, true);
        assert!(t.is_empty());
    }

    #[test]
    fn transform_delete_partial_vs_delete() {
        let op = vec![BinaryComponent::Delete(5)];
        let against = vec![BinaryComponent::Delete(3)];
        let t = transform(&op, &against, true);
        assert_eq!(t, vec![BinaryComponent::Delete(2)]);
    }

    #[test]
    fn transform_op_remaining_after_against_exhausted() {
        let op = vec![
            BinaryComponent::Retain(2),
            BinaryComponent::Insert(vec![99]),
        ];
        let against: BinaryOp = vec![];
        let t = transform(&op, &against, true);
        assert_eq!(
            t,
            vec![
                BinaryComponent::Retain(2),
                BinaryComponent::Insert(vec![99]),
            ]
        );
    }

    #[test]
    fn transform_convergence() {
        let data = &[1, 2, 3, 4, 5];
        let op_a = vec![
            BinaryComponent::Retain(5),
            BinaryComponent::Insert(vec![6, 7]),
        ];
        let op_b = vec![
            BinaryComponent::Delete(1),
            BinaryComponent::Insert(vec![10]),
        ];
        let t_a = transform(&op_a, &op_b, true);
        let t_b = transform(&op_b, &op_a, false);
        let result_a = apply(&apply(data, &op_b), &t_a);
        let result_b = apply(&apply(data, &op_a), &t_b);
        assert_eq!(result_a, result_b);
    }

    #[test]
    fn transform_retain_vs_delete_partial() {
        // op retains 3, against deletes 5 -> retain is fully absorbed
        let op = vec![BinaryComponent::Retain(3)];
        let against = vec![BinaryComponent::Delete(5)];
        let t = transform(&op, &against, true);
        assert!(t.is_empty());
    }

    #[test]
    fn transform_delete_vs_retain_partial() {
        // op deletes 5, against retains 3 -> delete 3, then remainder
        let op = vec![BinaryComponent::Delete(5)];
        let against = vec![BinaryComponent::Retain(3)];
        let t = transform(&op, &against, true);
        assert_eq!(t, vec![BinaryComponent::Delete(5)]);
    }

    #[test]
    fn transform_delete_vs_delete_partial_ag_larger() {
        let op = vec![BinaryComponent::Delete(3)];
        let against = vec![BinaryComponent::Delete(5)];
        let t = transform(&op, &against, true);
        assert!(t.is_empty());
    }
}
