//! Extension dependency management
//!
//! This module handles dependencies between extensions.

use crate::error::{ExtensionError, ExtensionResult};
use crate::{Extension, ExtensionConfig, ExtensionDiscovery};
use std::collections::{HashMap, HashSet};
use tracing::debug;

/// Dependency specification
#[derive(Debug, Clone)]
pub struct ExtensionDependency {
    /// Name of the required extension
    pub name: String,
    /// Version constraint (optional)
    pub version: Option<String>,
    /// Whether this dependency is optional
    pub optional: bool,
}

impl ExtensionDependency {
    /// Parse a dependency string
    ///
    /// Formats:
    /// - `extension-name`
    /// - `extension-name >= 1.0.0`
    /// - `extension-name ~= 1.0`
    /// - `?extension-name` (optional)
    pub fn parse(s: &str) -> Self {
        let s = s.trim();

        // Check for optional marker
        let (optional, s) = if let Some(rest) = s.strip_prefix('?') {
            (true, rest.trim())
        } else {
            (false, s)
        };

        // Split name and version constraint
        let parts: Vec<&str> = s.splitn(2, char::is_whitespace).collect();
        let name = parts[0].to_string();
        let version = parts.get(1).map(|v| v.trim().to_string());

        Self {
            name,
            version,
            optional,
        }
    }

    /// Check if a version satisfies this dependency
    pub fn satisfies(&self, version: &str) -> bool {
        match &self.version {
            None => true, // No constraint means any version is ok
            Some(constraint) => {
                // Simple version comparison for now
                // TODO: Implement proper semver comparison
                Self::check_version_constraint(version, constraint)
            }
        }
    }

    fn check_version_constraint(version: &str, constraint: &str) -> bool {
        let constraint = constraint.trim();

        // Handle different operators
        if let Some(required) = constraint.strip_prefix(">=") {
            Self::compare_versions(version, required.trim()) >= std::cmp::Ordering::Equal
        } else if let Some(required) = constraint.strip_prefix("<=") {
            Self::compare_versions(version, required.trim()) <= std::cmp::Ordering::Equal
        } else if let Some(required) = constraint.strip_prefix('>') {
            Self::compare_versions(version, required.trim()) == std::cmp::Ordering::Greater
        } else if let Some(required) = constraint.strip_prefix('<') {
            Self::compare_versions(version, required.trim()) == std::cmp::Ordering::Less
        } else if let Some(required) = constraint.strip_prefix("==") {
            version == required.trim()
        } else if let Some(required) = constraint.strip_prefix("~=") {
            // Compatible release (e.g., ~=1.4 means >=1.4, <2.0)
            Self::compatible_release(version, required.trim())
        } else if let Some(required) = constraint.strip_prefix('^') {
            // Caret requirement (e.g., ^1.4 means >=1.4.0, <2.0.0)
            Self::caret_requirement(version, required.trim())
        } else {
            // Exact match
            version == constraint
        }
    }

    fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
        let a_parts: Vec<u32> = a.split('.').filter_map(|s| s.parse().ok()).collect();
        let b_parts: Vec<u32> = b.split('.').filter_map(|s| s.parse().ok()).collect();

        for i in 0..std::cmp::max(a_parts.len(), b_parts.len()) {
            let a_val = a_parts.get(i).copied().unwrap_or(0);
            let b_val = b_parts.get(i).copied().unwrap_or(0);

            match a_val.cmp(&b_val) {
                std::cmp::Ordering::Equal => continue,
                other => return other,
            }
        }

