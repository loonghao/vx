//! Static metadata extractor for provider.star files.
//!
//! This module provides a lightweight parser that extracts metadata from
//! `provider.star` files **without executing the Starlark engine**.
//!
//! It reads static string/list literals from well-known function definitions
//! (e.g. `def name(): return "msvc"`) and the `runtimes` list, so that
//! Rust code can consume provider metadata at compile-time or startup without
//! spinning up a full Starlark VM.
//!
//! # Design
//!
//! The parser is intentionally simple: it only handles the subset of Starlark
//! that appears in vx provider files (string literals, list literals, dict
//! literals with string keys/values).  Dynamic expressions are ignored.
//!
//! # Example
//!
//! ```rust
//! use vx_starlark::metadata::StarMetadata;
//!
//! const STAR: &str = include_str!("../provider.star");
//! let meta = StarMetadata::parse(STAR);
//!
//! assert_eq!(meta.name, Some("msvc".to_string()));
//! assert_eq!(meta.description, Some("Microsoft Visual C++ Build Tools".to_string()));
//! ```

/// Metadata extracted from a `provider.star` file.
#[derive(Debug, Clone, Default)]
pub struct StarMetadata {
    /// Provider name (from `def name(): return "..."`)
    pub name: Option<String>,
    /// Provider description (from `def description(): return "..."`)
    pub description: Option<String>,
    /// Provider homepage (from `def homepage(): return "..."`)
    pub homepage: Option<String>,
    /// Provider repository (from `def repository(): return "..."`)
    pub repository: Option<String>,
    /// Provider license (from `def license(): return "..."`)
    pub license: Option<String>,
    /// Provider ecosystem (from `def ecosystem(): return "..."`)
    pub ecosystem: Option<String>,
    /// Supported platforms (from `def platforms(): return {...}`)
    pub platforms: Option<Vec<String>>,
    /// Runtime definitions extracted from the top-level `runtimes` list
    pub runtimes: Vec<StarRuntimeMeta>,
}

/// Metadata for a single runtime entry inside the `runtimes` list.
#[derive(Debug, Clone, Default)]
pub struct StarRuntimeMeta {
    /// Runtime name
    pub name: Option<String>,
    /// Executable name
    pub executable: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Aliases list
    pub aliases: Vec<String>,
    /// Platform constraint OS list
    pub platform_os: Vec<String>,
    /// Whether auto-installable
    pub auto_installable: Option<bool>,
}

impl StarMetadata {
    /// Parse metadata from the raw content of a `provider.star` file.
    ///
    /// This is a best-effort static parser.  Fields that cannot be determined
    /// statically are left as `None` / empty.
    pub fn parse(source: &str) -> Self {
        StarMetadata {
            name: extract_simple_return(source, "name"),
            description: extract_simple_return(source, "description"),
            homepage: extract_simple_return(source, "homepage"),
            repository: extract_simple_return(source, "repository"),
            license: extract_simple_return(source, "license"),
            ecosystem: extract_simple_return(source, "ecosystem"),
            platforms: extract_platforms_os(source),
            runtimes: extract_runtimes(source),
        }
    }

    /// Return the provider name, falling back to a default.
    pub fn name_or<'a>(&'a self, default: &'a str) -> &'a str {
        self.name.as_deref().unwrap_or(default)
    }

    /// Return the provider description, falling back to a default.
    pub fn description_or<'a>(&'a self, default: &'a str) -> &'a str {
        self.description.as_deref().unwrap_or(default)
    }

    /// Return the homepage URL, falling back to a default.
    pub fn homepage_or<'a>(&'a self, default: &'a str) -> &'a str {
        self.homepage.as_deref().unwrap_or(default)
    }

    /// Find a runtime by name.
    pub fn find_runtime(&self, name: &str) -> Option<&StarRuntimeMeta> {
        self.runtimes
            .iter()
            .find(|r| r.name.as_deref() == Some(name) || r.aliases.iter().any(|a| a == name))
    }

