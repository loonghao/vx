//! Environment builder for constructing environment variable sets
//!
//! Provides a fluent API for building environment configurations.

use std::collections::HashMap;
use std::path::PathBuf;

/// Builder for constructing environment variable sets
///
/// # Example
///
/// ```rust
/// use vx_env::EnvBuilder;
///
/// let env = EnvBuilder::new()
///     .path_prepend("/usr/local/node/bin")
///     .path_prepend("/usr/local/go/bin")
///     .var("NODE_ENV", "production")
///     .var("GOPATH", "/home/user/go")
///     .build();
/// ```
#[derive(Debug, Clone, Default)]
pub struct EnvBuilder {
    /// Environment variables to set
    vars: HashMap<String, String>,
    /// Paths to prepend to PATH
    path_prepends: Vec<PathBuf>,
    /// Paths to append to PATH
    path_appends: Vec<PathBuf>,
    /// Whether to inherit current environment
    inherit: bool,
}

impl EnvBuilder {
    /// Create a new environment builder
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            path_prepends: Vec::new(),
            path_appends: Vec::new(),
            inherit: true,
        }
    }

    /// Create a builder that doesn't inherit the current environment
    pub fn clean() -> Self {
        Self {
            inherit: false,
            ..Self::new()
        }
    }

    /// Set an environment variable
    pub fn var(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.vars.insert(name.into(), value.into());
        self
    }

    /// Set multiple environment variables
    pub fn vars(mut self, vars: impl IntoIterator<Item = (String, String)>) -> Self {
        self.vars.extend(vars);
        self
    }

    /// Prepend a path to PATH (will be searched first)
    pub fn path_prepend(mut self, path: impl Into<PathBuf>) -> Self {
        self.path_prepends.push(path.into());
        self
    }

    /// Prepend multiple paths to PATH
    pub fn path_prepends(mut self, paths: impl IntoIterator<Item = PathBuf>) -> Self {
        self.path_prepends.extend(paths);
        self
    }

    /// Append a path to PATH (will be searched last)
    pub fn path_append(mut self, path: impl Into<PathBuf>) -> Self {
        self.path_appends.push(path.into());
        self
    }

    /// Append multiple paths to PATH
    pub fn path_appends(mut self, paths: impl IntoIterator<Item = PathBuf>) -> Self {
        self.path_appends.extend(paths);
        self
    }

    /// Set whether to inherit the current environment
    pub fn inherit(mut self, inherit: bool) -> Self {
        self.inherit = inherit;
        self
    }

    /// Build the final environment variable map
    pub fn build(self) -> HashMap<String, String> {
        let mut env = self.vars;

        // Build PATH
        if !self.path_prepends.is_empty() || !self.path_appends.is_empty() {
            let current_path = if self.inherit {
                std::env::var("PATH").unwrap_or_default()
            } else {
                String::new()
            };

            let path_sep = if cfg!(windows) { ";" } else { ":" };

            let mut path_parts: Vec<String> = Vec::new();

            // Prepends go first (in order)
            for p in &self.path_prepends {
                path_parts.push(p.to_string_lossy().into_owned());
            }

            // Current PATH in the middle
            if !current_path.is_empty() {
                path_parts.push(current_path);
            }

            // Appends go last (in order)
            for p in &self.path_appends {
                path_parts.push(p.to_string_lossy().into_owned());
            }

            env.insert("PATH".to_string(), path_parts.join(path_sep));
        }

        env
    }

    /// Build and merge with existing environment variables
    pub fn build_merged(self, existing: &HashMap<String, String>) -> HashMap<String, String> {
        let mut result = existing.clone();
        result.extend(self.build());
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let env = EnvBuilder::new()
            .var("FOO", "bar")
            .var("BAZ", "qux")
            .build();

        assert_eq!(env.get("FOO"), Some(&"bar".to_string()));
        assert_eq!(env.get("BAZ"), Some(&"qux".to_string()));
    }

    #[test]
    fn test_builder_path_prepend() {
        let env = EnvBuilder::clean()
            .path_prepend("/first")
            .path_prepend("/second")
            .build();

        let path = env.get("PATH").unwrap();
        let sep = if cfg!(windows) { ";" } else { ":" };

        assert!(path.starts_with("/first"));
        assert!(path.contains(&format!("{}/second", sep)));
    }

    #[test]
    fn test_builder_path_append() {
        let env = EnvBuilder::clean().path_append("/last").build();

        let path = env.get("PATH").unwrap();
        assert!(path.ends_with("/last"));
    }

    #[test]
    fn test_builder_vars_batch() {
        let vars = vec![
            ("A".to_string(), "1".to_string()),
            ("B".to_string(), "2".to_string()),
        ];

        let env = EnvBuilder::new().vars(vars).build();

        assert_eq!(env.get("A"), Some(&"1".to_string()));
        assert_eq!(env.get("B"), Some(&"2".to_string()));
    }
}
