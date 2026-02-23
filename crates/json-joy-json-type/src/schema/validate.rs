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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn any() -> Schema {
        Schema::Any(AnySchema::default())
    }

    fn str_s() -> Schema {
        Schema::Str(StrSchema::default())
    }

    fn num_s() -> Schema {
        Schema::Num(NumSchema::default())
    }

    #[test]
    fn validate_any_ok() {
        assert!(validate_schema(&any()).is_ok());
    }

    #[test]
    fn validate_bool_ok() {
        assert!(validate_schema(&Schema::Bool(BoolSchema::default())).is_ok());
    }

    #[test]
    fn validate_con_ok() {
        let s = Schema::Con(ConSchema {
            base: SchemaBase::default(),
            value: json!(42),
        });
        assert!(validate_schema(&s).is_ok());
    }

    // -- Num validation --

    #[test]
    fn validate_num_ok() {
        assert!(validate_schema(&num_s()).is_ok());
    }

    #[test]
    fn validate_num_gt_and_gte_conflict() {
        let s = Schema::Num(NumSchema {
            gt: Some(1.0),
            gte: Some(1.0),
            ..Default::default()
        });
        assert_eq!(validate_schema(&s), Err("GT_GTE".into()));
    }

    #[test]
    fn validate_num_lt_and_lte_conflict() {
        let s = Schema::Num(NumSchema {
            lt: Some(10.0),
            lte: Some(10.0),
            ..Default::default()
        });
        assert_eq!(validate_schema(&s), Err("LT_LTE".into()));
    }

    #[test]
    fn validate_num_gt_exceeds_lt() {
        let s = Schema::Num(NumSchema {
            gt: Some(100.0),
            lt: Some(1.0),
            ..Default::default()
        });
        assert_eq!(validate_schema(&s), Err("GT_LT".into()));
    }

    #[test]
    fn validate_num_gte_exceeds_lte() {
        let s = Schema::Num(NumSchema {
            gte: Some(100.0),
            lte: Some(1.0),
            ..Default::default()
        });
        assert_eq!(validate_schema(&s), Err("GT_LT".into()));
    }

    #[test]
    fn validate_num_gt_equal_lt_is_ok() {
        // gt=5, lt=5 means lo == hi which is valid (not >)
        let s = Schema::Num(NumSchema {
            gt: Some(5.0),
            lt: Some(5.0),
            ..Default::default()
        });
        assert!(validate_schema(&s).is_ok());
    }

    // -- Str validation --

    #[test]
    fn validate_str_ok() {
        assert!(validate_schema(&str_s()).is_ok());
    }

    #[test]
    fn validate_str_min_exceeds_max() {
        let s = Schema::Str(StrSchema {
            min: Some(100),
            max: Some(10),
            ..Default::default()
        });
        assert_eq!(validate_schema(&s), Err("MIN_MAX".into()));
    }

    #[test]
    fn validate_str_min_equals_max_ok() {
        let s = Schema::Str(StrSchema {
            min: Some(5),
            max: Some(5),
            ..Default::default()
        });
        assert!(validate_schema(&s).is_ok());
    }

    // -- Bin validation --

    #[test]
    fn validate_bin_ok() {
        let s = Schema::Bin(BinSchema {
            base: SchemaBase::default(),
            type_: Box::new(any()),
            format: None,
            min: None,
            max: None,
        });
        assert!(validate_schema(&s).is_ok());
    }

    #[test]
    fn validate_bin_min_exceeds_max() {
        let s = Schema::Bin(BinSchema {
            base: SchemaBase::default(),
            type_: Box::new(any()),
            format: None,
            min: Some(100),
            max: Some(10),
        });
        assert_eq!(validate_schema(&s), Err("MIN_MAX".into()));
    }

    // -- Arr validation --

    #[test]
    fn validate_arr_with_type_ok() {
        let s = Schema::Arr(ArrSchema {
            type_: Some(Box::new(any())),
            ..Default::default()
        });
        assert!(validate_schema(&s).is_ok());
    }

    #[test]
    fn validate_arr_empty_err() {
        let s = Schema::Arr(ArrSchema::default());
        assert_eq!(validate_schema(&s), Err("EMPTY_ARR".into()));
    }

    #[test]
    fn validate_arr_lone_tail_err() {
        let s = Schema::Arr(ArrSchema {
            tail: Some(vec![any()]),
            ..Default::default()
        });
        assert_eq!(validate_schema(&s), Err("LONE_TAIL".into()));
    }

    #[test]
    fn validate_arr_min_exceeds_max() {
        let s = Schema::Arr(ArrSchema {
            type_: Some(Box::new(any())),
            min: Some(100),
            max: Some(10),
            ..Default::default()
        });
        assert_eq!(validate_schema(&s), Err("MIN_MAX".into()));
    }

    #[test]
    fn validate_arr_head_only_ok() {
        let s = Schema::Arr(ArrSchema {
            head: Some(vec![any(), str_s()]),
            ..Default::default()
        });
        assert!(validate_schema(&s).is_ok());
    }

    #[test]
    fn validate_arr_head_and_type_and_tail_ok() {
        let s = Schema::Arr(ArrSchema {
            head: Some(vec![str_s()]),
            type_: Some(Box::new(num_s())),
            tail: Some(vec![str_s()]),
            ..Default::default()
        });
        assert!(validate_schema(&s).is_ok());
    }

    // -- Obj validation --

    #[test]
    fn validate_obj_ok() {
        let s = Schema::Obj(ObjSchema {
            keys: vec![KeySchema {
                base: SchemaBase::default(),
                key: "name".into(),
                value: Box::new(str_s()),
                optional: None,
            }],
            ..Default::default()
        });
        assert!(validate_schema(&s).is_ok());
    }

    #[test]
    fn validate_obj_empty_key_err() {
        let s = Schema::Obj(ObjSchema {
            keys: vec![KeySchema {
                base: SchemaBase::default(),
                key: "".into(),
                value: Box::new(str_s()),
                optional: None,
            }],
            ..Default::default()
        });
        assert_eq!(validate_schema(&s), Err("KEY_EMPTY".into()));
    }

    // -- Key validation --

    #[test]
    fn validate_key_empty_err() {
        let s = Schema::Key(KeySchema {
            base: SchemaBase::default(),
            key: "".into(),
            value: Box::new(any()),
            optional: None,
        });
        assert_eq!(validate_schema(&s), Err("KEY_EMPTY".into()));
    }

    // -- Map validation --

    #[test]
    fn validate_map_ok() {
        let s = Schema::Map(MapSchema {
            base: SchemaBase::default(),
            key: None,
            value: Box::new(any()),
        });
        assert!(validate_schema(&s).is_ok());
    }

    #[test]
    fn validate_map_with_key_ok() {
        let s = Schema::Map(MapSchema {
            base: SchemaBase::default(),
            key: Some(Box::new(str_s())),
            value: Box::new(num_s()),
        });
        assert!(validate_schema(&s).is_ok());
    }

    // -- Ref validation --

    #[test]
    fn validate_ref_empty_err() {
        let s = Schema::Ref(RefSchema {
            base: SchemaBase::default(),
            ref_: "".into(),
        });
        assert_eq!(validate_schema(&s), Err("REF_EMPTY".into()));
    }

    #[test]
    fn validate_ref_ok() {
        let s = Schema::Ref(RefSchema {
            base: SchemaBase::default(),
            ref_: "MyType".into(),
        });
        assert!(validate_schema(&s).is_ok());
    }

    // -- Or validation --

    #[test]
    fn validate_or_empty_err() {
        let s = Schema::Or(OrSchema {
            base: SchemaBase::default(),
            types: vec![],
            discriminator: json!(null),
        });
        assert_eq!(validate_schema(&s), Err("TYPES_LENGTH".into()));
    }

    #[test]
    fn validate_or_ok() {
        let s = Schema::Or(OrSchema {
            base: SchemaBase::default(),
            types: vec![str_s(), num_s()],
            discriminator: json!(null),
        });
        assert!(validate_schema(&s).is_ok());
    }

    // -- Fn / FnRx validation --

    #[test]
    fn validate_fn_ok() {
        let s = Schema::Fn(FnSchema {
            base: SchemaBase::default(),
            req: Box::new(str_s()),
            res: Box::new(num_s()),
        });
        assert!(validate_schema(&s).is_ok());
    }

    #[test]
    fn validate_fn_rx_ok() {
        let s = Schema::FnRx(FnRxSchema {
            base: SchemaBase::default(),
            req: Box::new(any()),
            res: Box::new(any()),
        });
        assert!(validate_schema(&s).is_ok());
    }

    // -- Alias / Module validation --

    #[test]
    fn validate_alias_ok() {
        let s = Schema::Alias(AliasSchema {
            base: SchemaBase::default(),
            key: "Foo".into(),
            value: Box::new(str_s()),
            optional: None,
            pub_: None,
        });
        assert!(validate_schema(&s).is_ok());
    }

    #[test]
    fn validate_module_ok() {
        let s = Schema::Module(ModuleSchema {
            base: SchemaBase::default(),
            keys: vec![AliasSchema {
                base: SchemaBase::default(),
                key: "Foo".into(),
                value: Box::new(str_s()),
                optional: None,
                pub_: Some(true),
            }],
        });
        assert!(validate_schema(&s).is_ok());
    }

    #[test]
    fn validate_module_propagates_inner_error() {
        let s = Schema::Module(ModuleSchema {
            base: SchemaBase::default(),
            keys: vec![AliasSchema {
                base: SchemaBase::default(),
                key: "Bad".into(),
                value: Box::new(Schema::Ref(RefSchema {
                    base: SchemaBase::default(),
                    ref_: "".into(),
                })),
                optional: None,
                pub_: None,
            }],
        });
        assert_eq!(validate_schema(&s), Err("REF_EMPTY".into()));
    }
}
