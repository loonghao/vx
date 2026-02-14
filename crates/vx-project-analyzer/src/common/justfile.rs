//! Justfile analyzer
//!
//! `just` is a language-agnostic command runner. It can be used in any project
//! regardless of the programming language. This module provides justfile parsing
//! that is independent of any specific language analyzer.

use crate::error::AnalyzerResult;
use crate::script_parser::ScriptParser;
use crate::types::{Script, ScriptSource};
use std::path::Path;

/// Justfile analyzer for cross-language projects
pub struct JustfileAnalyzer {
    script_parser: ScriptParser,
}

impl JustfileAnalyzer {
    /// Create a new justfile analyzer
    pub fn new() -> Self {
        Self {
            script_parser: ScriptParser::new(),
        }
    }

    /// Detect if the project has a justfile
    pub fn detect(&self, root: &Path) -> bool {
        root.join("justfile").exists() || root.join("Justfile").exists()
    }

    /// Analyze scripts from justfile
    pub async fn analyze_scripts(&self, root: &Path) -> AnalyzerResult<Vec<Script>> {
        // Try both justfile and Justfile (case sensitivity varies by OS)
        let justfile_path = if root.join("justfile").exists() {
            root.join("justfile")
        } else if root.join("Justfile").exists() {
            root.join("Justfile")
        } else {
            return Ok(Vec::new());
        };

        let content = tokio::fs::read_to_string(&justfile_path).await?;
        parse_justfile_scripts(&content, &self.script_parser)
    }
}

impl Default for JustfileAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse scripts from justfile content
///
/// Justfile recipes have the format:
/// ```text
/// recipe-name [args]:
///     command1
///     command2
/// ```
///
/// We only extract top-level recipe names, not the commands inside.
pub fn parse_justfile_scripts(content: &str, parser: &ScriptParser) -> AnalyzerResult<Vec<Script>> {
    let mut scripts = Vec::new();

    for line in content.lines() {
        // Skip empty lines and comments
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }

        // Recipe lines must start at column 0 (no leading whitespace)
        // and contain a colon that's not inside quotes or brackets
        if !line.starts_with(' ')
            && !line.starts_with('\t')
            && let Some(recipe_name) = parse_recipe_name(line)
        {
            // Skip special justfile directives
            if is_justfile_directive(&recipe_name) {
                continue;
            }

            // Skip invalid recipe names
            if !is_valid_recipe_name(&recipe_name) {
                continue;
            }

            let cmd = format!("just {}", recipe_name);
            let mut script = Script::new(&recipe_name, &cmd, ScriptSource::Justfile);
            script.tools = parser.parse(&cmd);
            scripts.push(script);
        }
    }

    Ok(scripts)
}

/// Parse recipe name from a justfile line
///
/// Returns None if the line is not a valid recipe definition.
fn parse_recipe_name(line: &str) -> Option<String> {
    // Find the colon that marks the end of recipe name
    // But ignore colons inside quotes, brackets, or after @
    let mut in_quotes = false;
    let mut in_brackets = false;
    let mut quote_char = ' ';

    for (i, c) in line.char_indices() {
        match c {
            '"' | '\'' if !in_brackets => {
                if in_quotes && c == quote_char {
                    in_quotes = false;
                } else if !in_quotes {
                    in_quotes = true;
                    quote_char = c;
                }
            }
            '[' if !in_quotes => in_brackets = true,
            ']' if !in_quotes => in_brackets = false,
            ':' if !in_quotes && !in_brackets => {
                // Check if this is := (assignment) or : (recipe)
                let rest = &line[i..];
                if rest.starts_with(":=") {
                    return None; // This is a variable assignment
                }

                let name_part = line[..i].trim();

                // Handle dependencies: "recipe: dep1 dep2" -> extract "recipe"
                // Handle args: "recipe arg1 arg2:" -> extract "recipe"
                let recipe_name = name_part.split_whitespace().next()?;

                return Some(recipe_name.to_string());
            }
            _ => {}
        }
    }

    None
}

/// Check if this is a justfile directive (not a recipe)
fn is_justfile_directive(name: &str) -> bool {
    matches!(
        name,
        "set" | "alias" | "export" | "import" | "mod" | "private"
    )
}

/// Check if the recipe name is valid
fn is_valid_recipe_name(name: &str) -> bool {
    // Recipe names should:
    // - Not be empty
    // - Not start with @ (that's for quiet commands inside recipes)
    // - Not contain special characters except - and _
    // - Not look like a command (cd, echo, etc.)
    // - Be at least 2 characters (avoid single letter false positives)

    if name.is_empty() || name.len() < 2 {
        return false;
    }

    // Skip if starts with @
    if name.starts_with('@') {
        return false;
    }

    // Skip common command prefixes that might be misidentified
    let invalid_prefixes = ["cd ", "echo ", "if ", "for ", "while "];
    for prefix in invalid_prefixes {
        if name.starts_with(prefix) {
            return false;
        }
    }

    // Valid recipe names contain only alphanumeric, -, _
    name.chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_recipe_name() {
        assert_eq!(parse_recipe_name("build:"), Some("build".to_string()));
        assert_eq!(parse_recipe_name("test arg1:"), Some("test".to_string()));
        assert_eq!(
            parse_recipe_name("deploy: build test"),
            Some("deploy".to_string())
        );
        assert_eq!(parse_recipe_name("set shell := ['bash']"), None);
        assert_eq!(parse_recipe_name("export FOO := 'bar'"), None);
    }

    #[test]
    fn test_is_justfile_directive() {
        assert!(is_justfile_directive("set"));
        assert!(is_justfile_directive("alias"));
        assert!(is_justfile_directive("export"));
        assert!(!is_justfile_directive("build"));
        assert!(!is_justfile_directive("test"));
    }

    #[test]
    fn test_is_valid_recipe_name() {
        assert!(is_valid_recipe_name("build"));
        assert!(is_valid_recipe_name("test-all"));
        assert!(is_valid_recipe_name("build_release"));
        assert!(!is_valid_recipe_name("@echo"));
        assert!(!is_valid_recipe_name("a")); // too short
        assert!(!is_valid_recipe_name("")); // empty
    }
}
