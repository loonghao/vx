//! Argument type definitions

use serde::{Deserialize, Serialize};

/// Argument type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArgType {
    /// String argument (default)
    #[default]
    String,
    /// Boolean flag
    Flag,
    /// Array of values
    Array,
    /// Number
    Number,
}

impl std::fmt::Display for ArgType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgType::String => write!(f, "string"),
            ArgType::Flag => write!(f, "flag"),
            ArgType::Array => write!(f, "array"),
            ArgType::Number => write!(f, "number"),
        }
    }
}

/// Argument definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgDef {
    /// Argument name
    pub name: String,
    /// Argument type
    #[serde(default, rename = "type")]
    pub arg_type: ArgType,
    /// Whether the argument is required
    #[serde(default)]
    pub required: bool,
    /// Default value
    #[serde(default)]
    pub default: Option<String>,
    /// Valid choices
    #[serde(default)]
    pub choices: Vec<String>,
    /// Environment variable to read from
    #[serde(default)]
    pub env: Option<String>,
    /// Short flag (single character)
    #[serde(default)]
    pub short: Option<char>,
    /// Help text
    #[serde(default)]
    pub help: Option<String>,
    /// Validation pattern (regex)
    #[serde(default)]
    pub pattern: Option<String>,
    /// Whether this is a positional argument
    #[serde(default)]
    pub positional: bool,
}

impl ArgDef {
    /// Create a new argument definition
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            arg_type: ArgType::String,
            required: false,
            default: None,
            choices: Vec::new(),
            env: None,
            short: None,
            help: None,
            pattern: None,
            positional: false,
        }
    }

    /// Set argument type
    pub fn arg_type(mut self, arg_type: ArgType) -> Self {
        self.arg_type = arg_type;
        self
    }

    /// Set as required
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Set default value
    pub fn default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }

    /// Set valid choices
    pub fn choices(mut self, choices: Vec<impl Into<String>>) -> Self {
        self.choices = choices.into_iter().map(|c| c.into()).collect();
        self
    }

    /// Set environment variable
    pub fn env(mut self, env: impl Into<String>) -> Self {
        self.env = Some(env.into());
        self
    }

    /// Set short flag
    pub fn short(mut self, short: char) -> Self {
        self.short = Some(short);
        self
    }

    /// Set help text
    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Set validation pattern
    pub fn pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    /// Set as positional argument
    pub fn positional(mut self, positional: bool) -> Self {
        self.positional = positional;
        self
    }

    /// Check if this argument has a default or is optional
    pub fn is_optional(&self) -> bool {
        !self.required || self.default.is_some()
    }

    /// Get the long flag name (--name)
    pub fn long_flag(&self) -> String {
        format!("--{}", self.name.replace('_', "-"))
    }

    /// Get the short flag if available (-x)
    pub fn short_flag(&self) -> Option<String> {
        self.short.map(|c| format!("-{}", c))
    }
}

/// Parsed argument value
#[derive(Debug, Clone, PartialEq)]
pub enum ArgValue {
    /// String value
    String(String),
    /// Boolean value
    Bool(bool),
    /// Array of values
    Array(Vec<String>),
    /// Number value
    Number(f64),
    /// No value (for optional args)
    None,
}

impl ArgValue {
    /// Get as string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            ArgValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as bool
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ArgValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Get as array
    pub fn as_array(&self) -> Option<&[String]> {
        match self {
            ArgValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Get as number
    pub fn as_number(&self) -> Option<f64> {
        match self {
            ArgValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Check if value is none
    pub fn is_none(&self) -> bool {
        matches!(self, ArgValue::None)
    }

    /// Convert to string representation
    pub fn to_string_value(&self) -> String {
        match self {
            ArgValue::String(s) => s.clone(),
            ArgValue::Bool(b) => b.to_string(),
            ArgValue::Array(arr) => arr.join(" "),
            ArgValue::Number(n) => n.to_string(),
            ArgValue::None => String::new(),
        }
    }
}

impl std::fmt::Display for ArgValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgValue::String(s) => write!(f, "{}", s),
            ArgValue::Bool(b) => write!(f, "{}", b),
            ArgValue::Array(arr) => write!(f, "[{}]", arr.join(", ")),
            ArgValue::Number(n) => write!(f, "{}", n),
            ArgValue::None => write!(f, ""),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arg_def_builder() {
        let arg = ArgDef::new("environment")
            .required(true)
            .choices(vec!["dev", "prod"])
            .help("Target environment")
            .short('e');

        assert_eq!(arg.name, "environment");
        assert!(arg.required);
        assert_eq!(arg.choices, vec!["dev", "prod"]);
        assert_eq!(arg.short, Some('e'));
    }

    #[test]
    fn test_arg_def_flags() {
        let arg = ArgDef::new("dry_run").short('n');

        assert_eq!(arg.long_flag(), "--dry-run");
        assert_eq!(arg.short_flag(), Some("-n".to_string()));
    }

    #[test]
    fn test_arg_value_conversions() {
        let string_val = ArgValue::String("hello".to_string());
        assert_eq!(string_val.as_string(), Some("hello"));
        assert_eq!(string_val.as_bool(), None);

        let bool_val = ArgValue::Bool(true);
        assert_eq!(bool_val.as_bool(), Some(true));
        assert_eq!(bool_val.as_string(), None);

        let array_val = ArgValue::Array(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(
            array_val.as_array(),
            Some(&["a".to_string(), "b".to_string()][..])
        );
    }
}
