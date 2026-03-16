//! Static metadata extractor for provider.star files.
//!
//! This module provides a lightweight parser that extracts metadata from
//! `provider.star` files **without executing the Starlark engine**.

/// Metadata extracted from a `provider.star` file.
#[derive(Debug, Clone, Default)]
pub struct StarMetadata {
    /// Provider name (from `name = "..."` or `def name(): return "..."`)
    pub name: Option<String>,
    /// Provider description
    pub description: Option<String>,
    /// Provider homepage
    pub homepage: Option<String>,
    /// Provider repository
    pub repository: Option<String>,
    /// Provider license
    pub license: Option<String>,
    /// Provider ecosystem
    pub ecosystem: Option<String>,
    /// Supported platforms (from `platforms = {"os": [...]}`)
    pub platforms: Option<Vec<String>>,
    /// Runtime definitions extracted from the top-level `runtimes` list
    pub runtimes: Vec<StarRuntimeMeta>,
    /// pip package name (from `pip_package = "..."`)
    pub pip_package: Option<String>,
    /// Package alias (from `package_alias = {"ecosystem": "uvx", "package": "meson"}`)
    pub package_alias: Option<(String, String)>,
    /// Supported package prefixes for ecosystem:package syntax (RFC 0027)
    pub package_prefixes: Vec<String>,
    /// Minimum vx version required to use this provider (semver constraint)
    pub vx_version: Option<String>,
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
    pub install_deps: Vec<String>,
    /// Glob patterns for locating the executable on the system (for tools not on PATH, e.g. MSVC cl.exe)
    pub system_paths: Vec<String>,
    /// Priority (lower = higher priority)
    pub priority: Option<u32>,
}

impl StarMetadata {
    /// Parse metadata from the raw content of a `provider.star` file.
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
            vx_version: extract_simple_return(source, "vx_version"),
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
fn extract_simple_return(source: &str, fn_name: &str) -> Option<String> {
    // Try top-level variable format first: `name = "value"`
    for line in source.lines() {
        let trimmed = line.trim();
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
    let search_window = &after_def[..after_def.len().min(300)];
    let return_pos = search_window.find("return")?;
    let after_return = search_window[return_pos + 6..].trim_start();
    extract_string_literal(after_return)
}

/// Extract a quoted string literal from the beginning of `s`.
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
    // Try top-level variable format first: `platforms = {"os": [...]}`
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
    let window = &after_def[..after_def.len().min(500)];
    let os_pos = window.find("\"os\"")?;
    let after_os = &window[os_pos + 4..];
    let list_start = after_os.find('[')?;
    let list_content = &after_os[list_start + 1..];
    let list_end = list_content.find(']')?;
    let list_str = &list_content[..list_end];
    Some(extract_string_list_items(list_str))
}

/// Extract string items from a comma-separated list body (without brackets).
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
    let marker = "runtimes = [";
    let start = match source.find(marker) {
        Some(p) => p + marker.len(),
        None => return Vec::new(),
    };
    let list_body = match find_matching_bracket(source, start - 1, '[', ']') {
        Some(body) => body,
        None => return Vec::new(),
    };
    parse_runtime_entries(list_body, source)
}

