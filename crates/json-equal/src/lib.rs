//! json-joy-json-equal - Deep equality comparison for JSON values.
//!
//! Mirrors the upstream `@jsonjoy.com/json-equal` package, extracted from
//! `@jsonjoy.com/util` in json-joy v18.0.0.
//!
//! Provides [`deep_equal`] for recursively comparing two [`serde_json::Value`]
//! instances with strict type checking.

mod deep_equal;

pub use deep_equal::deep_equal;