        std::cmp::Ordering::Equal
    }

    fn compatible_release(version: &str, required: &str) -> bool {
        let req_parts: Vec<u32> = required.split('.').filter_map(|s| s.parse().ok()).collect();

        if req_parts.is_empty() {
            return true;
        }

        let ver_parts: Vec<u32> = version.split('.').filter_map(|s| s.parse().ok()).collect();

        // Must be >= required
        if Self::compare_versions(version, required) == std::cmp::Ordering::Less {
            return false;
        }

        // Major version must match
        if ver_parts.first() != req_parts.first() {
            return false;
        }

        true
    }

    fn caret_requirement(version: &str, required: &str) -> bool {
        let req_parts: Vec<u32> = required.split('.').filter_map(|s| s.parse().ok()).collect();

        if req_parts.is_empty() {
            return true;
        }

        let ver_parts: Vec<u32> = version.split('.').filter_map(|s| s.parse().ok()).collect();

        // Must be >= required
        if Self::compare_versions(version, required) == std::cmp::Ordering::Less {
            return false;
        }

        // For ^0.x.y, minor version must match
        // For ^x.y.z (x > 0), major version must match
        let major = req_parts.first().copied().unwrap_or(0);
        if major == 0 {
            // ^0.x.y means >=0.x.y, <0.(x+1).0
            if let (Some(&ver_minor), Some(&req_minor)) = (ver_parts.get(1), req_parts.get(1)) {
                return ver_minor == req_minor;
            }
        } else {
            // ^x.y.z means >=x.y.z, <(x+1).0.0
            if let Some(&ver_major) = ver_parts.first() {
                return ver_major == major;
            }
        }

        true
    }
}

/// Dependency resolver for extensions
pub struct DependencyResolver {
    discovery: ExtensionDiscovery,
}

impl DependencyResolver {
    /// Create a new dependency resolver
    pub fn new() -> ExtensionResult<Self> {
        Ok(Self {
            discovery: ExtensionDiscovery::new()?,
        })
    }

    /// Resolve dependencies for an extension
    pub async fn resolve(&self, extension_name: &str) -> ExtensionResult<DependencyResolution> {
        let extensions = self.discovery.discover_all().await?;
        let ext_map: HashMap<_, _> = extensions.iter().map(|e| (e.name.as_str(), e)).collect();

        let target =
            ext_map
                .get(extension_name)
                .ok_or_else(|| ExtensionError::ExtensionNotFound {
                    name: extension_name.to_string(),
                    available: extensions.iter().map(|e| e.name.clone()).collect(),
                    searched_paths: vec![],
                })?;

        let mut resolution = DependencyResolution::new(extension_name.to_string());
        let mut visited = HashSet::new();
        let mut stack = vec![extension_name.to_string()];

        self.resolve_recursive(target, &ext_map, &mut resolution, &mut visited, &mut stack)?;

        Ok(resolution)
    }

    fn resolve_recursive(
        &self,
        extension: &Extension,
        ext_map: &HashMap<&str, &Extension>,
        resolution: &mut DependencyResolution,
        visited: &mut HashSet<String>,
        stack: &mut Vec<String>,
    ) -> ExtensionResult<()> {
        if visited.contains(&extension.name) {
            return Ok(());
        }

        visited.insert(extension.name.clone());

        // Parse dependencies from config
        let deps = self.parse_dependencies(&extension.config);

        for dep in deps {
            // Check for circular dependency
            if stack.contains(&dep.name) {
                resolution.add_circular(stack.clone(), dep.name.clone());
                continue;
            }

            // Find the dependency
            match ext_map.get(dep.name.as_str()) {
                Some(dep_ext) => {
                    // Check version constraint
                    if !dep.satisfies(&dep_ext.config.extension.version) {
                        resolution.add_conflict(
                            dep.name.clone(),
                            dep.version.clone().unwrap_or_default(),
                            dep_ext.config.extension.version.clone(),
                        );
                        continue;
                    }

                    // Add to resolved
                    resolution.add_resolved(dep_ext.name.clone(), dep_ext.path.clone());

                    // Resolve transitive dependencies
                    stack.push(dep.name.clone());
                    self.resolve_recursive(dep_ext, ext_map, resolution, visited, stack)?;
                    stack.pop();
                }
                None => {
                    if dep.optional {
                        debug!("Optional dependency '{}' not found", dep.name);
                    } else {
                        resolution.add_missing(dep.name.clone(), dep.version.clone());
                    }
                }
            }
        }

        Ok(())
    }

