fn rewrite_op<F>(op: &DecodedOp, map: &mut F) -> DecodedOp
where
    F: FnMut(Timestamp) -> Timestamp,
{
    match op {
        DecodedOp::NewCon { id, value } => DecodedOp::NewCon {
            id: map(*id),
            value: match value {
                ConValue::Json(v) => ConValue::Json(v.clone()),
                ConValue::Ref(ts) => ConValue::Ref(map(*ts)),
                ConValue::Undef => ConValue::Undef,
            },
        },
        DecodedOp::NewVal { id } => DecodedOp::NewVal { id: map(*id) },
        DecodedOp::NewObj { id } => DecodedOp::NewObj { id: map(*id) },
        DecodedOp::NewVec { id } => DecodedOp::NewVec { id: map(*id) },
        DecodedOp::NewStr { id } => DecodedOp::NewStr { id: map(*id) },
        DecodedOp::NewBin { id } => DecodedOp::NewBin { id: map(*id) },
        DecodedOp::NewArr { id } => DecodedOp::NewArr { id: map(*id) },
        DecodedOp::InsVal { id, obj, val } => DecodedOp::InsVal {
            id: map(*id),
            obj: map(*obj),
            val: map(*val),
        },
        DecodedOp::InsObj { id, obj, data } => DecodedOp::InsObj {
            id: map(*id),
            obj: map(*obj),
            data: data.iter().map(|(k, v)| (k.clone(), map(*v))).collect(),
        },
        DecodedOp::InsVec { id, obj, data } => DecodedOp::InsVec {
            id: map(*id),
            obj: map(*obj),
            data: data.iter().map(|(k, v)| (*k, map(*v))).collect(),
        },
        DecodedOp::InsStr {
            id,
            obj,
            reference,
            data,
        } => DecodedOp::InsStr {
            id: map(*id),
            obj: map(*obj),
            reference: map(*reference),
            data: data.clone(),
        },
        DecodedOp::InsBin {
            id,
            obj,
            reference,
            data,
        } => DecodedOp::InsBin {
            id: map(*id),
            obj: map(*obj),
            reference: map(*reference),
            data: data.clone(),
        },
        DecodedOp::InsArr {
            id,
            obj,
            reference,
            data,
        } => DecodedOp::InsArr {
            id: map(*id),
            obj: map(*obj),
            reference: map(*reference),
            data: data.iter().map(|t| map(*t)).collect(),
        },
        DecodedOp::UpdArr {
            id,
            obj,
            reference,
            val,
        } => DecodedOp::UpdArr {
            id: map(*id),
            obj: map(*obj),
            reference: map(*reference),
            val: map(*val),
        },
        DecodedOp::Del { id, obj, what } => DecodedOp::Del {
            id: map(*id),
            obj: map(*obj),
            what: what
                .iter()
                .map(|span| {
                    let ts = map(Timestamp {
                        sid: span.sid,
                        time: span.time,
                    });
                    Timespan {
                        sid: ts.sid,
                        time: ts.time,
                        span: span.span,
                    }
                })
                .collect(),
        },
        DecodedOp::Nop { id, len } => DecodedOp::Nop {
            id: map(*id),
            len: *len,
        },
    }
}
