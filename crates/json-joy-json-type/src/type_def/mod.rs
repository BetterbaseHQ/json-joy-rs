//! Type class hierarchy — Rust port of json-type/src/type/
//!
//! The TypeScript class hierarchy (AbsType<S> → concrete classes) is ported as:
//! - `TypeNode` enum: the sum type of all possible type classes
//! - Individual structs: `AnyType`, `NumType`, etc.
//! - `TypeBuilder`: factory for constructing TypeNode values

pub mod abs_type;
pub mod builder;
pub mod classes;
pub mod discriminator;
pub mod module_type;

pub use abs_type::BaseInfo;
pub use builder::TypeBuilder;
pub use classes::*;
pub use module_type::ModuleType;

use crate::schema::Schema;

/// The unified enum covering all type class instances.
///
/// Equivalent to the TypeScript union type `Type`.
#[derive(Debug, Clone)]
pub enum TypeNode {
    Any(AnyType),
    Bool(BoolType),
    Num(NumType),
    Str(StrType),
    Bin(BinType),
    Con(ConType),
    Arr(ArrType),
    Obj(ObjType),
    Map(MapType),
    Ref(RefType),
    Or(OrType),
    Fn(FnType),
    FnRx(FnRxType),
    Key(KeyType),
    Alias(AliasType),
}

impl TypeNode {
    /// Returns the kind string, matching the TypeScript `kind()` method.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Any(t) => t.kind(),
            Self::Bool(t) => t.kind(),
            Self::Num(t) => t.kind(),
            Self::Str(t) => t.kind(),
            Self::Bin(t) => t.kind(),
            Self::Con(t) => t.kind(),
            Self::Arr(t) => t.kind(),
            Self::Obj(t) => t.kind(),
            Self::Map(t) => t.kind(),
            Self::Ref(t) => t.kind(),
            Self::Or(t) => t.kind(),
            Self::Fn(t) => t.kind(),
            Self::FnRx(t) => t.kind(),
            Self::Key(t) => t.kind(),
            Self::Alias(t) => t.kind(),
        }
    }

    /// Returns the schema representation of this type node.
    pub fn get_schema(&self) -> Schema {
        match self {
            Self::Any(t) => t.get_schema(),
            Self::Bool(t) => t.get_schema(),
            Self::Num(t) => t.get_schema(),
            Self::Str(t) => t.get_schema(),
            Self::Bin(t) => t.get_schema(),
            Self::Con(t) => t.get_schema(),
            Self::Arr(t) => t.get_schema(),
            Self::Obj(t) => t.get_schema(),
            Self::Map(t) => t.get_schema(),
            Self::Ref(t) => t.get_schema(),
            Self::Or(t) => t.get_schema(),
            Self::Fn(t) => t.get_schema(),
            Self::FnRx(t) => t.get_schema(),
            Self::Key(t) => t.get_schema(),
            Self::Alias(t) => t.get_schema(),
        }
    }

    /// Returns a reference to the shared base info.
    pub fn base(&self) -> &BaseInfo {
        match self {
            Self::Any(t) => &t.base,
            Self::Bool(t) => &t.base,
            Self::Num(t) => &t.base,
            Self::Str(t) => &t.base,
            Self::Bin(t) => &t.base,
            Self::Con(t) => &t.base,
            Self::Arr(t) => &t.base,
            Self::Obj(t) => &t.base,
            Self::Map(t) => &t.base,
            Self::Ref(t) => &t.base,
            Self::Or(t) => &t.base,
            Self::Fn(t) => &t.base,
            Self::FnRx(t) => &t.base,
            Self::Key(t) => &t.base,
            Self::Alias(t) => &t.base,
        }
    }
}