    /// Collect all aliases across all runtimes.
    pub fn all_aliases(&self) -> Vec<&str> {
        self.runtimes
            .iter()
            .flat_map(|r| r.aliases.iter().map(|a| a.as_str()))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Extract the return value of a simple `def <name>(): return "<value>"` function.
///
/// Handles both single-line and multi-line forms:
/// ```starlark
/// def name(): return "msvc"
///
/// def name():
///     return "msvc"
/// ```
fn extract_simple_return(source: &str, fn_name: &str) -> Option<String> {
    let pattern = format!("def {}()", fn_name);
    let start = source.find(&pattern)?;
    let after_def = &source[start + pattern.len()..];

    // Find the `return` keyword within the next ~200 chars
    let search_window = &after_def[..after_def.len().min(300)];
    let return_pos = search_window.find("return")?;
    let after_return = &search_window[return_pos + 6..].trim_start();

    // Extract the string literal
    extract_string_literal(after_return)
}

/// Extract a quoted string literal from the beginning of `s`.
///
/// Supports both `"..."` and `'...'` delimiters.
/// Returns the content without quotes.
fn extract_string_literal(s: &str) -> Option<String> {
    let s = s.trim_start();
    let quote = s.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let rest = &s[1..];
    let end = rest.find(quote)?;
    Some(rest[..end].to_string())
}

/// Extract the OS list from `def platforms(): return {"os": ["windows", ...]}`.
fn extract_platforms_os(source: &str) -> Option<Vec<String>> {
    let pattern = "def platforms()";
    let start = source.find(pattern)?;
    let after_def = &source[start + pattern.len()..];

    // Find `"os"` key within the next 500 chars
    let window = &after_def[..after_def.len().min(500)];
    let os_pos = window.find("\"os\"")?;
    let after_os = &window[os_pos + 4..];

    // Find the list `[...]`
    let list_start = after_os.find('[')?;
    let list_content = &after_os[list_start + 1..];
    let list_end = list_content.find(']')?;
    let list_str = &list_content[..list_end];

    Some(extract_string_list_items(list_str))
}

/// Extract string items from a comma-separated list body (without brackets).
///
/// e.g. `"windows", "linux"` → `["windows", "linux"]`
fn extract_string_list_items(s: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut remaining = s;
    while !remaining.is_empty() {
        remaining = remaining.trim_start();
        if remaining.is_empty() {
            break;
        }
        let quote = remaining.chars().next().unwrap();
        if quote != '"' && quote != '\'' {
            // Skip non-string tokens
            if let Some(pos) = remaining.find([',', ']']) {
                remaining = &remaining[pos + 1..];
            } else {
                break;
            }
            continue;
        }
        remaining = &remaining[1..];
        if let Some(end) = remaining.find(quote) {
            items.push(remaining[..end].to_string());
            remaining = &remaining[end + 1..];
            // Skip comma
            remaining = remaining.trim_start();
            if remaining.starts_with(',') {
                remaining = &remaining[1..];
            }
        } else {
            break;
        }
    }
    items
}

/// Extract the top-level `runtimes = [...]` list and parse each dict entry.
fn extract_runtimes(source: &str) -> Vec<StarRuntimeMeta> {
    // Find `runtimes = [`
    let marker = "runtimes = [";
    let start = match source.find(marker) {
        Some(p) => p + marker.len(),
        None => return Vec::new(),
    };

    // Find the matching closing `]` at the top level
    let list_body = match find_matching_bracket(source, start - 1, '[', ']') {
        Some(body) => body,
        None => return Vec::new(),
    };

    // Split into individual dict entries `{...}`
    parse_runtime_dicts(list_body)
}

/// Given the source and the position of an opening bracket, return the content
/// between the opening and its matching closing bracket.
fn find_matching_bracket(source: &str, open_pos: usize, open: char, close: char) -> Option<&str> {
    let bytes = source.as_bytes();
    if bytes[open_pos] != open as u8 {
        return None;
    }
    let mut depth = 0usize;
    let mut in_string = false;
    let mut string_char = b'"';
    let mut i = open_pos;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            if b == string_char && (i == 0 || bytes[i - 1] != b'\\') {
                in_string = false;
            }
        } else if b == b'"' || b == b'\'' {
            in_string = true;
            string_char = b;
        } else if b == open as u8 {
            depth += 1;
        } else if b == close as u8 {
            depth -= 1;
            if depth == 0 {
                return Some(&source[open_pos + 1..i]);
            }
        }
        i += 1;
    }
    None
}

