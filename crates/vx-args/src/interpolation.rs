//! Variable interpolation with {{var}} syntax

use crate::error::{ArgError, ArgResult};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::process::Command;

/// Variable source for interpolation
pub trait VarSource {
    /// Get a variable value
    fn get(&self, name: &str) -> Option<String>;
}

/// HashMap-based variable source
impl VarSource for HashMap<String, String> {
    fn get(&self, name: &str) -> Option<String> {
        HashMap::get(self, name).cloned()
    }
}

/// Environment variable source
#[allow(dead_code)]
pub struct EnvVarSource;

impl VarSource for EnvVarSource {
    fn get(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }
}

/// Combined variable source (checks multiple sources in order)
pub struct CombinedSource<'a> {
    sources: Vec<&'a dyn VarSource>,
}

impl<'a> CombinedSource<'a> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self { sources: vec![] }
    }

    #[allow(dead_code)]
    pub fn add(&mut self, source: &'a dyn VarSource) {
        self.sources.push(source);
    }
}

impl Default for CombinedSource<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl VarSource for CombinedSource<'_> {
    fn get(&self, name: &str) -> Option<String> {
        for source in &self.sources {
            if let Some(value) = source.get(name) {
                return Some(value);
            }
        }
        None
    }
}

/// Variable interpolator
pub struct Interpolator {
    /// Pattern for {{var}} syntax
    pattern: Regex,
    /// Pattern for command interpolation `cmd`
    cmd_pattern: Regex,
    /// Built-in variables
    builtins: HashMap<String, String>,
    /// Whether to allow missing variables
    allow_missing: bool,
}

impl Interpolator {
    /// Create a new interpolator
    pub fn new() -> Self {
        let mut builtins = HashMap::new();

        // Add built-in variables
        builtins.insert(
            "vx.version".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        );

        if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
            builtins.insert("home".to_string(), home);
        }

        if let Ok(vx_paths) = vx_paths::VxPaths::new() {
            builtins.insert(
                "vx.home".to_string(),
                vx_paths.base_dir.display().to_string(),
            );
            builtins.insert(
                "vx.runtimes".to_string(),
                vx_paths.store_dir.display().to_string(),
            );
        }

        if let Ok(cwd) = std::env::current_dir() {
            builtins.insert("project.root".to_string(), cwd.display().to_string());
            if let Some(name) = cwd.file_name() {
                builtins.insert(
                    "project.name".to_string(),
                    name.to_string_lossy().to_string(),
                );
            }
        }

        // OS info
        builtins.insert("os.name".to_string(), std::env::consts::OS.to_string());
        builtins.insert("os.arch".to_string(), std::env::consts::ARCH.to_string());

        // Date/time
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        builtins.insert("timestamp".to_string(), now.as_secs().to_string());

        Self {
            pattern: Regex::new(r"\{\{([^}]+)\}\}").unwrap(),
            cmd_pattern: Regex::new(r"`([^`]+)`").unwrap(),
            builtins,
            allow_missing: false,
        }
    }

    /// Allow missing variables (replace with empty string)
    pub fn allow_missing(mut self, allow: bool) -> Self {
        self.allow_missing = allow;
        self
    }

    /// Add a built-in variable
    pub fn with_builtin(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.builtins.insert(name.into(), value.into());
        self
    }

    /// Interpolate a string using the given variable source
    pub fn interpolate(&self, input: &str, vars: &dyn VarSource) -> ArgResult<String> {
        self.interpolate_with_tracking(input, vars, &mut HashSet::new())
    }

    /// Interpolate with cycle detection
    fn interpolate_with_tracking(
        &self,
        input: &str,
        vars: &dyn VarSource,
        seen: &mut HashSet<String>,
    ) -> ArgResult<String> {
        let mut result = input.to_string();

        // First, handle command interpolation
        result = self.interpolate_commands(&result)?;

        // Then, handle variable interpolation
        result = self.interpolate_vars(&result, vars, seen)?;

        Ok(result)
    }

    /// Interpolate commands (backtick syntax)
    fn interpolate_commands(&self, input: &str) -> ArgResult<String> {
        let mut result = input.to_string();

        for cap in self.cmd_pattern.captures_iter(input) {
            let full_match = cap.get(0).unwrap().as_str();
            let cmd = cap.get(1).unwrap().as_str();

            let output = self.execute_command(cmd)?;
            result = result.replace(full_match, &output);
        }

        Ok(result)
    }

    /// Execute a command and return its output
    fn execute_command(&self, cmd: &str) -> ArgResult<String> {
        let output = if cfg!(windows) {
            Command::new("cmd").args(["/C", cmd]).output()
        } else {
            Command::new("sh").args(["-c", cmd]).output()
        };

        match output {
            Ok(output) if output.status.success() => {
                Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
            }
            Ok(output) => Err(ArgError::CommandFailed {
                command: cmd.to_string(),
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            }),
            Err(e) => Err(ArgError::CommandFailed {
                command: cmd.to_string(),
                message: e.to_string(),
            }),
        }
    }

    /// Interpolate variables ({{var}} syntax)
    fn interpolate_vars(
        &self,
        input: &str,
        vars: &dyn VarSource,
        seen: &mut HashSet<String>,
    ) -> ArgResult<String> {
        let mut result = input.to_string();
        let mut changed = true;

        // Keep interpolating until no more changes (handles nested vars)
        while changed {
            changed = false;
            let current = result.clone();

            for cap in self.pattern.captures_iter(&current) {
                let full_match = cap.get(0).unwrap().as_str();
                let var_name = cap.get(1).unwrap().as_str().trim();

                // Check for cycles
                if seen.contains(var_name) {
                    return Err(ArgError::CircularReference {
                        chain: format!(
                            "{} -> {}",
                            seen.iter().cloned().collect::<Vec<_>>().join(" -> "),
                            var_name
                        ),
                    });
                }

                let value = self.resolve_var(var_name, vars)?;

                if let Some(val) = value {
                    // Track this variable for cycle detection
                    seen.insert(var_name.to_string());

                    // Recursively interpolate the value
                    let interpolated = self.interpolate_with_tracking(&val, vars, seen)?;

                    seen.remove(var_name);

                    result = result.replace(full_match, &interpolated);
                    changed = true;
                } else if self.allow_missing {
                    result = result.replace(full_match, "");
                    changed = true;
                } else {
                    return Err(ArgError::variable_not_found(var_name));
                }
            }
        }

        Ok(result)
    }

    /// Resolve a variable name to its value
    fn resolve_var(&self, name: &str, vars: &dyn VarSource) -> ArgResult<Option<String>> {
        // Check for env.VAR syntax
        if let Some(env_name) = name.strip_prefix("env.") {
            return Ok(std::env::var(env_name).ok());
        }

        // Check built-ins first
        if let Some(value) = self.builtins.get(name) {
            return Ok(Some(value.clone()));
        }

        // Check user variables
        if let Some(value) = vars.get(name) {
            return Ok(Some(value));
        }

        Ok(None)
    }

    /// Interpolate a HashMap of variables
    pub fn interpolate_map(
        &self,
        map: &HashMap<String, String>,
        vars: &dyn VarSource,
    ) -> ArgResult<HashMap<String, String>> {
        let mut result = HashMap::new();

        for (key, value) in map {
            let interpolated = self.interpolate(value, vars)?;
            result.insert(key.clone(), interpolated);
        }

        Ok(result)
    }
}