/// Parse a list body into runtime metadata structs.
///
/// Handles dict literals `{...}`, `runtime_def(...)`, and `bundled_runtime_def(...)`.
fn parse_runtime_entries<'a>(list_body: &'a str, source: &'a str) -> Vec<StarRuntimeMeta> {
    let mut runtimes = Vec::new();
    let mut remaining = list_body;

    while !remaining.is_empty() {
        remaining = remaining.trim_start();
        if remaining.is_empty() {
            break;
        }

        // Skip comment lines
        if remaining.starts_with('#') {
            if let Some(newline) = remaining.find('\n') {
                remaining = &remaining[newline + 1..];
            } else {
                break;
            }
            continue;
        }

        // Dict literal: `{...}`
        if remaining.starts_with('{') {
            let Some(dict_body) = find_matching_bracket(remaining, 0, '{', '}') else {
                break;
            };
            runtimes.push(parse_runtime_dict(dict_body));
            let end_pos = dict_body.len() + 2;
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

        // Function call: `bundled_runtime_def(...)`
        if remaining.starts_with("bundled_runtime_def(") {
            let call_start = "bundled_runtime_def(".len();
            let Some(args_body) = find_matching_bracket(remaining, call_start - 1, '(', ')') else {
                break;
            };
            runtimes.push(parse_bundled_runtime_def_call(args_body));
            let end_pos = call_start + args_body.len() + 1;
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

        // Function call: `runtime_def(...)`
        if remaining.starts_with("runtime_def(") {
            let call_start = "runtime_def(".len();
            let Some(args_body) = find_matching_bracket(remaining, call_start - 1, '(', ')') else {
                break;
            };
            runtimes.push(parse_runtime_def_call(args_body, source));
            let end_pos = call_start + args_body.len() + 1;
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

        // Skip unknown tokens
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
/// `source` is the full provider.star content, used to resolve variable references
/// in `system_paths` (e.g. `system_paths = _MSVC_PATHS` where `_MSVC_PATHS` is a
/// top-level list variable).
fn parse_runtime_def_call(args_body: &str, source: &str) -> StarRuntimeMeta {
    let name = extract_first_positional_string(args_body);
    let executable = extract_kwarg_string(args_body, "executable").or_else(|| name.clone());
    let description = extract_kwarg_string(args_body, "description");
    let aliases = extract_kwarg_string_list(args_body, "aliases");
    let platform_os = extract_kwarg_platform_os(args_body);
    let auto_installable = extract_kwarg_bool(args_body, "auto_installable");
    let bundled_with = extract_kwarg_string(args_body, "bundled_with");
    let priority = extract_kwarg_u32(args_body, "priority");

    // system_paths may be a direct list `[...]` or a variable reference like `_MSVC_PATHS`.
    // Try direct list first; fall back to variable reference resolution.
    let system_paths = extract_kwarg_string_list_or_var(args_body, "system_paths", source);

    StarRuntimeMeta {
        name,
        executable,
        description,
        aliases,
        platform_os,
        auto_installable,
        bundled_with,
        shells: Vec::new(),
        install_deps: Vec::new(),
        system_paths,
        priority,
    }
}

/// Parse a `bundled_runtime_def("name", bundled_with="parent", ...)` call.
fn parse_bundled_runtime_def_call(args_body: &str) -> StarRuntimeMeta {
    let name = extract_first_positional_string(args_body);
    let bundled_with = extract_kwarg_string(args_body, "bundled_with");
    let executable = extract_kwarg_string(args_body, "executable").or_else(|| name.clone());
    let description = extract_kwarg_string(args_body, "description");
    let aliases = extract_kwarg_string_list(args_body, "aliases");
    let platform_os = extract_kwarg_platform_os(args_body);
    let auto_installable = extract_kwarg_bool(args_body, "auto_installable");
    let priority = extract_kwarg_u32(args_body, "priority");

    StarRuntimeMeta {
        name,
        executable,
        description,
        aliases,
        platform_os,
        auto_installable,
        bundled_with,
        shells: Vec::new(),
        install_deps: Vec::new(),
        system_paths: Vec::new(),
        priority,
    }
}

/// Extract the first positional string argument from a function call args body.
fn extract_first_positional_string(args_body: &str) -> Option<String> {
    let trimmed = args_body.trim_start();
    extract_string_literal(trimmed)
}

/// Extract a keyword argument string value from a function call args body.
fn extract_kwarg_string(args_body: &str, key: &str) -> Option<String> {
    let mut search_start = 0;
    while let Some(pos) = args_body[search_start..].find(key) {
        let actual_pos = search_start + pos;
        let after_key = &args_body[actual_pos + key.len()..];

        let before = &args_body[..actual_pos];
        let is_kwarg = before.is_empty()
            || before
                .chars()
                .last()
                .map(|c| c.is_whitespace() || c == ',')
                .unwrap_or(true);
        if !is_kwarg {
            search_start = actual_pos + key.len();
            continue;
        }

        let after_key_trimmed = after_key.trim_start();
        if let Some(after_equals_raw) = after_key_trimmed.strip_prefix('=') {
            let after_equals = &after_equals_raw.trim_start();
            if let Some(val) = extract_string_literal(after_equals) {
                return Some(val);
            }
        }

        search_start = actual_pos + key.len();
    }
    None
}

/// Extract a keyword argument string list from a function call args body.
fn extract_kwarg_string_list(args_body: &str, key: &str) -> Vec<String> {
    let mut search_start = 0;
    while let Some(pos) = args_body[search_start..].find(key) {
        let actual_pos = search_start + pos;
        let after_key = &args_body[actual_pos + key.len()..];

        let before = &args_body[..actual_pos];
        let is_kwarg = before.is_empty()
            || before
                .chars()
                .last()
                .map(|c| c.is_whitespace() || c == ',')
                .unwrap_or(true);
        if !is_kwarg {
            search_start = actual_pos + key.len();
            continue;
        }

        let after_key_trimmed = after_key.trim_start();
        if let Some(after_equals_raw) = after_key_trimmed.strip_prefix('=') {
            let after_equals = &after_equals_raw.trim_start();
            if after_equals.starts_with('[')
                && let Some(list_body) = find_matching_bracket(after_equals, 0, '[', ']')
            {
                return extract_string_list_items(list_body);
            }
        }

        search_start = actual_pos + key.len();
    }
    Vec::new()
}

/// Extract a keyword argument string list, supporting both direct lists and variable references.
///
/// This handles two forms:
/// 1. Direct list: `system_paths = ["path1", "path2"]`
/// 2. Variable reference: `system_paths = _MSVC_PATHS` where `_MSVC_PATHS = [...]` is defined
///    at the top level of the source file.
fn extract_kwarg_string_list_or_var(args_body: &str, key: &str, source: &str) -> Vec<String> {
    let mut search_start = 0;
    while let Some(pos) = args_body[search_start..].find(key) {
        let actual_pos = search_start + pos;
        let after_key = &args_body[actual_pos + key.len()..];

        let before = &args_body[..actual_pos];
        let is_kwarg = before.is_empty()
            || before
                .chars()
                .last()
                .map(|c| c.is_whitespace() || c == ',')
                .unwrap_or(true);
        if !is_kwarg {
            search_start = actual_pos + key.len();
            continue;
        }

        let after_key_trimmed = after_key.trim_start();
        if let Some(after_equals_raw) = after_key_trimmed.strip_prefix('=') {
            let after_equals = after_equals_raw.trim_start();

            // Case 1: Direct list `[...]`
            if after_equals.starts_with('[')
                && let Some(list_body) = find_matching_bracket(after_equals, 0, '[', ']')
            {
                return extract_string_list_items(list_body);
            }

            // Case 2: Variable reference (identifier like `_MSVC_PATHS`)
            // Extract the identifier name (alphanumeric + underscore)
            let var_name: String = after_equals
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !var_name.is_empty()
                && !var_name
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
            {
                // Look up the variable in the source file: `VAR_NAME = [...]`
                if let Some(resolved) = resolve_list_variable(source, &var_name) {
                    return resolved;
                }
            }
        }

        search_start = actual_pos + key.len();
    }
    Vec::new()
}

/// Resolve a top-level list variable from the source file.
///
/// Looks for `VAR_NAME = [...]` at the top level and returns the string items.
fn resolve_list_variable(source: &str, var_name: &str) -> Option<Vec<String>> {
    // Search for `VAR_NAME = [` pattern
    let pattern = format!("{} = [", var_name);
    let start = source.find(&pattern)?;
    let list_start = start + pattern.len() - 1; // position of '['
    let list_body = find_matching_bracket(&source[list_start..], 0, '[', ']')?;
    let items = extract_string_list_items(list_body);
    if items.is_empty() { None } else { Some(items) }
}

/// Extract a `platform_constraint = {"os": [...]}` keyword argument from a function call args body.
fn extract_kwarg_platform_os(args_body: &str) -> Vec<String> {
    let key = "platform_constraint";
    let mut search_start = 0;
    while let Some(pos) = args_body[search_start..].find(key) {
        let actual_pos = search_start + pos;
        let after_key = &args_body[actual_pos + key.len()..];

        let before = &args_body[..actual_pos];
        let is_kwarg = before.is_empty()
            || before
                .chars()
                .last()
                .map(|c| c.is_whitespace() || c == ',')
                .unwrap_or(true);
        if !is_kwarg {
            search_start = actual_pos + key.len();
            continue;
        }

        let after_key_trimmed = after_key.trim_start();
        if let Some(after_equals_raw) = after_key_trimmed.strip_prefix('=') {
            let after_equals = after_equals_raw.trim_start();
            if after_equals.starts_with('{')
                && let Some(dict_body) = find_matching_bracket(after_equals, 0, '{', '}')
                && let Some(os_pos) = dict_body.find("\"os\"")
            {
                let after_os = &dict_body[os_pos + 4..];
                let after_colon = after_os.trim_start().trim_start_matches(':').trim_start();
                if after_colon.starts_with('[')
                    && let Some(list_body) = find_matching_bracket(after_colon, 0, '[', ']')
                {
                    return extract_string_list_items(list_body);
                }
            }
        }

        search_start = actual_pos + key.len();
    }
    Vec::new()
}

/// Extract a boolean keyword argument from a function call args body.
fn extract_kwarg_bool(args_body: &str, key: &str) -> Option<bool> {
    let mut search_start = 0;
    while let Some(pos) = args_body[search_start..].find(key) {
        let actual_pos = search_start + pos;
        let after_key = &args_body[actual_pos + key.len()..];

        let before = &args_body[..actual_pos];
        let is_kwarg = before.is_empty()
            || before
                .chars()
                .last()
                .map(|c| c.is_whitespace() || c == ',')
                .unwrap_or(true);
        if !is_kwarg {
            search_start = actual_pos + key.len();
            continue;
        }

        let after_key_trimmed = after_key.trim_start();
        if let Some(after_equals_raw) = after_key_trimmed.strip_prefix('=') {
            let after_equals = after_equals_raw.trim_start();
            if after_equals.starts_with("True") {
                return Some(true);
            } else if after_equals.starts_with("False") {
                return Some(false);
            }
        }

        search_start = actual_pos + key.len();
    }
    None
}

/// Extract a u32 keyword argument from a function call args body.
fn extract_kwarg_u32(args_body: &str, key: &str) -> Option<u32> {
    let mut search_start = 0;
    while let Some(pos) = args_body[search_start..].find(key) {
        let actual_pos = search_start + pos;
        let after_key = &args_body[actual_pos + key.len()..];

        let before = &args_body[..actual_pos];
        let is_kwarg = before.is_empty()
            || before
                .chars()
                .last()
                .map(|c| c.is_whitespace() || c == ',')
                .unwrap_or(true);
        if !is_kwarg {
            search_start = actual_pos + key.len();
            continue;
        }

        let after_key_trimmed = after_key.trim_start();
        if let Some(after_equals_raw) = after_key_trimmed.strip_prefix('=') {
            let after_equals = after_equals_raw.trim_start();
            let digits: String = after_equals
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if !digits.is_empty() {
                return digits.parse().ok();
            }
        }

        search_start = actual_pos + key.len();
    }
    None
}

/// Legacy wrapper kept for compatibility.
#[allow(dead_code)]
fn parse_runtime_dicts(list_body: &str) -> Vec<StarRuntimeMeta> {
    parse_runtime_entries(list_body, "")
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
        system_paths: extract_dict_string_list(body, "system_paths"),
        priority: extract_dict_u32_value(body, "priority"),
    }
}

/// Extract shells list from a dict body.
fn extract_dict_shells(body: &str) -> Vec<(String, String)> {
    let mut shells = Vec::new();
    let key_patterns = ["\"shells\"", "'shells'"];
    for key_pattern in key_patterns {
        if let Some(pos) = body.find(key_pattern) {
            let after_key = &body[pos + key_pattern.len()..];
            let after_colon = after_key.trim_start().trim_start_matches(':').trim_start();
            if after_colon.starts_with('[')
                && let Some(list_body) = find_matching_bracket(after_colon, 0, '[', ']')
            {
                let mut remaining = list_body;
                while let Some(dict_start) = remaining.find('{') {
                    if let Some(dict_body) = find_matching_bracket(remaining, dict_start, '{', '}')
                    {
                        let name = extract_dict_string_value(dict_body, "name");
                        let path = extract_dict_string_value(dict_body, "path");
                        if let (Some(name), Some(path)) = (name, path) {
                            shells.push((name, path));
                        }
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
            break;
        }
    }
    shells
}

/// Extract a string value for a given key from a dict body.
fn extract_dict_string_value(body: &str, key: &str) -> Option<String> {
    for key_str in &[format!("\"{}\"", key), format!("'{}'", key)] {
        if let Some(pos) = body.find(key_str.as_str()) {
            let after_key = &body[pos + key_str.len()..];
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

/// Extract a u32 value for a given key from a dict body.
fn extract_dict_u32_value(body: &str, key: &str) -> Option<u32> {
    for key_str in &[format!("\"{}\"", key), format!("'{}'", key)] {
        if let Some(pos) = body.find(key_str.as_str()) {
            let after_key = &body[pos + key_str.len()..];
            let after_colon = after_key.trim_start().trim_start_matches(':').trim_start();
            let num_str: String = after_colon
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if !num_str.is_empty() {
                return num_str.parse().ok();
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
fn extract_package_alias(source: &str) -> Option<(String, String)> {
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
