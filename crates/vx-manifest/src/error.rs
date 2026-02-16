//! Error types for manifest operations

use thiserror::Error;

/// Errors that can occur when working with manifests
#[derive(Debug, Error)]
pub enum ManifestError {
    /// IO error reading manifest file
    #[error("Failed to read manifest file: {0}")]
    Io(#[from] std::io::Error),

    /// TOML parsing error
    #[error("Failed to parse manifest TOML: {0}")]
    Parse(#[from] toml::de::Error),

    /// TOML parsing error with provider context
    #[error("Failed to parse manifest for provider '{provider}': {source}")]
    ParseWithContext {
        /// Provider name for context
        provider: String,
        /// The underlying TOML parse error
        source: toml::de::Error,
    },

    /// Validation error
    #[error("Manifest validation error: {0}")]
    Validation(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Structured diagnostic with hints
    #[error("{message}")]
    Diagnostic {
        /// Main error message
        message: String,
        /// Provider name (if known)
        provider: Option<String>,
        /// Hint for how to fix the error
        hint: Option<String>,
        /// Field that caused the error (if known)
        field: Option<String>,
    },
}

impl ManifestError {
    /// Create a parse error with provider context and diagnostic hints
    pub fn parse_with_context(provider: &str, source: toml::de::Error) -> Self {
        Self::ParseWithContext {
            provider: provider.to_string(),
            source,
        }
    }

    /// Create a diagnostic error with hints
    pub fn diagnostic(message: impl Into<String>) -> DiagnosticBuilder {
        DiagnosticBuilder {
            message: message.into(),
            provider: None,
            hint: None,
            field: None,
        }
    }

    /// Get a human-readable diagnostic message with hints for common errors
    pub fn diagnostic_message(&self) -> String {
        match self {
            Self::ParseWithContext { provider, source } => {
                let raw = source.to_string();
                let mut parts = vec![format!("  Provider: {}\n  Error: {}", provider, raw)];

                // Detect common error patterns and provide hints
                if let Some(hint) = detect_parse_hint(&raw) {
                    parts.push(format!("  💡 Hint: {}", hint));
                }

                parts.join("\n")
            }
            Self::Parse(source) => {
                let raw = source.to_string();
                let mut parts = vec![format!("  Error: {}", raw)];
                if let Some(hint) = detect_parse_hint(&raw) {
                    parts.push(format!("  💡 Hint: {}", hint));
                }
                parts.join("\n")
            }
            Self::Diagnostic {
                message,
                provider,
                hint,
                field,
            } => {
                let mut parts = Vec::new();
                if let Some(p) = provider {
                    parts.push(format!("  Provider: {}", p));
                }
                if let Some(f) = field {
                    parts.push(format!("  Field: {}", f));
                }
                parts.push(format!("  Error: {}", message));
                if let Some(h) = hint {
                    parts.push(format!("  💡 Hint: {}", h));
                }
                parts.join("\n")
            }
            _ => self.to_string(),
        }
    }
}

/// Builder for Diagnostic errors
pub struct DiagnosticBuilder {
    message: String,
    provider: Option<String>,
    hint: Option<String>,
    field: Option<String>,
}

impl DiagnosticBuilder {
    /// Set the provider name
    pub fn provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    /// Set a hint for fixing the error
    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Set the field that caused the error
    pub fn field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    /// Build the ManifestError
    pub fn build(self) -> ManifestError {
        ManifestError::Diagnostic {
            message: self.message,
            provider: self.provider,
            hint: self.hint,
            field: self.field,
        }
    }
}

/// Detect common parse errors and provide helpful hints
fn detect_parse_hint(error_message: &str) -> Option<String> {
    // Unknown variant for ecosystem field
    if error_message.contains("unknown variant") && error_message.contains("expected one of") {
        if error_message.contains("ecosystem")
            || error_message.contains("cpp")
            || error_message.contains("node")
        {
            return Some(
                "Check the 'ecosystem' field value. Supported values: nodejs, python, rust, go, ruby, java, dotnet, devtools, container, cloud, ai, cpp, zig, system"
                    .to_string(),
            );
        }
        // Generic unknown variant hint
        return Some(
            "An enum field has an unrecognized value. Check the field name shown in the error and verify the value matches the expected variants."
                .to_string(),
        );
    }

    // Type mismatch (e.g., when = { os = "windows" } instead of when = "*")
    if error_message.contains("invalid type: map, expected a string") {
        return Some(
            "A field expected a string but got a table/map. For constraint rules, use 'when = \"*\"' (string) with a separate 'platform = \"windows\"' field instead of 'when = { os = \"windows\" }'."
                .to_string(),
        );
    }

    // Missing field errors
    if error_message.contains("missing field") {
        let field_name = extract_quoted_value(error_message, "missing field");
        if let Some(name) = field_name {
            return Some(format!(
                "Required field '{}' is missing. Check your provider.toml for this field.",
                name
            ));
        }
    }

    // download_type errors
    if error_message.contains("download_type")
        || (error_message.contains("unknown variant") && error_message.contains("archive"))
    {
        return Some(
            "Check the 'download_type' field. Supported values: archive, binary, installer, git_clone (note: use snake_case, not kebab-case)"
                .to_string(),
        );
    }

    // serde rename_all snake_case issues
    if error_message.contains("unknown variant") {
        let value = extract_quoted_value(error_message, "unknown variant");
        if let Some(v) = &value {
            if v.contains('-') {
                return Some(format!(
                    "Value '{}' uses kebab-case but snake_case is expected. Try '{}' instead.",
                    v,
                    v.replace('-', "_")
                ));
            }
        }
    }

    // Invalid type: integer expected string (common when version numbers are unquoted)
    if error_message.contains("invalid type: integer, expected a string") {
        return Some(
            "A string field received an integer value. Make sure version numbers and similar values are quoted, e.g., version = \"1.0\" instead of version = 1.0"
                .to_string(),
        );
    }

    None
}

/// Extract a quoted value from an error message following a pattern
fn extract_quoted_value(message: &str, prefix: &str) -> Option<String> {
    if let Some(pos) = message.find(prefix) {
        let after = &message[pos + prefix.len()..];
        // Look for `foo` or 'foo' patterns
        if let Some(start) = after.find('`').or_else(|| after.find('\'')) {
            let rest = &after[start + 1..];
            if let Some(end) = rest.find('`').or_else(|| rest.find('\'')) {
                return Some(rest[..end].to_string());
            }
        }
    }
    None
}
