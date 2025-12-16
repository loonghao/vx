//! Tool suggestion system for friendly error messages
//!
//! This module provides:
//! - Tool name aliases (e.g., "rust" -> "cargo", "python" -> "uv")
//! - Fuzzy matching using Levenshtein distance for typo suggestions
//! - GitHub issue links for unsupported tool requests

use strsim::levenshtein;

/// GitHub repository for issue creation
const GITHUB_REPO: &str = "https://github.com/loonghao/vx";

/// Threshold for Levenshtein distance to consider a match
const SIMILARITY_THRESHOLD: usize = 3;

/// Tool alias mapping - maps common tool names/aliases to their vx-supported equivalents
const TOOL_ALIASES: &[(&str, &str, &str)] = &[
    // Rust ecosystem
    ("rust", "cargo", "Rust's package manager and build tool"),
    ("rustc", "cargo", "Use 'cargo' for Rust development"),
    ("rustup", "cargo", "Use 'cargo' for Rust development"),
    // Python ecosystem
    ("python", "uv", "Fast Python package manager (uv)"),
    ("python3", "uv", "Fast Python package manager (uv)"),
    ("pip", "uv", "Fast Python package manager (uv)"),
    ("pip3", "uv", "Fast Python package manager (uv)"),
    (
        "poetry",
        "uv",
        "Consider using 'uv' for Python package management",
    ),
    (
        "pipenv",
        "uv",
        "Consider using 'uv' for Python package management",
    ),
    (
        "conda",
        "uv",
        "Consider using 'uv' for Python package management",
    ),
    // Node.js ecosystem
    ("nodejs", "node", "Node.js runtime"),
    ("js", "node", "Node.js JavaScript runtime"),
    ("javascript", "node", "Node.js JavaScript runtime"),
    ("npx", "npm", "npm package runner (use 'npm' or 'pnpm')"),
    ("deno", "bun", "Consider 'bun' as a fast JavaScript runtime"),
    // Go ecosystem
    ("golang", "go", "Go programming language"),
];

/// Represents a tool suggestion with context
#[derive(Debug, Clone)]
pub struct ToolSuggestion {
    /// The suggested tool name
    pub suggested_tool: String,
    /// Description or reason for the suggestion
    pub description: String,
    /// Whether this is an alias match (exact) or fuzzy match
    pub is_alias: bool,
}

/// Get suggestions for an unknown tool name
///
/// Returns suggestions based on:
/// 1. Known aliases (e.g., "rust" -> "cargo")
/// 2. Fuzzy matching against available tools
pub fn get_tool_suggestions(unknown_tool: &str, available_tools: &[String]) -> Vec<ToolSuggestion> {
    let mut suggestions = Vec::new();
    let unknown_lower = unknown_tool.to_lowercase();

    // Check for known aliases first
    for (alias, target, description) in TOOL_ALIASES {
        if unknown_lower == *alias {
            suggestions.push(ToolSuggestion {
                suggested_tool: target.to_string(),
                description: description.to_string(),
                is_alias: true,
            });
        }
    }

    // If we found an alias match, return it immediately
    if !suggestions.is_empty() {
        return suggestions;
    }

    // Fuzzy match against available tools
    let mut fuzzy_matches: Vec<(String, usize)> = available_tools
        .iter()
        .filter_map(|tool| {
            let distance = levenshtein(&unknown_lower, &tool.to_lowercase());
            if distance <= SIMILARITY_THRESHOLD && distance > 0 {
                Some((tool.clone(), distance))
            } else {
                None
            }
        })
        .collect();

    // Sort by distance (closest matches first)
    fuzzy_matches.sort_by_key(|(_, distance)| *distance);

    // Take top 3 matches
    for (tool, _) in fuzzy_matches.into_iter().take(3) {
        suggestions.push(ToolSuggestion {
            suggested_tool: tool,
            description: String::new(),
            is_alias: false,
        });
    }

    suggestions
}

/// Generate a GitHub issue URL for requesting a new tool
pub fn get_feature_request_url(tool_name: &str) -> String {
    let title = format!("Feature Request: Support for '{}'", tool_name);
    let body = format!(
        "## Tool Request\n\n\
        **Tool name:** `{}`\n\n\
        **Why would this tool be useful?**\n\n\
        <!-- Please describe why you'd like vx to support this tool -->\n\n\
        **Additional context**\n\n\
        <!-- Any other information about the tool -->",
        tool_name
    );

    format!(
        "{}/issues/new?title={}&body={}&labels=enhancement",
        GITHUB_REPO,
        urlencoding::encode(&title),
        urlencoding::encode(&body)
    )
}

/// Simple URL encoding for the issue URL
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut result = String::new();
        for c in s.chars() {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                    result.push(c);
                }
                ' ' => result.push_str("%20"),
                '\n' => result.push_str("%0A"),
                '#' => result.push_str("%23"),
                '&' => result.push_str("%26"),
                '\'' => result.push_str("%27"),
                '(' => result.push_str("%28"),
                ')' => result.push_str("%29"),
                ':' => result.push_str("%3A"),
                '/' => result.push_str("%2F"),
                '?' => result.push_str("%3F"),
                '=' => result.push_str("%3D"),
                '`' => result.push_str("%60"),
                '[' => result.push_str("%5B"),
                ']' => result.push_str("%5D"),
                '*' => result.push_str("%2A"),
                _ => {
                    for byte in c.to_string().as_bytes() {
                        result.push_str(&format!("%{:02X}", byte));
                    }
                }
            }
        }
        result
    }
}

/// Format suggestions for display
pub fn format_suggestions(suggestions: &[ToolSuggestion]) -> Vec<String> {
    suggestions
        .iter()
        .map(|s| {
            if s.description.is_empty() {
                s.suggested_tool.clone()
            } else {
                format!("{} - {}", s.suggested_tool, s.description)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_suggestion() {
        let available = vec!["cargo".to_string(), "node".to_string(), "uv".to_string()];
        let suggestions = get_tool_suggestions("rust", &available);

        assert!(!suggestions.is_empty());
        assert!(suggestions[0].is_alias);
        assert_eq!(suggestions[0].suggested_tool, "cargo");
    }

    #[test]
    fn test_fuzzy_suggestion() {
        let available = vec!["cargo".to_string(), "node".to_string(), "pnpm".to_string()];
        let suggestions = get_tool_suggestions("nod", &available);

        assert!(!suggestions.is_empty());
        assert!(!suggestions[0].is_alias);
        assert_eq!(suggestions[0].suggested_tool, "node");
    }

    #[test]
    fn test_python_alias() {
        let available = vec!["uv".to_string(), "node".to_string()];
        let suggestions = get_tool_suggestions("python", &available);

        assert!(!suggestions.is_empty());
        assert!(suggestions[0].is_alias);
        assert_eq!(suggestions[0].suggested_tool, "uv");
    }

    #[test]
    fn test_no_suggestion_for_unrelated() {
        let available = vec!["cargo".to_string(), "node".to_string()];
        let suggestions = get_tool_suggestions("zzzzzzz", &available);

        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_feature_request_url() {
        let url = get_feature_request_url("mytool");
        assert!(url.contains("github.com/loonghao/vx/issues/new"));
        assert!(url.contains("mytool"));
    }
}
