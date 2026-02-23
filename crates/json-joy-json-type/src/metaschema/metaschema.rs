//! Metaschema — describes the json-type schema system using its own schema language.
//!
//! Upstream reference: json-type/src/metaschema/metaschema.ts

use serde_json::json;

use crate::schema::*;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn str_schema() -> Schema {
    Schema::Str(StrSchema::default())
}

fn any_schema() -> Schema {
    Schema::Any(AnySchema::default())
}

fn bool_schema() -> Schema {
    Schema::Bool(BoolSchema::default())
}

fn num_schema() -> Schema {
    Schema::Num(NumSchema::default())
}

fn con_schema(value: serde_json::Value) -> Schema {
    Schema::Con(ConSchema {
        base: SchemaBase::default(),
        value,
    })
}

fn ref_schema(name: &str) -> Schema {
    Schema::Ref(RefSchema {
        base: SchemaBase::default(),
        ref_: name.to_string(),
    })
}

fn arr_schema(item: Schema) -> Schema {
    Schema::Arr(ArrSchema {
        type_: Some(Box::new(item)),
        ..Default::default()
    })
}

fn map_schema(value: Schema) -> Schema {
    Schema::Map(MapSchema {
        base: SchemaBase::default(),
        key: None,
        value: Box::new(value),
    })
}

fn or_schema(types: Vec<Schema>) -> Schema {
    Schema::Or(OrSchema {
        base: SchemaBase::default(),
        types,
        discriminator: json!(["num", -1]),
    })
}

fn key(k: &str, v: Schema) -> KeySchema {
    KeySchema {
        base: SchemaBase::default(),
        key: k.to_string(),
        value: Box::new(v),
        optional: None,
    }
}

fn key_opt(k: &str, v: Schema) -> KeySchema {
    KeySchema {
        base: SchemaBase::default(),
        key: k.to_string(),
        value: Box::new(v),
        optional: Some(true),
    }
}

fn obj(keys: Vec<KeySchema>) -> Schema {
    Schema::Obj(ObjSchema {
        keys,
        ..Default::default()
    })
}

fn alias(id: &str, value: Schema) -> AliasSchema {
    AliasSchema {
        base: SchemaBase::default(),
        key: id.to_string(),
        value: Box::new(value),
        optional: None,
        pub_: Some(true),
    }
}

// ── Individual type definitions ───────────────────────────────────────────────

fn display() -> AliasSchema {
    alias(
        "Display",
        obj(vec![
            key_opt("title", str_schema()),
            key_opt("intro", str_schema()),
            key_opt("description", str_schema()),
        ]),
    )
}

fn schema_example() -> AliasSchema {
    alias(
        "SchemaExample",
        obj(vec![
            key_opt("title", str_schema()),
            key_opt("intro", str_schema()),
            key_opt("description", str_schema()),
            key("value", any_schema()),
        ]),
    )
}

fn schema_base() -> AliasSchema {
    alias(
        "SchemaBase",
        obj(vec![
            key_opt("title", str_schema()),
            key_opt("intro", str_schema()),
            key_opt("description", str_schema()),
            key("kind", str_schema()),
            key_opt("meta", map_schema(any_schema())),
            key_opt("default", any_schema()),
            key_opt("examples", arr_schema(ref_schema("SchemaExample"))),
            key_opt("deprecated", obj(vec![key_opt("info", str_schema())])),
            key_opt("metadata", map_schema(any_schema())),
        ]),
    )
}

fn any_schema_def() -> AliasSchema {
    alias(
        "AnySchema",
        obj(vec![
            key_opt("title", str_schema()),
            key_opt("intro", str_schema()),
            key_opt("description", str_schema()),
            key("kind", con_schema(json!("any"))),
        ]),
    )
}

fn con_schema_def() -> AliasSchema {
    alias(
        "ConSchema",
        obj(vec![
            key_opt("title", str_schema()),
            key_opt("intro", str_schema()),
            key_opt("description", str_schema()),
            key("kind", con_schema(json!("con"))),
            key("value", any_schema()),
        ]),
    )
}

