fn choose_sequence_insert_reference(
    slots: &[Timestamp],
    container: Timestamp,
    lcp: usize,
    ins_len: usize,
    del_len: usize,
    old_len: usize,
) -> Timestamp {
    // Upstream str/bin differ emits inserts while walking edits from the end.
    // For mixed replace windows this frequently anchors insertion on the last
    // deleted atom rather than the prefix atom.
    if ins_len > 0 && del_len > 0 {
        let idx = lcp.saturating_add(del_len).saturating_sub(1);
        if idx < slots.len() {
            return slots[idx];
        }
    }
    if lcp == 0 && del_len == 0 {
        return container;
    }
    if lcp > 0 && lcp - 1 < slots.len() {
        return slots[lcp - 1];
    }
    if old_len > 0 && del_len == old_len {
        return slots[old_len - 1];
    }
    slots.first().copied().unwrap_or(container)
}


fn try_emit_child_recursive_diff(
    runtime: &RuntimeModel,
    emitter: &mut NativeEmitter,
    child: Timestamp,
    old_opt: Option<&Value>,
    new_v: &Value,
) -> Result<bool, DiffError> {
    match old_opt {
        Some(Value::String(old)) if matches!(new_v, Value::String(_)) => {
            let child = match runtime
                .resolve_string_node(child)
                .or_else(|| runtime.find_string_node_by_value(old))
            {
                Some(id) => id,
                None => return Ok(false),
            };
            let new = match new_v {
                Value::String(v) => v,
                _ => unreachable!(),
            };
            let slots = match runtime.string_visible_slots(child) {
                Some(v) => v,
                None => return Ok(false),
            };
            let old_chars: Vec<char> = old.chars().collect();
            if old_chars.len() != slots.len() {
                return Ok(false);
            }
            let new_chars: Vec<char> = new.chars().collect();
            let mut lcp = 0usize;
            while lcp < old_chars.len() && lcp < new_chars.len() && old_chars[lcp] == new_chars[lcp] {
                lcp += 1;
            }
            let mut lcs = 0usize;
            while lcs < (old_chars.len() - lcp)
                && lcs < (new_chars.len() - lcp)
                && old_chars[old_chars.len() - 1 - lcs] == new_chars[new_chars.len() - 1 - lcs]
            {
                lcs += 1;
            }
            let del_len = old_chars.len().saturating_sub(lcp + lcs);
            let ins: String = new_chars[lcp..new_chars.len().saturating_sub(lcs)]
                .iter()
                .collect();
            if !ins.is_empty() {
                let reference = choose_sequence_insert_reference(
                    &slots,
                    child,
                    lcp,
                    ins.chars().count(),
                    del_len,
                    old_chars.len(),
                );
                emitter.push(DecodedOp::InsStr {
                    id: emitter.next_id(),
                    obj: child,
                    reference,
                    data: ins,
                });
            }
            if del_len > 0 {
                let del_slots = &slots[lcp..lcp + del_len];
                let mut spans: Vec<crate::patch::Timespan> = Vec::new();
                for slot in del_slots {
                    if let Some(last) = spans.last_mut() {
                        if last.sid == slot.sid && last.time + last.span == slot.time {
                            last.span += 1;
                            continue;
                        }
                    }
                    spans.push(crate::patch::Timespan {
                        sid: slot.sid,
                        time: slot.time,
                        span: 1,
                    });
                }
                emitter.push(DecodedOp::Del {
                    id: emitter.next_id(),
                    obj: child,
                    what: spans,
                });
            }
            return Ok(true);
        }
        Some(old) if runtime.resolve_bin_node(child).is_some() => {
            let child = runtime.resolve_bin_node(child).expect("checked is_some");
            let old_bin = match parse_bin_object(old) {
                Some(v) => v,
                None => return Ok(false),
            };
            let new_bin = match parse_bin_object(new_v) {
                Some(v) => v,
                None => return Ok(false),
            };
            let slots = match runtime.bin_visible_slots(child) {
                Some(v) => v,
                None => return Ok(false),
            };
            if slots.len() != old_bin.len() {
                return Ok(false);
            }
            let mut lcp = 0usize;
            while lcp < old_bin.len() && lcp < new_bin.len() && old_bin[lcp] == new_bin[lcp] {
                lcp += 1;
            }
            let mut lcs = 0usize;
            while lcs < (old_bin.len() - lcp)
                && lcs < (new_bin.len() - lcp)
                && old_bin[old_bin.len() - 1 - lcs] == new_bin[new_bin.len() - 1 - lcs]
            {
                lcs += 1;
            }
            let del_len = old_bin.len().saturating_sub(lcp + lcs);
            let ins_bytes = &new_bin[lcp..new_bin.len().saturating_sub(lcs)];
            if !ins_bytes.is_empty() {
                let reference = choose_sequence_insert_reference(
                    &slots,
                    child,
                    lcp,
                    ins_bytes.len(),
                    del_len,
                    old_bin.len(),
                );
                emitter.push(DecodedOp::InsBin {
                    id: emitter.next_id(),
                    obj: child,
                    reference,
                    data: ins_bytes.to_vec(),
                });
            }
            if del_len > 0 {
                let del_slots = &slots[lcp..lcp + del_len];
                let mut spans: Vec<crate::patch::Timespan> = Vec::new();
                for slot in del_slots {
                    if let Some(last) = spans.last_mut() {
                        if last.sid == slot.sid && last.time + last.span == slot.time {
                            last.span += 1;
                            continue;
                        }
                    }
                    spans.push(crate::patch::Timespan {
                        sid: slot.sid,
                        time: slot.time,
                        span: 1,
                    });
                }
                emitter.push(DecodedOp::Del {
                    id: emitter.next_id(),
                    obj: child,
                    what: spans,
                });
            }
            return Ok(true);
        }
        Some(Value::Array(old_arr))
            if matches!(new_v, Value::Array(_)) && runtime.resolve_array_node(child).is_some() =>
        {
            let child = runtime.resolve_array_node(child).expect("checked is_some");
            let new_arr = match new_v {
                Value::Array(v) => v,
                _ => unreachable!(),
            };
            if let (Some(slots), Some(values)) = (
                runtime.array_visible_slots(child),
                runtime.array_visible_values(child),
            ) {
                if try_emit_array_indexwise_diff(
                    runtime,
                    emitter,
                    child,
                    &slots,
                    &values,
                    old_arr,
                    new_arr,
                )? {
                    return Ok(true);
                }
            }
            if old_arr.len() == new_arr.len() && !old_arr.is_empty() {
                let mut any_change = false;
                let mut all_changed_are_object_mutations = true;
                if let Some(values) = runtime.array_visible_values(child) {
                    for i in 0..old_arr.len() {
                        if old_arr[i] == new_arr[i] {
                            continue;
                        }
                        any_change = true;
                        let (Some(old_obj), Some(new_obj)) = (old_arr[i].as_object(), new_arr[i].as_object()) else {
                            all_changed_are_object_mutations = false;
                            break;
                        };
                        if i >= values.len() || runtime.resolve_object_node(values[i]).is_none() {
                            all_changed_are_object_mutations = false;
                            break;
                        }
                        let value_obj = runtime
                            .resolve_object_node(values[i])
                            .expect("checked is_some");
                        let _ = try_emit_object_recursive_diff(
                            runtime,
                            emitter,
                            value_obj,
                            old_obj,
                            new_obj,
                        )?;
                    }
                    if any_change && all_changed_are_object_mutations {
                        return Ok(true);
                    }
                }
            }
            if old_arr.iter().any(|v| !is_array_native_supported(v))
                || new_arr.iter().any(|v| !is_array_native_supported(v))
            {
                return Ok(false);
            }
            let slots = match runtime.array_visible_slots(child) {
                Some(v) => v,
                None => return Ok(false),
            };
            if slots.len() != old_arr.len() {
                return Ok(false);
            }
            emit_array_delta_ops(emitter, child, &slots, old_arr, new_arr);
            return Ok(true);
        }
        Some(Value::Array(_))
            if matches!(new_v, Value::Array(_)) && runtime.resolve_vec_node(child).is_some() =>
        {
            let child = runtime.resolve_vec_node(child).expect("checked is_some");
            let new_arr = match new_v {
                Value::Array(v) => v,
                _ => unreachable!(),
            };
            emit_vec_delta_ops(runtime, emitter, child, new_arr)?;
            return Ok(true);
        }
        Some(Value::Object(old_obj))
            if matches!(new_v, Value::Object(_)) && runtime.resolve_object_node(child).is_some() =>
        {
            let child = runtime.resolve_object_node(child).expect("checked is_some");
            let new_obj = match new_v {
                Value::Object(v) => v,
                _ => unreachable!(),
            };
            return try_emit_object_recursive_diff(runtime, emitter, child, old_obj, new_obj);
        }
        _ => {}
    }
    Ok(false)
}

