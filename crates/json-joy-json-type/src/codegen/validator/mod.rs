pub mod types;
pub mod validator;

pub use types::{ErrorMode, ValidatorOptions, ValidationResult};
pub use validator::validate;
