//! Binary diff — wraps the string diff algorithm over byte arrays.
//!
//! Mirrors `packages/json-joy/src/util/diff/bin.ts`.

use super::str::{self, Patch, PatchOpType};

/// Encode a byte slice as a string with one char per byte (code points 0–255).
pub fn to_str(buf: &[u8]) -> String {
    buf.iter().map(|&b| b as char).collect()
}

/// Decode a string (one char per byte) back to bytes.
pub fn to_bin(s: &str) -> Vec<u8> {
    s.chars().map(|c| c as u8).collect()
}

/// Compute a diff between two byte slices.
pub fn diff(src: &[u8], dst: &[u8]) -> Patch {
    str::diff(&to_str(src), &to_str(dst))
}

/// Apply a binary patch, calling callbacks for insertions and deletions.
pub fn apply<FIns, FDel>(patch: &Patch, src_len: usize, mut on_insert: FIns, mut on_delete: FDel)
where
    FIns: FnMut(usize, Vec<u8>),
    FDel: FnMut(usize, usize),
{
    str::apply(
        patch,
        src_len,
        |pos, s| on_insert(pos, to_bin(s)),
        |pos, len, _| on_delete(pos, len),
    );
}

/// Reconstruct the source bytes from a binary patch.
pub fn patch_src(patch: &Patch) -> Vec<u8> {
    to_bin(&str::patch_src(patch))
}

/// Reconstruct the destination bytes from a binary patch.
pub fn patch_dst(patch: &Patch) -> Vec<u8> {
    to_bin(&str::patch_dst(patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_to_str_to_bin() {
        let original = vec![0u8, 1, 127, 200, 255];
        assert_eq!(to_bin(&to_str(&original)), original);
    }

    #[test]
    fn diff_and_reconstruct() {
        let src = b"hello world";
        let dst = b"hello rust";
        let patch = diff(src, dst);
        assert_eq!(patch_src(&patch), src.to_vec());
        assert_eq!(patch_dst(&patch), dst.to_vec());
    }

    #[test]
    fn binary_diff_operations() {
        let src = b"abcdef";
        let dst = b"abXdef";
        let patch = diff(src, dst);
        let mut del_count = 0usize;
        let mut ins_count = 0usize;
        for (op_type, txt) in &patch {
            match op_type {
                PatchOpType::Del => del_count += txt.chars().count(),
                PatchOpType::Ins => ins_count += txt.chars().count(),
                _ => {}
            }
        }
        assert_eq!(del_count, 1); // 'c'
        assert_eq!(ins_count, 1); // 'X'
    }
}