impl std::fmt::Display for TypeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn t() -> TypeBuilder {
        TypeBuilder::new()
    }

    #[test]
    fn type_node_kind_any() {
        assert_eq!(t().any().kind(), "any");
    }

    #[test]
    fn type_node_kind_bool() {
        assert_eq!(t().bool().kind(), "bool");
    }

    #[test]
    fn type_node_kind_num() {
        assert_eq!(t().num().kind(), "num");
    }

    #[test]
    fn type_node_kind_str() {
        assert_eq!(t().str().kind(), "str");
    }

    #[test]
    fn type_node_kind_bin() {
        assert_eq!(t().bin().kind(), "bin");
    }

    #[test]
    fn type_node_kind_con() {
        assert_eq!(t().Const(json!(42), None).kind(), "con");
    }

    #[test]
    fn type_node_kind_arr() {
        assert_eq!(t().arr().kind(), "arr");
    }

    #[test]
    fn type_node_kind_obj() {
        assert_eq!(t().obj().kind(), "obj");
    }

    #[test]
    fn type_node_kind_map() {
        assert_eq!(t().map().kind(), "map");
    }

    #[test]
    fn type_node_kind_ref() {
        assert_eq!(t().Ref("Foo").kind(), "ref");
    }

    #[test]
    fn type_node_kind_or() {
        assert_eq!(t().Or(vec![t().str(), t().num()]).kind(), "or");
    }

    #[test]
    fn type_node_kind_fn() {
        assert_eq!(t().fn_().kind(), "fn");
    }

    #[test]
    fn type_node_kind_fn_rx() {
        assert_eq!(t().fn_rx().kind(), "fn$");
    }

    #[test]
    fn type_node_kind_key() {
        assert_eq!(t().Key("x", t().str()).kind(), "key");
    }

    #[test]
    fn type_node_display_matches_kind() {
        let node = t().str();
        assert_eq!(format!("{}", node), "str");

        let node = t().num();
        assert_eq!(format!("{}", node), "num");

        let node = t().bool();
        assert_eq!(format!("{}", node), "bool");
    }

    #[test]
    fn type_node_get_schema_any() {
        let s = t().any().get_schema();
        assert_eq!(s.kind(), "any");
    }

    #[test]
    fn type_node_get_schema_bool() {
        let s = t().bool().get_schema();
        assert_eq!(s.kind(), "bool");
    }

    #[test]
    fn type_node_get_schema_num() {
        let s = t().num().get_schema();
        assert_eq!(s.kind(), "num");
    }

    #[test]
    fn type_node_get_schema_str() {
        let s = t().str().get_schema();
        assert_eq!(s.kind(), "str");
    }

    #[test]
    fn type_node_get_schema_bin() {
        let s = t().bin().get_schema();
        assert_eq!(s.kind(), "bin");
    }

    #[test]
    fn type_node_get_schema_con() {
        let s = t().Const(json!("hello"), None).get_schema();
        assert_eq!(s.kind(), "con");
    }

    #[test]
    fn type_node_get_schema_arr() {
        let s = t().arr().get_schema();
        assert_eq!(s.kind(), "arr");
    }

    #[test]
    fn type_node_get_schema_obj() {
        let s = t().obj().get_schema();
        assert_eq!(s.kind(), "obj");
    }

    #[test]
    fn type_node_get_schema_map() {
        let s = t().map().get_schema();
        assert_eq!(s.kind(), "map");
    }

    #[test]
    fn type_node_get_schema_ref() {
        let s = t().Ref("Foo").get_schema();
        assert_eq!(s.kind(), "ref");
    }

    #[test]
    fn type_node_get_schema_fn() {
        let s = t().fn_().get_schema();
        assert_eq!(s.kind(), "fn");
    }

    #[test]
    fn type_node_get_schema_fn_rx() {
        let s = t().fn_rx().get_schema();
        assert_eq!(s.kind(), "fn$");
    }

    #[test]
    fn type_node_base_returns_base_info() {
        let node = t().any();
        let base = node.base();
        assert!(base.title.is_none());
    }

    #[test]
    fn type_node_base_for_all_variants() {
        // Ensure base() doesn't panic for any variant
        let nodes = vec![
            t().any(),
            t().bool(),
            t().num(),
            t().str(),
            t().bin(),
            t().Const(json!(null), None),
            t().arr(),
            t().obj(),
            t().map(),
            t().Ref("X"),
            t().Or(vec![t().str()]),
            t().fn_(),
            t().fn_rx(),
            t().Key("k", t().str()),
        ];
        for node in &nodes {
            let _ = node.base();
        }
    }
}
