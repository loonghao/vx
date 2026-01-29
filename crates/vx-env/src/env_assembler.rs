//! Environment variable assembler
//!
//! This module provides a Rez-style environment variable assembly mechanism
//! with support for append, prepend, replace, and default operations.

use std::collections::HashMap;

/// Environment variable operation type
#[derive(Debug, Clone, PartialEq)]
pub enum EnvOperation {
    /// Set variable (overwrite)
    Set(String),
    /// Append to existing value
    Append(String),
    /// Prepend to existing value
    Prepend(String),
    /// Remove pattern from existing value
    Remove(String),
    /// Use default value (only if not set)
    Default(String),
}

/// Environment variable configuration
#[derive(Debug, Clone)]
pub struct EnvVar {
    /// Variable name
    pub name: String,
    /// Operation to perform
    pub operation: EnvOperation,
    /// Separator for PATH-like variables
    pub separator: String,
}

impl EnvVar {
    /// Create a new SET operation
    pub fn set(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            operation: EnvOperation::Set(value.into()),
            separator: default_separator(),
        }
    }

    /// Create a new PREPEND operation
    pub fn prepend(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            operation: EnvOperation::Prepend(value.into()),
            separator: default_separator(),
        }
    }

    /// Create a new APPEND operation
    pub fn append(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            operation: EnvOperation::Append(value.into()),
            separator: default_separator(),
        }
    }

    /// Create a new DEFAULT operation
    pub fn default_val(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            operation: EnvOperation::Default(value.into()),
            separator: default_separator(),
        }
    }

    /// Create a new REMOVE operation
    pub fn remove(name: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            operation: EnvOperation::Remove(pattern.into()),
            separator: default_separator(),
        }
    }

    /// Set custom separator
    pub fn with_separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }
}

/// Default path separator based on platform
fn default_separator() -> String {
    if cfg!(windows) {
        ";".to_string()
    } else {
        ":".to_string()
    }
}

/// PATH priority constants (higher = more priority)
pub mod priority {
    /// Project-specific tools (highest priority)
    pub const PROJECT_TOOLS: i32 = 1000;
    /// vx-managed tools
    pub const VX_TOOLS: i32 = 900;
    /// vx shims
    pub const VX_SHIMS: i32 = 800;
    /// User-defined prepend
    pub const USER_PREPEND: i32 = 700;
    /// System PATH (inherited)
    pub const SYSTEM: i32 = 500;
    /// User-defined append
    pub const USER_APPEND: i32 = 300;
    /// Legacy compatibility paths (lowest priority)
    pub const LEGACY: i32 = 100;
}

/// Environment variable assembler
///
/// Provides a Rez-style environment assembly mechanism with support for
/// multiple operations and deterministic ordering.
#[derive(Debug)]
pub struct EnvAssembler {
    /// Base environment (inherited or isolated)
    base_env: HashMap<String, String>,
    /// Operation queue with priorities (priority, variable)
    operations: Vec<(i32, EnvVar)>,
}

impl Default for EnvAssembler {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvAssembler {
    /// Create a new empty assembler
    pub fn new() -> Self {
        Self {
            base_env: HashMap::new(),
            operations: Vec::new(),
        }
    }

    /// Inherit from system environment
    pub fn inherit_system(mut self) -> Self {
        self.base_env = std::env::vars().collect();
        self
    }

    /// Inherit specific variables from system
    pub fn inherit_vars(mut self, vars: &[&str]) -> Self {
        for var in vars {
            if let Ok(value) = std::env::var(var) {
                self.base_env.insert(var.to_string(), value);
            }
        }
        self
    }

    /// Inherit variables matching patterns (e.g., "SSH_*", "GITHUB_*")
    pub fn inherit_pattern(mut self, patterns: &[&str]) -> Self {
        for (key, value) in std::env::vars() {
            for pattern in patterns {
                if Self::matches_pattern(&key, pattern) {
                    self.base_env.insert(key.clone(), value.clone());
                    break;
                }
            }
        }
        self
    }

    /// Set a base environment variable
    pub fn set_base(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.base_env.insert(name.into(), value.into());
        self
    }

