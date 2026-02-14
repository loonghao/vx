//! Runtime mapping
//!
//! This module provides a comprehensive mapping of runtimes to their dependencies,
//! supporting various ecosystems (Node.js, Python, Rust, Go, etc.)
//!
//! ## RFC 0017: Declarative RuntimeMap
//!
//! RuntimeMap is built entirely from provider.toml files using `from_manifests()`.
//! This ensures a single source of truth for runtime specifications.

use crate::runtime_spec::{Ecosystem, RuntimeDependency, RuntimeSpec};
use std::collections::HashMap;
use vx_manifest::{ProviderManifest, RuntimeDef, SystemDepTypeDef};

/// A registry of runtime specifications and their dependencies
#[derive(Debug, Default)]
pub struct RuntimeMap {
    /// Map of runtime name to specification
    runtimes: HashMap<String, RuntimeSpec>,
    /// Map of alias to primary runtime name
    aliases: HashMap<String, String>,
    /// Map of runtime name to original RuntimeDef (for version-specific constraint queries)
    runtime_defs: HashMap<String, RuntimeDef>,
}

impl RuntimeMap {
    /// Create an empty runtime map (for testing)
    pub fn empty() -> Self {
        Self::default()
    }

    /// Build RuntimeMap entirely from provider manifests
    ///
    /// This is the standard way to create a RuntimeMap, using provider.toml
    /// files as the single source of truth for runtime specifications.
    ///
    /// # Example
    /// ```ignore
    /// let manifests = load_manifests_with_overrides();
    /// let map = RuntimeMap::from_manifests(&manifests);
    /// ```
    pub fn from_manifests(manifests: &[ProviderManifest]) -> Self {
        let mut map = Self::default();

        for manifest in manifests {
            let ecosystem = manifest
                .provider
                .ecosystem
                .map(Self::convert_ecosystem)
                .unwrap_or_default();

            for runtime in &manifest.runtimes {
                let spec = Self::runtime_def_to_spec(runtime, ecosystem);
                // Store the original RuntimeDef for version-specific constraint queries
                map.runtime_defs
                    .insert(runtime.name.clone(), runtime.clone());
                map.register(spec);
            }
        }

        map
    }

    /// Convert a RuntimeDef from manifest to RuntimeSpec
    fn runtime_def_to_spec(runtime: &RuntimeDef, ecosystem: Ecosystem) -> RuntimeSpec {
        let mut spec =
            RuntimeSpec::new(&runtime.name, runtime.description.as_deref().unwrap_or(""));

        // Basic fields
        spec.executable = Some(runtime.executable.clone());
        spec.aliases = runtime.aliases.clone();
        spec.command_prefix = runtime.command_prefix.clone();
        spec.ecosystem = ecosystem;

        // Priority and auto_installable from RFC 0018
        if let Some(priority) = runtime.priority {
            spec.priority = priority;
        }
        if let Some(auto_installable) = runtime.auto_installable {
            spec.auto_installable = auto_installable;
        }

        // Convert bundled_with to dependency
        if let Some(ref bundled_with) = runtime.bundled_with {
            let dep = RuntimeDependency::required(
                bundled_with.clone(),
                format!("{} is bundled with {}", runtime.name, bundled_with),
            )
            .provided_by(bundled_with.clone());
            spec.dependencies.push(dep);
        }

        // Convert managed_by to dependency
        if let Some(ref managed_by) = runtime.managed_by {
            let dep = RuntimeDependency::required(
                managed_by.clone(),
                format!("{} is managed by {}", runtime.name, managed_by),
            )
            .provided_by(managed_by.clone());
            spec.dependencies.push(dep);
        }

        // Convert constraints to dependencies
        // For now, we take the "*" (any version) constraints as default dependencies
        for constraint in &runtime.constraints {
            // Only process universal constraints for now
            if constraint.when == "*" {
                for req in &constraint.requires {
                    let mut dep = RuntimeDependency::required(
                        &req.runtime,
                        req.reason.as_deref().unwrap_or("Required dependency"),
                    );
                    // Parse version constraint
                    if !req.version.is_empty() && req.version != "*" {
                        // Try to extract min version from constraint like ">=12"
                        if let Some(min) = Self::extract_min_version(&req.version) {
                            dep = dep.with_min_version(min);
                        }
                    }
                    if let Some(ref recommended) = req.recommended {
                        dep = dep.with_recommended_version(recommended.clone());
                    }
                    // Set provided_by if specified (for proxy-managed runtimes like yarn 2.x+)
                    if let Some(ref provided_by) = req.provided_by {
                        dep = dep.provided_by(provided_by.clone());
                    }
                    spec.dependencies.push(dep);
                }
            }
        }

        // Environment variables from RFC 0018
        if let Some(ref env_config) = runtime.env_config {
            spec.env_vars = env_config.get_vars_for_version(&runtime.name);
            spec.env_config = Some(env_config.clone());
        }

        // RFC 0021: Convert system_deps to RuntimeDependency
        // Only include Runtime type dependencies that match the current platform
        if let Some(ref system_deps) = runtime.system_deps {
            let current_platform = Self::current_platform_name();

            for dep in &system_deps.pre_depends {
                // Only process Runtime type dependencies (vx-managed runtimes)
                if dep.dep_type != SystemDepTypeDef::Runtime {
                    continue;
                }

                // Check platform filter
                if !dep.platforms.is_empty() && !dep.platforms.iter().any(|p| p == current_platform)
                {
                    continue;
                }

                // Create RuntimeDependency
                let reason = dep.reason.as_deref().unwrap_or("System dependency");
                let mut runtime_dep = if dep.optional {
                    RuntimeDependency::optional(&dep.id, reason)
                } else {
                    RuntimeDependency::required(&dep.id, reason)
                };

                if let Some(ref version) = dep.version
                    && let Some(min) = Self::extract_min_version(version)
                {
                    runtime_dep = runtime_dep.with_min_version(min);
                }

                spec.dependencies.push(runtime_dep);
            }
        }

        spec
    }