    fn parse_dependencies(&self, _config: &ExtensionConfig) -> Vec<ExtensionDependency> {
        // Dependencies can be specified in the runtime section
        // or in a dedicated [dependencies] section (future)

        // For now, we use the runtime.dependencies field
        // which typically contains Python/Node packages
        // Extension dependencies would be in a separate field

        // TODO: Add [extension.dependencies] section to config

        Vec::new()
    }

    /// Check if all dependencies are satisfied for an extension
    pub async fn check_dependencies(&self, extension_name: &str) -> ExtensionResult<bool> {
        let resolution = self.resolve(extension_name).await?;
        Ok(resolution.is_satisfied())
    }
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new().expect("Failed to create dependency resolver")
    }
}

/// Result of dependency resolution
#[derive(Debug, Clone)]
pub struct DependencyResolution {
    /// The extension being resolved
    pub target: String,
    /// Resolved dependencies (name -> path)
    pub resolved: HashMap<String, std::path::PathBuf>,
    /// Missing dependencies (name -> version constraint)
    pub missing: Vec<MissingDependency>,
    /// Version conflicts
    pub conflicts: Vec<VersionConflict>,
    /// Circular dependencies
    pub circular: Vec<CircularDependency>,
}

impl DependencyResolution {
    fn new(target: String) -> Self {
        Self {
            target,
            resolved: HashMap::new(),
            missing: Vec::new(),
            conflicts: Vec::new(),
            circular: Vec::new(),
        }
    }

    fn add_resolved(&mut self, name: String, path: std::path::PathBuf) {
        self.resolved.insert(name, path);
    }

    fn add_missing(&mut self, name: String, version: Option<String>) {
        self.missing.push(MissingDependency { name, version });
    }

    fn add_conflict(&mut self, name: String, required: String, found: String) {
        self.conflicts.push(VersionConflict {
            name,
            required,
            found,
        });
    }

    fn add_circular(&mut self, chain: Vec<String>, target: String) {
        self.circular.push(CircularDependency { chain, target });
    }

    /// Check if all dependencies are satisfied
    pub fn is_satisfied(&self) -> bool {
        self.missing.is_empty() && self.conflicts.is_empty() && self.circular.is_empty()
    }

    /// Get a human-readable summary
    pub fn summary(&self) -> String {
        let mut summary = String::new();

        if self.is_satisfied() {
            summary.push_str(&format!(
                "All dependencies satisfied ({} resolved)\n",
                self.resolved.len()
            ));
        } else {
            if !self.missing.is_empty() {
                summary.push_str("Missing dependencies:\n");
                for dep in &self.missing {
                    let version = dep
                        .version
                        .as_ref()
                        .map(|v| format!(" {}", v))
                        .unwrap_or_default();
                    summary.push_str(&format!("  - {}{}\n", dep.name, version));
                }
            }

            if !self.conflicts.is_empty() {
                summary.push_str("Version conflicts:\n");
                for conflict in &self.conflicts {
                    summary.push_str(&format!(
                        "  - {}: required {}, found {}\n",
                        conflict.name, conflict.required, conflict.found
                    ));
                }
            }

            if !self.circular.is_empty() {
                summary.push_str("Circular dependencies:\n");
                for circular in &self.circular {
                    summary.push_str(&format!(
                        "  - {} -> {}\n",
                        circular.chain.join(" -> "),
                        circular.target
                    ));
                }
            }
        }

        summary
    }
}

/// A missing dependency
#[derive(Debug, Clone)]
pub struct MissingDependency {
    /// Dependency name
    pub name: String,
    /// Version constraint
    pub version: Option<String>,
}

/// A version conflict
#[derive(Debug, Clone)]
pub struct VersionConflict {
    /// Dependency name
    pub name: String,
    /// Required version
    pub required: String,
    /// Found version
    pub found: String,
}

/// A circular dependency
#[derive(Debug, Clone)]
pub struct CircularDependency {
    /// Dependency chain leading to the cycle
    pub chain: Vec<String>,
    /// Target that creates the cycle
    pub target: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_dependency() {
        let dep = ExtensionDependency::parse("my-extension");
        assert_eq!(dep.name, "my-extension");
        assert!(dep.version.is_none());
        assert!(!dep.optional);
    }

