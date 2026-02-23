//! All 16 JSON CRDT Patch operations as a single Rust enum.
//!
//! Mirrors the 16 operation classes in
//! `packages/json-joy/src/json-crdt-patch/operations.ts`.

use crate::json_crdt_patch::clock::{print_ts, Ts, Tss};
use json_joy_json_pack::PackValue;

// ── ConValue ───────────────────────────────────────────────────────────────

/// The value stored in a `new_con` operation.
///
/// Upstream type is `unknown | undefined | ITimestampStruct`:
/// - A `Timestamp` → reference to another CRDT node.
/// - Anything else → a constant JSON-compatible value encoded as CBOR.
#[derive(Debug, Clone, PartialEq)]
pub enum ConValue {
    /// A timestamp reference to another CRDT node.
    Ref(Ts),
    /// A constant value (null, bool, number, string, binary, array, object, undefined).
    Val(PackValue),
}

// ── Operation ──────────────────────────────────────────────────────────────

/// A single JSON CRDT Patch operation.
///
/// Each variant carries an `id: Ts` identifying the operation in the
/// global logical clock space.
///
/// Span (the number of clock ticks consumed):
/// - Most operations consume 1 tick.
/// - `InsStr` consumes UTF-16 code-unit length (upstream `string.length`).
/// - `InsBin`, `InsArr` consume `data.len()` ticks.
/// - `Nop` consumes `len` ticks.
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    // ── Creation operations ──────────────────────────────────────────────
    /// Create a new constant `con` value.
    NewCon { id: Ts, val: ConValue },
    /// Create a new LWW-Register `val` object.
    NewVal { id: Ts },
    /// Create a new LWW-Map `obj` object.
    NewObj { id: Ts },
    /// Create a new LWW-Vector `vec` object.
    NewVec { id: Ts },
    /// Create a new RGA-String `str` object.
    NewStr { id: Ts },
    /// Create a new RGA-Binary `bin` object.
    NewBin { id: Ts },
    /// Create a new RGA-Array `arr` object.
    NewArr { id: Ts },

    // ── Mutation operations ──────────────────────────────────────────────
    /// Set the value of a `val` register.
    InsVal { id: Ts, obj: Ts, val: Ts },
    /// Set key→value pairs in an `obj` map.
    InsObj {
        id: Ts,
        obj: Ts,
        data: Vec<(String, Ts)>,
    },
    /// Set index→value pairs in a `vec` vector.
    InsVec {
        id: Ts,
        obj: Ts,
        data: Vec<(u8, Ts)>,
    },
    /// Insert a string into a `str` RGA.
    InsStr {
        id: Ts,
        obj: Ts,
        after: Ts,
        data: String,
    },
    /// Insert binary data into a `bin` RGA.
    InsBin {
        id: Ts,
        obj: Ts,
        after: Ts,
        data: Vec<u8>,
    },
    /// Insert elements into an `arr` RGA.
    InsArr {
        id: Ts,
        obj: Ts,
        after: Ts,
        data: Vec<Ts>,
    },
    /// Update an existing element in an `arr` array.
    UpdArr { id: Ts, obj: Ts, after: Ts, val: Ts },
    /// Delete ranges of operations in an object (str/bin/arr).
    Del { id: Ts, obj: Ts, what: Vec<Tss> },
    /// No-op — skips clock cycles without performing any CRDT action.
    Nop { id: Ts, len: u64 },
}

impl Op {
    /// Returns the ID (first timestamp) of this operation.
    pub fn id(&self) -> Ts {
        match self {
            Op::NewCon { id, .. }
            | Op::NewVal { id }
            | Op::NewObj { id }
            | Op::NewVec { id }
            | Op::NewStr { id }
            | Op::NewBin { id }
            | Op::NewArr { id }
            | Op::InsVal { id, .. }
            | Op::InsObj { id, .. }
            | Op::InsVec { id, .. }
            | Op::InsStr { id, .. }
            | Op::InsBin { id, .. }
            | Op::InsArr { id, .. }
            | Op::UpdArr { id, .. }
            | Op::Del { id, .. }
            | Op::Nop { id, .. } => *id,
        }
    }

    /// Number of logical clock cycles consumed by this operation.
    pub fn span(&self) -> u64 {
        match self {
            Op::InsStr { data, .. } => data.encode_utf16().count() as u64,
            Op::InsBin { data, .. } => data.len() as u64,
            Op::InsArr { data, .. } => data.len() as u64,
            Op::Nop { len, .. } => *len,
            _ => 1,
        }
    }