    /// Extract minimum version from a version constraint like ">=12" or ">=12, <23"
    fn extract_min_version(constraint: &str) -> Option<String> {
        // Simple parsing for common patterns
        for part in constraint.split(',') {
            let part = part.trim();
            if let Some(version) = part.strip_prefix(">=") {
                return Some(version.trim().to_string());
            }
        }
        None
    }

    /// Get the current platform name (for system_deps platform filtering)
    fn current_platform_name() -> &'static str {
        #[cfg(target_os = "windows")]
        {
            "windows"
        }
        #[cfg(target_os = "macos")]
        {
            "macos"
        }
        #[cfg(target_os = "linux")]
        {
            "linux"
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            "unknown"
        }
    }

    /// Convert vx_manifest::Ecosystem to vx_resolver::Ecosystem
    fn convert_ecosystem(eco: vx_manifest::Ecosystem) -> Ecosystem {
        match eco {
            vx_manifest::Ecosystem::NodeJs => Ecosystem::Node,
            vx_manifest::Ecosystem::Python => Ecosystem::Python,
            vx_manifest::Ecosystem::Rust => Ecosystem::Rust,
            vx_manifest::Ecosystem::Go => Ecosystem::Go,
            vx_manifest::Ecosystem::Java => Ecosystem::Java,
            // All other ecosystems map to Generic for now
            _ => Ecosystem::Generic,
        }
    }

    /// Register a runtime specification
    pub fn register(&mut self, spec: RuntimeSpec) {
        // Register aliases
        for alias in &spec.aliases {
            self.aliases.insert(alias.clone(), spec.name.clone());
        }
        self.runtimes.insert(spec.name.clone(), spec);
    }

    /// Get a runtime specification by name or alias
    pub fn get(&self, name: &str) -> Option<&RuntimeSpec> {
        // First try direct lookup
        if let Some(spec) = self.runtimes.get(name) {
            return Some(spec);
        }
        // Then try alias lookup
        if let Some(primary) = self.aliases.get(name) {
            return self.runtimes.get(primary);
        }
        None
    }

    /// Check if a runtime is known
    pub fn contains(&self, name: &str) -> bool {
        self.runtimes.contains_key(name) || self.aliases.contains_key(name)
    }

    /// Get all runtime names
    pub fn runtime_names(&self) -> Vec<&str> {
        self.runtimes.keys().map(|s| s.as_str()).collect()
    }

    /// Get runtimes by ecosystem
    pub fn by_ecosystem(&self, ecosystem: Ecosystem) -> Vec<&RuntimeSpec> {
        self.runtimes
            .values()
            .filter(|spec| spec.ecosystem == ecosystem)
            .collect()
    }

    /// Resolve the primary runtime name from a name or alias
    pub fn resolve_name<'a>(&'a self, name: &'a str) -> Option<&'a str> {
        if self.runtimes.contains_key(name) {
            Some(name)
        } else {
            self.aliases.get(name).map(|s| s.as_str())
        }
    }

    /// Get version-specific dependencies for a runtime
    ///
    /// This method queries the original RuntimeDef constraints to find
    /// dependencies that apply to a specific version. This is useful for
    /// runtimes like Yarn 2.x+ where different versions have different
    /// dependency requirements (e.g., Yarn 2+ requires Node.js via corepack).
    ///
    /// Returns a list of RuntimeDependency for the given version.
    pub fn get_dependencies_for_version(
        &self,
        runtime_name: &str,
        version: &str,
    ) -> Vec<RuntimeDependency> {
        // First, resolve the name (in case it's an alias)
        let resolved_name = self.resolve_name(runtime_name).unwrap_or(runtime_name);

        // Get the original RuntimeDef
        let Some(runtime_def) = self.runtime_defs.get(resolved_name) else {
            return vec![];
        };

        // Get version-specific dependencies from constraints
        let deps = runtime_def.get_dependencies_for_version(version);

        deps.iter()
            .map(|dep_def| {
                let mut dep = RuntimeDependency::required(
                    &dep_def.runtime,
                    dep_def.reason.as_deref().unwrap_or("Required dependency"),
                );

                // Parse version constraint
                if !dep_def.version.is_empty()
                    && dep_def.version != "*"
                    && let Some(min) = Self::extract_min_version(&dep_def.version)
                {
                    dep = dep.with_min_version(min);
                }

                if let Some(ref recommended) = dep_def.recommended {
                    dep = dep.with_recommended_version(recommended.clone());
                }

                // Set provided_by if specified
                if let Some(ref provided_by) = dep_def.provided_by {
                    dep = dep.provided_by(provided_by.clone());
                }

                dep
            })
            .collect()
    }

    /// Get the parent runtime (provided_by) for a specific version
    ///
    /// This is a convenience method that returns the first dependency
    /// with `provided_by` set for the given version. Useful for determining
    /// which runtime needs to be installed to provide the requested runtime.
    pub fn get_parent_runtime_for_version(
        &self,
        runtime_name: &str,
        version: &str,
    ) -> Option<String> {
        // First check static dependencies
        if let Some(spec) = self.get(runtime_name)
            && let Some(parent) = spec
                .dependencies
                .iter()
                .find(|dep| dep.required && dep.provided_by.is_some())
                .and_then(|dep| dep.provided_by.clone())
        {
            return Some(parent);
        }

        // Then check version-specific dependencies
        self.get_dependencies_for_version(runtime_name, version)
            .iter()
            .find(|dep| dep.required && dep.provided_by.is_some())
            .and_then(|dep| dep.provided_by.clone())
    }

    /// Get detection system_paths for a runtime (from provider.toml detection config)
    ///
    /// These are glob patterns pointing to known installation locations
    /// (e.g., Visual Studio paths for cl.exe). Used by the Resolver to
    /// find executables that are not in the vx store or system PATH.
    pub fn get_detection_system_paths(&self, runtime_name: &str) -> Vec<String> {
        let resolved_name = self.resolve_name(runtime_name).unwrap_or(runtime_name);
        self.runtime_defs
            .get(resolved_name)
            .and_then(|def| def.detection.as_ref())
            .map(|detection| detection.system_paths.clone())
            .unwrap_or_default()
    }

    /// Get the installation order for a runtime and its dependencies
    ///
    /// Returns a topologically sorted list of runtimes to install,
    /// with dependencies coming before dependents.
    pub fn get_install_order<'a>(&'a self, runtime_name: &'a str) -> Vec<&'a str> {
        let mut order = Vec::new();
        let mut visited = std::collections::HashSet::new();

        self.visit_dependencies(runtime_name, &mut order, &mut visited);
        order
    }

    /// Recursively visit dependencies (DFS)
    fn visit_dependencies<'a>(
        &'a self,
        runtime_name: &'a str,
        order: &mut Vec<&'a str>,
        visited: &mut std::collections::HashSet<&'a str>,
    ) {
        if visited.contains(runtime_name) {
            return;
        }
        visited.insert(runtime_name);

        if let Some(spec) = self.get(runtime_name) {
            // Visit dependencies first
            for dep in &spec.dependencies {
                if dep.required {
                    // Use the provider if specified, otherwise the dependency name
                    let dep_name = dep.provided_by.as_deref().unwrap_or(&dep.runtime_name);
                    self.visit_dependencies(dep_name, order, visited);
                }
            }
            // Then add this runtime
            order.push(&spec.name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_manifests_basic() {
        let toml = r#"
[provider]
name = "node"
ecosystem = "nodejs"

[[runtimes]]
name = "node"
description = "Node.js runtime"
executable = "node"
aliases = ["nodejs"]
priority = 100

[[runtimes]]
name = "npm"
description = "Node Package Manager"
executable = "npm"
bundled_with = "node"
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();
        let map = RuntimeMap::from_manifests(&[manifest]);

        // Check node was registered
        assert!(map.contains("node"));
        assert!(map.contains("nodejs")); // alias

        let node_spec = map.get("node").unwrap();
        assert_eq!(node_spec.name, "node");
        assert_eq!(node_spec.ecosystem, Ecosystem::Node);
        assert_eq!(node_spec.priority, 100);

        // Check npm was registered with dependency
        assert!(map.contains("npm"));
        let npm_spec = map.get("npm").unwrap();
        assert_eq!(npm_spec.dependencies.len(), 1);
        assert_eq!(npm_spec.dependencies[0].runtime_name, "node");
        assert!(npm_spec.dependencies[0].required);
    }

    #[test]
    fn test_from_manifests_with_constraints() {
        let toml = r#"
[provider]
name = "yarn"
ecosystem = "nodejs"

[[runtimes]]
name = "yarn"
description = "Yarn package manager"
executable = "yarn"

[[runtimes.constraints]]
when = "*"
requires = [
    { runtime = "node", version = ">=12", recommended = "20", reason = "Yarn requires Node.js" }
]
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();
        let map = RuntimeMap::from_manifests(&[manifest]);

        let yarn_spec = map.get("yarn").unwrap();
        assert_eq!(yarn_spec.dependencies.len(), 1);
        assert_eq!(yarn_spec.dependencies[0].runtime_name, "node");
        assert_eq!(
            yarn_spec.dependencies[0].min_version,
            Some("12".to_string())
        );
        assert_eq!(
            yarn_spec.dependencies[0].recommended_version,
            Some("20".to_string())
        );
    }

    #[test]
    fn test_from_manifests_multiple_providers() {
        let node_toml = r#"
[provider]
name = "node"
ecosystem = "nodejs"

[[runtimes]]
name = "node"
executable = "node"
"#;
        let python_toml = r#"
[provider]
name = "python"
ecosystem = "python"

[[runtimes]]
name = "python"
executable = "python"
aliases = ["python3", "py"]
"#;
        let node_manifest = ProviderManifest::parse(node_toml).unwrap();
        let python_manifest = ProviderManifest::parse(python_toml).unwrap();
        let map = RuntimeMap::from_manifests(&[node_manifest, python_manifest]);

        assert!(map.contains("node"));
        assert!(map.contains("python"));
        assert!(map.contains("python3")); // alias
        assert!(map.contains("py")); // alias

        assert_eq!(map.get("node").unwrap().ecosystem, Ecosystem::Node);
        assert_eq!(map.get("python").unwrap().ecosystem, Ecosystem::Python);
    }

    #[test]
    fn test_from_manifests_managed_by() {
        let toml = r#"
[provider]
name = "rust"
ecosystem = "rust"

[[runtimes]]
name = "rustup"
executable = "rustup"

[[runtimes]]
name = "rustc"
executable = "rustc"
managed_by = "rustup"

[[runtimes]]
name = "cargo"
executable = "cargo"
managed_by = "rustup"
"#;
        let manifest = ProviderManifest::parse(toml).unwrap();
        let map = RuntimeMap::from_manifests(&[manifest]);

        let rustc_spec = map.get("rustc").unwrap();
        assert_eq!(rustc_spec.dependencies.len(), 1);
        assert_eq!(rustc_spec.dependencies[0].runtime_name, "rustup");

        let cargo_spec = map.get("cargo").unwrap();
        assert_eq!(cargo_spec.dependencies.len(), 1);
        assert_eq!(cargo_spec.dependencies[0].runtime_name, "rustup");
    }

    #[test]
    fn test_extract_min_version() {
        assert_eq!(
            RuntimeMap::extract_min_version(">=12"),
            Some("12".to_string())
        );
        assert_eq!(
            RuntimeMap::extract_min_version(">=12, <23"),
            Some("12".to_string())
        );
        assert_eq!(
            RuntimeMap::extract_min_version(">=18.0.0"),
            Some("18.0.0".to_string())
        );
        assert_eq!(RuntimeMap::extract_min_version("*"), None);
        assert_eq!(RuntimeMap::extract_min_version("<20"), None);
    }
}
