use thiserror::Error;

/// Errors raised by the pure domain layer.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CoreError {
    /// A required value was empty or malformed.
    #[error("invalid value: {0}")]
    Invalid(String),
    /// A rule could not be evaluated because its threshold is incomplete.
    #[error("invalid alert rule: {0}")]
    InvalidRule(String),
    /// A date or number could not be parsed.
    #[error("parse error: {0}")]
    Parse(String),
}

/// Result alias for the core crate.
pub type Result<T> = std::result::Result<T, CoreError>;
