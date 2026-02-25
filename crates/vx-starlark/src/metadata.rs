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
//! // RFC 0038 v5 top-level variable format
//! const STAR: &str = r#"
//! name = "msvc"
//! description = "Microsoft Visual C++ Build Tools"
//! ecosystem = "system"
//!
//! runtimes = [
//!     {"name": "msvc", "executable": "cl"},
//! ]
//! "#;
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
    /// pip package name (from `pip_package = "..."`).
    ///
    /// When set, `build_runtimes` will create a `ManifestDrivenRuntime` with
    /// `pip_package` set, enabling PyPI version fetching and pip installation.
    pub pip_package: Option<String>,
    /// Package alias (from `package_alias = {"ecosystem": "uvx", "package": "meson"}`).
    ///
    /// When set, `vx <name>` is routed to `vx <ecosystem>:<package>` (RFC 0033).
    /// For example, `package_alias = {"ecosystem": "uvx", "package": "meson"}`
    /// makes `vx meson` equivalent to `vx uvx meson`.
    pub package_alias: Option<(String, String)>,
    /// Supported package prefixes for ecosystem:package syntax (RFC 0027).
    ///
    /// When set, `vx <prefix>:<package>` will be routed to this provider.
    /// Example: `package_prefixes = ["deno"]` enables `vx deno:cowsay`.
    ///
    /// Extracted from `package_prefixes = ["deno", "npm"]` in provider.star.
    pub package_prefixes: Vec<String>,
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
    /// Parent runtime name (for bundled tools like ctest/cpack bundled with cmake)
    pub bundled_with: Option<String>,
    /// Shells provided by this runtime (RFC 0038)
    /// Each shell is (name, relative_path)
    pub shells: Vec<(String, String)>,
    /// Install dependencies (vx-managed runtimes that must be installed first)
    /// Format: ["7zip", "node>=18", ...] - each entry is a runtime name with optional version constraint
    pub install_deps: Vec<String>,
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
            pip_package: extract_simple_return(source, "pip_package"),
            package_alias: extract_package_alias(source),
            package_prefixes: extract_string_list_var(source, "package_prefixes"),
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

/// Extract a string value for a top-level variable or function return.
///
/// Supports two formats (RFC 0038 v5 top-level variables take priority):
/// 1. Top-level variable: `name = "value"` or `name = 'value'` (any spacing around `=`)
/// 2. Function return: `def name(): return "value"` (single or multi-line)
fn extract_simple_return(source: &str, fn_name: &str) -> Option<String> {
    // Try top-level variable format first (RFC 0038 v5): `name = "value"`
    // Handles any amount of whitespace around `=`, e.g. `name        = "node"`
    for line in source.lines() {
        let trimmed = line.trim();
        // Must start with the exact variable name followed by optional spaces then `=`
        if let Some(rest) = trimmed.strip_prefix(fn_name) {
            let rest = rest.trim_start();
            if let Some(after_eq_raw) = rest.strip_prefix('=') {
                let after_eq = after_eq_raw.trim_start();
                if let Some(val) = extract_string_literal(after_eq) {
                    return Some(val);
                }
            }
        }
    }

    // Fall back to function return format: `def name(): return "value"`
    let pattern = format!("def {}()", fn_name);
    let start = source.find(&pattern)?;
    let after_def = &source[start + pattern.len()..];

    // Find the `return` keyword within the next ~300 chars
    let search_window = &after_def[..after_def.len().min(300)];
    let return_pos = search_window.find("return")?;
    let after_return = search_window[return_pos + 6..].trim_start();

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

/// Extract the OS list from `platforms = {"os": [...]}` or `def platforms(): return {"os": [...]}`.
fn extract_platforms_os(source: &str) -> Option<Vec<String>> {
    // Try top-level variable format first (RFC 0038 v5): `platforms = {"os": [...]}`
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(after_prefix) = trimmed.strip_prefix("platforms =") {
            let after_eq = after_prefix.trim_start();
            if after_eq.starts_with('{')
                && let Some(dict_body) = find_matching_bracket(after_eq, 0, '{', '}')
                && let Some(os_pos) = dict_body.find("\"os\"")
            {
                let after_os = &dict_body[os_pos + 4..];
                let after_colon = after_os.trim_start().trim_start_matches(':').trim_start();
                if after_colon.starts_with('[')
                    && let Some(list_body) = find_matching_bracket(after_colon, 0, '[', ']')
                {
                    return Some(extract_string_list_items(list_body));
                }
            }
        }
    }

    // Fall back to function format: `def platforms(): return {"os": [...]}`
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

    // Split into individual dict entries `{...}` or function calls `runtime_def(...)`
    parse_runtime_entries(list_body)
}