    #[test]
    fn test_parse_dependency_with_version() {
        let dep = ExtensionDependency::parse("my-extension >= 1.0.0");
        assert_eq!(dep.name, "my-extension");
        assert_eq!(dep.version, Some(">= 1.0.0".to_string()));
        assert!(!dep.optional);
    }

    #[test]
    fn test_parse_optional_dependency() {
        let dep = ExtensionDependency::parse("?optional-ext");
        assert_eq!(dep.name, "optional-ext");
        assert!(dep.optional);
    }

    #[test]
    fn test_version_comparison() {
        assert!(
            ExtensionDependency::compare_versions("1.0.0", "1.0.0") == std::cmp::Ordering::Equal
        );
        assert!(
            ExtensionDependency::compare_versions("2.0.0", "1.0.0") == std::cmp::Ordering::Greater
        );
        assert!(
            ExtensionDependency::compare_versions("1.0.0", "2.0.0") == std::cmp::Ordering::Less
        );
        assert!(
            ExtensionDependency::compare_versions("1.2.0", "1.1.0") == std::cmp::Ordering::Greater
        );
        assert!(
            ExtensionDependency::compare_versions("1.0.1", "1.0.0") == std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn test_satisfies_no_constraint() {
        let dep = ExtensionDependency::parse("my-ext");
        assert!(dep.satisfies("1.0.0"));
        assert!(dep.satisfies("2.0.0"));
    }

    #[test]
    fn test_satisfies_gte() {
        let dep = ExtensionDependency::parse("my-ext >= 1.0.0");
        assert!(dep.satisfies("1.0.0"));
        assert!(dep.satisfies("1.0.1"));
        assert!(dep.satisfies("2.0.0"));
        assert!(!dep.satisfies("0.9.0"));
    }

    #[test]
    fn test_satisfies_lt() {
        let dep = ExtensionDependency::parse("my-ext < 2.0.0");
        assert!(dep.satisfies("1.0.0"));
        assert!(dep.satisfies("1.9.9"));
        assert!(!dep.satisfies("2.0.0"));
        assert!(!dep.satisfies("3.0.0"));
    }

    #[test]
    fn test_satisfies_exact() {
        let dep = ExtensionDependency::parse("my-ext == 1.0.0");
        assert!(dep.satisfies("1.0.0"));
        assert!(!dep.satisfies("1.0.1"));
        assert!(!dep.satisfies("0.9.0"));
    }

    #[test]
    fn test_caret_requirement() {
        // ^1.2.3 means >=1.2.3, <2.0.0
        assert!(ExtensionDependency::caret_requirement("1.2.3", "1.2.3"));
        assert!(ExtensionDependency::caret_requirement("1.9.9", "1.2.3"));
        assert!(!ExtensionDependency::caret_requirement("2.0.0", "1.2.3"));
        assert!(!ExtensionDependency::caret_requirement("1.2.2", "1.2.3"));
    }

    #[test]
    fn test_compatible_release() {
        // ~=1.4 means >=1.4, <2.0
        assert!(ExtensionDependency::compatible_release("1.4.0", "1.4"));
        assert!(ExtensionDependency::compatible_release("1.9.0", "1.4"));
        assert!(!ExtensionDependency::compatible_release("2.0.0", "1.4"));
        assert!(!ExtensionDependency::compatible_release("1.3.0", "1.4"));
    }

    #[test]
    fn test_dependency_resolution_satisfied() {
        let mut resolution = DependencyResolution::new("test".to_string());
        resolution.add_resolved(
            "dep1".to_string(),
            std::path::PathBuf::from("/path/to/dep1"),
        );
        assert!(resolution.is_satisfied());
    }

    #[test]
    fn test_dependency_resolution_missing() {
        let mut resolution = DependencyResolution::new("test".to_string());
        resolution.add_missing("missing-dep".to_string(), Some(">= 1.0".to_string()));
        assert!(!resolution.is_satisfied());
    }
}