fn bool_schema_def() -> AliasSchema {
    alias(
        "BoolSchema",
        obj(vec![
            key_opt("title", str_schema()),
            key_opt("intro", str_schema()),
            key_opt("description", str_schema()),
            key("kind", con_schema(json!("bool"))),
        ]),
    )
}

fn num_schema_def() -> AliasSchema {
    alias(
        "NumSchema",
        obj(vec![
            key_opt("title", str_schema()),
            key_opt("intro", str_schema()),
            key_opt("description", str_schema()),
            key("kind", con_schema(json!("num"))),
            key_opt(
                "format",
                or_schema(vec![
                    con_schema(json!("i")),
                    con_schema(json!("u")),
                    con_schema(json!("f")),
                    con_schema(json!("i8")),
                    con_schema(json!("i16")),
                    con_schema(json!("i32")),
                    con_schema(json!("i64")),
                    con_schema(json!("u8")),
                    con_schema(json!("u16")),
                    con_schema(json!("u32")),
                    con_schema(json!("u64")),
                    con_schema(json!("f32")),
                    con_schema(json!("f64")),
                ]),
            ),
            key_opt("gt", num_schema()),
            key_opt("gte", num_schema()),
            key_opt("lt", num_schema()),
            key_opt("lte", num_schema()),
        ]),
    )
}

fn str_schema_def() -> AliasSchema {
    alias(
        "StrSchema",
        obj(vec![
            key_opt("title", str_schema()),
            key_opt("intro", str_schema()),
            key_opt("description", str_schema()),
            key("kind", con_schema(json!("str"))),
            key_opt(
                "format",
                or_schema(vec![con_schema(json!("ascii")), con_schema(json!("utf8"))]),
            ),
            key_opt("ascii", bool_schema()),
            key_opt("noJsonEscape", bool_schema()),
            key_opt("min", num_schema()),
            key_opt("max", num_schema()),
        ]),
    )
}

fn bin_schema_def() -> AliasSchema {
    alias(
        "BinSchema",
        obj(vec![
            key_opt("title", str_schema()),
            key_opt("intro", str_schema()),
            key_opt("description", str_schema()),
            key("kind", con_schema(json!("bin"))),
            key("type", ref_schema("Schema")),
            key_opt(
                "format",
                or_schema(vec![
                    con_schema(json!("json")),
                    con_schema(json!("cbor")),
                    con_schema(json!("msgpack")),
                    con_schema(json!("resp3")),
                    con_schema(json!("ion")),
                    con_schema(json!("bson")),
                    con_schema(json!("ubjson")),
                    con_schema(json!("bencode")),
                ]),
            ),
            key_opt("min", num_schema()),
            key_opt("max", num_schema()),
        ]),
    )
}

fn arr_schema_def() -> AliasSchema {
    alias(
        "ArrSchema",
        obj(vec![
            key_opt("title", str_schema()),
            key_opt("intro", str_schema()),
            key_opt("description", str_schema()),
            key("kind", con_schema(json!("arr"))),
            key_opt("type", ref_schema("Schema")),
            key_opt("head", arr_schema(ref_schema("Schema"))),
            key_opt("tail", arr_schema(ref_schema("Schema"))),
            key_opt("min", num_schema()),
            key_opt("max", num_schema()),
        ]),
    )
}

fn key_schema_def() -> AliasSchema {
    alias(
        "KeySchema",
        obj(vec![
            key_opt("title", str_schema()),
            key_opt("intro", str_schema()),
            key_opt("description", str_schema()),
            key("kind", con_schema(json!("key"))),
            key("key", str_schema()),
            key("value", ref_schema("Schema")),
            key_opt("optional", bool_schema()),
        ]),
    )
}