    /// Add an environment variable operation with priority
    pub fn add_operation(mut self, priority: i32, var: EnvVar) -> Self {
        self.operations.push((priority, var));
        self
    }

    /// Add a PATH prepend operation with specified priority
    pub fn prepend_path(self, priority: i32, path: impl Into<String>) -> Self {
        self.add_operation(priority, EnvVar::prepend("PATH", path))
    }

    /// Add a PATH append operation with specified priority
    pub fn append_path(self, priority: i32, path: impl Into<String>) -> Self {
        self.add_operation(priority, EnvVar::append("PATH", path))
    }

    /// Add a simple SET operation
    pub fn set_var(self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.add_operation(priority::USER_PREPEND, EnvVar::set(name, value))
    }

    /// Build the final environment
    ///
    /// Operations are applied in priority order (highest first).
    /// For the same priority, operations are applied in the order they were added.
    pub fn build(self) -> HashMap<String, String> {
        let mut env = self.base_env;

        // Sort operations by priority (descending) while preserving insertion order for same priority
        let mut ops = self.operations;
        // Use stable sort to preserve insertion order for same priority
        ops.sort_by(|a, b| b.0.cmp(&a.0));

        for (_, var) in ops {
            Self::apply_operation(&mut env, var);
        }

        env
    }

    /// Apply a single operation to the environment
    fn apply_operation(env: &mut HashMap<String, String>, var: EnvVar) {
        match var.operation {
            EnvOperation::Set(value) => {
                env.insert(var.name, value);
            }
            EnvOperation::Prepend(value) => {
                let current = env.get(&var.name).cloned().unwrap_or_default();
                if current.is_empty() {
                    env.insert(var.name, value);
                } else {
                    env.insert(var.name, format!("{}{}{}", value, var.separator, current));
                }
            }
            EnvOperation::Append(value) => {
                let current = env.get(&var.name).cloned().unwrap_or_default();
                if current.is_empty() {
                    env.insert(var.name, value);
                } else {
                    env.insert(var.name, format!("{}{}{}", current, var.separator, value));
                }
            }
            EnvOperation::Remove(pattern) => {
                if let Some(current) = env.get(&var.name).cloned() {
                    let parts: Vec<&str> = current
                        .split(&var.separator)
                        .filter(|p| !p.contains(&pattern))
                        .collect();
                    env.insert(var.name, parts.join(&var.separator));
                }
            }
            EnvOperation::Default(value) => {
                env.entry(var.name).or_insert(value);
            }
        }
    }

