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
