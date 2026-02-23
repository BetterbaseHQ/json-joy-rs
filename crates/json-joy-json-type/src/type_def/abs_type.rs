//! Base type node and shared info.
//!
//! Upstream reference: json-type/src/type/classes/AbsType.ts

use serde_json::Value;
use std::sync::Arc;

use super::module_type::ModuleType;

/// Validator function type: receives a JSON value, returns None if ok or Some(error) if invalid.
pub type ValidatorFn = Arc<dyn Fn(&Value) -> Option<String> + Send + Sync>;

/// Shared fields for all type nodes (metadata + validators + system reference).
#[derive(Clone, Default)]
pub struct BaseInfo {
    pub title: Option<String>,
    pub intro: Option<String>,
    pub description: Option<String>,
    pub default: Option<Value>,
    pub examples: Vec<Value>,
    pub validators: Vec<(ValidatorFn, Option<String>)>,
    pub system: Option<Arc<ModuleType>>,
}

impl std::fmt::Debug for BaseInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BaseInfo")
            .field("title", &self.title)
            .field("intro", &self.intro)
            .field("description", &self.description)
            .field("default", &self.default)
            .field("validators_count", &self.validators.len())
            .finish()
    }
}

impl BaseInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_system(mut self, system: Option<Arc<ModuleType>>) -> Self {
        self.system = system;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn base_info_default_all_none() {
        let b = BaseInfo::default();
        assert!(b.title.is_none());
        assert!(b.intro.is_none());
        assert!(b.description.is_none());
        assert!(b.default.is_none());
        assert!(b.examples.is_empty());
        assert!(b.validators.is_empty());
        assert!(b.system.is_none());
    }

    #[test]
    fn base_info_new_same_as_default() {
        let b1 = BaseInfo::new();
        let b2 = BaseInfo::default();
        assert_eq!(b1.title, b2.title);
        assert_eq!(b1.validators.len(), b2.validators.len());
    }

    #[test]
    fn base_info_with_system_sets_system() {
        let module = Arc::new(ModuleType::new());
        let b = BaseInfo::new().with_system(Some(module.clone()));
        assert!(b.system.is_some());
    }

    #[test]
    fn base_info_with_system_none() {
        let b = BaseInfo::new().with_system(None);
        assert!(b.system.is_none());
    }

    #[test]
    fn base_info_debug_shows_validator_count() {
        let b = BaseInfo {
            title: Some("Test".into()),
            validators: vec![(
                Arc::new(|_v: &Value| -> Option<String> { None }),
                Some("rule".into()),
            )],
            ..Default::default()
        };
        let debug = format!("{:?}", b);
        assert!(debug.contains("validators_count: 1"));
        assert!(debug.contains("Test"));
    }

    #[test]
    fn base_info_debug_without_validators() {
        let b = BaseInfo::default();
        let debug = format!("{:?}", b);
        assert!(debug.contains("validators_count: 0"));
    }

    #[test]
    fn base_info_clone() {
        let b = BaseInfo {
            title: Some("Clone me".into()),
            description: Some("A description".into()),
            default: Some(json!(42)),
            examples: vec![json!("example")],
            ..Default::default()
        };
        let b2 = b.clone();
        assert_eq!(b2.title, Some("Clone me".into()));
        assert_eq!(b2.description, Some("A description".into()));
        assert_eq!(b2.default, Some(json!(42)));
        assert_eq!(b2.examples.len(), 1);
    }

    #[test]
    fn validator_fn_can_be_called() {
        let validator: ValidatorFn = Arc::new(|v: &Value| {
            if v.is_null() {
                Some("Cannot be null".into())
            } else {
                None
            }
        });
        assert!(validator(&json!(null)).is_some());
        assert!(validator(&json!(42)).is_none());
    }
}
