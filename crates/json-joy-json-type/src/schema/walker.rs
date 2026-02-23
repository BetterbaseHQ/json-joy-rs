//! Schema walker — port of Walker.ts.

use super::schema::Schema;

/// Walks every node in a schema tree, calling the visitor for each node.
pub struct Walker;

impl Walker {
    /// Walk the entire schema tree rooted at `schema`, calling `on_type` for every node.
    pub fn walk(schema: &Schema, on_type: &mut dyn FnMut(&Schema)) {
        let mut w = Walker;
        w.walk_node(schema, on_type);
    }

    fn walk_node(&mut self, schema: &Schema, on_type: &mut dyn FnMut(&Schema)) {
        match schema {
            Schema::Key(s) => {
                on_type(schema);
                self.walk_node(&s.value, on_type);
            }
            Schema::Any(_)
            | Schema::Con(_)
            | Schema::Bool(_)
            | Schema::Num(_)
            | Schema::Str(_)
            | Schema::Bin(_)
            | Schema::Ref(_) => {
                on_type(schema);
            }
            Schema::Arr(s) => {
                on_type(schema);
                if let Some(head) = &s.head {
                    for h in head {
                        self.walk_node(h, on_type);
                    }
                }
                if let Some(t) = &s.type_ {
                    self.walk_node(t, on_type);
                }
                if let Some(tail) = &s.tail {
                    for t in tail {
                        self.walk_node(t, on_type);
                    }
                }
            }
            Schema::Obj(s) => {
                on_type(schema);
                for key in &s.keys {
                    self.walk_node(&key.value, on_type);
                }
            }
            Schema::Map(s) => {
                on_type(schema);
                self.walk_node(&s.value, on_type);
                if let Some(key) = &s.key {
                    self.walk_node(key, on_type);
                }
            }
            Schema::Or(s) => {
                on_type(schema);
                for t in &s.types {
                    self.walk_node(t, on_type);
                }
            }
            Schema::Fn(s) => {
                on_type(schema);
                self.walk_node(&s.req, on_type);
                self.walk_node(&s.res, on_type);
            }
            Schema::FnRx(s) => {
                on_type(schema);
                self.walk_node(&s.req, on_type);
                self.walk_node(&s.res, on_type);
            }
            Schema::Module(s) => {
                on_type(schema);
                for alias in &s.keys {
                    self.walk_node(&alias.value, on_type);
                }
            }
            Schema::Alias(s) => {
                on_type(schema);
                self.walk_node(&s.value, on_type);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::*;
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
    fn walk_any_visits_once() {
        let mut visited = Vec::new();
        Walker::walk(&any(), &mut |s| visited.push(s.kind().to_string()));
        assert_eq!(visited, vec!["any"]);
    }

    #[test]
    fn walk_con_visits_once() {
        let s = Schema::Con(ConSchema {
            base: SchemaBase::default(),
            value: json!(42),
        });
        let mut count = 0;
        Walker::walk(&s, &mut |_| count += 1);
        assert_eq!(count, 1);
    }

    #[test]
    fn walk_bool_visits_once() {
        let mut count = 0;
        Walker::walk(&Schema::Bool(BoolSchema::default()), &mut |_| count += 1);
        assert_eq!(count, 1);
    }

    #[test]
    fn walk_str_visits_once() {
        let mut count = 0;
        Walker::walk(&str_s(), &mut |_| count += 1);
        assert_eq!(count, 1);
    }

    #[test]
    fn walk_bin_visits_once() {
        let s = Schema::Bin(BinSchema {
            base: SchemaBase::default(),
            type_: Box::new(any()),
            format: None,
            min: None,
            max: None,
        });
        let mut count = 0;
        Walker::walk(&s, &mut |_| count += 1);
        assert_eq!(count, 1);
    }

    #[test]
    fn walk_ref_visits_once() {
        let s = Schema::Ref(RefSchema {
            base: SchemaBase::default(),
            ref_: "Foo".into(),
        });
        let mut count = 0;
        Walker::walk(&s, &mut |_| count += 1);
        assert_eq!(count, 1);
    }

    #[test]
    fn walk_key_visits_key_and_value() {
        let s = Schema::Key(KeySchema {
            base: SchemaBase::default(),
            key: "name".into(),
            value: Box::new(str_s()),
            optional: None,
        });
        let mut visited = Vec::new();
        Walker::walk(&s, &mut |s| visited.push(s.kind().to_string()));
        assert_eq!(visited, vec!["key", "str"]);
    }

    #[test]
    fn walk_arr_visits_head_type_tail() {
        let s = Schema::Arr(ArrSchema {
            head: Some(vec![str_s()]),
            type_: Some(Box::new(num_s())),
            tail: Some(vec![any()]),
            ..Default::default()
        });
        let mut visited = Vec::new();
        Walker::walk(&s, &mut |s| visited.push(s.kind().to_string()));
        assert_eq!(visited, vec!["arr", "str", "num", "any"]);
    }

    #[test]
    fn walk_arr_without_head_tail() {
        let s = Schema::Arr(ArrSchema {
            type_: Some(Box::new(num_s())),
            ..Default::default()
        });
        let mut visited = Vec::new();
        Walker::walk(&s, &mut |s| visited.push(s.kind().to_string()));
        assert_eq!(visited, vec!["arr", "num"]);
    }

    #[test]
    fn walk_obj_visits_each_key_value() {
        let s = Schema::Obj(ObjSchema {
            keys: vec![
                KeySchema {
                    base: SchemaBase::default(),
                    key: "a".into(),
                    value: Box::new(str_s()),
                    optional: None,
                },
                KeySchema {
                    base: SchemaBase::default(),
                    key: "b".into(),
                    value: Box::new(num_s()),
                    optional: None,
                },
            ],
            ..Default::default()
        });
        let mut visited = Vec::new();
        Walker::walk(&s, &mut |s| visited.push(s.kind().to_string()));
        assert_eq!(visited, vec!["obj", "str", "num"]);
    }

    #[test]
    fn walk_map_visits_value_and_key() {
        let s = Schema::Map(MapSchema {
            base: SchemaBase::default(),
            key: Some(Box::new(str_s())),
            value: Box::new(num_s()),
        });
        let mut visited = Vec::new();
        Walker::walk(&s, &mut |s| visited.push(s.kind().to_string()));
        assert_eq!(visited, vec!["map", "num", "str"]);
    }

    #[test]
    fn walk_map_without_key() {
        let s = Schema::Map(MapSchema {
            base: SchemaBase::default(),
            key: None,
            value: Box::new(num_s()),
        });
        let mut visited = Vec::new();
        Walker::walk(&s, &mut |s| visited.push(s.kind().to_string()));
        assert_eq!(visited, vec!["map", "num"]);
    }

    #[test]
    fn walk_or_visits_all_types() {
        let s = Schema::Or(OrSchema {
            base: SchemaBase::default(),
            types: vec![str_s(), num_s(), any()],
            discriminator: json!(null),
        });
        let mut visited = Vec::new();
        Walker::walk(&s, &mut |s| visited.push(s.kind().to_string()));
        assert_eq!(visited, vec!["or", "str", "num", "any"]);
    }

    #[test]
    fn walk_fn_visits_req_and_res() {
        let s = Schema::Fn(FnSchema {
            base: SchemaBase::default(),
            req: Box::new(str_s()),
            res: Box::new(num_s()),
        });
        let mut visited = Vec::new();
        Walker::walk(&s, &mut |s| visited.push(s.kind().to_string()));
        assert_eq!(visited, vec!["fn", "str", "num"]);
    }

    #[test]
    fn walk_fn_rx_visits_req_and_res() {
        let s = Schema::FnRx(FnRxSchema {
            base: SchemaBase::default(),
            req: Box::new(str_s()),
            res: Box::new(num_s()),
        });
        let mut visited = Vec::new();
        Walker::walk(&s, &mut |s| visited.push(s.kind().to_string()));
        assert_eq!(visited, vec!["fn$", "str", "num"]);
    }

    #[test]
    fn walk_module_visits_all_alias_values() {
        let s = Schema::Module(ModuleSchema {
            base: SchemaBase::default(),
            keys: vec![
                AliasSchema {
                    base: SchemaBase::default(),
                    key: "A".into(),
                    value: Box::new(str_s()),
                    optional: None,
                    pub_: None,
                },
                AliasSchema {
                    base: SchemaBase::default(),
                    key: "B".into(),
                    value: Box::new(num_s()),
                    optional: None,
                    pub_: None,
                },
            ],
        });
        let mut visited = Vec::new();
        Walker::walk(&s, &mut |s| visited.push(s.kind().to_string()));
        assert_eq!(visited, vec!["module", "str", "num"]);
    }

    #[test]
    fn walk_alias_visits_inner() {
        let s = Schema::Alias(AliasSchema {
            base: SchemaBase::default(),
            key: "MyAlias".into(),
            value: Box::new(str_s()),
            optional: None,
            pub_: None,
        });
        let mut visited = Vec::new();
        Walker::walk(&s, &mut |s| visited.push(s.kind().to_string()));
        // Alias kind is "key", inner is "str"
        assert_eq!(visited, vec!["key", "str"]);
    }

    #[test]
    fn walk_nested_structure() {
        // obj { arr(str) }
        let inner_arr = Schema::Arr(ArrSchema {
            type_: Some(Box::new(str_s())),
            ..Default::default()
        });
        let s = Schema::Obj(ObjSchema {
            keys: vec![KeySchema {
                base: SchemaBase::default(),
                key: "items".into(),
                value: Box::new(inner_arr),
                optional: None,
            }],
            ..Default::default()
        });
        let mut visited = Vec::new();
        Walker::walk(&s, &mut |s| visited.push(s.kind().to_string()));
        assert_eq!(visited, vec!["obj", "arr", "str"]);
    }
}