/// Parse a list body into runtime metadata structs.
///
/// Handles two formats:
/// 1. Dict literals: `{"name": "foo", ...}`
/// 2. Function calls: `runtime_def("foo", ...)` and `bundled_runtime_def("foo", bundled_with="bar", ...)`
fn parse_runtime_entries(list_body: &str) -> Vec<StarRuntimeMeta> {
    let mut runtimes = Vec::new();
    let mut remaining = list_body;

    while !remaining.is_empty() {
        remaining = remaining.trim_start();
        if remaining.is_empty() {
            break;
        }

        // Dict literal: `{...}`
        if remaining.starts_with('{') {
            let Some(dict_body) = find_matching_bracket(remaining, 0, '{', '}') else {
                break;
            };
            runtimes.push(parse_runtime_dict(dict_body));
            let end_pos = dict_body.len() + 2; // +2 for `{` and `}`
            if end_pos >= remaining.len() {
                break;
            }
            remaining = &remaining[end_pos..];
            // Skip comma
            remaining = remaining.trim_start();
            if remaining.starts_with(',') {
                remaining = &remaining[1..];
            }
            continue;
        }

        // Function call: `runtime_def(...)` or `bundled_runtime_def(...)`
        if remaining.starts_with("bundled_runtime_def(") {
            let call_start = "bundled_runtime_def(".len();
            let Some(args_body) = find_matching_bracket(remaining, call_start - 1, '(', ')') else {
                break;
            };
            runtimes.push(parse_bundled_runtime_def_call(args_body));
            let end_pos = call_start + args_body.len() + 1; // +1 for closing `)`
            if end_pos >= remaining.len() {
                break;
            }
            remaining = &remaining[end_pos..];
            remaining = remaining.trim_start();
            if remaining.starts_with(',') {
                remaining = &remaining[1..];
            }
            continue;
        }

        if remaining.starts_with("runtime_def(") {
            let call_start = "runtime_def(".len();
            let Some(args_body) = find_matching_bracket(remaining, call_start - 1, '(', ')') else {
                break;
            };
            runtimes.push(parse_runtime_def_call(args_body));
            let end_pos = call_start + args_body.len() + 1; // +1 for closing `)`
            if end_pos >= remaining.len() {
                break;
            }
            remaining = &remaining[end_pos..];
            remaining = remaining.trim_start();
            if remaining.starts_with(',') {
                remaining = &remaining[1..];
            }
            continue;
        }

        // Skip unknown tokens (comments, whitespace, etc.)
        if let Some(pos) = remaining.find([',', '{', '}', '(', ')']) {
            if pos == 0 {
                remaining = &remaining[1..];
            } else {
                remaining = &remaining[pos..];
            }
        } else {
            break;
        }
    }

    runtimes
}

/// Parse a `runtime_def("name", aliases=[...], ...)` call.
///
/// Extracts: name (positional arg 0), aliases, executable, description, priority.
fn parse_runtime_def_call(args_body: &str) -> StarRuntimeMeta {
    // First positional argument is the name (a string literal)
    let name = extract_first_positional_string(args_body);
    let executable = extract_kwarg_string(args_body, "executable").or_else(|| name.clone());
    let description = extract_kwarg_string(args_body, "description");
    let aliases = extract_kwarg_string_list(args_body, "aliases");

    StarRuntimeMeta {
        name,
        executable,
        description,
        aliases,
        platform_os: Vec::new(),
        auto_installable: None,
        bundled_with: None,
        shells: Vec::new(),
        install_deps: Vec::new(),
    }
}

/// Parse a `bundled_runtime_def("name", bundled_with="parent", ...)` call.
///
/// Extracts: name (positional arg 0), bundled_with, executable, description.
fn parse_bundled_runtime_def_call(args_body: &str) -> StarRuntimeMeta {
    let name = extract_first_positional_string(args_body);
    let bundled_with = extract_kwarg_string(args_body, "bundled_with");
    let executable = extract_kwarg_string(args_body, "executable").or_else(|| name.clone());
    let description = extract_kwarg_string(args_body, "description");
    let aliases = extract_kwarg_string_list(args_body, "aliases");

    StarRuntimeMeta {
        name,
        executable,
        description,
        aliases,
        platform_os: Vec::new(),
        auto_installable: None,
        bundled_with,
        shells: Vec::new(),
        install_deps: Vec::new(),
    }
}

