use crate::patch::Timestamp;
use serde_json::Value;

use super::types::{ConCell, Id, RuntimeNode};
use super::RuntimeModel;

impl RuntimeModel {
    pub(crate) fn root_object_field(&self, key: &str) -> Option<Timestamp> {
        let root = self.root?;
        match self.nodes.get(&root)? {
            RuntimeNode::Obj(entries) => entries
                .iter()
                .rev()
                .find(|(k, _)| k == key)
                .map(|(_, id)| (*id).into()),
            RuntimeNode::Val(child) => match self.nodes.get(child)? {
                RuntimeNode::Obj(entries) => entries
                    .iter()
                    .rev()
                    .find(|(k, _)| k == key)
                    .map(|(_, id)| (*id).into()),
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn root_id(&self) -> Option<Timestamp> {
        self.root.map(Into::into)
    }

    pub(crate) fn object_field(&self, obj: Timestamp, key: &str) -> Option<Timestamp> {
        match self.nodes.get(&Id::from(obj))? {
            RuntimeNode::Obj(entries) => entries
                .iter()
                .rev()
                .find(|(k, _)| k == key)
                .map(|(_, id)| (*id).into()),
            RuntimeNode::Val(child) => match self.nodes.get(child)? {
                RuntimeNode::Obj(entries) => entries
                    .iter()
                    .rev()
                    .find(|(k, _)| k == key)
                    .map(|(_, id)| (*id).into()),
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn node_is_string(&self, id: Timestamp) -> bool {
        matches!(self.nodes.get(&Id::from(id)), Some(RuntimeNode::Str(_)))
    }

    pub(crate) fn node_is_array(&self, id: Timestamp) -> bool {
        matches!(self.nodes.get(&Id::from(id)), Some(RuntimeNode::Arr(_)))
    }

    pub(crate) fn node_is_bin(&self, id: Timestamp) -> bool {
        matches!(self.nodes.get(&Id::from(id)), Some(RuntimeNode::Bin(_)))
    }

    pub(crate) fn node_is_object(&self, id: Timestamp) -> bool {
        matches!(self.nodes.get(&Id::from(id)), Some(RuntimeNode::Obj(_)))
    }

    pub(crate) fn node_is_vec(&self, id: Timestamp) -> bool {
        matches!(self.nodes.get(&Id::from(id)), Some(RuntimeNode::Vec(_)))
    }

    pub(crate) fn node_is_val(&self, id: Timestamp) -> bool {
        matches!(self.nodes.get(&Id::from(id)), Some(RuntimeNode::Val(_)))
    }

    pub(crate) fn val_child(&self, id: Timestamp) -> Option<Timestamp> {
        match self.nodes.get(&Id::from(id))? {
            RuntimeNode::Val(child) => Some((*child).into()),
            _ => None,
        }
    }

    pub(crate) fn resolve_string_node(&self, id: Timestamp) -> Option<Timestamp> {
        if self.node_is_string(id) {
            return Some(id);
        }
        let child = self.val_child(id)?;
        self.node_is_string(child).then_some(child)
    }

    pub(crate) fn find_string_node_by_value(&self, expected: &str) -> Option<Timestamp> {
        let mut found: Option<Id> = None;
        for (id, node) in &self.nodes {
            let RuntimeNode::Str(atoms) = node else {
                continue;
            };
            let mut s = String::new();
            for atom in atoms {
                if let Some(ch) = atom.ch {
                    s.push(ch);
                }
            }
            if s == expected {
                if found.is_some() {
                    return None;
                }
                found = Some(*id);
            }
        }
        found.map(Into::into)
    }

    pub(crate) fn resolve_bin_node(&self, id: Timestamp) -> Option<Timestamp> {
        if self.node_is_bin(id) {
            return Some(id);
        }
        let child = self.val_child(id)?;
        self.node_is_bin(child).then_some(child)
    }

    pub(crate) fn resolve_array_node(&self, id: Timestamp) -> Option<Timestamp> {
        if self.node_is_array(id) {
            return Some(id);
        }
        let child = self.val_child(id)?;
        self.node_is_array(child).then_some(child)
    }

    pub(crate) fn resolve_vec_node(&self, id: Timestamp) -> Option<Timestamp> {
        if self.node_is_vec(id) {
            return Some(id);
        }
        let child = self.val_child(id)?;
        self.node_is_vec(child).then_some(child)
    }

    pub(crate) fn resolve_object_node(&self, id: Timestamp) -> Option<Timestamp> {
        if self.node_is_object(id) {
            return Some(id);
        }
        let child = self.val_child(id)?;
        self.node_is_object(child).then_some(child)
    }

    pub(crate) fn vec_index_value(&self, id: Timestamp, index: u64) -> Option<Timestamp> {
        let node = self.nodes.get(&Id::from(id))?;
        if let RuntimeNode::Vec(map) = node {
            map.get(&index).copied().map(Into::into)
        } else {
            None
        }
    }

    pub(crate) fn vec_max_index(&self, id: Timestamp) -> Option<u64> {
        let node = self.nodes.get(&Id::from(id))?;
        if let RuntimeNode::Vec(map) = node {
            map.keys().copied().max()
        } else {
            None
        }
    }

    pub(crate) fn node_json_value(&self, id: Timestamp) -> Option<Value> {
        self.node_view(Id::from(id))
    }

    pub(crate) fn node_is_deleted_or_missing(&self, id: Timestamp) -> bool {
        let key = Id::from(id);
        match self.nodes.get(&key) {
            None => true,
            Some(RuntimeNode::Con(ConCell::Undef)) => true,
            _ => false,
        }
    }

    pub(crate) fn string_visible_slots(&self, id: Timestamp) -> Option<Vec<Timestamp>> {
        let node = self.nodes.get(&Id::from(id))?;
        if let RuntimeNode::Str(atoms) = node {
            let mut out = Vec::new();
            for atom in atoms {
                if atom.ch.is_some() {
                    out.push(atom.slot.into());
                }
            }
            Some(out)
        } else {
            None
        }
    }

    pub(crate) fn array_visible_slots(&self, id: Timestamp) -> Option<Vec<Timestamp>> {
        let node = self.nodes.get(&Id::from(id))?;
        if let RuntimeNode::Arr(atoms) = node {
            let mut out = Vec::new();
            for atom in atoms {
                if atom.value.is_some() {
                    out.push(atom.slot.into());
                }
            }
            Some(out)
        } else {
            None
        }
    }

    pub(crate) fn array_visible_values(&self, id: Timestamp) -> Option<Vec<Timestamp>> {
        let node = self.nodes.get(&Id::from(id))?;
        if let RuntimeNode::Arr(atoms) = node {
            let mut out = Vec::new();
            for atom in atoms {
                if let Some(value) = atom.value {
                    out.push(value.into());
                }
            }
            Some(out)
        } else {
            None
        }
    }

    pub(crate) fn bin_visible_slots(&self, id: Timestamp) -> Option<Vec<Timestamp>> {
        let node = self.nodes.get(&Id::from(id))?;
        if let RuntimeNode::Bin(atoms) = node {
            let mut out = Vec::new();
            for atom in atoms {
                if atom.byte.is_some() {
                    out.push(atom.slot.into());
                }
            }
            Some(out)
        } else {
            None
        }
    }
}
