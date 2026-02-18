//! Validator output types.

/// Result of running a validator.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    /// Validation passed.
    Ok,
    /// Validation failed, boolean mode (just true/false).
    BoolError,
    /// Validation failed, string mode (JSON-encoded path).
    StringError(String),
    /// Validation failed, object mode (detailed error).
    ObjectError {
        code: String,
        errno: u8,
        message: String,
        path: Vec<serde_json::Value>,
    },
}

impl ValidationResult {
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }

    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
}

/// Mode controlling how validation errors are reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ErrorMode {
    /// Returns a boolean (true = error).
    Boolean,
    /// Returns a JSON-encoded error path string.
    String,
    /// Returns a structured error object.
    #[default]
    Object,
}

/// Options for the validator.
#[derive(Debug, Clone, Default)]
pub struct ValidatorOptions {
    pub errors: ErrorMode,
    pub skip_object_extra_fields_check: bool,
    pub unsafe_mode: bool,
}
