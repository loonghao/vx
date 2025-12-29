//! Python dependency parsing
//!
//! Parses dependencies from pyproject.toml and requirements.txt

use crate::dependency::{Dependency, DependencySource};
use crate::ecosystem::Ecosystem;
use crate::error::AnalyzerResult;
use std::path::Path;

/// Parse dependencies from pyproject.toml
pub fn parse_pyproject_dependencies(content: &str, path: &Path) -> AnalyzerResult<Vec<Dependency>> {
    let mut deps = Vec::new();

    // Parse as TOML
    let doc: toml::Value = toml::from_str(content)?;

    // Parse [project.dependencies]
    if let Some(project) = doc.get("project") {
        if let Some(dependencies) = project.get("dependencies") {
            if let Some(deps_array) = dependencies.as_array() {
                for dep_str in deps_array {
                    if let Some(s) = dep_str.as_str() {
                        if let Some(dep) = parse_dependency_string(s, path, "project.dependencies")
                        {
                            deps.push(dep);
                        }
                    }
                }
            }
        }

        // Parse [project.optional-dependencies]
        if let Some(optional) = project.get("optional-dependencies") {
            if let Some(optional_table) = optional.as_table() {
                for (group, group_deps) in optional_table {
                    if let Some(deps_array) = group_deps.as_array() {
                        for dep_str in deps_array {
                            if let Some(s) = dep_str.as_str() {
                                let section = format!("project.optional-dependencies.{}", group);
                                if let Some(mut dep) = parse_dependency_string(s, path, &section) {
                                    dep.is_dev = group == "dev" || group == "test";
                                    deps.push(dep);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Parse [dependency-groups] (PEP 735)
    if let Some(groups) = doc.get("dependency-groups") {
        if let Some(groups_table) = groups.as_table() {
            for (group, group_deps) in groups_table {
                if let Some(deps_array) = group_deps.as_array() {
                    for dep_str in deps_array {
                        if let Some(s) = dep_str.as_str() {
                            let section = format!("dependency-groups.{}", group);
                            if let Some(mut dep) = parse_dependency_string(s, path, &section) {
                                dep.is_dev = group == "dev" || group == "test";
                                deps.push(dep);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(deps)
}

/// Parse a single dependency string (e.g., "requests>=2.0")
fn parse_dependency_string(s: &str, path: &Path, section: &str) -> Option<Dependency> {
    let s = s.trim();
    if s.is_empty() || s.starts_with('#') {
        return None;
    }

    // Handle extras: package[extra1,extra2]
    let name_end = s.find(['[', '>', '<', '=', '!', ';']).unwrap_or(s.len());

    let name = s[..name_end].trim().to_string();
    if name.is_empty() {
        return None;
    }

    // Extract version if present
    let version = if name_end < s.len() {
        let rest = &s[name_end..];
        if let Some(idx) = rest.find(|c: char| c.is_ascii_digit()) {
            let version_start = idx;
            let version_end = rest[version_start..]
                .find([',', ';', '['])
                .map(|i| version_start + i)
                .unwrap_or(rest.len());
            Some(rest[version_start..version_end].trim().to_string())
        } else {
            None
        }
    } else {
        None
    };

    let mut dep = Dependency::new(
        name,
        Ecosystem::Python,
        DependencySource::ConfigFile {
            path: path.to_path_buf(),
            section: section.to_string(),
        },
    );

    if let Some(v) = version {
        dep = dep.with_version(v);
    }

    Some(dep)
}

/// Parse requirements.txt
pub fn parse_requirements_txt(content: &str, path: &Path) -> AnalyzerResult<Vec<Dependency>> {
    let mut deps = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') || line.starts_with('-') {
            continue;
        }

        if let Some(dep) = parse_dependency_string(line, path, "requirements.txt") {
            deps.push(dep);
        }
    }

    Ok(deps)
}