fn try_emit_object_recursive_diff(
    runtime: &RuntimeModel,
    emitter: &mut NativeEmitter,
    obj_node: Timestamp,
    old_obj: &serde_json::Map<String, Value>,
    new_obj: &serde_json::Map<String, Value>,
) -> Result<bool, DiffError> {
    let mut pairs: Vec<(String, Timestamp)> = Vec::new();

    for (k, _) in old_obj {
        if !new_obj.contains_key(k) {
            let id = emitter.next_id();
            emitter.push(DecodedOp::NewCon {
                id,
                value: ConValue::Undef,
            });
            pairs.push((k.clone(), id));
        }
    }

    for (k, v) in new_obj {
        if old_obj.get(k) == Some(v) {
            continue;
        }
        if let Some(child_id) = runtime.object_field(obj_node, k) {
            if try_emit_child_recursive_diff(runtime, emitter, child_id, old_obj.get(k), v)? {
                continue;
            }
        }
        let id = emitter.emit_value(v);
        pairs.push((k.clone(), id));
    }

    if !pairs.is_empty() {
        emitter.push(DecodedOp::InsObj {
            id: emitter.next_id(),
            obj: obj_node,
            data: pairs,
        });
    }
    Ok(true)
}


struct NativeEmitter {
    sid: u64,
    cursor: u64,
    ops: Vec<DecodedOp>,
}