    /// Short mnemonic name of this operation (used in verbose JSON codec).
    pub fn name(&self) -> &'static str {
        match self {
            Op::NewCon { .. } => "new_con",
            Op::NewVal { .. } => "new_val",
            Op::NewObj { .. } => "new_obj",
            Op::NewVec { .. } => "new_vec",
            Op::NewStr { .. } => "new_str",
            Op::NewBin { .. } => "new_bin",
            Op::NewArr { .. } => "new_arr",
            Op::InsVal { .. } => "ins_val",
            Op::InsObj { .. } => "ins_obj",
            Op::InsVec { .. } => "ins_vec",
            Op::InsStr { .. } => "ins_str",
            Op::InsBin { .. } => "ins_bin",
            Op::InsArr { .. } => "ins_arr",
            Op::UpdArr { .. } => "upd_arr",
            Op::Del { .. } => "del",
            Op::Nop { .. } => "nop",
        }
    }
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = self.id();
        let span = self.span();
        let base = if span > 1 {
            format!("{} {}!{}", self.name(), print_ts(id), span)
        } else {
            format!("{} {}", self.name(), print_ts(id))
        };
        match self {
            Op::InsVal { obj, val, .. } => write!(
                f,
                "{}, obj = {}, val = {}",
                base,
                print_ts(*obj),
                print_ts(*val)
            ),
            Op::InsStr {
                obj, after, data, ..
            } => write!(
                f,
                "{}, obj = {} {{ {} ← {:?} }}",
                base,
                print_ts(*obj),
                print_ts(*after),
                data
            ),
            Op::InsBin {
                obj, after, data, ..
            } => write!(
                f,
                "{}, obj = {} {{ {} ← {:?} }}",
                base,
                print_ts(*obj),
                print_ts(*after),
                data
            ),
            Op::Del { obj, what, .. } => {
                let spans: Vec<_> = what
                    .iter()
                    .map(|s| format!("{}!{}", print_ts(s.ts()), s.span))
                    .collect();
                write!(
                    f,
                    "{}, obj = {} {{ {} }}",
                    base,
                    print_ts(*obj),
                    spans.join(", ")
                )
            }
            _ => write!(f, "{}", base),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json_crdt_patch::clock::ts;

    #[test]
    fn span_of_nop() {
        let op = Op::Nop {
            id: ts(1, 0),
            len: 5,
        };
        assert_eq!(op.span(), 5);
    }

    #[test]
    fn span_of_ins_str() {
        let op = Op::InsStr {
            id: ts(1, 0),
            obj: ts(1, 0),
            after: ts(1, 0),
            data: "hello".into(),
        };
        assert_eq!(op.span(), 5);
    }

    #[test]
    fn span_of_creation_op() {
        let op = Op::NewObj { id: ts(1, 0) };
        assert_eq!(op.span(), 1);
    }

    #[test]
    fn op_name() {
        assert_eq!(
            Op::NewCon {
                id: ts(1, 0),
                val: ConValue::Val(PackValue::Null)
            }
            .name(),
            "new_con"
        );
        assert_eq!(
            Op::Del {
                id: ts(1, 0),
                obj: ts(1, 0),
                what: vec![]
            }
            .name(),
            "del"
        );
    }

    #[test]
    fn op_name_all_variants() {
        assert_eq!(Op::NewVal { id: ts(1, 0) }.name(), "new_val");
        assert_eq!(Op::NewObj { id: ts(1, 0) }.name(), "new_obj");
        assert_eq!(Op::NewVec { id: ts(1, 0) }.name(), "new_vec");
        assert_eq!(Op::NewStr { id: ts(1, 0) }.name(), "new_str");
        assert_eq!(Op::NewBin { id: ts(1, 0) }.name(), "new_bin");
        assert_eq!(Op::NewArr { id: ts(1, 0) }.name(), "new_arr");
        assert_eq!(
            Op::InsVal {
                id: ts(1, 0),
                obj: ts(1, 0),
                val: ts(1, 0)
            }
            .name(),
            "ins_val"
        );
        assert_eq!(
            Op::InsObj {
                id: ts(1, 0),
                obj: ts(1, 0),
                data: vec![]
            }
            .name(),
            "ins_obj"
        );
        assert_eq!(
            Op::InsVec {
                id: ts(1, 0),
                obj: ts(1, 0),
                data: vec![]
            }
            .name(),
            "ins_vec"
        );
        assert_eq!(
            Op::InsStr {
                id: ts(1, 0),
                obj: ts(1, 0),
                after: ts(1, 0),
                data: String::new()
            }
            .name(),
            "ins_str"
        );
        assert_eq!(
            Op::InsBin {
                id: ts(1, 0),
                obj: ts(1, 0),
                after: ts(1, 0),
                data: vec![]
            }
            .name(),
            "ins_bin"
        );
        assert_eq!(
            Op::InsArr {
                id: ts(1, 0),
                obj: ts(1, 0),
                after: ts(1, 0),
                data: vec![]
            }
            .name(),
            "ins_arr"
        );
        assert_eq!(
            Op::UpdArr {
                id: ts(1, 0),
                obj: ts(1, 0),
                after: ts(1, 0),
                val: ts(1, 0)
            }
            .name(),
            "upd_arr"
        );
        assert_eq!(
            Op::Nop {
                id: ts(1, 0),
                len: 1
            }
            .name(),
            "nop"
        );
    }

    #[test]
    fn id_returns_correct_ts() {
        let op = Op::InsVal {
            id: ts(5, 42),
            obj: ts(1, 0),
            val: ts(1, 1),
        };
        assert_eq!(op.id(), ts(5, 42));
    }

    #[test]
    fn span_of_ins_bin() {
        let op = Op::InsBin {
            id: ts(1, 0),
            obj: ts(1, 0),
            after: ts(1, 0),
            data: vec![1, 2, 3, 4],
        };
        assert_eq!(op.span(), 4);
    }

    #[test]
    fn span_of_ins_arr() {
        let op = Op::InsArr {
            id: ts(1, 0),
            obj: ts(1, 0),
            after: ts(1, 0),
            data: vec![ts(1, 1), ts(1, 2)],
        };
        assert_eq!(op.span(), 2);
    }

    #[test]
    fn span_of_ins_str_utf16_surrogate() {
        // Emoji uses 2 UTF-16 code units
        let op = Op::InsStr {
            id: ts(1, 0),
            obj: ts(1, 0),
            after: ts(1, 0),
            data: "a😀".into(),
        };
        // 'a' = 1 + emoji = 2 → 3
        assert_eq!(op.span(), 3);
    }

    #[test]
    fn display_new_obj() {
        let op = Op::NewObj { id: ts(1, 5) };
        let s = format!("{}", op);
        assert!(s.contains("new_obj"));
    }

    #[test]
    fn display_ins_val() {
        let op = Op::InsVal {
            id: ts(1, 5),
            obj: ts(1, 0),
            val: ts(1, 3),
        };
        let s = format!("{}", op);
        assert!(s.contains("ins_val"));
        assert!(s.contains("obj"));
        assert!(s.contains("val"));
    }

    #[test]
    fn display_ins_str() {
        let op = Op::InsStr {
            id: ts(1, 5),
            obj: ts(1, 0),
            after: ts(1, 0),
            data: "hi".into(),
        };
        let s = format!("{}", op);
        assert!(s.contains("ins_str"));
        assert!(s.contains("hi"));
    }

    #[test]
    fn display_ins_bin() {
        let op = Op::InsBin {
            id: ts(1, 5),
            obj: ts(1, 0),
            after: ts(1, 0),
            data: vec![0xAB],
        };
        let s = format!("{}", op);
        assert!(s.contains("ins_bin"));
    }

    #[test]
    fn display_del() {
        use crate::json_crdt_patch::clock::Tss;
        let op = Op::Del {
            id: ts(1, 5),
            obj: ts(1, 0),
            what: vec![Tss::new(1, 2, 3)],
        };
        let s = format!("{}", op);
        assert!(s.contains("del"));
    }

    #[test]
    fn display_nop_with_span() {
        let op = Op::Nop {
            id: ts(1, 5),
            len: 10,
        };
        let s = format!("{}", op);
        assert!(s.contains("nop"));
        assert!(s.contains("!10"));
    }

    #[test]
    fn con_value_equality() {
        let a = ConValue::Val(PackValue::Integer(42));
        let b = ConValue::Val(PackValue::Integer(42));
        assert_eq!(a, b);

        let c = ConValue::Ref(ts(1, 5));
        let d = ConValue::Ref(ts(1, 5));
        assert_eq!(c, d);

        assert_ne!(a, c);
    }

    #[test]
    fn con_value_ref_different_ts() {
        let a = ConValue::Ref(ts(1, 5));
        let b = ConValue::Ref(ts(1, 6));
        assert_ne!(a, b);
    }

    #[test]
    fn con_value_val_different_types() {
        let a = ConValue::Val(PackValue::Integer(1));
        let b = ConValue::Val(PackValue::Str("hello".into()));
        assert_ne!(a, b);
    }

    #[test]
    fn con_value_val_null() {
        let a = ConValue::Val(PackValue::Null);
        let b = ConValue::Val(PackValue::Null);
        assert_eq!(a, b);
    }

    #[test]
    fn con_value_val_bool() {
        assert_eq!(
            ConValue::Val(PackValue::Bool(true)),
            ConValue::Val(PackValue::Bool(true))
        );
        assert_ne!(
            ConValue::Val(PackValue::Bool(true)),
            ConValue::Val(PackValue::Bool(false))
        );
    }

    #[test]
    fn id_of_all_creation_ops() {
        let cases: Vec<Op> = vec![
            Op::NewCon {
                id: ts(1, 10),
                val: ConValue::Val(PackValue::Null),
            },
            Op::NewVal { id: ts(1, 10) },
            Op::NewObj { id: ts(1, 10) },
            Op::NewVec { id: ts(1, 10) },
            Op::NewStr { id: ts(1, 10) },
            Op::NewBin { id: ts(1, 10) },
            Op::NewArr { id: ts(1, 10) },
        ];
        for op in &cases {
            assert_eq!(op.id(), ts(1, 10), "id mismatch for {}", op.name());
        }
    }

    #[test]
    fn id_of_all_mutation_ops() {
        let cases: Vec<Op> = vec![
            Op::InsVal {
                id: ts(2, 7),
                obj: ts(1, 0),
                val: ts(1, 1),
            },
            Op::InsObj {
                id: ts(2, 7),
                obj: ts(1, 0),
                data: vec![],
            },
            Op::InsVec {
                id: ts(2, 7),
                obj: ts(1, 0),
                data: vec![],
            },
            Op::InsStr {
                id: ts(2, 7),
                obj: ts(1, 0),
                after: ts(1, 0),
                data: String::new(),
            },
            Op::InsBin {
                id: ts(2, 7),
                obj: ts(1, 0),
                after: ts(1, 0),
                data: vec![],
            },
            Op::InsArr {
                id: ts(2, 7),
                obj: ts(1, 0),
                after: ts(1, 0),
                data: vec![],
            },
            Op::UpdArr {
                id: ts(2, 7),
                obj: ts(1, 0),
                after: ts(1, 0),
                val: ts(1, 1),
            },
            Op::Del {
                id: ts(2, 7),
                obj: ts(1, 0),
                what: vec![],
            },
            Op::Nop {
                id: ts(2, 7),
                len: 1,
            },
        ];
        for op in &cases {
            assert_eq!(op.id(), ts(2, 7), "id mismatch for {}", op.name());
        }
    }

    #[test]
    fn span_of_empty_ins_str() {
        let op = Op::InsStr {
            id: ts(1, 0),
            obj: ts(1, 0),
            after: ts(1, 0),
            data: String::new(),
        };
        assert_eq!(op.span(), 0);
    }

    #[test]
    fn span_of_empty_ins_bin() {
        let op = Op::InsBin {
            id: ts(1, 0),
            obj: ts(1, 0),
            after: ts(1, 0),
            data: vec![],
        };
        assert_eq!(op.span(), 0);
    }

    #[test]
    fn span_of_empty_ins_arr() {
        let op = Op::InsArr {
            id: ts(1, 0),
            obj: ts(1, 0),
            after: ts(1, 0),
            data: vec![],
        };
        assert_eq!(op.span(), 0);
    }

    #[test]
    fn display_ins_obj() {
        let op = Op::InsObj {
            id: ts(1, 5),
            obj: ts(1, 0),
            data: vec![("key".into(), ts(1, 3))],
        };
        let s = format!("{}", op);
        assert!(s.contains("ins_obj"));
    }

    #[test]
    fn display_ins_vec() {
        let op = Op::InsVec {
            id: ts(1, 5),
            obj: ts(1, 0),
            data: vec![(0, ts(1, 3))],
        };
        let s = format!("{}", op);
        assert!(s.contains("ins_vec"));
    }

    #[test]
    fn display_ins_arr() {
        let op = Op::InsArr {
            id: ts(1, 5),
            obj: ts(1, 0),
            after: ts(1, 0),
            data: vec![ts(1, 3)],
        };
        let s = format!("{}", op);
        assert!(s.contains("ins_arr"));
    }

    #[test]
    fn display_upd_arr() {
        let op = Op::UpdArr {
            id: ts(1, 5),
            obj: ts(1, 0),
            after: ts(1, 3),
            val: ts(1, 4),
        };
        let s = format!("{}", op);
        assert!(s.contains("upd_arr"));
    }

    #[test]
    fn op_clone() {
        let op = Op::InsStr {
            id: ts(1, 0),
            obj: ts(1, 0),
            after: ts(1, 0),
            data: "hello".into(),
        };
        let cloned = op.clone();
        assert_eq!(op, cloned);
    }

    #[test]
    fn op_debug() {
        let op = Op::NewCon {
            id: ts(1, 0),
            val: ConValue::Val(PackValue::Null),
        };
        let debug = format!("{:?}", op);
        assert!(debug.contains("NewCon"));
    }

    #[test]
    fn con_value_ref_clone() {
        let val = ConValue::Ref(ts(5, 10));
        let cloned = val.clone();
        assert_eq!(val, cloned);
    }
}
