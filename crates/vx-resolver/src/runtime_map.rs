//! Runtime mapping
//!
//! This module provides a comprehensive mapping of runtimes to their dependencies,
//! supporting various ecosystems (Node.js, Python, Rust, Go, etc.)

use crate::runtime_spec::{Ecosystem, RuntimeDependency, RuntimeSpec};
use std::collections::HashMap;

/// A registry of runtime specifications and their dependencies
#[derive(Debug, Default)]
pub struct RuntimeMap {
    /// Map of runtime name to specification
    runtimes: HashMap<String, RuntimeSpec>,
    /// Map of alias to primary runtime name
    aliases: HashMap<String, String>,
}

impl RuntimeMap {
    /// Create a new runtime map with built-in runtime definitions
    pub fn new() -> Self {
        let mut map = Self::default();
        map.register_builtin_runtimes();
        map
    }

    /// Create an empty runtime map (for testing)
    pub fn empty() -> Self {
        Self::default()
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

    /// Register all built-in runtime definitions
    fn register_builtin_runtimes(&mut self) {
        // ============ Node.js Ecosystem ============

        // Node.js runtime
        self.register(
            RuntimeSpec::new("node", "Node.js JavaScript runtime")
                .with_alias("nodejs")
                .with_ecosystem(Ecosystem::Node)
                .with_priority(100), // High priority - base runtime
        );

        // npm - bundled with Node.js
        self.register(
            RuntimeSpec::new("npm", "Node.js package manager")
                .with_ecosystem(Ecosystem::Node)
                .with_dependency(
                    RuntimeDependency::required("node", "npm is bundled with Node.js")
                        .provided_by("node"),
                ),
        );

        // npx - bundled with Node.js
        self.register(
            RuntimeSpec::new("npx", "Node.js package runner")
                .with_ecosystem(Ecosystem::Node)
                .with_dependency(
                    RuntimeDependency::required("node", "npx is bundled with Node.js")
                        .provided_by("node"),
                ),
        );

        // yarn - requires Node.js
        // Note: Yarn 1.x has compatibility issues with Node.js 23+ due to native module compilation
        // Recommend Node.js 20 LTS for best compatibility
        self.register(
            RuntimeSpec::new("yarn", "Fast, reliable, and secure dependency management")
                .with_ecosystem(Ecosystem::Node)
                .with_dependency(
                    RuntimeDependency::required("node", "yarn requires Node.js runtime")
                        .with_min_version("12.0.0")
                        .with_max_version("22.99.99")
                        .with_recommended_version("20"),
                ),
        );

        // pnpm - requires Node.js
        self.register(
            RuntimeSpec::new("pnpm", "Fast, disk space efficient package manager")
                .with_ecosystem(Ecosystem::Node)
                .with_dependency(RuntimeDependency::required(
                    "node",
                    "pnpm requires Node.js runtime",
                )),
        );

        // bun - standalone runtime (no dependencies)
        self.register(
            RuntimeSpec::new("bun", "Incredibly fast JavaScript runtime and toolkit")
                .with_alias("bunx")
                .with_ecosystem(Ecosystem::Node)
                .with_priority(90),
        );

        // ============ Python Ecosystem ============

        // uv - standalone Python package manager
        self.register(
            RuntimeSpec::new(
                "uv",
                "An extremely fast Python package installer and resolver",
            )
            .with_ecosystem(Ecosystem::Python)
            .with_priority(100), // Standalone, no dependencies
        );

        // uvx - uv runtime runner (alias for "uv tool run")
        self.register(
            RuntimeSpec::new("uvx", "Python application runner")
                .with_ecosystem(Ecosystem::Python)
                .with_executable("uv")
                .with_command_prefix(vec!["tool", "run"])
                .with_dependency(
                    RuntimeDependency::required("uv", "uvx is part of uv").provided_by("uv"),
                ),
        );

        // pip - requires Python (but uv can replace it)
        self.register(
            RuntimeSpec::new("pip", "Python package installer")
                .with_alias("pip3")
                .with_ecosystem(Ecosystem::Python)
                .with_dependency(RuntimeDependency::required(
                    "python",
                    "pip requires Python runtime",
                )),
        );

        // pipx - Python application runner
        self.register(
            RuntimeSpec::new(
                "pipx",
                "Install and run Python applications in isolated environments",
            )
            .with_ecosystem(Ecosystem::Python)
            .with_dependency(RuntimeDependency::required(
                "python",
                "pipx requires Python runtime",
            )),
        );

        // ============ Rust Ecosystem ============

        // rustup - Rust toolchain installer
        self.register(
            RuntimeSpec::new("rustup", "The Rust toolchain installer")
                .with_ecosystem(Ecosystem::Rust)
                .with_priority(100),
        );

        // cargo - Rust package manager
        self.register(
            RuntimeSpec::new("cargo", "Rust package manager and build tool")
                .with_ecosystem(Ecosystem::Rust)
                .with_dependency(
                    RuntimeDependency::required("rustup", "cargo is installed via rustup")
                        .provided_by("rustup"),
                ),
        );

        // rustc - Rust compiler
        self.register(
            RuntimeSpec::new("rustc", "The Rust compiler")
                .with_ecosystem(Ecosystem::Rust)
                .with_dependency(
                    RuntimeDependency::required("rustup", "rustc is installed via rustup")
                        .provided_by("rustup"),
                ),
        );

        // cargo-binstall - Binary installation for cargo
        self.register(
            RuntimeSpec::new("cargo-binstall", "Binary installation for Rust projects")
                .with_ecosystem(Ecosystem::Rust)
                .with_dependency(RuntimeDependency::required(
                    "cargo",
                    "cargo-binstall requires cargo",
                )),
        );

        // ============ Go Ecosystem ============

        // go - Go programming language
        self.register(
            RuntimeSpec::new("go", "The Go programming language")
                .with_alias("golang")
                .with_ecosystem(Ecosystem::Go)
                .with_priority(100),
        );

        // ============ Java Ecosystem ============

        // java - Java Development Kit (Eclipse Temurin)
        self.register(
            RuntimeSpec::new("java", "Java Development Kit (Eclipse Temurin)")
                .with_alias("jdk")
                .with_alias("temurin")
                .with_alias("openjdk")
                .with_ecosystem(Ecosystem::Java)
                .with_priority(100),
        );

        // javac - Java compiler (bundled with JDK)
        self.register(
            RuntimeSpec::new("javac", "Java compiler")
                .with_ecosystem(Ecosystem::Java)
                .with_dependency(
                    RuntimeDependency::required("java", "javac is bundled with JDK")
                        .provided_by("java"),
                ),
        );

        // jar - Java archive tool (bundled with JDK)
        self.register(
            RuntimeSpec::new("jar", "Java archive tool")
                .with_ecosystem(Ecosystem::Java)
                .with_dependency(
                    RuntimeDependency::required("java", "jar is bundled with JDK")
                        .provided_by("java"),
                ),
        );

        // ============ Generic Runtimes ============

        // MSVC Build Tools - Windows C/C++ toolchain
        self.register(
            RuntimeSpec::new("msvc", "Microsoft Visual C++ Build Tools (cl, nmake)")
                .with_aliases(vec!["cl", "nmake", "msvc-tools", "vs-build-tools"])
                .with_executable("cl")
                .with_ecosystem(Ecosystem::Generic),
        );

        // git - Version control
        self.register(RuntimeSpec::new(
            "git",
            "Distributed version control system",
        ));

        // make - Build automation
        self.register(RuntimeSpec::new("make", "Build automation tool").with_alias("gmake"));

        // cmake - Cross-platform build system
        self.register(RuntimeSpec::new(
            "cmake",
            "Cross-platform build system generator",
        ));

        // docker - Container platform
        self.register(RuntimeSpec::new("docker", "Container platform"));

        // kubectl - Kubernetes CLI
        self.register(
            RuntimeSpec::new("kubectl", "Kubernetes command-line tool").with_alias("k8s"),
        );
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