impl NativeEmitter {
    fn new(sid: u64, start_time: u64) -> Self {
        Self {
            sid,
            cursor: start_time,
            ops: Vec::new(),
        }
    }

    fn next_id(&self) -> Timestamp {
        Timestamp {
            sid: self.sid,
            time: self.cursor,
        }
    }

    fn push(&mut self, op: DecodedOp) {
        self.cursor = self.cursor.saturating_add(op.span());
        self.ops.push(op);
    }

    fn emit_value(&mut self, value: &Value) -> Timestamp {
        match value {
            Value::Null | Value::Bool(_) | Value::Number(_) => {
                let id = self.next_id();
                self.push(DecodedOp::NewCon {
                    id,
                    value: ConValue::Json(value.clone()),
                });
                id
            }
            Value::String(s) => {
                let str_id = self.next_id();
                self.push(DecodedOp::NewStr { id: str_id });
                if !s.is_empty() {
                    let ins_id = self.next_id();
                    self.push(DecodedOp::InsStr {
                        id: ins_id,
                        obj: str_id,
                        reference: str_id,
                        data: s.clone(),
                    });
                }
                str_id
            }
            Value::Array(items) => {
                let arr_id = self.next_id();
                self.push(DecodedOp::NewArr { id: arr_id });
                if !items.is_empty() {
                    let mut children = Vec::with_capacity(items.len());
                    for item in items {
                        if is_con_scalar(item) {
                            // Array scalar elements are emitted as VAL wrappers
                            // around CON nodes to mirror upstream diff op shape.
                            let val_id = self.next_id();
                            self.push(DecodedOp::NewVal { id: val_id });
                            let con_id = self.emit_value(item);
                            let ins_id = self.next_id();
                            self.push(DecodedOp::InsVal {
                                id: ins_id,
                                obj: val_id,
                                val: con_id,
                            });
                            children.push(val_id);
                        } else {
                            children.push(self.emit_value(item));
                        }
                    }
                    let ins_id = self.next_id();
                    self.push(DecodedOp::InsArr {
                        id: ins_id,
                        obj: arr_id,
                        reference: arr_id,
                        data: children,
                    });
                }
                arr_id
            }
            Value::Object(map) => {
                let obj_id = self.next_id();
                self.push(DecodedOp::NewObj { id: obj_id });
                if !map.is_empty() {
                    let mut pairs = Vec::with_capacity(map.len());
                    for (k, v) in map {
                        let id = self.emit_value(v);
                        pairs.push((k.clone(), id));
                    }
                    let ins_id = self.next_id();
                    self.push(DecodedOp::InsObj {
                        id: ins_id,
                        obj: obj_id,
                        data: pairs,
                    });
                }
                obj_id
            }
        }
    }
}

fn is_con_scalar(value: &Value) -> bool {
    matches!(value, Value::Null | Value::Bool(_) | Value::Number(_))
}

fn is_array_native_supported(value: &Value) -> bool {
    is_con_scalar(value) || matches!(value, Value::String(_))
}


fn object_single_scalar_key_delta(
    old: &serde_json::Map<String, Value>,
    new: &serde_json::Map<String, Value>,
) -> Option<(String, ConValue)> {
    let mut changed: Option<(String, ConValue)> = None;

    for (k, old_v) in old {
        match new.get(k) {
            Some(new_v) => {
                if old_v == new_v {
                    continue;
                }
                if !is_con_scalar(new_v) {
                    return None;
                }
                if changed.is_some() {
                    return None;
                }
                changed = Some((k.clone(), ConValue::Json(new_v.clone())));
            }
            None => {
                if changed.is_some() {
                    return None;
                }
                changed = Some((k.clone(), ConValue::Undef));
            }
        }
    }

    for (k, new_v) in new {
        if old.contains_key(k) {
            continue;
        }
        if !is_con_scalar(new_v) {
            return None;
        }
        if changed.is_some() {
            return None;
        }
        changed = Some((k.clone(), ConValue::Json(new_v.clone())));
    }

    changed
}
