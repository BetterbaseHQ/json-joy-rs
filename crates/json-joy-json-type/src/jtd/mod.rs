//! JSON Type Definition (JTD) converter.
//!
//! Upstream reference: json-type/src/jtd/

pub mod converter;
pub mod types;

pub use converter::to_jtd_form;
pub use types::JtdForm;