/// Extract the first positional string argument from a function call args body.
///
/// e.g. `"node", aliases=["nodejs"]` → `Some("node")`
fn extract_first_positional_string(args_body: &str) -> Option<String> {
    let trimmed = args_body.trim_start();
    extract_string_literal(trimmed)
}

/// Extract a keyword argument string value from a function call args body.
///
/// e.g. `"node", bundled_with = "go"` with key `"bundled_with"` → `Some("go")`
fn extract_kwarg_string(args_body: &str, key: &str) -> Option<String> {
    // Look for `key = "value"` or `key="value"` pattern
    let pattern = format!("{} =", key);
    let pattern2 = format!("{}=", key);

    for pat in &[pattern.as_str(), pattern2.as_str()] {
        if let Some(pos) = args_body.find(pat) {
            // Make sure this is a keyword arg (preceded by comma/whitespace/newline, not part of a string)
            let before = &args_body[..pos];
            // Simple heuristic: the char before the key should be whitespace, comma, or newline
            let is_kwarg = before.is_empty()
                || before
                    .chars()
                    .last()
                    .map(|c| c.is_whitespace() || c == ',')
                    .unwrap_or(true);
            if !is_kwarg {
                continue;
            }
            let after = &args_body[pos + pat.len()..];
            let after = after.trim_start();
            if let Some(val) = extract_string_literal(after) {
                return Some(val);
            }
        }
    }
    None
}

/// Extract a keyword argument string list from a function call args body.
///
/// e.g. `"node", aliases = ["nodejs", "node-js"]` with key `"aliases"` → `["nodejs", "node-js"]`
fn extract_kwarg_string_list(args_body: &str, key: &str) -> Vec<String> {
    let pattern = format!("{} =", key);
    let pattern2 = format!("{}=", key);

    for pat in &[pattern.as_str(), pattern2.as_str()] {
        if let Some(pos) = args_body.find(pat) {
            let before = &args_body[..pos];
            let is_kwarg = before.is_empty()
                || before
                    .chars()
                    .last()
                    .map(|c| c.is_whitespace() || c == ',')
                    .unwrap_or(true);
            if !is_kwarg {
                continue;
            }
            let after = &args_body[pos + pat.len()..];
            let after = after.trim_start();
            if after.starts_with('[')
                && let Some(list_body) = find_matching_bracket(after, 0, '[', ']')
            {
                return extract_string_list_items(list_body);
            }
        }
    }
    Vec::new()
}