fn obj_schema_def() -> AliasSchema {
    alias(
        "ObjSchema",
        obj(vec![
            key_opt("title", str_schema()),
            key_opt("intro", str_schema()),
            key_opt("description", str_schema()),
            key("kind", con_schema(json!("obj"))),
            key("keys", arr_schema(ref_schema("KeySchema"))),
            key_opt("extends", arr_schema(str_schema())),
            key_opt("decodeUnknownKeys", bool_schema()),
            key_opt("encodeUnknownKeys", bool_schema()),
        ]),
    )
}

fn map_schema_def() -> AliasSchema {
    alias(
        "MapSchema",
        obj(vec![
            key_opt("title", str_schema()),
            key_opt("intro", str_schema()),
            key_opt("description", str_schema()),
            key("kind", con_schema(json!("map"))),
            key_opt("key", ref_schema("Schema")),
            key("value", ref_schema("Schema")),
        ]),
    )
}

fn ref_schema_def() -> AliasSchema {
    alias(
        "RefSchema",
        obj(vec![
            key_opt("title", str_schema()),
            key_opt("intro", str_schema()),
            key_opt("description", str_schema()),
            key("kind", con_schema(json!("ref"))),
            key("ref", str_schema()),
        ]),
    )
}

fn or_schema_def() -> AliasSchema {
    alias(
        "OrSchema",
        obj(vec![
            key_opt("title", str_schema()),
            key_opt("intro", str_schema()),
            key_opt("description", str_schema()),
            key("kind", con_schema(json!("or"))),
            key("types", arr_schema(ref_schema("Schema"))),
            key("discriminator", any_schema()),
        ]),
    )
}

fn fn_schema_def() -> AliasSchema {
    alias(
        "FnSchema",
        obj(vec![
            key_opt("title", str_schema()),
            key_opt("intro", str_schema()),
            key_opt("description", str_schema()),
            key("kind", con_schema(json!("fn"))),
            key("req", ref_schema("Schema")),
            key("res", ref_schema("Schema")),
        ]),
    )
}

fn fn_rx_schema_def() -> AliasSchema {
    alias(
        "FnRxSchema",
        obj(vec![
            key_opt("title", str_schema()),
            key_opt("intro", str_schema()),
            key_opt("description", str_schema()),
            key("kind", con_schema(json!("fn$"))),
            key("req", ref_schema("Schema")),
            key("res", ref_schema("Schema")),
        ]),
    )
}

fn alias_schema_def() -> AliasSchema {
    alias(
        "AliasSchema",
        obj(vec![
            key_opt("title", str_schema()),
            key_opt("intro", str_schema()),
            key_opt("description", str_schema()),
            key("kind", con_schema(json!("key"))),
            key("key", str_schema()),
            key("value", ref_schema("Schema")),
            key_opt("optional", bool_schema()),
            key_opt("pub", bool_schema()),
        ]),
    )
}

fn module_schema_def() -> AliasSchema {
    alias(
        "ModuleSchema",
        obj(vec![
            key("kind", con_schema(json!("module"))),
            key("keys", arr_schema(ref_schema("AliasSchema"))),
        ]),
    )
}

fn json_schema_def() -> AliasSchema {
    alias(
        "JsonSchema",
        or_schema(vec![
            ref_schema("BoolSchema"),
            ref_schema("NumSchema"),
            ref_schema("StrSchema"),
            ref_schema("BinSchema"),
            ref_schema("ArrSchema"),
            ref_schema("ConSchema"),
            ref_schema("ObjSchema"),
            ref_schema("KeySchema"),
            ref_schema("MapSchema"),
        ]),
    )
}

