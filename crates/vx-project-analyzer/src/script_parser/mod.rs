//! Script command parsing to detect tool dependencies
//!
//! This module provides an extensible architecture for parsing script commands
//! and detecting tool dependencies. It uses a trait-based design to allow
//! language-specific pattern providers to be added easily.
//!
//! ## Architecture
//!
//! - [`ScriptPatternProvider`]: Trait for language-specific pattern matching
//! - [`PatternRegistry`]: Collects all registered pattern providers
//! - [`ScriptParser`]: Facade that delegates to registered providers
//! - [`ScriptTool`]: Represents a detected tool in a script
//! - [`ToolInvocation`]: How a tool is invoked (uvx, npx, etc.)

mod registry;
mod types;

pub use registry::{PatternRegistry, ScriptPatternProvider};
pub use types::{ParseContext, ScriptTool, ToolInvocation};

use registry::DEFAULT_REGISTRY;

/// Script parser for detecting tool dependencies
///
/// This is the main entry point for script parsing. It delegates to
/// registered pattern providers to detect tools in script commands.
pub struct ScriptParser {
    registry: &'static PatternRegistry,
}

impl Default for ScriptParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptParser {
    /// Create a new script parser with the default registry
    pub fn new() -> Self {
        Self {
            registry: &DEFAULT_REGISTRY,
        }
    }

    /// Parse a script command and extract tool dependencies
    pub fn parse(&self, command: &str) -> Vec<ScriptTool> {
        self.parse_with_context(command, &[])
    }

    /// Parse a script command with context about known script names.
    ///
    /// When `known_scripts` is provided, references to those scripts via
    /// package manager commands (pnpm, yarn, npm) will be filtered out
    /// since they are internal script references, not external tools.
    pub fn parse_with_context(&self, command: &str, known_scripts: &[&str]) -> Vec<ScriptTool> {
        let mut tools = Vec::new();

        // Split by common command separators
        let parts = split_commands(command);

        let context = ParseContext { known_scripts };

        for part in parts {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            // Try each registered provider
            for provider in self.registry.providers() {
                if let Some(tool) = provider.parse(part, &context) {
                    tools.push(tool);
                    break; // Only one provider should match per command part
                }
            }
        }

        tools
    }
}

/// Split command by common separators (&&, ||, ;)
fn split_commands(command: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut current = command;

    while !current.is_empty() {
        // Find next separator
        let next_sep = current
            .find("&&")
            .map(|i| (i, 2))
            .into_iter()
            .chain(current.find("||").map(|i| (i, 2)))
            .chain(current.find(';').map(|i| (i, 1)))
            .min_by_key(|(i, _)| *i);

        if let Some((idx, len)) = next_sep {
            let part = &current[..idx];
            if !part.trim().is_empty() {
                parts.push(part.trim());
            }
            current = &current[idx + len..];
        } else {
            if !current.trim().is_empty() {
                parts.push(current.trim());
            }
            break;
        }
    }

    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_commands() {
        assert_eq!(split_commands("a && b"), vec!["a", "b"]);
        assert_eq!(split_commands("a || b"), vec!["a", "b"]);
        assert_eq!(split_commands("a; b"), vec!["a", "b"]);
        assert_eq!(split_commands("a && b || c"), vec!["a", "b", "c"]);
    }
}
