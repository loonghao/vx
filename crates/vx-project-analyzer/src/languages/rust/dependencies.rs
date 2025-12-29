//! Rust dependency parsing
//!
//! Parses dependencies from Cargo.toml files.

use crate::dependency::{Dependency, DependencySource};
use crate::ecosystem::Ecosystem;
use crate::error::AnalyzerResult;
use std::path::Path;

/// Parse dependencies from Cargo.toml content
pub fn parse_cargo_dependencies(
    content: &str,
    cargo_toml_path: &Path,
) -> AnalyzerResult<Vec<Dependency>> {
    let mut deps = Vec::new();
    let doc: toml::Value = toml::from_str(content)?;

    // Parse [dependencies]
    if let Some(dependencies) = doc.get("dependencies") {
        if let Some(deps_table) = dependencies.as_table() {
            for (name, value) in deps_table {
                let version = extract_version(value);
                let mut dep = Dependency::new(
                    name.clone(),
                    Ecosystem::Rust,
                    DependencySource::ConfigFile {
                        path: cargo_toml_path.to_path_buf(),
                        section: "dependencies".to_string(),
                    },
                );
                if let Some(v) = version {
                    dep = dep.with_version(v);
                }
                deps.push(dep);
            }
        }
    }

    // Parse [dev-dependencies]
    if let Some(dev_deps) = doc.get("dev-dependencies") {
        if let Some(deps_table) = dev_deps.as_table() {
            for (name, value) in deps_table {
                let version = extract_version(value);
                let mut dep = Dependency::new(
                    name.clone(),
                    Ecosystem::Rust,
                    DependencySource::ConfigFile {
                        path: cargo_toml_path.to_path_buf(),
                        section: "dev-dependencies".to_string(),
                    },
                );
                if let Some(v) = version {
                    dep = dep.with_version(v);
                }
                dep = dep.as_dev();
                deps.push(dep);
            }
        }
    }

    // Parse [build-dependencies]
    if let Some(build_deps) = doc.get("build-dependencies") {
        if let Some(deps_table) = build_deps.as_table() {
            for (name, value) in deps_table {
                let version = extract_version(value);
                let mut dep = Dependency::new(
                    name.clone(),
                    Ecosystem::Rust,
                    DependencySource::ConfigFile {
                        path: cargo_toml_path.to_path_buf(),
                        section: "build-dependencies".to_string(),
                    },
                );
                if let Some(v) = version {
                    dep = dep.with_version(v);
                }
                deps.push(dep);
            }
        }
    }

    Ok(deps)
}

/// Extract version from Cargo.toml dependency value
fn extract_version(value: &toml::Value) -> Option<String> {
    match value {
        toml::Value::String(s) => Some(s.clone()),
        toml::Value::Table(t) => t.get("version").and_then(|v| v.as_str().map(String::from)),
        _ => None,
    }
}