impl Default for Interpolator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_interpolation() {
        let interpolator = Interpolator::new();
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "world".to_string());

        let result = interpolator.interpolate("Hello, {{name}}!", &vars).unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_multiple_vars() {
        let interpolator = Interpolator::new();
        let mut vars = HashMap::new();
        vars.insert("project".to_string(), "my-app".to_string());
        vars.insert("version".to_string(), "1.0.0".to_string());

        let result = interpolator
            .interpolate("Building {{project}} v{{version}}", &vars)
            .unwrap();
        assert_eq!(result, "Building my-app v1.0.0");
    }

    #[test]
    fn test_nested_vars() {
        let interpolator = Interpolator::new();
        let mut vars = HashMap::new();
        vars.insert("base".to_string(), "dist".to_string());
        vars.insert("dir".to_string(), "{{base}}/output".to_string());

        let result = interpolator.interpolate("{{dir}}", &vars).unwrap();
        assert_eq!(result, "dist/output");
    }

    #[test]
    fn test_builtin_vars() {
        let interpolator = Interpolator::new();
        let vars = HashMap::new();

        let result = interpolator.interpolate("OS: {{os.name}}", &vars).unwrap();
        assert!(result.starts_with("OS: "));
        assert!(!result.contains("{{"));
    }

    #[test]
    fn test_env_var() {
        std::env::set_var("TEST_VAR_123", "test_value");
        let interpolator = Interpolator::new();
        let vars = HashMap::new();

        let result = interpolator
            .interpolate("Value: {{env.TEST_VAR_123}}", &vars)
            .unwrap();
        assert_eq!(result, "Value: test_value");

        std::env::remove_var("TEST_VAR_123");
    }

    #[test]
    fn test_missing_var() {
        let interpolator = Interpolator::new();
        let vars = HashMap::new();

        let result = interpolator.interpolate("{{missing}}", &vars);
        assert!(result.is_err());
    }

    #[test]
    fn test_allow_missing() {
        let interpolator = Interpolator::new().allow_missing(true);
        let vars = HashMap::new();

        let result = interpolator
            .interpolate("Value: {{missing}}", &vars)
            .unwrap();
        assert_eq!(result, "Value: ");
    }

    #[test]
    fn test_circular_reference() {
        let interpolator = Interpolator::new();
        let mut vars = HashMap::new();
        vars.insert("a".to_string(), "{{b}}".to_string());
        vars.insert("b".to_string(), "{{a}}".to_string());

        let result = interpolator.interpolate("{{a}}", &vars);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ArgError::CircularReference { .. }
        ));
    }

    #[test]
    fn test_command_interpolation() {
        let interpolator = Interpolator::new();
        let vars = HashMap::new();

        // Simple echo command
        let cmd = if cfg!(windows) {
            "echo hello"
        } else {
            "echo hello"
        };

        let result = interpolator
            .interpolate(&format!("Output: `{}`", cmd), &vars)
            .unwrap();
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_combined_source() {
        let mut vars1 = HashMap::new();
        vars1.insert("a".to_string(), "1".to_string());

        let mut vars2 = HashMap::new();
        vars2.insert("b".to_string(), "2".to_string());

        let mut combined = CombinedSource::new();
        combined.add(&vars1);
        combined.add(&vars2);

        assert_eq!(combined.get("a"), Some("1".to_string()));
        assert_eq!(combined.get("b"), Some("2".to_string()));
        assert_eq!(combined.get("c"), None);
    }
}