/// Parse a list body (content between `[` and `]`) into runtime metadata structs.
fn parse_runtime_dicts(list_body: &str) -> Vec<StarRuntimeMeta> {
    let mut runtimes = Vec::new();
    let mut remaining = list_body;

    while let Some(dict_start) = remaining.find('{') {
        let Some(dict_body) = find_matching_bracket(remaining, dict_start, '{', '}') else {
            break;
        };

        runtimes.push(parse_runtime_dict(dict_body));

        // Advance past this dict
        let end_pos = dict_start + dict_body.len() + 2; // +2 for `{` and `}`
        if end_pos >= remaining.len() {
            break;
        }
        remaining = &remaining[end_pos..];
    }

    runtimes
}

/// Parse a single runtime dict body (content between `{` and `}`).
fn parse_runtime_dict(body: &str) -> StarRuntimeMeta {
    StarRuntimeMeta {
        name: extract_dict_string_value(body, "name"),
        executable: extract_dict_string_value(body, "executable"),
        description: extract_dict_string_value(body, "description"),
        aliases: extract_dict_string_list(body, "aliases"),
        platform_os: extract_dict_platform_os(body),
        auto_installable: extract_dict_bool_value(body, "auto_installable"),
    }
}

/// Extract a string value for a given key from a dict body.
///
/// Handles `"key": "value"` and `"key":  "value"` patterns.
fn extract_dict_string_value(body: &str, key: &str) -> Option<String> {
    // Try both quote styles for the key
    for key_str in &[format!("\"{}\"", key), format!("'{}'", key)] {
        if let Some(pos) = body.find(key_str.as_str()) {
            let after_key = &body[pos + key_str.len()..];
            // Skip `:` and whitespace
            let after_colon = after_key.trim_start();
            let after_colon = after_colon.trim_start_matches(':').trim_start();
            if let Some(val) = extract_string_literal(after_colon) {
                return Some(val);
            }
        }
    }
    None
}

/// Extract a bool value for a given key from a dict body.
fn extract_dict_bool_value(body: &str, key: &str) -> Option<bool> {
    for key_str in &[format!("\"{}\"", key), format!("'{}'", key)] {
        if let Some(pos) = body.find(key_str.as_str()) {
            let after_key = &body[pos + key_str.len()..];
            let after_colon = after_key.trim_start().trim_start_matches(':').trim_start();
            if after_colon.starts_with("True") {
                return Some(true);
            } else if after_colon.starts_with("False") {
                return Some(false);
            }
        }
    }
    None
}

/// Extract a string list value for a given key from a dict body.
fn extract_dict_string_list(body: &str, key: &str) -> Vec<String> {
    for key_str in &[format!("\"{}\"", key), format!("'{}'", key)] {
        if let Some(pos) = body.find(key_str.as_str()) {
            let after_key = &body[pos + key_str.len()..];
            let after_colon = after_key.trim_start().trim_start_matches(':').trim_start();
            if after_colon.starts_with('[')
                && let Some(list_body) = find_matching_bracket(after_colon, 0, '[', ']')
            {
                return extract_string_list_items(list_body);
            }
        }
    }
    Vec::new()
}