fn schema_def() -> AliasSchema {
    alias(
        "Schema",
        or_schema(vec![
            ref_schema("JsonSchema"),
            ref_schema("RefSchema"),
            ref_schema("OrSchema"),
            ref_schema("AnySchema"),
            ref_schema("FnSchema"),
            ref_schema("FnRxSchema"),
        ]),
    )
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Returns the metaschema: a `ModuleSchema` that describes the entire
/// json-type schema system using its own schema language.
///
/// Ports the `module` export from `json-type/src/metaschema/metaschema.ts`.
pub fn module() -> ModuleSchema {
    ModuleSchema {
        base: SchemaBase::default(),
        keys: vec![
            display(),
            schema_example(),
            schema_base(),
            any_schema_def(),
            con_schema_def(),
            bool_schema_def(),
            num_schema_def(),
            str_schema_def(),
            bin_schema_def(),
            arr_schema_def(),
            key_schema_def(),
            obj_schema_def(),
            map_schema_def(),
            ref_schema_def(),
            or_schema_def(),
            fn_schema_def(),
            fn_rx_schema_def(),
            alias_schema_def(),
            module_schema_def(),
            json_schema_def(),
            schema_def(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::validate::validate_schema;

    #[test]
    fn module_returns_expected_alias_count() {
        let m = module();
        assert_eq!(m.keys.len(), 21);
    }

    #[test]
    fn module_first_alias_is_display() {
        let m = module();
        assert_eq!(m.keys[0].key, "Display");
    }

    #[test]
    fn module_last_alias_is_schema() {
        let m = module();
        assert_eq!(m.keys.last().unwrap().key, "Schema");
    }

    #[test]
    fn module_contains_expected_alias_names() {
        let m = module();
        let names: Vec<&str> = m.keys.iter().map(|a| a.key.as_str()).collect();
        assert!(names.contains(&"Display"));
        assert!(names.contains(&"SchemaExample"));
        assert!(names.contains(&"SchemaBase"));
        assert!(names.contains(&"AnySchema"));
        assert!(names.contains(&"ConSchema"));
        assert!(names.contains(&"BoolSchema"));
        assert!(names.contains(&"NumSchema"));
        assert!(names.contains(&"StrSchema"));
        assert!(names.contains(&"BinSchema"));
        assert!(names.contains(&"ArrSchema"));
        assert!(names.contains(&"KeySchema"));
        assert!(names.contains(&"ObjSchema"));
        assert!(names.contains(&"MapSchema"));
        assert!(names.contains(&"RefSchema"));
        assert!(names.contains(&"OrSchema"));
        assert!(names.contains(&"FnSchema"));
        assert!(names.contains(&"FnRxSchema"));
        assert!(names.contains(&"AliasSchema"));
        assert!(names.contains(&"ModuleSchema"));
        assert!(names.contains(&"JsonSchema"));
        assert!(names.contains(&"Schema"));
    }

    #[test]
    fn all_aliases_are_public() {
        let m = module();
        for alias in &m.keys {
            assert_eq!(
                alias.pub_,
                Some(true),
                "Alias {} should be public",
                alias.key
            );
        }
    }

    #[test]
    fn all_alias_values_are_valid_schemas() {
        let m = module();
        for alias in &m.keys {
            let result = validate_schema(&alias.value);
            assert!(
                result.is_ok(),
                "Alias {} has invalid schema: {:?}",
                alias.key,
                result
            );
        }
    }

    #[test]
    fn module_schema_is_itself_valid() {
        let m = module();
        let schema = Schema::Module(m);
        assert!(validate_schema(&schema).is_ok());
    }

    #[test]
    fn display_alias_has_obj_with_optional_keys() {
        let m = module();
        let display_alias = m.keys.iter().find(|a| a.key == "Display").unwrap();
        if let Schema::Obj(obj) = display_alias.value.as_ref() {
            assert_eq!(obj.keys.len(), 3);
            // All keys should be optional
            for key in &obj.keys {
                assert_eq!(key.optional, Some(true));
            }
        } else {
            panic!("Display should be an Obj schema");
        }
    }

    #[test]
    fn schema_example_has_required_value_key() {
        let m = module();
        let example = m.keys.iter().find(|a| a.key == "SchemaExample").unwrap();
        if let Schema::Obj(obj) = example.value.as_ref() {
            let value_key = obj.keys.iter().find(|k| k.key == "value").unwrap();
            // value should be required (optional is None)
            assert!(value_key.optional.is_none());
        } else {
            panic!("SchemaExample should be an Obj schema");
        }
    }

    #[test]
    fn any_schema_def_has_kind_con() {
        let m = module();
        let any_def = m.keys.iter().find(|a| a.key == "AnySchema").unwrap();
        if let Schema::Obj(obj) = any_def.value.as_ref() {
            let kind_key = obj.keys.iter().find(|k| k.key == "kind").unwrap();
            if let Schema::Con(con) = kind_key.value.as_ref() {
                assert_eq!(con.value, json!("any"));
            } else {
                panic!("kind should be a Con schema");
            }
        } else {
            panic!("AnySchema should be an Obj schema");
        }
    }

    #[test]
    fn schema_def_is_or_type() {
        let m = module();
        let schema_def = m.keys.iter().find(|a| a.key == "Schema").unwrap();
        if let Schema::Or(or) = schema_def.value.as_ref() {
            assert!(!or.types.is_empty());
        } else {
            panic!("Schema should be an Or schema");
        }
    }

    #[test]
    fn json_schema_def_is_or_type() {
        let m = module();
        let json_schema_def = m.keys.iter().find(|a| a.key == "JsonSchema").unwrap();
        if let Schema::Or(or) = json_schema_def.value.as_ref() {
            assert_eq!(or.types.len(), 9);
        } else {
            panic!("JsonSchema should be an Or schema");
        }
    }

    // -- Helper function tests --

    #[test]
    fn helper_str_schema() {
        assert_eq!(str_schema().kind(), "str");
    }

    #[test]
    fn helper_any_schema() {
        assert_eq!(any_schema().kind(), "any");
    }

    #[test]
    fn helper_bool_schema() {
        assert_eq!(bool_schema().kind(), "bool");
    }

    #[test]
    fn helper_num_schema() {
        assert_eq!(num_schema().kind(), "num");
    }

    #[test]
    fn helper_con_schema() {
        let s = con_schema(json!("test"));
        if let Schema::Con(con) = &s {
            assert_eq!(con.value, json!("test"));
        } else {
            panic!("Expected Con");
        }
    }

    #[test]
    fn helper_ref_schema() {
        let s = ref_schema("Foo");
        if let Schema::Ref(r) = &s {
            assert_eq!(r.ref_, "Foo");
        } else {
            panic!("Expected Ref");
        }
    }

    #[test]
    fn helper_arr_schema() {
        let s = arr_schema(str_schema());
        assert_eq!(s.kind(), "arr");
    }

    #[test]
    fn helper_map_schema() {
        let s = map_schema(num_schema());
        assert_eq!(s.kind(), "map");
    }

    #[test]
    fn helper_or_schema() {
        let s = or_schema(vec![str_schema(), num_schema()]);
        if let Schema::Or(or) = &s {
            assert_eq!(or.types.len(), 2);
        } else {
            panic!("Expected Or");
        }
    }

    #[test]
    fn helper_key_is_required() {
        let k = key("name", str_schema());
        assert_eq!(k.key, "name");
        assert!(k.optional.is_none());
    }

    #[test]
    fn helper_key_opt_is_optional() {
        let k = key_opt("name", str_schema());
        assert_eq!(k.key, "name");
        assert_eq!(k.optional, Some(true));
    }

    #[test]
    fn helper_obj_creates_obj_schema() {
        let s = obj(vec![key("a", str_schema())]);
        assert_eq!(s.kind(), "obj");
    }

    #[test]
    fn helper_alias_creates_alias_schema() {
        let a = alias("MyType", str_schema());
        assert_eq!(a.key, "MyType");
        assert_eq!(a.pub_, Some(true));
        assert!(a.optional.is_none());
    }
}
