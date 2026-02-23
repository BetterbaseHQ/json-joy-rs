//! Diff patch cleanup utilities.
//!
//! Mirrors `packages/json-joy/src/util/diff/str-utils.ts`.

use super::str::{overlap, sfx, Patch, PatchOpType};

/// Removes redundant equalities and adjusts diff operations for better
/// semantic alignment (word / line boundaries).
pub fn cleanup_patch(patch: &mut Patch) {
    let mut changes = false;
    let mut equalities: Vec<usize> = Vec::new();
    let mut last_equality: Option<String> = None;
    let mut pointer = 0usize;
    let mut len_ins1 = 0usize;
    let mut len_del1 = 0usize;
    let mut len_ins2 = 0usize;
    let mut len_del2 = 0usize;

    while pointer < patch.len() {
        if patch[pointer].0 == PatchOpType::Eql {
            equalities.push(pointer);
            len_ins1 = len_ins2;
            len_del1 = len_del2;
            len_ins2 = 0;
            len_del2 = 0;
            last_equality = Some(patch[pointer].1.clone());
        } else {
            if patch[pointer].0 == PatchOpType::Ins {
                len_ins2 += patch[pointer].1.chars().count();
            } else {
                len_del2 += patch[pointer].1.chars().count();
            }

            if let Some(ref le) = last_equality {
                let le_len = le.chars().count();
                if le_len <= len_ins1.max(len_del1) && le_len <= len_ins2.max(len_del2) {
                    let eq_idx = *equalities.last().unwrap();
                    let le_text = le.clone();
                    // Insert a DEL of the equality before it
                    patch.insert(eq_idx, (PatchOpType::Del, le_text));
                    // Change the equality itself to INS
                    patch[eq_idx + 1].0 = PatchOpType::Ins;
                    // Pop the last two equalities (the one we just processed and the one before)
                    equalities.pop();
                    equalities.pop();
                    pointer = if let Some(&p) = equalities.last() {
                        p
                    } else {
                        0
                    };
                    // Reset to just before this position
                    if pointer > 0 {
                        pointer = pointer.saturating_sub(1);
                    }
                    len_ins1 = 0;
                    len_del1 = 0;
                    len_ins2 = 0;
                    len_del2 = 0;
                    last_equality = None;
                    changes = true;
                }
            }
        }
        pointer += 1;
    }

    if changes {
        cleanup_patch(patch);
    }
    cleanup_semantic_lossless(patch);

    // Find overlaps between deletions and insertions
    pointer = 1;
    while pointer < patch.len() {
        if patch[pointer - 1].0 == PatchOpType::Del && patch[pointer].0 == PatchOpType::Ins {
            let deletion = patch[pointer - 1].1.clone();
            let insertion = patch[pointer].1.clone();
            let del_chars = deletion.chars().count();
            let ins_chars = insertion.chars().count();
            let ov1 = overlap(&deletion, &insertion);
            let ov2 = overlap(&insertion, &deletion);
            if ov1 >= ov2 {
                if ov1 * 2 >= del_chars || ov1 * 2 >= ins_chars {
                    let eq_str: String = insertion.chars().take(ov1).collect();
                    let del_str: String = deletion.chars().take(del_chars - ov1).collect();
                    let ins_str: String = insertion.chars().skip(ov1).collect();
                    patch[pointer - 1].1 = del_str;
                    patch.insert(pointer, (PatchOpType::Eql, eq_str));
                    patch[pointer + 1].1 = ins_str;
                    pointer += 1;
                }
            } else if ov2 * 2 >= del_chars || ov2 * 2 >= ins_chars {
                let eq_str: String = deletion.chars().take(ov2).collect();
                let ins_str: String = insertion.chars().take(ins_chars - ov2).collect();
                let del_str: String = deletion.chars().skip(ov2).collect();
                patch.insert(pointer, (PatchOpType::Eql, eq_str));
                patch[pointer - 1].0 = PatchOpType::Ins;
                patch[pointer - 1].1 = ins_str;
                patch[pointer + 1].0 = PatchOpType::Del;
                patch[pointer + 1].1 = del_str;
                pointer += 1;
            }
            pointer += 1;
        }
        pointer += 1;
    }
}

