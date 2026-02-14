//! Configuration validation

use crate::types::VxConfig;

/// Validation warning (non-fatal)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ValidationWarning {
    pub message: String,
}

/// Validation result
#[derive(Debug, Default)]
pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Check if validation passed (no errors)
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// Add an error
    pub fn error(&mut self, message: impl Into<String>) {
        self.errors.push(message.into());
    }

    /// Add a warning
    pub fn warn(&mut self, message: impl Into<String>) {
        self.warnings.push(message.into());
    }
}

/// Validate configuration
pub fn validate_config(config: &VxConfig) -> ValidationResult {
    let mut result = ValidationResult::default();

    // Validate min_version if specified
    if let Some(min_version) = &config.min_version
        && let Err(e) = validate_version_requirement(min_version)
    {
        result.error(e);
    }

    // Validate tool versions
    for name in config.tools.keys() {
        validate_tool_name(name, &mut result);
    }

    // Validate scripts
    for name in config.scripts.keys() {
        validate_script_name(name, &mut result);
    }

    // Validate services
    for (name, service) in &config.services {
        validate_service(name, service, &mut result);
    }

    result
}

/// Validate version requirement
fn validate_version_requirement(version: &str) -> Result<(), String> {
    // Parse version requirement
    let parts: Vec<&str> = version.split('.').collect();
    if parts.is_empty() || parts.len() > 3 {
        return Err(format!("Invalid version format: {}", version));
    }

    // Check each part is a valid number
    for part in parts {
        if part.parse::<u32>().is_err() {
            return Err(format!("Invalid version number: {}", part));
        }
    }

    Ok(())
}

/// Validate tool name
fn validate_tool_name(name: &str, result: &mut ValidationResult) {
    // Check for valid characters
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        result.warn(format!("Tool name '{}' contains unusual characters", name));
    }
}

/// Validate script name
fn validate_script_name(name: &str, result: &mut ValidationResult) {
    // Check for valid characters
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ':')
    {
        result.warn(format!(
            "Script name '{}' contains unusual characters",
            name
        ));
    }
}

/// Validate service configuration
fn validate_service(
    name: &str,
    service: &crate::types::ServiceConfig,
    result: &mut ValidationResult,
) {
    // Service must have either image or command
    if service.image.is_none() && service.command.is_none() {
        result.warn(format!(
            "Service '{}' has neither 'image' nor 'command' specified",
            name
        ));
    }

    // Validate port format
    for port in &service.ports {
        if !is_valid_port_mapping(port) {
            result.warn(format!(
                "Service '{}' has invalid port mapping: {}",
                name, port
            ));
        }
    }
}

/// Check if port mapping is valid (e.g., "8080:80" or "8080")
fn is_valid_port_mapping(port: &str) -> bool {
    let parts: Vec<&str> = port.split(':').collect();
    match parts.len() {
        1 => parts[0].parse::<u16>().is_ok(),
        2 => parts[0].parse::<u16>().is_ok() && parts[1].parse::<u16>().is_ok(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_config_str;

    #[test]
    fn test_validate_valid_config() {
        let content = r#"
min_version = "0.6.0"

[tools]
node = "20"

[scripts]
dev = "npm run dev"
"#;
        let config = parse_config_str(content).unwrap();
        let result = validate_config(&config);
        assert!(result.is_ok());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validate_invalid_version() {
        let content = r#"
min_version = "invalid"
"#;
        let config = parse_config_str(content).unwrap();
        let result = validate_config(&config);
        assert!(!result.is_ok());
    }

    #[test]
    fn test_validate_service_warning() {
        let content = r#"
[services.empty]
# No image or command
"#;
        let config = parse_config_str(content).unwrap();
        let result = validate_config(&config);
        assert!(!result.warnings.is_empty());
    }
}
