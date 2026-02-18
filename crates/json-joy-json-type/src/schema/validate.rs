//! Schema integrity validator.
//!
//! Upstream reference: json-type/src/schema/validate.ts

use super::schema::*;

/// Validate a schema for structural integrity.
///
/// Returns `Ok(())` if the schema is valid, or `Err(msg)` with a description.
pub fn validate_schema(schema: &Schema) -> Result<(), String> {
    match schema {
        Schema::Any(_) => Ok(()),
        Schema::Bool(_) => Ok(()),
        Schema::Con(_) => Ok(()),
        Schema::Num(s) => validate_num(s),
        Schema::Str(s) => validate_str(s),
        Schema::Bin(s) => validate_bin(s),
        Schema::Arr(s) => validate_arr(s),
        Schema::Obj(s) => validate_obj(s),
        Schema::Key(s) => validate_key(s),
        Schema::Map(s) => validate_map(s),
        Schema::Ref(s) => validate_ref(s),
        Schema::Or(s) => validate_or(s),
        Schema::Fn(s) => {
            validate_schema(&s.req)?;
            validate_schema(&s.res)
        }
        Schema::FnRx(s) => {
            validate_schema(&s.req)?;
            validate_schema(&s.res)
        }
        Schema::Alias(s) => validate_schema(&s.value),
        Schema::Module(s) => {
            for alias in &s.keys {
                validate_schema(&alias.value)?;
            }
            Ok(())
        }
    }
}

fn validate_num(s: &NumSchema) -> Result<(), String> {
    if s.gt.is_some() && s.gte.is_some() {
        return Err("GT_GTE".into());
    }
    if s.lt.is_some() && s.lte.is_some() {
        return Err("LT_LTE".into());
    }
    let lo = s.gt.or(s.gte);
    let hi = s.lt.or(s.lte);
    if let (Some(lo), Some(hi)) = (lo, hi) {
        if lo > hi {
            return Err("GT_LT".into());
        }
    }
    Ok(())
}

fn validate_str(s: &StrSchema) -> Result<(), String> {
    if let (Some(min), Some(max)) = (s.min, s.max) {
        if min > max {
            return Err("MIN_MAX".into());
        }
    }
    Ok(())
}

fn validate_bin(s: &BinSchema) -> Result<(), String> {
    if let (Some(min), Some(max)) = (s.min, s.max) {
        if min > max {
            return Err("MIN_MAX".into());
        }
    }
    validate_schema(&s.type_)
}

fn validate_arr(s: &ArrSchema) -> Result<(), String> {
    if s.head.is_none() && s.type_.is_none() && s.tail.is_none() {
        return Err("EMPTY_ARR".into());
    }
    if s.tail.is_some() && s.type_.is_none() {
        return Err("LONE_TAIL".into());
    }
    if let (Some(min), Some(max)) = (s.min, s.max) {
        if min > max {
            return Err("MIN_MAX".into());
        }
    }
    if let Some(t) = &s.type_ {
        validate_schema(t)?;
    }
    if let Some(head) = &s.head {
        for h in head {
            validate_schema(h)?;
        }
    }
    if let Some(tail) = &s.tail {
        for t in tail {
            validate_schema(t)?;
        }
    }
    Ok(())
}

fn validate_obj(s: &ObjSchema) -> Result<(), String> {
    for key in &s.keys {
        validate_schema(&Schema::Key(KeySchema {
            base: key.base.clone(),
            key: key.key.clone(),
            value: key.value.clone(),
            optional: key.optional,
        }))?;
    }
    Ok(())
}

fn validate_key(s: &KeySchema) -> Result<(), String> {
    if s.key.is_empty() {
        return Err("KEY_EMPTY".into());
    }
    validate_schema(&s.value)
}

fn validate_map(s: &MapSchema) -> Result<(), String> {
    validate_schema(&s.value)?;
    if let Some(key) = &s.key {
        validate_schema(key)?;
    }
    Ok(())
}

fn validate_ref(s: &RefSchema) -> Result<(), String> {
    if s.ref_.is_empty() {
        return Err("REF_EMPTY".into());
    }
    Ok(())
}

fn validate_or(s: &OrSchema) -> Result<(), String> {
    if s.types.is_empty() {
        return Err("TYPES_LENGTH".into());
    }
    for t in &s.types {
        validate_schema(t)?;
    }
    Ok(())
}
