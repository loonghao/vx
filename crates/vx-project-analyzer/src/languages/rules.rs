//! Script detection rules framework
//!
//! This module provides a declarative rule system for detecting scripts
//! in projects. Each language can define its own rules.

use crate::script_parser::ScriptParser;
use crate::types::{Script, ScriptSource};
use std::collections::HashSet;
use std::path::Path;

/// A rule for detecting scripts based on file presence
#[derive(Debug, Clone)]
pub struct ScriptRule {
    /// Script name (e.g., "test", "lint", "nox")
    pub name: &'static str,
    /// Command to run
    pub command: &'static str,
    /// Description of the script
    pub description: &'static str,
    /// Files that trigger this rule (any match)
    pub trigger_files: &'static [&'static str],
    /// Files that must NOT exist for this rule to apply
    pub exclude_if_exists: &'static [&'static str],
    /// Priority (higher = preferred when multiple rules match same name)
    pub priority: u8,
}

impl ScriptRule {
    /// Create a new script rule
    pub const fn new(name: &'static str, command: &'static str, description: &'static str) -> Self {
        Self {
            name,
            command,
            description,
            trigger_files: &[],
            exclude_if_exists: &[],
            priority: 50,
        }
    }

    /// Set trigger files (any match triggers the rule)
    pub const fn triggers(mut self, files: &'static [&'static str]) -> Self {
        self.trigger_files = files;
        self
    }

    /// Set exclusion files (rule won't apply if any exist)
    pub const fn excludes(mut self, files: &'static [&'static str]) -> Self {
        self.exclude_if_exists = files;
        self
    }

    /// Set priority (higher = preferred)
    pub const fn priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Check if this rule matches the given project root
    pub fn matches(&self, root: &Path) -> bool {
        // Check exclusions first
        for exclude in self.exclude_if_exists {
            if root.join(exclude).exists() {
                return false;
            }
        }

        // Check triggers
        if self.trigger_files.is_empty() {
            return false;
        }

        self.trigger_files
            .iter()
            .any(|file| root.join(file).exists())
    }

    /// Convert to a Script if the rule matches
    pub fn to_script(&self, parser: &ScriptParser) -> Script {
        let mut script = Script::new(
            self.name,
            self.command,
            ScriptSource::Detected {
                reason: format!(
                    "{} detected",
                    self.trigger_files.first().unwrap_or(&"config")
                ),
            },
        );
        script.tools = parser.parse(self.command);
        script.description = Some(self.description.to_string());
        script
    }
}

/// Apply a set of script rules to detect scripts for a project
///
/// Rules are evaluated in priority order (highest first).
/// For each script name, only the highest priority matching rule is used.
pub fn apply_rules(root: &Path, rules: &[ScriptRule], parser: &ScriptParser) -> Vec<Script> {
    let mut scripts = Vec::new();
    let mut seen_names: HashSet<&str> = HashSet::new();

    // Sort rules by priority (descending) - we need to collect since rules is a slice
    let mut sorted_rules: Vec<_> = rules.iter().collect();
    sorted_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

    for rule in sorted_rules {
        // Skip if we already have a script with this name
        if seen_names.contains(rule.name) {
            continue;
        }

        if rule.matches(root) {
            scripts.push(rule.to_script(parser));
            seen_names.insert(rule.name);
        }
    }

    scripts
}

/// Merge detected scripts with explicit scripts
///
/// Explicit scripts (from config files) take priority over detected ones.
pub fn merge_scripts(explicit: Vec<Script>, detected: Vec<Script>) -> Vec<Script> {
    let explicit_names: HashSet<String> = explicit.iter().map(|s| s.name.clone()).collect();

    let mut result = explicit;
    for script in detected {
        if !explicit_names.contains(&script.name) {
            result.push(script);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_rule_matches_trigger() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("noxfile.py"), "").unwrap();

        let rule = ScriptRule::new("nox", "uvx nox", "Run nox").triggers(&["noxfile.py"]);

        assert!(rule.matches(temp.path()));
    }

    #[test]
    fn test_rule_excludes() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("pytest.ini"), "").unwrap();
        std::fs::write(temp.path().join("noxfile.py"), "").unwrap();

        let rule = ScriptRule::new("test", "pytest", "Run tests")
            .triggers(&["pytest.ini"])
            .excludes(&["noxfile.py"]);

        assert!(!rule.matches(temp.path()));
    }

    #[test]
    fn test_priority_ordering() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("noxfile.py"), "").unwrap();
        std::fs::write(temp.path().join("pytest.ini"), "").unwrap();

        let rules = &[
            ScriptRule::new("test", "pytest", "pytest")
                .triggers(&["pytest.ini"])
                .priority(50),
            ScriptRule::new("test", "nox -s tests", "nox")
                .triggers(&["noxfile.py"])
                .priority(100),
        ];

        let parser = ScriptParser::new();
        let scripts = apply_rules(temp.path(), rules, &parser);

        assert_eq!(scripts.len(), 1);
        assert_eq!(scripts[0].command, "nox -s tests");
    }
}
