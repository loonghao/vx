//! File parsers for different project configuration formats

use crate::{error::ConfigError, Result};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Generic function to read and parse TOML files
pub fn read_and_parse_toml(path: &PathBuf, file_type: &str) -> Result<toml::Value> {
    let content = fs::read_to_string(path).map_err(|e| ConfigError::Io {
        message: format!("Failed to read {}: {}", file_type, e),
        source: e,
    })?;
    toml::from_str(&content).map_err(|e| ConfigError::Parse {
        message: e.to_string(),
        file_type: file_type.to_string(),
    })
}

/// Generic function to read and parse JSON files
pub fn read_and_parse_json(path: &PathBuf, file_type: &str) -> Result<JsonValue> {
    let content = fs::read_to_string(path).map_err(|e| ConfigError::Io {
        message: format!("Failed to read {}: {}", file_type, e),
        source: e,
    })?;
    serde_json::from_str(&content).map_err(|e| ConfigError::Parse {
        message: e.to_string(),
        file_type: file_type.to_string(),
    })
}

/// Generic function to read text files
pub fn read_text_file(path: &PathBuf, file_type: &str) -> Result<String> {
    fs::read_to_string(path).map_err(|e| ConfigError::Io {
        message: format!("Failed to read {}: {}", file_type, e),
        source: e,
    })
}

/// Parse pyproject.toml for tool version requirements
pub fn parse_pyproject_toml(path: &PathBuf) -> Result<HashMap<String, String>> {
    let parsed: toml::Value = read_and_parse_toml(path, "pyproject.toml")?;
    let mut versions = HashMap::new();

    // Check for Python version requirement
    if let Some(project) = parsed.get("project") {
        if let Some(requires_python) = project.get("requires-python") {
            if let Some(version_str) = requires_python.as_str() {
                // Parse version requirement like ">=3.8" to "3.8"
                let version = parse_version_requirement(version_str);
                versions.insert("python".to_string(), version);
            }
        }
    }

    // Check for tool.uv configuration
    if let Some(tool) = parsed.get("tool") {
        if let Some(uv) = tool.get("uv") {
            if let Some(version) = uv.get("version") {
                if let Some(version_str) = version.as_str() {
                    versions.insert("uv".to_string(), version_str.to_string());
                }
            }
        }
    }

    Ok(versions)
}

/// Parse Cargo.toml for tool version requirements
pub fn parse_cargo_toml(path: &PathBuf) -> Result<HashMap<String, String>> {
    let parsed: toml::Value = read_and_parse_toml(path, "Cargo.toml")?;
    let mut versions = HashMap::new();

    // Check for Rust version requirement
    if let Some(package) = parsed.get("package") {
        if let Some(rust_version) = package.get("rust-version") {
            if let Some(version_str) = rust_version.as_str() {
                versions.insert("rust".to_string(), version_str.to_string());
            }
        }
    }

    Ok(versions)
}

/// Parse package.json for tool version requirements
pub fn parse_package_json(path: &PathBuf) -> Result<HashMap<String, String>> {
    let parsed: JsonValue = read_and_parse_json(path, "package.json")?;
    let mut versions = HashMap::new();

    // Check for Node.js version requirement in engines
    if let Some(engines) = parsed.get("engines") {
        if let Some(node_version) = engines.get("node") {
            if let Some(version_str) = node_version.as_str() {
                let version = parse_version_requirement(version_str);
                versions.insert("node".to_string(), version);
            }
        }
        if let Some(npm_version) = engines.get("npm") {
            if let Some(version_str) = npm_version.as_str() {
                let version = parse_version_requirement(version_str);
                versions.insert("npm".to_string(), version);
            }
        }
    }

    Ok(versions)
}

/// Parse go.mod for Go version requirement
pub fn parse_go_mod(path: &PathBuf) -> Result<HashMap<String, String>> {
    let content = read_text_file(path, "go.mod")?;
    let mut versions = HashMap::new();

    // Parse go.mod format: "go 1.21"
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("go ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                versions.insert("go".to_string(), parts[1].to_string());
            }
            break;
        }
    }

    Ok(versions)
}

/// Parse version requirement string to extract version
pub fn parse_version_requirement(requirement: &str) -> String {
    // Remove common prefixes like >=, ^, ~, etc.
    let cleaned = requirement
        .trim_start_matches(">=")
        .trim_start_matches("^")
        .trim_start_matches("~")
        .trim_start_matches("=")
        .trim_start_matches(">");

    // Take the first version number found
    if let Some(space_pos) = cleaned.find(' ') {
        cleaned[..space_pos].to_string()
    } else {
        cleaned.to_string()
    }
}
