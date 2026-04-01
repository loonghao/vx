//! Input validation for Agent DX (Google Cloud CLI best practices)
//!
//! AI agents can hallucinate adversarial inputs — treat all CLI inputs as
//! untrusted, similar to web API inputs. This module implements the defensive
//! checks described in Justin Poehnelt's "Rewrite Your CLI for AI Agents":
//!
//! - Path traversal: `../../.ssh`
//! - Control characters: ASCII < 0x20
//! - Embedded query parameters: `fileId?fields=name`
//! - URL-encoded tricks: `%2e%2e` (double encoding)
//! - Null bytes in strings

use anyhow::{Result, bail};

/// Validate a tool/runtime name from user input.
///
/// Accepts: alphanumeric, `-`, `_`, `.`, `@`, `:` (for ecosystem:pkg syntax)
/// Rejects: control characters, path separators, query params, null bytes
pub fn validate_runtime_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Runtime name cannot be empty");
    }

    // Reject null bytes
    if name.contains('\0') {
        bail!("Invalid runtime name: contains null byte");
    }

    // Reject control characters (ASCII < 0x20, excluding common whitespace handled elsewhere)
    if name.chars().any(|c| (c as u32) < 0x20) {
        bail!(
            "Invalid runtime name '{}': contains control characters",
            name
        );
    }

    // Reject path traversal patterns
    if name.contains("..") {
        bail!(
            "Invalid runtime name '{}': path traversal not allowed",
            name
        );
    }

    // Reject embedded query parameters (AI hallucination pattern: `node?version=20`)
    if name.contains('?') || name.contains('#') {
        bail!(
            "Invalid runtime name '{}': embedded query parameters not allowed",
            name
        );
    }

    // Reject URL-encoded percent sequences (double-encoding: `%2e%2e`)
    if name.contains('%') {
        bail!(
            "Invalid runtime name '{}': URL-encoded characters not allowed",
            name
        );
    }

    // Reject shell metacharacters that could enable injection
    for ch in [';', '&', '|', '`', '$', '(', ')', '{', '}', '<', '>'] {
        if name.contains(ch) {
            bail!(
                "Invalid runtime name '{}': shell metacharacter '{}' not allowed",
                name,
                ch
            );
        }
    }

    Ok(())
}

/// Validate a file path provided by an agent/user.
///
/// Sandboxes paths to the current working directory — agents cannot escape
/// to system paths like `../../.ssh/id_rsa`.
pub fn validate_safe_path(path: &str) -> Result<()> {
    if path.is_empty() {
        bail!("Path cannot be empty");
    }

    // Reject null bytes
    if path.contains('\0') {
        bail!("Invalid path: contains null byte");
    }

    // Reject control characters
    if path.chars().any(|c| (c as u32) < 0x20) {
        bail!("Invalid path '{}': contains control characters", path);
    }

    // Reject embedded query parameters (AI confuses paths with URLs)
    if path.contains('?') || path.contains('#') {
        bail!(
            "Invalid path '{}': embedded query parameters not allowed",
            path
        );
    }

    // Reject URL-encoded percent sequences
    if path.contains('%') {
        bail!(
            "Invalid path '{}': URL-encoded characters not allowed",
            path
        );
    }

    // Normalize and check for path traversal
    let normalized = std::path::Path::new(path);
    for component in normalized.components() {
        if component == std::path::Component::ParentDir {
            bail!("Invalid path '{}': path traversal (..) not allowed", path);
        }
    }

    Ok(())
}

/// Validate a version string.
///
/// Versions should be semver-like: `1.2.3`, `latest`, `lts`, `^1.2`, `>=1.0`.
/// Rejects control characters, path separators, and injection patterns.
pub fn validate_version(version: &str) -> Result<()> {
    if version.is_empty() {
        bail!("Version cannot be empty");
    }

    // Reject null bytes
    if version.contains('\0') {
        bail!("Invalid version: contains null byte");
    }

    // Reject control characters
    if version.chars().any(|c| (c as u32) < 0x20) {
        bail!("Invalid version '{}': contains control characters", version);
    }

    // Reject shell metacharacters
    for ch in [';', '&', '|', '`', '$', '(', ')', '{', '}'] {
        if version.contains(ch) {
            bail!(
                "Invalid version '{}': shell metacharacter '{}' not allowed",
                version,
                ch
            );
        }
    }

    // Reject URL-encoded percent sequences
    if version.contains('%') {
        bail!(
            "Invalid version '{}': URL-encoded characters not allowed",
            version
        );
    }

    Ok(())
}

/// Validate a field mask (comma-separated field names from `--fields`).
///
/// Field names should be simple identifiers: `name`, `version`, `path`, etc.
pub fn validate_field_name(field: &str) -> Result<()> {
    if field.is_empty() {
        bail!("Field name cannot be empty");
    }

    // Only allow alphanumeric and underscore in field names
    if !field
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        bail!(
            "Invalid field name '{}': only alphanumeric, '-', '_' allowed",
            field
        );
    }

    Ok(())
}

/// Validate all field names in a field mask.
pub fn validate_fields(fields: &[String]) -> Result<()> {
    for field in fields {
        validate_field_name(field)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_runtime_names() {
        assert!(validate_runtime_name("node").is_ok());
        assert!(validate_runtime_name("node@20").is_ok());
        assert!(validate_runtime_name("npm:typescript").is_ok());
        assert!(validate_runtime_name("go").is_ok());
        assert!(validate_runtime_name("uv").is_ok());
    }

    #[test]
    fn test_invalid_runtime_names() {
        // Path traversal
        assert!(validate_runtime_name("../../.ssh").is_err());
        assert!(validate_runtime_name("node..etc").is_err());

        // Embedded query params (AI hallucination)
        assert!(validate_runtime_name("node?version=20").is_err());
        assert!(validate_runtime_name("node#latest").is_err());

        // URL encoding
        assert!(validate_runtime_name("%2enode").is_err());

        // Control characters
        assert!(validate_runtime_name("node\x00").is_err());
        assert!(validate_runtime_name("node\x01").is_err());

        // Shell injection
        assert!(validate_runtime_name("node;rm -rf").is_err());
        assert!(validate_runtime_name("node|cat /etc/passwd").is_err());
    }

    #[test]
    fn test_path_traversal_rejection() {
        assert!(validate_safe_path("../../.ssh/id_rsa").is_err());
        assert!(validate_safe_path("some/normal/path").is_ok());
        assert!(validate_safe_path("relative/path.txt").is_ok());
    }

    #[test]
    fn test_valid_versions() {
        assert!(validate_version("1.2.3").is_ok());
        assert!(validate_version("latest").is_ok());
        assert!(validate_version("lts").is_ok());
        assert!(validate_version("^1.2.0").is_ok());
        assert!(validate_version(">=18").is_ok());
        assert!(validate_version("v20.0.0").is_ok());
    }

    #[test]
    fn test_invalid_versions() {
        assert!(validate_version("1.2.3;rm -rf /").is_err());
        assert!(validate_version("1.2.3%20evil").is_err());
    }
}
