//! Node.js dependency parsing
//!
//! Parses dependencies from package.json files.

use crate::dependency::{Dependency, DependencySource};
use crate::ecosystem::Ecosystem;
use crate::error::AnalyzerResult;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// Minimal package.json structure for dependency parsing
#[derive(Debug, Deserialize)]
pub struct PackageJson {
    #[serde(default)]
    pub dependencies: Option<HashMap<String, String>>,
    #[serde(default, rename = "devDependencies")]
    pub dev_dependencies: Option<HashMap<String, String>>,
    #[serde(default)]
    pub scripts: Option<HashMap<String, String>>,
}

/// Parse dependencies from package.json content
pub fn parse_package_json_dependencies(
    content: &str,
    package_json_path: &Path,
) -> AnalyzerResult<Vec<Dependency>> {
    let mut deps = Vec::new();
    let pkg: PackageJson = serde_json::from_str(content)?;

    // Parse dependencies
    for (name, version) in pkg.dependencies.unwrap_or_default() {
        let mut dep = Dependency::new(
            name,
            Ecosystem::NodeJs,
            DependencySource::ConfigFile {
                path: package_json_path.to_path_buf(),
                section: "dependencies".to_string(),
            },
        );
        dep = dep.with_version(version);
        deps.push(dep);
    }

    // Parse devDependencies
    for (name, version) in pkg.dev_dependencies.unwrap_or_default() {
        let mut dep = Dependency::new(
            name,
            Ecosystem::NodeJs,
            DependencySource::ConfigFile {
                path: package_json_path.to_path_buf(),
                section: "devDependencies".to_string(),
            },
        );
        dep = dep.with_version(version).as_dev();
        deps.push(dep);
    }

    Ok(deps)
}