fn semantic_score(one: &str, two: &str) -> u8 {
    if one.is_empty() || two.is_empty() {
        return 6;
    }
    let char1 = one.chars().last().unwrap();
    let char2 = two.chars().next().unwrap();
    let non_alnum1 = !char1.is_alphanumeric();
    let non_alnum2 = !char2.is_alphanumeric();
    let ws1 = non_alnum1 && char1.is_whitespace();
    let ws2 = non_alnum2 && char2.is_whitespace();
    let lb1 = ws1 && (char1 == '\r' || char1 == '\n');
    let lb2 = ws2 && (char2 == '\r' || char2 == '\n');
    let bl1 = lb1 && (one.ends_with("\n\r\n") || one.ends_with("\n\n"));
    let bl2 = lb2 && (two.starts_with("\r\n\r\n") || two.starts_with("\n\n"));
    if bl1 || bl2 {
        return 5;
    }
    if lb1 || lb2 {
        return 4;
    }
    if non_alnum1 && !ws1 && ws2 {
        return 3;
    }
    if ws1 || ws2 {
        return 2;
    }
    if non_alnum1 || non_alnum2 {
        return 1;
    }
    0
}

fn cleanup_semantic_lossless(patch: &mut Patch) {
    let mut pointer = 1usize;
    while pointer + 1 < patch.len() {
        let prev_type = patch[pointer - 1].0;
        let next_type = patch[pointer + 1].0;
        if prev_type == PatchOpType::Eql && next_type == PatchOpType::Eql {
            let mut equality1 = patch[pointer - 1].1.clone();
            let mut edit = patch[pointer].1.clone();
            let mut equality2 = patch[pointer + 1].1.clone();

            // Shift edit as far left as possible
            let common = sfx(&equality1, &edit);
            if common > 0 {
                let e1_chars: Vec<char> = equality1.chars().collect();
                let edit_chars: Vec<char> = edit.chars().collect();
                let common_str: String = edit_chars[edit_chars.len() - common..].iter().collect();
                equality1 = e1_chars[..e1_chars.len() - common].iter().collect();
                edit = common_str.clone()
                    + &edit_chars[..edit_chars.len() - common]
                        .iter()
                        .collect::<String>();
                equality2 = common_str + &equality2;
            }

            // Step right to find best semantic fit
            let mut best_eq1 = equality1.clone();
            let mut best_edit = edit.clone();
            let mut best_eq2 = equality2.clone();
            let mut best_score =
                semantic_score(&equality1, &edit) + semantic_score(&edit, &equality2);

            let edit_chars: Vec<char> = edit.chars().collect();
            let eq2_chars: Vec<char> = equality2.chars().collect();
            let mut eq1 = equality1.clone();
            let mut ed = edit.clone();
            let mut eq2 = equality2.clone();

            while !ed.is_empty() && !eq2.is_empty() {
                let ed_chars: Vec<char> = ed.chars().collect();
                let eq2_chars_cur: Vec<char> = eq2.chars().collect();
                if ed_chars[0] != eq2_chars_cur[0] {
                    break;
                }
                let c = ed_chars[0];
                eq1.push(c);
                ed = ed_chars[1..].iter().collect::<String>() + &c.to_string();
                eq2 = eq2_chars_cur[1..].iter().collect();
                let score = semantic_score(&eq1, &ed) + semantic_score(&ed, &eq2);
                if score >= best_score {
                    best_score = score;
                    best_eq1 = eq1.clone();
                    best_edit = ed.clone();
                    best_eq2 = eq2.clone();
                }
            }
            let _ = (edit_chars, eq2_chars); // suppress warnings

            if patch[pointer - 1].1 != best_eq1 {
                if best_eq1.is_empty() {
                    patch.remove(pointer - 1);
                    pointer = pointer.saturating_sub(1);
                } else {
                    patch[pointer - 1].1 = best_eq1;
                }
                if let Some(p) = patch.get_mut(pointer) {
                    p.1 = best_edit;
                }
                if pointer + 1 < patch.len() {
                    if best_eq2.is_empty() {
                        patch.remove(pointer + 1);
                        pointer = pointer.saturating_sub(1);
                    } else {
                        patch[pointer + 1].1 = best_eq2;
                    }
                }
            }
        }
        pointer += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util_inner::diff::str::diff;

    #[test]
    fn cleanup_patch_basic() {
        let mut p = diff("the cat sat on the mat", "the cat sat on the bat");
        cleanup_patch(&mut p);
        let src: String = p
            .iter()
            .filter(|(t, _)| *t != PatchOpType::Ins)
            .map(|(_, s)| s.as_str())
            .collect();
        let dst: String = p
            .iter()
            .filter(|(t, _)| *t != PatchOpType::Del)
            .map(|(_, s)| s.as_str())
            .collect();
        assert_eq!(src, "the cat sat on the mat");
        assert_eq!(dst, "the cat sat on the bat");
    }

    // ── Helper to verify patch integrity ────────────────────────────────

    fn assert_patch_reconstructs(src: &str, dst: &str, patch: &Patch) {
        let reconstructed_src: String = patch
            .iter()
            .filter(|(t, _)| *t != PatchOpType::Ins)
            .map(|(_, s)| s.as_str())
            .collect();
        let reconstructed_dst: String = patch
            .iter()
            .filter(|(t, _)| *t != PatchOpType::Del)
            .map(|(_, s)| s.as_str())
            .collect();
        assert_eq!(reconstructed_src, src, "source reconstruction failed");
        assert_eq!(reconstructed_dst, dst, "destination reconstruction failed");
    }

    // ── semantic_score ──────────────────────────────────────────────────

    #[test]
    fn semantic_score_both_empty() {
        assert_eq!(semantic_score("", ""), 6);
    }

    #[test]
    fn semantic_score_one_empty() {
        assert_eq!(semantic_score("abc", ""), 6);
        assert_eq!(semantic_score("", "abc"), 6);
    }

    #[test]
    fn semantic_score_both_alphanumeric() {
        assert_eq!(semantic_score("abc", "def"), 0);
    }

    #[test]
    fn semantic_score_non_alphanumeric_boundary() {
        assert_eq!(semantic_score("abc.", "def"), 1);
        assert_eq!(semantic_score("abc", ".def"), 1);
    }

    #[test]
    fn semantic_score_whitespace_boundary() {
        assert_eq!(semantic_score("abc ", "def"), 2);
        assert_eq!(semantic_score("abc", " def"), 2);
    }

    #[test]
    fn semantic_score_punctuation_then_whitespace() {
        // non_alnum1 && !ws1 && ws2
        assert_eq!(semantic_score("abc.", " def"), 3);
    }

    #[test]
    fn semantic_score_line_break() {
        assert_eq!(semantic_score("abc\n", "def"), 4);
        assert_eq!(semantic_score("abc", "\ndef"), 4);
    }

    #[test]
    fn semantic_score_blank_line() {
        assert_eq!(semantic_score("abc\n\n", "def"), 5);
        assert_eq!(semantic_score("abc", "\n\ndef"), 5);
    }

    #[test]
    fn semantic_score_crlf_blank_line() {
        assert_eq!(semantic_score("abc", "\r\n\r\ndef"), 5);
    }

    // ── cleanup_patch with identical strings ────────────────────────────

    #[test]
    fn cleanup_patch_identical() {
        let mut p = diff("hello", "hello");
        cleanup_patch(&mut p);
        // Should just be one Eql
        assert!(p.iter().all(|(t, _)| *t == PatchOpType::Eql));
        assert_patch_reconstructs("hello", "hello", &p);
    }

    // ── cleanup_patch with empty strings ────────────────────────────────

    #[test]
    fn cleanup_patch_empty_to_nonempty() {
        let mut p = diff("", "hello");
        cleanup_patch(&mut p);
        assert_patch_reconstructs("", "hello", &p);
    }

    #[test]
    fn cleanup_patch_nonempty_to_empty() {
        let mut p = diff("hello", "");
        cleanup_patch(&mut p);
        assert_patch_reconstructs("hello", "", &p);
    }

    // ── cleanup_patch overlap detection ─────────────────────────────────

    #[test]
    fn cleanup_patch_with_overlap() {
        let mut p = diff("abcdef", "abxyzef");
        cleanup_patch(&mut p);
        assert_patch_reconstructs("abcdef", "abxyzef", &p);
    }

    #[test]
    fn cleanup_patch_word_boundary() {
        let mut p = diff("The quick brown fox", "The slow brown fox");
        cleanup_patch(&mut p);
        assert_patch_reconstructs("The quick brown fox", "The slow brown fox", &p);
    }

    #[test]
    fn cleanup_patch_multiple_changes() {
        let mut p = diff("alpha beta gamma", "alpha delta gamma");
        cleanup_patch(&mut p);
        assert_patch_reconstructs("alpha beta gamma", "alpha delta gamma", &p);
    }

    // ── cleanup_semantic_lossless ───────────────────────────────────────

    #[test]
    fn cleanup_semantic_lossless_shifts_to_word_boundary() {
        // Manually create a patch: Eql("The "), Del("c"), Eql("at sat")
        // After lossless cleanup it should shift to a better boundary
        let mut p: Patch = vec![
            (PatchOpType::Eql, "The ".to_string()),
            (PatchOpType::Del, "c".to_string()),
            (PatchOpType::Eql, "at sat".to_string()),
        ];
        cleanup_semantic_lossless(&mut p);
        // The patch should still reconstruct correctly
        let src: String = p
            .iter()
            .filter(|(t, _)| *t != PatchOpType::Ins)
            .map(|(_, s)| s.as_str())
            .collect();
        assert_eq!(src, "The cat sat");
    }

    #[test]
    fn cleanup_semantic_lossless_no_equalities() {
        let mut p: Patch = vec![
            (PatchOpType::Del, "abc".to_string()),
            (PatchOpType::Ins, "xyz".to_string()),
        ];
        cleanup_semantic_lossless(&mut p);
        // No equalities to shift around, patch stays the same
        assert_eq!(p.len(), 2);
    }

    #[test]
    fn cleanup_semantic_lossless_empty_equality_removed() {
        // When lossless cleanup shifts text, empty equalities should be removed
        let mut p: Patch = vec![
            (PatchOpType::Eql, "a".to_string()),
            (PatchOpType::Ins, "b".to_string()),
            (PatchOpType::Eql, "cde".to_string()),
        ];
        cleanup_semantic_lossless(&mut p);
        // Should still be valid
        let dst: String = p
            .iter()
            .filter(|(t, _)| *t != PatchOpType::Del)
            .map(|(_, s)| s.as_str())
            .collect();
        assert_eq!(dst, "abcde");
    }

    // ── cleanup_patch with line-oriented content ────────────────────────

    #[test]
    fn cleanup_patch_line_changes() {
        let src = "line1\nline2\nline3\n";
        let dst = "line1\nmodified\nline3\n";
        let mut p = diff(src, dst);
        cleanup_patch(&mut p);
        assert_patch_reconstructs(src, dst, &p);
    }

    // ── cleanup_patch with large overlap ────────────────────────────────

    #[test]
    fn cleanup_patch_large_overlap_insertion() {
        // Tests the ov1 >= ov2 branch with significant overlap
        let mut p: Patch = vec![
            (PatchOpType::Del, "abcxyz".to_string()),
            (PatchOpType::Ins, "xyzdef".to_string()),
        ];
        cleanup_patch(&mut p);
        let src: String = p
            .iter()
            .filter(|(t, _)| *t != PatchOpType::Ins)
            .map(|(_, s)| s.as_str())
            .collect();
        let dst: String = p
            .iter()
            .filter(|(t, _)| *t != PatchOpType::Del)
            .map(|(_, s)| s.as_str())
            .collect();
        assert_eq!(src, "abcxyz");
        assert_eq!(dst, "xyzdef");
    }

    #[test]
    fn cleanup_patch_large_overlap_reverse() {
        // Tests the ov2 > ov1 branch
        let mut p: Patch = vec![
            (PatchOpType::Del, "xyzabc".to_string()),
            (PatchOpType::Ins, "defxyz".to_string()),
        ];
        cleanup_patch(&mut p);
        let src: String = p
            .iter()
            .filter(|(t, _)| *t != PatchOpType::Ins)
            .map(|(_, s)| s.as_str())
            .collect();
        let dst: String = p
            .iter()
            .filter(|(t, _)| *t != PatchOpType::Del)
            .map(|(_, s)| s.as_str())
            .collect();
        assert_eq!(src, "xyzabc");
        assert_eq!(dst, "defxyz");
    }

    // ── cleanup_patch redundant equality elimination ────────────────────

    #[test]
    fn cleanup_patch_eliminates_small_equalities() {
        // Small equality surrounded by larger diffs should be eliminated
        let mut p: Patch = vec![
            (PatchOpType::Del, "aaaa".to_string()),
            (PatchOpType::Eql, "x".to_string()),
            (PatchOpType::Del, "bbbb".to_string()),
        ];
        cleanup_patch(&mut p);
        let src: String = p
            .iter()
            .filter(|(t, _)| *t != PatchOpType::Ins)
            .map(|(_, s)| s.as_str())
            .collect();
        assert_eq!(src, "aaaaxbbbb");
    }

    // ── cleanup_patch recursion ─────────────────────────────────────────

    #[test]
    fn cleanup_patch_handles_recursion() {
        // A case that triggers the recursive cleanup path
        let src = "aXbXcXd";
        let dst = "a1b2c3d";
        let mut p = diff(src, dst);
        cleanup_patch(&mut p);
        assert_patch_reconstructs(src, dst, &p);
    }

    // ── cleanup_patch unicode ───────────────────────────────────────────

    #[test]
    fn cleanup_patch_unicode() {
        let src = "héllo wörld";
        let dst = "héllo wörld!";
        let mut p = diff(src, dst);
        cleanup_patch(&mut p);
        assert_patch_reconstructs(src, dst, &p);
    }

    #[test]
    fn cleanup_patch_empty_both() {
        let mut p = diff("", "");
        cleanup_patch(&mut p);
        assert!(p.is_empty() || p.iter().all(|(_, s)| s.is_empty()));
    }
}
