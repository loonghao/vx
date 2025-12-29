//! Error types for argument parsing

use thiserror::Error;

/// Result type for argument operations
pub type ArgResult<T> = Result<T, ArgError>;

/// Argument parsing errors
#[derive(Debug, Error)]
pub enum ArgError {
    /// Missing required argument
    #[error("Missing required argument: {name}")]
    MissingRequired {
        /// Argument name
        name: String,
        /// Help text
        help: Option<String>,
    },

    /// Invalid argument value
    #[error("Invalid value for '{name}': {reason}")]
    InvalidValue {
        /// Argument name
        name: String,
        /// Reason for invalidity
        reason: String,
        /// Expected values
        expected: Option<Vec<String>>,
    },

    /// Unknown argument
    #[error("Unknown argument: {name}")]
    UnknownArgument {
        /// Argument name
        name: String,
        /// Similar arguments (for suggestions)
        similar: Vec<String>,
    },

    /// Invalid argument type
    #[error("Expected {expected} for '{name}', got {actual}")]
    TypeMismatch {
        /// Argument name
        name: String,
        /// Expected type
        expected: String,
        /// Actual value
        actual: String,
    },

    /// Too many positional arguments
    #[error("Too many arguments: expected at most {max}, got {actual}")]
    TooManyArguments {
        /// Maximum expected
        max: usize,
        /// Actual count
        actual: usize,
    },

    /// Variable not found during interpolation
    #[error("Variable not found: {name}")]
    VariableNotFound {
        /// Variable name
        name: String,
    },

    /// Command execution failed during interpolation
    #[error("Command failed: {command}")]
    CommandFailed {
        /// Command that failed
        command: String,
        /// Error message
        message: String,
    },

    /// Invalid pattern in argument definition
    #[error("Invalid pattern for '{name}': {pattern}")]
    InvalidPattern {
        /// Argument name
        name: String,
        /// Invalid pattern
        pattern: String,
        /// Error message
        message: String,
    },

    /// Circular variable reference
    #[error("Circular variable reference: {chain}")]
    CircularReference {
        /// Reference chain
        chain: String,
    },
}

impl ArgError {
    /// Create a missing required error
    pub fn missing_required(name: impl Into<String>, help: Option<String>) -> Self {
        Self::MissingRequired {
            name: name.into(),
            help,
        }
    }

    /// Create an invalid value error
    pub fn invalid_value(
        name: impl Into<String>,
        reason: impl Into<String>,
        expected: Option<Vec<String>>,
    ) -> Self {
        Self::InvalidValue {
            name: name.into(),
            reason: reason.into(),
            expected,
        }
    }

    /// Create an unknown argument error
    pub fn unknown_argument(name: impl Into<String>, similar: Vec<String>) -> Self {
        Self::UnknownArgument {
            name: name.into(),
            similar,
        }
    }

    /// Create a type mismatch error
    pub fn type_mismatch(
        name: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self::TypeMismatch {
            name: name.into(),
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    /// Create a variable not found error
    pub fn variable_not_found(name: impl Into<String>) -> Self {
        Self::VariableNotFound { name: name.into() }
    }

    /// Get user-friendly error message with suggestions
    pub fn user_message(&self) -> String {
        match self {
            Self::MissingRequired { name, help } => {
                let mut msg = format!("Missing required argument: {}", name);
                if let Some(h) = help {
                    msg.push_str(&format!("\n  {}", h));
                }
                msg
            }
            Self::InvalidValue {
                name,
                reason,
                expected,
            } => {
                let mut msg = format!("Invalid value for '{}': {}", name, reason);
                if let Some(choices) = expected {
                    msg.push_str(&format!("\n  Valid choices: {}", choices.join(", ")));
                }
                msg
            }
            Self::UnknownArgument { name, similar } => {
                let mut msg = format!("Unknown argument: {}", name);
                if !similar.is_empty() {
                    msg.push_str(&format!("\n  Did you mean: {}?", similar.join(", ")));
                }
                msg
            }
            _ => self.to_string(),
        }
    }
}
