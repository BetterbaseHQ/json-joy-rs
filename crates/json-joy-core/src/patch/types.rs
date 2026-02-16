#[derive(Debug, Error)]
pub enum PatchError {
    #[error("patch decode overflow")]
    Overflow,
    #[error("unknown patch opcode: {0}")]
    UnknownOpcode(u8),
    #[error("invalid cbor in patch")]
    InvalidCbor,
    #[error("trailing bytes in patch")]
    TrailingBytes,
}

#[derive(Debug, Error)]
pub enum PatchTransformError {
    #[error("empty patch")]
    EmptyPatch,
    #[error("patch timeline rewrite failed: {0}")]
    Build(#[from] crate::patch_builder::PatchBuildError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Timestamp {
    pub sid: u64,
    pub time: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Timespan {
    pub sid: u64,
    pub time: u64,
    pub span: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConValue {
    Json(serde_json::Value),
    Ref(Timestamp),
    Undef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodedOp {
    NewCon { id: Timestamp, value: ConValue },
    NewVal { id: Timestamp },
    NewObj { id: Timestamp },
    NewVec { id: Timestamp },
    NewStr { id: Timestamp },
    NewBin { id: Timestamp },
    NewArr { id: Timestamp },
    InsVal { id: Timestamp, obj: Timestamp, val: Timestamp },
    InsObj {
        id: Timestamp,
        obj: Timestamp,
        data: Vec<(String, Timestamp)>,
    },
    InsVec {
        id: Timestamp,
        obj: Timestamp,
        data: Vec<(u64, Timestamp)>,
    },
    InsStr {
        id: Timestamp,
        obj: Timestamp,
        reference: Timestamp,
        data: String,
    },
    InsBin {
        id: Timestamp,
        obj: Timestamp,
        reference: Timestamp,
        data: Vec<u8>,
    },
    InsArr {
        id: Timestamp,
        obj: Timestamp,
        reference: Timestamp,
        data: Vec<Timestamp>,
    },
    UpdArr {
        id: Timestamp,
        obj: Timestamp,
        reference: Timestamp,
        val: Timestamp,
    },
    Del {
        id: Timestamp,
        obj: Timestamp,
        what: Vec<Timespan>,
    },
    Nop { id: Timestamp, len: u64 },
}

impl DecodedOp {
    pub fn id(&self) -> Timestamp {
        match self {
            DecodedOp::NewCon { id, .. }
            | DecodedOp::NewVal { id }
            | DecodedOp::NewObj { id }
            | DecodedOp::NewVec { id }
            | DecodedOp::NewStr { id }
            | DecodedOp::NewBin { id }
            | DecodedOp::NewArr { id }
            | DecodedOp::InsVal { id, .. }
            | DecodedOp::InsObj { id, .. }
            | DecodedOp::InsVec { id, .. }
            | DecodedOp::InsStr { id, .. }
            | DecodedOp::InsBin { id, .. }
            | DecodedOp::InsArr { id, .. }
            | DecodedOp::UpdArr { id, .. }
            | DecodedOp::Del { id, .. }
            | DecodedOp::Nop { id, .. } => *id,
        }
    }

    pub fn span(&self) -> u64 {
        match self {
            DecodedOp::InsStr { data, .. } => data.chars().count() as u64,
            DecodedOp::InsBin { data, .. } => data.len() as u64,
            DecodedOp::InsArr { data, .. } => data.len() as u64,
            DecodedOp::Nop { len, .. } => *len,
            _ => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Patch {
    /// Original binary payload, preserved for exact wire round-trips.
    bytes: Vec<u8>,
    op_count: u64,
    span: u64,
    sid: u64,
    time: u64,
    opcodes: Vec<u8>,
    decoded_ops: Vec<DecodedOp>,
}

impl Patch {
    pub fn from_binary(data: &[u8]) -> Result<Self, PatchError> {
        let mut reader = Reader::new(data);
        let decoded = decode_patch(&mut reader);
        if let Err(err) = decoded {
            // json-joy's JS decoder is permissive for many malformed inputs.
            // This compatibility behavior is fixture-driven (see
            // tests/compat/fixtures/* and patch_codec_from_fixtures.rs).
            if matches!(err, PatchError::InvalidCbor) {
                // Fixture corpus currently shows ASCII JSON payload
                // (`0x7b` / '{') is rejected upstream.
                if data.first() == Some(&0x7b) {
                    return Err(err);
                }
            }
            return Ok(Self {
                bytes: data.to_vec(),
                op_count: 0,
                span: 0,
                sid: 0,
                time: 0,
                opcodes: Vec::new(),
                decoded_ops: Vec::new(),
            });
        }
        if !reader.is_eof() {
            return Ok(Self {
                bytes: data.to_vec(),
                op_count: 0,
                span: 0,
                sid: 0,
                time: 0,
                opcodes: Vec::new(),
                decoded_ops: Vec::new(),
            });
        }
        let (sid, time, op_count, span, opcodes, decoded_ops) = decoded.expect("checked above");
        Ok(Self {
            bytes: data.to_vec(),
            op_count,
            span,
            sid,
            time,
            opcodes,
            decoded_ops,
        })
    }

    pub fn to_binary(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    pub fn op_count(&self) -> u64 {
        self.op_count
    }

    pub fn span(&self) -> u64 {
        self.span
    }

    pub fn id(&self) -> Option<(u64, u64)> {
        if self.op_count == 0 {
            None
        } else {
            Some((self.sid, self.time))
        }
    }

    pub fn next_time(&self) -> u64 {
        if self.op_count == 0 {
            0
        } else {
            self.time.saturating_add(self.span)
        }
    }

    pub fn opcodes(&self) -> &[u8] {
        &self.opcodes
    }

    pub fn decoded_ops(&self) -> &[DecodedOp] {
        &self.decoded_ops
    }

    pub fn rewrite_time<F>(&self, mut map: F) -> Result<Self, PatchTransformError>
    where
        F: FnMut(Timestamp) -> Timestamp,
    {
        if self.op_count == 0 {
            return Err(PatchTransformError::EmptyPatch);
        }
        let mut ops = Vec::with_capacity(self.decoded_ops.len());
        for op in &self.decoded_ops {
            ops.push(rewrite_op(op, &mut map));
        }
        let first = ops.first().expect("checked non-empty patch");
        let first_id = first.id();
        let bytes = crate::patch_builder::encode_patch_from_ops(first_id.sid, first_id.time, &ops)?;
        Ok(Patch::from_binary(&bytes).expect("encoded patch must decode"))
    }

    pub fn rebase(
        &self,
        new_time: u64,
        transform_after: Option<u64>,
    ) -> Result<Self, PatchTransformError> {
        if self.op_count == 0 {
            return Err(PatchTransformError::EmptyPatch);
        }
        let patch_sid = self.sid;
        let patch_start = self.time;
        let horizon = transform_after.unwrap_or(patch_start);
        if patch_start == new_time {
            return Ok(self.clone());
        }
        let delta = new_time as i128 - patch_start as i128;
        self.rewrite_time(|id| {
            if id.sid != patch_sid || id.time < horizon {
                return id;
            }
            let next = (id.time as i128 + delta).max(0) as u64;
            Timestamp {
                sid: id.sid,
                time: next,
            }
        })
    }
}