/// Extract the OS list from `"platform_constraint": {"os": [...]}` in a dict body.
fn extract_dict_platform_os(body: &str) -> Vec<String> {
    let key = "platform_constraint";
    for key_str in &[format!("\"{}\"", key), format!("'{}'", key)] {
        if let Some(pos) = body.find(key_str.as_str()) {
            let after_key = &body[pos + key_str.len()..];
            let after_colon = after_key.trim_start().trim_start_matches(':').trim_start();
            if after_colon.starts_with('{')
                && let Some(dict_body) = find_matching_bracket(after_colon, 0, '{', '}')
                && let Some(os_pos) = dict_body.find("\"os\"")
            {
                let after_os = &dict_body[os_pos + 4..];
                let after_colon2 = after_os.trim_start().trim_start_matches(':').trim_start();
                if after_colon2.starts_with('[')
                    && let Some(list_body) = find_matching_bracket(after_colon2, 0, '[', ']')
                {
                    return extract_string_list_items(list_body);
                }
            }
        }
    }
    Vec::new()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_STAR: &str = r#"
def name():
    return "msvc"

def description():
    return "Microsoft Visual C++ Build Tools"

def homepage():
    return "https://visualstudio.microsoft.com/visual-cpp-build-tools/"

def ecosystem():
    return "system"

def platforms():
    return {"os": ["windows"]}

runtimes = [
    {
        "name":             "msvc",
        "executable":       "cl",
        "description":      "Microsoft Visual C++ compiler",
        "aliases":          ["cl", "vs-build-tools", "msvc-tools"],
        "priority":         100,
        "auto_installable": True,
        "platform_constraint": {"os": ["windows"]},
    },
    {
        "name":             "nmake",
        "executable":       "nmake",
        "description":      "Microsoft Program Maintenance Utility",
        "bundled_with":     "msvc",
        "auto_installable": False,
        "platform_constraint": {"os": ["windows"]},
    },
]
"#;

    #[test]
    fn test_parse_name() {
        let meta = StarMetadata::parse(SAMPLE_STAR);
        assert_eq!(meta.name, Some("msvc".to_string()));
    }

    #[test]
    fn test_parse_description() {
        let meta = StarMetadata::parse(SAMPLE_STAR);
        assert_eq!(
            meta.description,
            Some("Microsoft Visual C++ Build Tools".to_string())
        );
    }

    #[test]
    fn test_parse_homepage() {
        let meta = StarMetadata::parse(SAMPLE_STAR);
        assert_eq!(
            meta.homepage,
            Some("https://visualstudio.microsoft.com/visual-cpp-build-tools/".to_string())
        );
    }

    #[test]
    fn test_parse_ecosystem() {
        let meta = StarMetadata::parse(SAMPLE_STAR);
        assert_eq!(meta.ecosystem, Some("system".to_string()));
    }

    #[test]
    fn test_parse_platforms() {
        let meta = StarMetadata::parse(SAMPLE_STAR);
        assert_eq!(meta.platforms, Some(vec!["windows".to_string()]));
    }

    #[test]
    fn test_parse_runtimes_count() {
        let meta = StarMetadata::parse(SAMPLE_STAR);
        assert_eq!(meta.runtimes.len(), 2);
    }

    #[test]
    fn test_parse_runtime_name() {
        let meta = StarMetadata::parse(SAMPLE_STAR);
        assert_eq!(meta.runtimes[0].name, Some("msvc".to_string()));
        assert_eq!(meta.runtimes[1].name, Some("nmake".to_string()));
    }

    #[test]
    fn test_parse_runtime_aliases() {
        let meta = StarMetadata::parse(SAMPLE_STAR);
        assert_eq!(
            meta.runtimes[0].aliases,
            vec!["cl", "vs-build-tools", "msvc-tools"]
        );
    }

    #[test]
    fn test_parse_runtime_auto_installable() {
        let meta = StarMetadata::parse(SAMPLE_STAR);
        assert_eq!(meta.runtimes[0].auto_installable, Some(true));
        assert_eq!(meta.runtimes[1].auto_installable, Some(false));
    }

    #[test]
    fn test_find_runtime_by_alias() {
        let meta = StarMetadata::parse(SAMPLE_STAR);
        let rt = meta.find_runtime("cl");
        assert!(rt.is_some());
        assert_eq!(rt.unwrap().name, Some("msvc".to_string()));
    }

    #[test]
    fn test_name_or_fallback() {
        let meta = StarMetadata::default();
        assert_eq!(meta.name_or("fallback"), "fallback");
    }
}
