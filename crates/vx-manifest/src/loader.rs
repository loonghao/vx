//! Manifest loader for discovering and loading provider.toml and provider.star files

use crate::{
    Ecosystem, ManifestError, PlatformConstraint, ProviderManifest, ProviderOverride, Result,
    apply_override, extract_provider_name,
    provider::{ProviderMeta, RuntimeDef},
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Manifest loader - discovers and loads provider.toml files
#[derive(Debug, Default)]
pub struct ManifestLoader {
    /// Loaded manifests by provider name
    manifests: HashMap<String, ProviderManifest>,
    /// Manifest file paths by provider name
    paths: HashMap<String, PathBuf>,
    /// Pending overrides by provider name (applied when building)
    overrides: HashMap<String, Vec<ProviderOverride>>,
}

impl ManifestLoader {
    /// Create a new manifest loader
    pub fn new() -> Self {
        Self::default()
    }

    /// Load all manifests from a providers directory
    pub fn load_from_dir(&mut self, providers_dir: &Path) -> Result<usize> {
        let mut count = 0;

        if !providers_dir.exists() {
            return Ok(0);
        }

        let entries = std::fs::read_dir(providers_dir).map_err(ManifestError::Io)?;

        for entry in entries {
            let entry = entry.map_err(ManifestError::Io)?;
            let path = entry.path();

            if path.is_dir() {
                let toml_path = path.join("provider.toml");
                let star_path = path.join("provider.star");

                if toml_path.exists() {
                    match ProviderManifest::load(&toml_path) {
                        Ok(manifest) => {
                            let name = manifest.provider.name.clone();
                            self.paths.insert(name.clone(), toml_path);
                            self.manifests.insert(name, manifest);
                            count += 1;
                        }
                        Err(e) => {
                            // Log warning but continue loading other manifests
                            tracing::warn!("Failed to load manifest from {:?}: {}", toml_path, e);
                        }
                    }
                } else if star_path.exists() {
                    match std::fs::read_to_string(&star_path) {
                        Ok(content) => match star_to_manifest(&content) {
                            Some(manifest) => {
                                let name = manifest.provider.name.clone();
                                self.paths.insert(name.clone(), star_path);
                                self.manifests.insert(name, manifest);
                                count += 1;
                            }
                            None => {
                                tracing::warn!(
                                    "Failed to parse provider.star from {:?}: missing name",
                                    star_path
                                );
                            }
                        },
                        Err(e) => {
                            tracing::warn!(
                                "Failed to read provider.star from {:?}: {}",
                                star_path,
                                e
                            );
                        }
                    }
                }
            }
        }

        Ok(count)
    }

    /// Load manifests from embedded (name, content) tuples.
    /// Later entries with the same provider name override earlier ones.
    pub fn load_embedded<'a, I>(&mut self, manifests: I) -> Result<usize>
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        let mut count = 0;
        let mut parse_errors: Vec<String> = Vec::new();

        for (name, content) in manifests {
            match ProviderManifest::parse(content) {
                Ok(manifest) => {
                    let provider_name = manifest.provider.name.clone();
                    if provider_name != name {
                        tracing::warn!(
                            "Manifest name mismatch: embedded key '{}' differs from provider '{}'; using provider name",
                            name,
                            provider_name
                        );
                    }
                    self.insert(manifest);
                    count += 1;
                }
                Err(e) => {
                    // Create enhanced error with provider context
                    let context_error = match e {
                        ManifestError::Parse(toml_err) => {
                            ManifestError::parse_with_context(name, toml_err)
                        }
                        other => other,
                    };

                    let diagnostic = context_error.diagnostic_message();
                    tracing::warn!(
                        "Failed to parse manifest for provider '{}':\n{}",
                        name,
                        diagnostic
                    );
                    parse_errors.push(format!("  - {}: {}", name, context_error));
                }
            }
        }

        if !parse_errors.is_empty() {
            tracing::info!(
                "{} manifest(s) failed to parse. Run with --debug for details. Affected providers:\n{}",
                parse_errors.len(),
                parse_errors.join("\n")
            );
        }

        Ok(count)
    }

    /// Insert a manifest directly (used for overlays/overrides).
    pub fn insert(&mut self, manifest: ProviderManifest) {
        let name = manifest.provider.name.clone();
        self.manifests.insert(name.clone(), manifest);
        // Unknown path for embedded/override entries; use empty PathBuf as placeholder.
        self.paths.entry(name).or_default();
    }

    /// Load override files from a directory
    ///
    /// Override files are named `<provider>.override.toml` and contain
    /// constraint overrides for the corresponding provider.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// loader.load_overrides_from_dir(Path::new("~/.vx/providers"))?;
    /// ```
    pub fn load_overrides_from_dir(&mut self, dir: &Path) -> Result<usize> {
        let mut count = 0;

        if !dir.exists() {
            return Ok(0);
        }

        let entries = std::fs::read_dir(dir).map_err(ManifestError::Io)?;

        for entry in entries {
            let entry = entry.map_err(ManifestError::Io)?;
            let path = entry.path();

            if path.is_file()
                && let Some(filename) = path.file_name().and_then(|n| n.to_str())
                && let Some(provider_name) = extract_provider_name(filename)
            {
                match ProviderOverride::load(&path) {
                    Ok(override_config) => {
                        if !override_config.is_empty() {
                            self.overrides
                                .entry(provider_name.to_string())
                                .or_default()
                                .push(override_config);
                            count += 1;
                            tracing::debug!(
                                "Loaded override for '{}' from {:?}",
                                provider_name,
                                path
                            );
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load override from {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(count)
    }

    /// Apply all loaded overrides to manifests
    ///
    /// This should be called after loading all manifests and overrides.
    pub fn apply_overrides(&mut self) {
        for (provider_name, overrides) in &self.overrides {
            if let Some(manifest) = self.manifests.get_mut(provider_name) {
                for override_config in overrides {
                    apply_override(manifest, override_config);
                }
                tracing::debug!(
                    "Applied {} override(s) to provider '{}'",
                    overrides.len(),
                    provider_name
                );
            } else {
                tracing::warn!(
                    "Override for '{}' has no matching manifest - ignored",
                    provider_name
                );
            }
        }
    }

    /// Consume the loader and return all loaded manifests with overrides applied.
    pub fn into_manifests(mut self) -> Vec<ProviderManifest> {
        self.apply_overrides();
        self.manifests.into_values().collect()
    }

    /// Load a single manifest file
    pub fn load_file(&mut self, path: &Path) -> Result<()> {
        let manifest = ProviderManifest::load(path)?;
        let name = manifest.provider.name.clone();
        self.paths.insert(name.clone(), path.to_path_buf());
        self.manifests.insert(name, manifest);
        Ok(())
    }

    /// Get a manifest by provider name
    pub fn get(&self, name: &str) -> Option<&ProviderManifest> {
        self.manifests.get(name)
    }

    /// Get all loaded manifests
    pub fn all(&self) -> impl Iterator<Item = &ProviderManifest> {
        self.manifests.values()
    }

    /// Get the number of loaded manifests
    pub fn len(&self) -> usize {
        self.manifests.len()
    }

    /// Check if no manifests are loaded
    pub fn is_empty(&self) -> bool {
        self.manifests.is_empty()
    }

    /// Get manifest file path for a provider
    pub fn get_path(&self, name: &str) -> Option<&Path> {
        self.paths.get(name).map(|p| p.as_path())
    }

    /// Find a runtime definition across all manifests
    pub fn find_runtime(
        &self,
        runtime_name: &str,
    ) -> Option<(&ProviderManifest, &crate::RuntimeDef)> {
        for manifest in self.manifests.values() {
            if let Some(runtime) = manifest.get_runtime(runtime_name) {
                return Some((manifest, runtime));
            }
        }
        None
    }
}

// ============================================================================
// Inline provider.star parser
//
// This is a lightweight copy of the parsing logic from vx-starlark/src/metadata.rs.
// We duplicate it here to avoid a circular dependency:
//   vx-manifest → vx-starlark → vx-runtime → vx-manifest
//
// Keep in sync with vx-starlark/src/metadata.rs when the format changes.
// ============================================================================

/// Convert a `provider.star` file content into a `ProviderManifest`.
/// Returns `None` if the provider name cannot be determined.
fn star_to_manifest(content: &str) -> Option<ProviderManifest> {
    let name = star_extract_simple_return(content, "name")?;
    let description = star_extract_simple_return(content, "description");
    let homepage = star_extract_simple_return(content, "homepage");
    let repository = star_extract_simple_return(content, "repository");
    let ecosystem_str = star_extract_simple_return(content, "ecosystem");
    let platforms_os = star_extract_platforms_os(content);

    let ecosystem = match ecosystem_str.as_deref() {
        Some("nodejs") | Some("node") => Some(Ecosystem::NodeJs),
        Some("python") => Some(Ecosystem::Python),
        Some("rust") => Some(Ecosystem::Rust),
        Some("go") | Some("golang") => Some(Ecosystem::Go),
        Some("ruby") => Some(Ecosystem::Ruby),
        Some("java") => Some(Ecosystem::Java),
        Some("dotnet") | Some(".net") => Some(Ecosystem::DotNet),
        Some("devtools") => Some(Ecosystem::DevTools),
        Some("container") => Some(Ecosystem::Container),
        Some("cloud") => Some(Ecosystem::Cloud),
        Some("ai") => Some(Ecosystem::Ai),
        Some("cpp") | Some("c++") => Some(Ecosystem::Cpp),
        Some("zig") => Some(Ecosystem::Zig),
        Some("system") | Some(_) => Some(Ecosystem::System),
        None => None,
    };

    let platform_constraint = platforms_os.and_then(|os_list| {
        let os_vec: Vec<crate::Os> = os_list
            .iter()
            .filter_map(|s| match s.as_str() {
                "windows" => Some(crate::Os::Windows),
                "macos" | "darwin" => Some(crate::Os::MacOS),
                "linux" => Some(crate::Os::Linux),
                _ => None,
            })
            .collect();
        if os_vec.is_empty() {
            None
        } else {
            Some(PlatformConstraint {
                os: os_vec,
                ..Default::default()
            })
        }
    });

    let provider = ProviderMeta {
        name: name.clone(),
        description: description.clone(),
        homepage,
        repository,
        ecosystem,
        platform_constraint,
        package_alias: None,
    };

    let star_runtimes = star_extract_runtimes(content);
    let runtimes = if star_runtimes.is_empty() {
        vec![RuntimeDef {
            name: name.clone(),
            executable: name.clone(),
            description,
            aliases: vec![],
            bundled_with: None,
            managed_by: None,
            command_prefix: vec![],
            constraints: vec![],
            hooks: None,
            platforms: None,
            platform_constraint: None,
            versions: None,
            executable_config: None,
            layout: None,
            download: None,
            priority: None,
            auto_installable: None,
            detection: None,
            env_config: None,
            system_install: None,
            test: None,
            health: None,
            output: None,
            shell: None,
            system_deps: None,
            cache: None,
            mirrors: vec![],
            mirror_strategy: None,
            commands: vec![],
            normalize: None,
            version_ranges: None,
            bundled: None,
        }]
    } else {
        star_runtimes
            .iter()
            .map(|rt| {
                let rt_name = rt.name.clone().unwrap_or_else(|| name.clone());
                let executable = rt.executable.clone().unwrap_or_else(|| rt_name.clone());
                let rt_description = rt.description.clone();

                let rt_platform = if rt.platform_os.is_empty() {
                    None
                } else {
                    let os_vec: Vec<crate::Os> = rt
                        .platform_os
                        .iter()
                        .filter_map(|s| match s.as_str() {
                            "windows" => Some(crate::Os::Windows),
                            "macos" | "darwin" => Some(crate::Os::MacOS),
                            "linux" => Some(crate::Os::Linux),
                            _ => None,
                        })
                        .collect();
                    if os_vec.is_empty() {
                        None
                    } else {
                        Some(PlatformConstraint {
                            os: os_vec,
                            ..Default::default()
                        })
                    }
                };

                RuntimeDef {
                    name: rt_name,
                    executable,
                    description: rt_description,
                    aliases: rt.aliases.clone(),
                    platform_constraint: rt_platform,
                    bundled_with: rt.bundled_with.clone(),
                    managed_by: None,
                    command_prefix: rt.command_prefix.clone(),
                    constraints: vec![],
                    hooks: None,
                    platforms: None,
                    versions: None,
                    executable_config: None,
                    layout: None,
                    download: None,
                    priority: rt.priority.map(|p| p as i32),
                    auto_installable: rt.auto_installable,
                    detection: None,
                    env_config: None,
                    system_install: None,
                    test: None,
                    health: None,
                    output: None,
                    shell: None,
                    system_deps: None,
                    cache: None,
                    mirrors: vec![],
                    mirror_strategy: None,
                    commands: vec![],
                    normalize: None,
                    version_ranges: None,
                    bundled: None,
                }
            })
            .collect()
    };

    Some(ProviderManifest { provider, runtimes })
}

/// Metadata for a single runtime entry inside the `runtimes` list.
struct StarRuntimeMeta {
    name: Option<String>,
    executable: Option<String>,
    description: Option<String>,
    aliases: Vec<String>,
    platform_os: Vec<String>,
    auto_installable: Option<bool>,
    bundled_with: Option<String>,
    priority: Option<u32>,
    command_prefix: Vec<String>,
}

/// Extract a string value for a top-level variable or function return.
///
/// Supports two formats (RFC 0038 v5 top-level variables take priority):
/// 1. Top-level variable: `name = "value"` or `name = 'value'` (any spacing around `=`)
/// 2. Function return: `def name(): return "value"`
fn star_extract_simple_return(source: &str, fn_name: &str) -> Option<String> {
    // Try top-level variable format first (RFC 0038 v5): `name = "value"`
    // Handles any amount of whitespace around `=`, e.g. `name        = "node"`
    for line in source.lines() {
        let trimmed = line.trim();
        // Must start with the exact variable name followed by optional spaces then `=`
        if let Some(rest) = trimmed.strip_prefix(fn_name) {
            let rest = rest.trim_start();
            if rest.starts_with('=')
                && let after_eq = rest[1..].trim_start()
                && let Some(val) = star_extract_string_literal(after_eq)
            {
                return Some(val);
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
    star_extract_string_literal(after_return)
}

/// Extract a quoted string literal from the beginning of `s`.
fn star_extract_string_literal(s: &str) -> Option<String> {
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
fn star_extract_platforms_os(source: &str) -> Option<Vec<String>> {
    // Try top-level variable format first (RFC 0038 v5): `platforms = {"os": [...]}`
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(after_prefix) = trimmed.strip_prefix("platforms =") {
            let after_eq = after_prefix.trim_start();
            if after_eq.starts_with('{') {
                // Find the matching closing brace
                if let Some(dict_body) = star_find_matching_bracket(after_eq, 0, '{', '}')
                    && let Some(os_pos) = dict_body.find("\"os\"")
                {
                    let after_os = &dict_body[os_pos + 4..];
                    let after_colon = after_os.trim_start().trim_start_matches(':').trim_start();
                    if after_colon.starts_with('[')
                        && let Some(list_body) =
                            star_find_matching_bracket(after_colon, 0, '[', ']')
                    {
                        return Some(star_extract_string_list_items(list_body));
                    }
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
    Some(star_extract_string_list_items(list_str))
}

/// Extract string items from a comma-separated list body (without brackets).
fn star_extract_string_list_items(s: &str) -> Vec<String> {
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
fn star_extract_runtimes(source: &str) -> Vec<StarRuntimeMeta> {
    let marker = "runtimes = [";
    let start = match source.find(marker) {
        Some(p) => p + marker.len(),
        None => return Vec::new(),
    };
    let list_body = match star_find_matching_bracket(source, start - 1, '[', ']') {
        Some(body) => body,
        None => return Vec::new(),
    };
    star_parse_runtime_dicts(list_body)
}

/// Given the source and the position of an opening bracket, return the content
/// between the opening and its matching closing bracket.
fn star_find_matching_bracket(
    source: &str,
    open_pos: usize,
    open: char,
    close: char,
) -> Option<&str> {
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

/// Parse a list body into runtime metadata structs.
fn star_parse_runtime_dicts(list_body: &str) -> Vec<StarRuntimeMeta> {
    let mut runtimes = Vec::new();
    let mut remaining = list_body;
    while let Some(dict_start) = remaining.find('{') {
        let Some(dict_body) = star_find_matching_bracket(remaining, dict_start, '{', '}') else {
            break;
        };
        runtimes.push(star_parse_runtime_dict(dict_body));
        let end_pos = dict_start + dict_body.len() + 2;
        if end_pos >= remaining.len() {
            break;
        }
        remaining = &remaining[end_pos..];
    }
    runtimes
}

/// Parse a single runtime dict body.
fn star_parse_runtime_dict(body: &str) -> StarRuntimeMeta {
    StarRuntimeMeta {
        name: star_extract_dict_string_value(body, "name"),
        executable: star_extract_dict_string_value(body, "executable"),
        description: star_extract_dict_string_value(body, "description"),
        aliases: star_extract_dict_string_list(body, "aliases"),
        platform_os: star_extract_dict_platform_os(body),
        auto_installable: star_extract_dict_bool_value(body, "auto_installable"),
        bundled_with: star_extract_dict_string_value(body, "bundled_with"),
        priority: star_extract_dict_u32_value(body, "priority"),
        command_prefix: star_extract_dict_string_list(body, "command_prefix"),
    }
}

/// Extract a string value for a given key from a dict body.
fn star_extract_dict_string_value(body: &str, key: &str) -> Option<String> {
    for key_str in &[format!("\"{}\"", key), format!("'{}'", key)] {
        if let Some(pos) = body.find(key_str.as_str()) {
            let after_key = &body[pos + key_str.len()..];
            let after_colon = after_key.trim_start().trim_start_matches(':').trim_start();
            if let Some(val) = star_extract_string_literal(after_colon) {
                return Some(val);
            }
        }
    }
    None
}

/// Extract a bool value for a given key from a dict body.
fn star_extract_dict_bool_value(body: &str, key: &str) -> Option<bool> {
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
fn star_extract_dict_u32_value(body: &str, key: &str) -> Option<u32> {
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
fn star_extract_dict_string_list(body: &str, key: &str) -> Vec<String> {
    for key_str in &[format!("\"{}\"", key), format!("'{}'", key)] {
        if let Some(pos) = body.find(key_str.as_str()) {
            let after_key = &body[pos + key_str.len()..];
            let after_colon = after_key.trim_start().trim_start_matches(':').trim_start();
            if after_colon.starts_with('[')
                && let Some(list_body) = star_find_matching_bracket(after_colon, 0, '[', ']')
            {
                return star_extract_string_list_items(list_body);
            }
        }
    }
    Vec::new()
}

/// Extract the OS list from `"platform_constraint": {"os": [...]}` in a dict body.
fn star_extract_dict_platform_os(body: &str) -> Vec<String> {
    let key = "platform_constraint";
    for key_str in &[format!("\"{}\"", key), format!("'{}'", key)] {
        if let Some(pos) = body.find(key_str.as_str()) {
            let after_key = &body[pos + key_str.len()..];
            let after_colon = after_key.trim_start().trim_start_matches(':').trim_start();
            if after_colon.starts_with('{')
                && let Some(dict_body) = star_find_matching_bracket(after_colon, 0, '{', '}')
                && let Some(os_pos) = dict_body.find("\"os\"")
            {
                let after_os = &dict_body[os_pos + 4..];
                let after_colon2 = after_os.trim_start().trim_start_matches(':').trim_start();
                if after_colon2.starts_with('[')
                    && let Some(list_body) = star_find_matching_bracket(after_colon2, 0, '[', ']')
                {
                    return star_extract_string_list_items(list_body);
                }
            }
        }
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_manifest(dir: &Path, name: &str) {
        let provider_dir = dir.join(name);
        fs::create_dir_all(&provider_dir).unwrap();

        let manifest = format!(
            r#"
[provider]
name = "{name}"

[[runtimes]]
name = "{name}"
executable = "{name}"
"#
        );

        fs::write(provider_dir.join("provider.toml"), manifest).unwrap();
    }

    fn create_test_manifest_with_constraints(dir: &Path, name: &str) {
        let provider_dir = dir.join(name);
        fs::create_dir_all(&provider_dir).unwrap();

        let manifest = format!(
            r#"
[provider]
name = "{name}"

[[runtimes]]
name = "{name}"
executable = "{name}"

[[runtimes.constraints]]
when = "^1"
requires = [
    {{ runtime = "node", version = ">=12, <23" }}
]
"#
        );

        fs::write(provider_dir.join("provider.toml"), manifest).unwrap();
    }

    fn create_override_file(dir: &Path, provider_name: &str, content: &str) {
        let filename = format!("{}.override.toml", provider_name);
        fs::write(dir.join(filename), content).unwrap();
    }

    #[test]
    fn test_load_from_dir() {
        let temp_dir = TempDir::new().unwrap();

        create_test_manifest(temp_dir.path(), "test1");
        create_test_manifest(temp_dir.path(), "test2");

        let mut loader = ManifestLoader::new();
        let count = loader.load_from_dir(temp_dir.path()).unwrap();

        assert_eq!(count, 2);
        assert_eq!(loader.len(), 2);
        assert!(loader.get("test1").is_some());
        assert!(loader.get("test2").is_some());
    }

    #[test]
    fn test_find_runtime() {
        let temp_dir = TempDir::new().unwrap();
        create_test_manifest(temp_dir.path(), "myruntime");

        let mut loader = ManifestLoader::new();
        loader.load_from_dir(temp_dir.path()).unwrap();

        let result = loader.find_runtime("myruntime");
        assert!(result.is_some());

        let (manifest, runtime) = result.unwrap();
        assert_eq!(manifest.provider.name, "myruntime");
        assert_eq!(runtime.name, "myruntime");
    }

    #[test]
    fn test_load_overrides_from_dir() {
        let temp_dir = TempDir::new().unwrap();

        // Create a provider manifest
        create_test_manifest_with_constraints(temp_dir.path(), "yarn");

        // Create an override file
        let override_content = r#"
[[constraints]]
when = "^1"
requires = [
    { runtime = "node", version = ">=14, <21" }
]
"#;
        create_override_file(temp_dir.path(), "yarn", override_content);

        let mut loader = ManifestLoader::new();
        loader.load_from_dir(temp_dir.path()).unwrap();
        let override_count = loader.load_overrides_from_dir(temp_dir.path()).unwrap();

        assert_eq!(override_count, 1);

        // Get manifests with overrides applied
        let manifests = loader.into_manifests();
        assert_eq!(manifests.len(), 1);

        let manifest = &manifests[0];
        let runtime = manifest.get_runtime("yarn").unwrap();
        assert_eq!(runtime.constraints.len(), 1);
        assert_eq!(runtime.constraints[0].requires[0].version, ">=14, <21");
    }

    #[test]
    fn test_override_without_manifest() {
        let temp_dir = TempDir::new().unwrap();

        // Create an override file without a corresponding manifest
        let override_content = r#"
[[constraints]]
when = "*"
requires = [
    { runtime = "node", version = ">=18" }
]
"#;
        create_override_file(temp_dir.path(), "nonexistent", override_content);

        let mut loader = ManifestLoader::new();
        let override_count = loader.load_overrides_from_dir(temp_dir.path()).unwrap();

        // Override is loaded but won't be applied (no matching manifest)
        assert_eq!(override_count, 1);

        let manifests = loader.into_manifests();
        assert!(manifests.is_empty());
    }

    #[test]
    fn test_multiple_overrides() {
        let temp_dir = TempDir::new().unwrap();
        let user_dir = temp_dir.path().join("user");
        let project_dir = temp_dir.path().join("project");
        fs::create_dir_all(&user_dir).unwrap();
        fs::create_dir_all(&project_dir).unwrap();

        // Create a provider manifest in user dir
        create_test_manifest_with_constraints(&user_dir, "yarn");

        // Create user-level override
        let user_override = r#"
[[constraints]]
when = "^1"
requires = [
    { runtime = "node", version = ">=14, <21" }
]
"#;
        create_override_file(&user_dir, "yarn", user_override);

        // Create project-level override (should take precedence)
        let project_override = r#"
[[constraints]]
when = "^1"
requires = [
    { runtime = "node", version = ">=16, <20" }
]
"#;
        create_override_file(&project_dir, "yarn", project_override);

        let mut loader = ManifestLoader::new();
        loader.load_from_dir(&user_dir).unwrap();
        loader.load_overrides_from_dir(&user_dir).unwrap();
        loader.load_overrides_from_dir(&project_dir).unwrap();

        let manifests = loader.into_manifests();
        let manifest = &manifests[0];
        let runtime = manifest.get_runtime("yarn").unwrap();

        // Project override should win (applied last)
        assert_eq!(runtime.constraints[0].requires[0].version, ">=16, <20");
    }
}