/// Parse a list body (content between `[` and `]`) into runtime metadata structs.
///
/// Legacy function kept for compatibility — delegates to `parse_runtime_entries`.
#[allow(dead_code)]
fn parse_runtime_dicts(list_body: &str) -> Vec<StarRuntimeMeta> {
    parse_runtime_entries(list_body)
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

/// Parse a single runtime dict body (content between `{` and `}`).
fn parse_runtime_dict(body: &str) -> StarRuntimeMeta {
    StarRuntimeMeta {
        name: extract_dict_string_value(body, "name"),
        executable: extract_dict_string_value(body, "executable"),
        description: extract_dict_string_value(body, "description"),
        aliases: extract_dict_string_list(body, "aliases"),
        platform_os: extract_dict_platform_os(body),
        auto_installable: extract_dict_bool_value(body, "auto_installable"),
        bundled_with: extract_dict_string_value(body, "bundled_with"),
        shells: extract_dict_shells(body),
        install_deps: extract_dict_string_list(body, "install_deps"),
    }
}

/// Extract shells list from a dict body.
///
/// Format: `"shells": [{"name": "git-bash", "path": "git-bash.exe"}, ...]`
fn extract_dict_shells(body: &str) -> Vec<(String, String)> {
    let mut shells = Vec::new();

    // Find "shells": [
    let key_patterns = ["\"shells\"", "'shells'"];
    for key_pattern in key_patterns {
        if let Some(pos) = body.find(key_pattern) {
            let after_key = &body[pos + key_pattern.len()..];
            // Skip : and whitespace, then find [
            let after_colon = after_key.trim_start().trim_start_matches(':').trim_start();
            if after_colon.starts_with('[') {
                // Find the matching ]
                if let Some(list_body) = find_matching_bracket(after_colon, 0, '[', ']') {
                    // Parse each dict in the list
                    let mut remaining = list_body;
                    while let Some(dict_start) = remaining.find('{') {
                        if let Some(dict_body) =
                            find_matching_bracket(remaining, dict_start, '{', '}')
                        {
                            let name = extract_dict_string_value(dict_body, "name");
                            let path = extract_dict_string_value(dict_body, "path");
                            if let (Some(name), Some(path)) = (name, path) {
                                shells.push((name, path));
                            }
                            // Advance past this dict
                            let end_pos = dict_start + dict_body.len() + 2;
                            if end_pos >= remaining.len() {
                                break;
                            }
                            remaining = &remaining[end_pos..];
                        } else {
                            break;
                        }
                    }
                }
            }
            break;
        }
    }

    shells
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

/// Extract `package_alias = {"ecosystem": "...", "package": "..."}` from provider.star.
///
/// Returns `Some((ecosystem, package))` if found, `None` otherwise.
fn extract_package_alias(source: &str) -> Option<(String, String)> {
    // Find `package_alias = {`
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("package_alias") {
            let rest = rest.trim_start();
            if let Some(after_eq_raw) = rest.strip_prefix('=') {
                let after_eq = after_eq_raw.trim_start();
                if after_eq.starts_with('{')
                    && let Some(dict_body) = find_matching_bracket(after_eq, 0, '{', '}')
                {
                    let ecosystem = extract_dict_string_value(dict_body, "ecosystem")?;
                    let package = extract_dict_string_value(dict_body, "package")?;
                    return Some((ecosystem, package));
                }
            }
        }
    }
    None
}

/// Extract a top-level string list variable like `package_prefixes = ["deno", "npm"]`.
///
/// Supports both formats:
/// 1. Top-level variable: `package_prefixes = ["deno", "npm"]`
/// 2. Function return: `def package_prefixes(): return ["deno", "npm"]`
fn extract_string_list_var(source: &str, var_name: &str) -> Vec<String> {
    // Try top-level variable format first: `var_name = [...]`
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(var_name) {
            let rest = rest.trim_start();
            if let Some(after_eq_raw) = rest.strip_prefix('=') {
                let after_eq = after_eq_raw.trim_start();
                if after_eq.starts_with('[')
                    && let Some(list_body) = find_matching_bracket(after_eq, 0, '[', ']')
                {
                    return extract_string_list_items(list_body);
                }
            }
        }
    }

    // Fall back to function return format: `def var_name(): return [...]`
    let pattern = format!("def {}()", var_name);
    if let Some(start) = source.find(&pattern) {
        let after_def = &source[start + pattern.len()..];
        let window = &after_def[..after_def.len().min(500)];
        if let Some(return_pos) = window.find("return") {
            let after_return = &window[return_pos + 6..].trim_start();
            if after_return.starts_with('[')
                && let Some(list_body) = find_matching_bracket(after_return, 0, '[', ']')
            {
                return extract_string_list_items(list_body);
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

    // RFC 0038 v5: top-level variable format
    const SAMPLE_STAR_V5: &str = r#"
name        = "node"
description = "Node.js - JavaScript runtime built on Chrome's V8 engine"
homepage    = "https://nodejs.org"
repository  = "https://github.com/nodejs/node"
ecosystem   = "nodejs"

runtimes = [
    {
        "name":       "node",
        "executable": "node",
        "aliases":    ["nodejs"],
        "priority":   100,
    },
    {"name": "npm",  "executable": "npm",  "bundled_with": "node"},
    {"name": "npx",  "executable": "npx",  "bundled_with": "node"},
]
"#;

    #[test]
    fn test_parse_v5_name() {
        let meta = StarMetadata::parse(SAMPLE_STAR_V5);
        assert_eq!(meta.name, Some("node".to_string()));
    }

    #[test]
    fn test_parse_v5_description() {
        let meta = StarMetadata::parse(SAMPLE_STAR_V5);
        assert_eq!(
            meta.description,
            Some("Node.js - JavaScript runtime built on Chrome's V8 engine".to_string())
        );
    }

    #[test]
    fn test_parse_v5_ecosystem() {
        let meta = StarMetadata::parse(SAMPLE_STAR_V5);
        assert_eq!(meta.ecosystem, Some("nodejs".to_string()));
    }

    #[test]
    fn test_parse_v5_runtimes() {
        let meta = StarMetadata::parse(SAMPLE_STAR_V5);
        assert_eq!(meta.runtimes.len(), 3);
        assert_eq!(meta.runtimes[0].name, Some("node".to_string()));
        assert_eq!(meta.runtimes[0].aliases, vec!["nodejs"]);
        assert_eq!(meta.runtimes[1].name, Some("npm".to_string()));
        assert_eq!(meta.runtimes[1].bundled_with, Some("node".to_string()));
    }

    // RFC 0038: runtime_def() / bundled_runtime_def() function call format
    const SAMPLE_STAR_FUNC_CALLS: &str = r#"
name        = "node"
description = "Node.js JavaScript runtime"
ecosystem   = "nodejs"

runtimes = [
    runtime_def("node",
        aliases = ["nodejs"],
    ),
    bundled_runtime_def("npm",  bundled_with = "node"),
    bundled_runtime_def("npx",  bundled_with = "node"),
]
"#;

    #[test]
    fn test_parse_runtime_def_calls_count() {
        let meta = StarMetadata::parse(SAMPLE_STAR_FUNC_CALLS);
        assert_eq!(
            meta.runtimes.len(),
            3,
            "Expected 3 runtimes from runtime_def/bundled_runtime_def calls"
        );
    }

    #[test]
    fn test_parse_runtime_def_name() {
        let meta = StarMetadata::parse(SAMPLE_STAR_FUNC_CALLS);
        assert_eq!(meta.runtimes[0].name, Some("node".to_string()));
    }

    #[test]
    fn test_parse_runtime_def_aliases() {
        let meta = StarMetadata::parse(SAMPLE_STAR_FUNC_CALLS);
        assert_eq!(meta.runtimes[0].aliases, vec!["nodejs"]);
    }

    #[test]
    fn test_parse_bundled_runtime_def_name() {
        let meta = StarMetadata::parse(SAMPLE_STAR_FUNC_CALLS);
        assert_eq!(meta.runtimes[1].name, Some("npm".to_string()));
        assert_eq!(meta.runtimes[2].name, Some("npx".to_string()));
    }

    #[test]
    fn test_parse_bundled_runtime_def_bundled_with() {
        let meta = StarMetadata::parse(SAMPLE_STAR_FUNC_CALLS);
        assert_eq!(meta.runtimes[1].bundled_with, Some("node".to_string()));
        assert_eq!(meta.runtimes[2].bundled_with, Some("node".to_string()));
    }

    #[test]
    fn test_parse_node_provider_star() {
        // Test with the actual node provider.star content
        let content = vx_provider_node_star();
        let meta = StarMetadata::parse(content);
        assert_eq!(meta.name, Some("node".to_string()));
        let names: Vec<_> = meta
            .runtimes
            .iter()
            .filter_map(|r| r.name.as_deref())
            .collect();
        assert!(
            names.contains(&"node"),
            "Expected 'node' in runtimes, got: {:?}",
            names
        );
        assert!(
            names.contains(&"npm"),
            "Expected 'npm' in runtimes, got: {:?}",
            names
        );
        assert!(
            names.contains(&"npx"),
            "Expected 'npx' in runtimes, got: {:?}",
            names
        );
    }

    fn vx_provider_node_star() -> &'static str {
        // Inline a minimal version of node's provider.star for testing
        r#"
load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def",
     "fetch_versions_from_api",
     "system_permissions",
     "bin_subdir_layout", "bin_subdir_env", "bin_subdir_execute_path",
     "post_extract_permissions", "pre_run_ensure_deps")
load("@vx//stdlib:env.star", "env_prepend")

name        = "node"
description = "Node.js - JavaScript runtime built on Chrome's V8 engine"
homepage    = "https://nodejs.org"
repository  = "https://github.com/nodejs/node"
license     = "MIT"
ecosystem   = "nodejs"
aliases     = ["nodejs"]

runtimes = [
    runtime_def("node",
        aliases = ["nodejs"],
        test_commands = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "^v?\\d+\\.\\d+\\.\\d+"},
        ],
    ),
    bundled_runtime_def("npm",  bundled_with = "node",
        version_pattern = "^\\d+\\.\\d+\\.\\d+"),
    bundled_runtime_def("npx",  bundled_with = "node",
        version_pattern = "^\\d+\\.\\d+\\.\\d+"),
]
"#
    }
}
