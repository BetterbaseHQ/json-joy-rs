use serde_json::{Map, Number, Value};

use super::types::{ConCell, Id, RuntimeNode};
use super::RuntimeModel;

impl RuntimeModel {
    pub(super) fn node_view(&self, id: Id) -> Option<Value> {
        match self.nodes.get(&id)? {
            RuntimeNode::Con(ConCell::Json(v)) => Some(v.clone()),
            RuntimeNode::Con(ConCell::Ref(id)) => Some(rid_to_json(*id)),
            RuntimeNode::Con(ConCell::Undef) => None,
            RuntimeNode::Val(child) => Some(self.node_view(*child).unwrap_or(Value::Null)),
            RuntimeNode::Obj(entries) => {
                let mut out = Map::new();
                for (k, v) in entries {
                    if let Some(val) = self.node_view(*v) {
                        out.insert(k.clone(), val);
                    }
                }
                Some(Value::Object(out))
            }
            RuntimeNode::Vec(map) => {
                let max = map.keys().copied().max().unwrap_or(0);
                let mut out = vec![Value::Null; max as usize + 1];
                for (i, id) in map {
                    if let Some(v) = self.node_view(*id) {
                        out[*i as usize] = v;
                    }
                }
                Some(Value::Array(out))
            }
            RuntimeNode::Str(atoms) => {
                let mut s = String::new();
                for a in atoms {
                    if let Some(ch) = a.ch {
                        s.push(ch);
                    }
                }
                Some(Value::String(s))
            }
            RuntimeNode::Bin(atoms) => {
                let mut map = Map::new();
                let mut idx = 0usize;
                for a in atoms {
                    if let Some(b) = a.byte {
                        map.insert(idx.to_string(), Value::Number(Number::from(b)));
                        idx += 1;
                    }
                }
                Some(Value::Object(map))
            }
            RuntimeNode::Arr(atoms) => {
                let mut out = Vec::new();
                for a in atoms {
                    if let Some(value_id) = a.value {
                        if let Some(v) = self.node_view(value_id) {
                            out.push(v);
                        }
                    }
                }
                Some(Value::Array(out))
            }
        }
    }
}

fn rid_to_json(id: Id) -> Value {
    let mut m = Map::new();
    m.insert("sid".to_string(), Value::Number(Number::from(id.sid)));
    m.insert("time".to_string(), Value::Number(Number::from(id.time)));
    Value::Object(m)
}
