#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchOpType {
    Del = -1,
    Eql = 0,
    Ins = 1,
}

pub type PatchOperation = (PatchOpType, String);
pub type Patch = Vec<PatchOperation>;

pub fn normalize(patch: Patch) -> Patch {
    if patch.len() < 2 {
        return patch;
    }
    let mut out: Patch = Vec::with_capacity(patch.len());
    for (t, s) in patch {
        if s.is_empty() {
            continue;
        }
        if let Some((lt, ls)) = out.last_mut() {
            if *lt == t {
                ls.push_str(&s);
                continue;
            }
        }
        out.push((t, s));
    }
    out
}

pub fn pfx(a: &str, b: &str) -> usize {
    let mut count = 0usize;
    let mut ia = a.chars();
    let mut ib = b.chars();
    loop {
        match (ia.next(), ib.next()) {
            (Some(ca), Some(cb)) if ca == cb => count += ca.len_utf8(),
            _ => break,
        }
    }
    count
}

pub fn sfx(a: &str, b: &str) -> usize {
    let ac: Vec<char> = a.chars().collect();
    let bc: Vec<char> = b.chars().collect();
    let mut i = 0usize;
    while i < ac.len() && i < bc.len() && ac[ac.len() - 1 - i] == bc[bc.len() - 1 - i] {
        i += 1;
    }
    ac[ac.len().saturating_sub(i)..].iter().map(|c| c.len_utf8()).sum()
}

pub fn overlap(mut a: &str, mut b: &str) -> usize {
    if a.is_empty() || b.is_empty() {
        return 0;
    }
    if a.len() > b.len() {
        a = &a[a.len() - b.len()..];
    } else if a.len() < b.len() {
        b = &b[..a.len()];
    }
    if a == b {
        return a.len();
    }
    let mut best = 0usize;
    let mut length = 1usize;
    loop {
        let start = a.len().saturating_sub(length);
        let pat = &a[start..];
        if let Some(found) = b.find(pat) {
            length += found;
            let start2 = a.len().saturating_sub(length);
            if found == 0 || a[start2..] == b[..length] {
                best = length;
                length += 1;
                continue;
            }
            continue;
        }
        return best;
    }
}

pub fn diff(src: &str, dst: &str) -> Patch {
    if src == dst {
        return if src.is_empty() {
            vec![]
        } else {
            vec![(PatchOpType::Eql, src.to_string())]
        };
    }
    let prefix = pfx(src, dst);
    let src_rest = &src[prefix..];
    let dst_rest = &dst[prefix..];
    let suffix = sfx(src_rest, dst_rest);
    let src_mid = &src_rest[..src_rest.len().saturating_sub(suffix)];
    let dst_mid = &dst_rest[..dst_rest.len().saturating_sub(suffix)];
    let mut out = Vec::new();
    if prefix > 0 {
        out.push((PatchOpType::Eql, src[..prefix].to_string()));
    }
    if !src_mid.is_empty() {
        out.push((PatchOpType::Del, src_mid.to_string()));
    }
    if !dst_mid.is_empty() {
        out.push((PatchOpType::Ins, dst_mid.to_string()));
    }
    if suffix > 0 {
        out.push((PatchOpType::Eql, src[src.len() - suffix..].to_string()));
    }
    normalize(out)
}

pub fn diff_edit(src: &str, dst: &str, _caret: isize) -> Patch {
    diff(src, dst)
}

pub fn src(patch: &Patch) -> String {
    let mut out = String::new();
    for (t, s) in patch {
        if *t != PatchOpType::Ins {
            out.push_str(s);
        }
    }
    out
}

pub fn dst(patch: &Patch) -> String {
    let mut out = String::new();
    for (t, s) in patch {
        if *t != PatchOpType::Del {
            out.push_str(s);
        }
    }
    out
}

pub fn invert(patch: &Patch) -> Patch {
    patch
        .iter()
        .map(|(t, s)| match t {
            PatchOpType::Eql => (PatchOpType::Eql, s.clone()),
            PatchOpType::Ins => (PatchOpType::Del, s.clone()),
            PatchOpType::Del => (PatchOpType::Ins, s.clone()),
        })
        .collect()
}

pub fn apply<FIns, FDel>(patch: &Patch, src_len: usize, mut on_insert: FIns, mut on_delete: FDel)
where
    FIns: FnMut(usize, &str),
    FDel: FnMut(usize, usize, &str),
{
    let mut pos = src_len;
    for (t, s) in patch.iter().rev() {
        match t {
            PatchOpType::Eql => pos = pos.saturating_sub(s.len()),
            PatchOpType::Ins => on_insert(pos, s),
            PatchOpType::Del => {
                let len = s.len();
                pos = pos.saturating_sub(len);
                on_delete(pos, len, s);
            }
        }
    }
}