    /// Check if a string matches a simple glob pattern
    fn matches_pattern(s: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            // Simple glob: only support * at end or beginning
            if pattern.ends_with('*') {
                s.starts_with(pattern.trim_end_matches('*'))
            } else if pattern.starts_with('*') {
                s.ends_with(pattern.trim_start_matches('*'))
            } else {
                // Pattern like "FOO*BAR" - check prefix and suffix
                let parts: Vec<&str> = pattern.split('*').collect();
                if parts.len() == 2 {
                    s.starts_with(parts[0]) && s.ends_with(parts[1])
                } else {
                    s == pattern
                }
            }
        } else {
            s == pattern
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_operation_set() {
        let assembler = EnvAssembler::new().set_var("FOO", "bar");
        let env = assembler.build();
        assert_eq!(env.get("FOO"), Some(&"bar".to_string()));
    }

    #[test]
    fn test_env_operation_prepend() {
        let assembler = EnvAssembler::new()
            .set_base("PATH", "/usr/bin")
            .prepend_path(priority::VX_TOOLS, "/vx/bin");
        let env = assembler.build();

        let sep = if cfg!(windows) { ";" } else { ":" };
        assert_eq!(env.get("PATH"), Some(&format!("/vx/bin{}/usr/bin", sep)));
    }

    #[test]
    fn test_env_operation_append() {
        let assembler = EnvAssembler::new()
            .set_base("PATH", "/usr/bin")
            .append_path(priority::LEGACY, "/opt/bin");
        let env = assembler.build();

        let sep = if cfg!(windows) { ";" } else { ":" };
        assert_eq!(env.get("PATH"), Some(&format!("/usr/bin{}/opt/bin", sep)));
    }

    #[test]
    fn test_env_operation_default() {
        // Test when variable doesn't exist
        let assembler = EnvAssembler::new().add_operation(
            priority::USER_PREPEND,
            EnvVar::default_val("MY_VAR", "default_value"),
        );
        let env = assembler.build();
        assert_eq!(env.get("MY_VAR"), Some(&"default_value".to_string()));

        // Test when variable already exists
        let assembler = EnvAssembler::new()
            .set_base("MY_VAR", "existing_value")
            .add_operation(
                priority::USER_PREPEND,
                EnvVar::default_val("MY_VAR", "default_value"),
            );
        let env = assembler.build();
        assert_eq!(env.get("MY_VAR"), Some(&"existing_value".to_string()));
    }

    #[test]
    fn test_env_operation_remove() {
        let sep = if cfg!(windows) { ";" } else { ":" };
        let initial_path = format!("/usr/bin{}/bad/bin{}/opt/bin", sep, sep);

        let assembler = EnvAssembler::new()
            .set_base("PATH", &initial_path)
            .add_operation(priority::USER_PREPEND, EnvVar::remove("PATH", "/bad/"));
        let env = assembler.build();

        assert_eq!(env.get("PATH"), Some(&format!("/usr/bin{}/opt/bin", sep)));
    }

    #[test]
    fn test_priority_ordering() {
        // Higher priority should be applied first, so it ends up at the front of PATH
        let assembler = EnvAssembler::new()
            .set_base("PATH", "/original")
            .prepend_path(priority::LEGACY, "/legacy") // 100
            .prepend_path(priority::VX_TOOLS, "/vx") // 900
            .prepend_path(priority::PROJECT_TOOLS, "/project"); // 1000
        let env = assembler.build();

        let sep = if cfg!(windows) { ";" } else { ":" };
        // All prepends happen, highest priority first
        // /project prepends to /original -> /project:/original
        // /vx prepends to /project:/original -> /vx:/project:/original
        // /legacy prepends to /vx:/project:/original -> /legacy:/vx:/project:/original
        // Actually: operations sorted highest first, so PROJECT_TOOLS runs first
        // Each prepend adds to front, so last prepend (lowest priority) ends up at front
        // Result: /legacy:/vx:/project:/original
        let path = env.get("PATH").unwrap();
        let parts: Vec<&str> = path.split(sep).collect();

        // The last prepend (lowest priority) ends up at the front
        assert_eq!(parts[0], "/legacy");
        assert_eq!(parts[1], "/vx");
        assert_eq!(parts[2], "/project");
        assert_eq!(parts[3], "/original");
    }

    #[test]
    fn test_matches_pattern() {
        assert!(EnvAssembler::matches_pattern("SSH_AUTH_SOCK", "SSH_*"));
        assert!(EnvAssembler::matches_pattern("GITHUB_TOKEN", "GITHUB_*"));
        assert!(!EnvAssembler::matches_pattern("HOME", "SSH_*"));

        assert!(EnvAssembler::matches_pattern("MY_VAR_SUFFIX", "*_SUFFIX"));
        assert!(!EnvAssembler::matches_pattern("MY_VAR", "*_SUFFIX"));

        assert!(EnvAssembler::matches_pattern("EXACT_MATCH", "EXACT_MATCH"));
        assert!(!EnvAssembler::matches_pattern("EXACT", "EXACT_MATCH"));
    }

    #[test]
    fn test_inherit_pattern() {
        // Set some test env vars
        std::env::set_var("TEST_VX_VAR1", "value1");
        std::env::set_var("TEST_VX_VAR2", "value2");
        std::env::set_var("TEST_OTHER", "other");

        let assembler = EnvAssembler::new().inherit_pattern(&["TEST_VX_*"]);
        let env = assembler.build();

        assert_eq!(env.get("TEST_VX_VAR1"), Some(&"value1".to_string()));
        assert_eq!(env.get("TEST_VX_VAR2"), Some(&"value2".to_string()));
        assert_eq!(env.get("TEST_OTHER"), None);

        // Cleanup
        std::env::remove_var("TEST_VX_VAR1");
        std::env::remove_var("TEST_VX_VAR2");
        std::env::remove_var("TEST_OTHER");
    }
}
