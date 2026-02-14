# VX Provider Code Templates

Complete code templates for creating a new vx provider. Replace `{name}` with the tool name (lowercase), `{Name}` with PascalCase, and `{NAME}` with UPPERCASE.

## provider.toml (Required)

The provider manifest defines metadata, runtimes, and constraints declaratively.

### Simple Provider (Single Runtime, GitHub Releases)

```toml
# {Name} Provider Manifest
# This file defines the metadata, runtimes, and dependency constraints for {Name}.

[provider]
name = "{name}"
description = "{Description of the tool}"
homepage = "https://github.com/{owner}/{repo}"
repository = "https://github.com/{owner}/{repo}"
ecosystem = "devtools"  # Options: nodejs, python, rust, go, devtools, system, zig
license = "MIT"  # SPDX identifier of upstream tool's license (REQUIRED)
# license_note = "..."  # Optional: special notes about license implications

# {Name} runtime
[[runtimes]]
name = "{name}"
description = "{Name} - {brief description}"
executable = "{name}"
# aliases = ["{alias1}", "{alias2}"]  # Optional alternative names

[runtimes.versions]
source = "github-releases"  # Most common source
owner = "{owner}"
repo = "{repo}"
strip_v_prefix = true  # Set true if versions have 'v' prefix (e.g., v1.0.0 -> 1.0.0)

[runtimes.platforms.windows]
executable_extensions = [".exe"]

[runtimes.platforms.unix]
executable_extensions = []

# No external dependencies (statically compiled binary)
[[runtimes.constraints]]
when = "*"
recommends = []
```

### Provider with Dependencies

```toml
[provider]
name = "{name}"
description = "{Description}"
homepage = "https://example.com"
ecosystem = "nodejs"

[[runtimes]]
name = "{name}"
description = "{Name} tool"
executable = "{name}"

[runtimes.versions]
source = "github-releases"
owner = "{owner}"
repo = "{repo}"
strip_v_prefix = true

[runtimes.platforms.windows]
executable_extensions = [".exe"]

[runtimes.platforms.unix]
executable_extensions = []

# Version-specific dependency constraints
[[runtimes.constraints]]
when = "^1"  # Applies to 1.x versions
requires = [
    { runtime = "node", version = ">=12, <23", reason = "{Name} 1.x requires Node.js 12-22" }
]

[[runtimes.constraints]]
when = ">=2"  # Applies to 2.x and above
requires = [
    { runtime = "node", version = ">=18", recommended = "22", reason = "{Name} 2.x requires Node.js 18+" }
]
```

### Provider with Multiple Runtimes (Bundled Tools)

```toml
[provider]
name = "{name}"
description = "{Description}"
homepage = "https://example.com"
ecosystem = "python"

# Main runtime
[[runtimes]]
name = "{name}"
description = "{Name} main tool"
executable = "{name}"
aliases = ["{name}3"]

[runtimes.versions]
source = "github-releases"
owner = "{owner}"
repo = "{repo}"
strip_v_prefix = true

[runtimes.executable_config]
dir_pattern = "{name}-{version}"  # If extracted to subdirectory

[runtimes.platforms.windows]
executable_extensions = [".exe"]

[runtimes.platforms.unix]
executable_extensions = []

[[runtimes.constraints]]
when = ">=1.0"
recommends = [
    { runtime = "uv", version = "*", reason = "UV provides faster operations" }
]

# Bundled tool (shares installation with main runtime)
[[runtimes]]
name = "{name}x"
description = "{Name} runner tool"
executable = "{name}x"
bundled_with = "{name}"  # Uses same installation as main runtime

[runtimes.platforms.windows]
executable_extensions = [".exe"]

[runtimes.platforms.unix]
executable_extensions = []

[[runtimes.constraints]]
when = "*"
requires = [
    { runtime = "{name}", version = "*", reason = "{name}x is bundled with {name}" }
]
```

### Platform-Specific Provider (Windows Only)

```toml
[provider]
name = "{name}"
description = "{Description}"
homepage = "https://example.com"
ecosystem = "system"

# Provider-level platform constraint
[provider.platforms]
os = ["windows"]  # Only available on Windows

[[runtimes]]
name = "{name}"
description = "{Name} Windows tool"
executable = "{name}"

[runtimes.platforms.windows]
executable_extensions = [".exe"]
```

### Version Source Options

```toml
# GitHub Releases (most common)
[runtimes.versions]
source = "github-releases"
owner = "{owner}"
repo = "{repo}"
strip_v_prefix = true  # Remove 'v' prefix from versions

# GitHub Tags
[runtimes.versions]
source = "github-tags"
owner = "{owner}"
repo = "{repo}"
strip_v_prefix = true

# Node.js official
[runtimes.versions]
source = "nodejs-org"
lts_pattern = "lts/*"

# Python standalone builds
[runtimes.versions]
source = "python-build-standalone"
stable_pattern = "3.*"

# Go official
[runtimes.versions]
source = "go-dev"
stable_pattern = "stable"

# Zig official
[runtimes.versions]
source = "zig-download"
stable_pattern = "stable"
```

### Executable Configuration Options

```toml
# Default: executable in root directory
# No executable_config needed

# Executable in versioned subdirectory
[runtimes.executable_config]
dir_pattern = "{name}-{version}"
# Result: {name}-1.0.0/{name}.exe

# Executable with platform in path
[runtimes.executable_config]
dir_pattern = "{name}-v{version}-{platform}-{arch}"
# Result: {name}-v1.0.0-win-x64/{name}.exe

# Special extensions (.cmd for npm-style tools)
[runtimes.executable_config]
extensions = [".cmd", ".exe"]  # .cmd takes priority on Windows
```

## Cargo.toml

```toml
[package]
name = "vx-provider-{name}"
version.workspace = true
edition.workspace = true
description = "{Name} provider for vx"
license.workspace = true
repository.workspace = true
homepage.workspace = true
authors.workspace = true
keywords.workspace = true
categories.workspace = true
rust-version.workspace = true

[dependencies]
vx-runtime = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
async-trait = { workspace = true }
tokio = { workspace = true }
reqwest = { workspace = true }
chrono = { workspace = true }

[dev-dependencies]
rstest = { workspace = true }
```

## src/lib.rs

```rust
//! {Name} provider for vx
//!
//! This crate provides the {Name} provider for vx.
//! {Description of the tool}
//!
//! # Example
//!
//! ```ignore
//! use vx_provider_{name}::create_provider;
//!
//! let provider = create_provider();
//! assert_eq!(provider.name(), "{name}");
//! ```

mod config;
mod provider;
mod runtime;

pub use config::{Name}UrlBuilder;
pub use provider::{Name}Provider;
pub use runtime::{Name}Runtime;

use std::sync::Arc;
use vx_runtime::Provider;

/// Create a new {Name} provider instance
///
/// This is the main entry point for the provider, used by the registry.
pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new({Name}Provider::new())
}
```

## src/provider.rs

```rust
//! {Name} provider implementation
//!
//! Provides the {Name} runtime.

use crate::runtime::{Name}Runtime;
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};

/// {Name} provider
#[derive(Debug, Default)]
pub struct {Name}Provider;

impl {Name}Provider {
    /// Create a new {Name} provider
    pub fn new() -> Self {
        Self
    }
}

impl Provider for {Name}Provider {
    fn name(&self) -> &str {
        "{name}"
    }

    fn description(&self) -> &str {
        "{Description}"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new({Name}Runtime::new())]
    }

    fn supports(&self, name: &str) -> bool {
        name == "{name}"
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        if name == "{name}" {
            Some(Arc::new({Name}Runtime::new()))
        } else {
            None
        }
    }
}
```

## src/runtime.rs

```rust
//! {Name} runtime implementation
//!
//! {Description of the tool}
//! {Homepage URL}

use crate::config::{Name}UrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Ecosystem, GitHubReleaseOptions, Platform, Runtime, RuntimeContext, VerificationResult,
    VersionInfo,
};

/// {Name} runtime implementation
#[derive(Debug, Clone, Default)]
pub struct {Name}Runtime;

impl {Name}Runtime {
    /// Create a new {Name} runtime
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for {Name}Runtime {
    fn name(&self) -> &str {
        "{name}"
    }

    fn description(&self) -> &str {
        "{Description}"
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System  // Or: NodeJs, Python, Rust, Go
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://github.com/{owner}/{repo}".to_string(),
        );
        meta.insert(
            "documentation".to_string(),
            "{documentation_url}".to_string(),
        );
        meta.insert("category".to_string(), "{category}".to_string());
        meta
    }

    // ========== Executable Path Configuration ==========
    // Most providers only need to override 1-2 of these methods.
    // The framework automatically handles platform-specific extensions and path construction.

    // Override if executable name differs from runtime name
    // fn executable_name(&self) -> &str {
    //     "custom-exe-name"
    // }

    // Override for tools that use .cmd/.bat on Windows (like npm, yarn)
    // fn executable_extensions(&self) -> &[&str] {
    //     &[".cmd", ".exe"]
    // }

    // Override if executable is in a subdirectory
    // fn executable_dir_path(&self, version: &str, platform: &Platform) -> Option<String> {
    //     Some(format!("{name}-{}", version))
    // }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        ctx.fetch_github_releases(
            "{name}",
            "{owner}",
            "{repo}",
            GitHubReleaseOptions::new()
                .strip_v_prefix(false)  // true if versions have 'v' prefix
                .skip_prereleases(true),
        )
        .await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok({Name}UrlBuilder::download_url(version, platform))
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_name = {Name}UrlBuilder::get_executable_name(platform);
        let exe_path = install_path.join(exe_name);

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "{Name} executable not found at expected path: {}",
                    exe_path.display()
                )],
                vec!["Try reinstalling the runtime".to_string()],
            )
        }
    }
}
```

## src/config.rs

```rust
//! URL builder and platform configuration for {Name}
//!
//! {Name} releases are available at: https://github.com/{owner}/{repo}/releases
//! Download URL format: https://github.com/{owner}/{repo}/releases/download/{version}/{name}-{version}-{target}.{ext}

use vx_runtime::{Arch, Os, Platform};

/// URL builder for {Name} downloads
pub struct {Name}UrlBuilder;

impl {Name}UrlBuilder {
    /// Base URL for {Name} releases
    const BASE_URL: &'static str = "https://github.com/{owner}/{repo}/releases/download";

    /// Build the download URL for a specific version and platform
    pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
        let target = Self::get_target_triple(platform)?;
        let ext = Self::get_archive_extension(platform);
        Some(format!(
            "{}/{}/{name}-{}-{}.{}",
            Self::BASE_URL,
            version,
            version,
            target,
            ext
        ))
    }

    /// Get the target triple for the platform
    pub fn get_target_triple(platform: &Platform) -> Option<String> {
        match (&platform.os, &platform.arch) {
            // Windows
            (Os::Windows, Arch::X86_64) => Some("x86_64-pc-windows-msvc".to_string()),
            (Os::Windows, Arch::Aarch64) => Some("aarch64-pc-windows-msvc".to_string()),

            // macOS
            (Os::MacOS, Arch::X86_64) => Some("x86_64-apple-darwin".to_string()),
            (Os::MacOS, Arch::Aarch64) => Some("aarch64-apple-darwin".to_string()),

            // Linux (using musl for better compatibility)
            (Os::Linux, Arch::X86_64) => Some("x86_64-unknown-linux-musl".to_string()),
            (Os::Linux, Arch::Aarch64) => Some("aarch64-unknown-linux-musl".to_string()),

            _ => None,
        }
    }

    /// Get the archive extension for the platform
    pub fn get_archive_extension(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "zip",
            _ => "tar.gz",
        }
    }

    /// Get the executable name for the platform
    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "{name}.exe",
            _ => "{name}",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_url_linux_x64() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        let url = {Name}UrlBuilder::download_url("1.0.0", &platform);
        assert!(url.is_some());
        let url = url.unwrap();
        assert!(url.contains("github.com/{owner}/{repo}"));
        assert!(url.contains("1.0.0"));
    }

    #[test]
    fn test_download_url_windows_x64() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        let url = {Name}UrlBuilder::download_url("1.0.0", &platform);
        assert!(url.is_some());
        assert!(url.unwrap().ends_with(".zip"));
    }

    #[test]
    fn test_download_url_macos_arm64() {
        let platform = Platform {
            os: Os::MacOS,
            arch: Arch::Aarch64,
        };
        let url = {Name}UrlBuilder::download_url("1.0.0", &platform);
        assert!(url.is_some());
        assert!(url.unwrap().ends_with(".tar.gz"));
    }
}
```

## tests/runtime_tests.rs

```rust
//! Tests for {Name} runtime

use rstest::rstest;
use vx_provider_{name}::{{Name}Provider, {Name}Runtime, {Name}UrlBuilder};
use vx_runtime::{Arch, Ecosystem, Os, Platform, Provider, Runtime};

#[test]
fn test_runtime_name() {
    let runtime = {Name}Runtime::new();
    assert_eq!(runtime.name(), "{name}");
}

#[test]
fn test_runtime_description() {
    let runtime = {Name}Runtime::new();
    assert!(runtime.description().contains("{Name}"));
}

#[test]
fn test_runtime_ecosystem() {
    let runtime = {Name}Runtime::new();
    assert_eq!(runtime.ecosystem(), Ecosystem::System);
}

#[test]
fn test_runtime_metadata() {
    let runtime = {Name}Runtime::new();
    let meta = runtime.metadata();
    assert!(meta.contains_key("homepage"));
}

#[test]
fn test_provider_name() {
    let provider = {Name}Provider::new();
    assert_eq!(provider.name(), "{name}");
}

#[test]
fn test_provider_supports() {
    let provider = {Name}Provider::new();
    assert!(provider.supports("{name}"));
    assert!(!provider.supports("other"));
}

#[test]
fn test_provider_runtimes() {
    let provider = {Name}Provider::new();
    let runtimes = provider.runtimes();
    assert_eq!(runtimes.len(), 1);
    assert_eq!(runtimes[0].name(), "{name}");
}

#[test]
fn test_provider_get_runtime() {
    let provider = {Name}Provider::new();
    assert!(provider.get_runtime("{name}").is_some());
    assert!(provider.get_runtime("other").is_none());
}

#[rstest]
#[case(Os::Linux, Arch::X86_64, "x86_64-unknown-linux-musl")]
#[case(Os::Linux, Arch::Aarch64, "aarch64-unknown-linux-musl")]
#[case(Os::MacOS, Arch::X86_64, "x86_64-apple-darwin")]
#[case(Os::MacOS, Arch::Aarch64, "aarch64-apple-darwin")]
#[case(Os::Windows, Arch::X86_64, "x86_64-pc-windows-msvc")]
#[case(Os::Windows, Arch::Aarch64, "aarch64-pc-windows-msvc")]
fn test_target_triple(#[case] os: Os, #[case] arch: Arch, #[case] expected: &str) {
    let platform = Platform { os, arch };
    let triple = {Name}UrlBuilder::get_target_triple(&platform);
    assert_eq!(triple, Some(expected.to_string()));
}

#[rstest]
#[case(Os::Windows, "zip")]
#[case(Os::Linux, "tar.gz")]
#[case(Os::MacOS, "tar.gz")]
fn test_archive_extension(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform {
        os,
        arch: Arch::X86_64,
    };
    let ext = {Name}UrlBuilder::get_archive_extension(&platform);
    assert_eq!(ext, expected);
}

#[rstest]
#[case(Os::Windows, "{name}.exe")]
#[case(Os::Linux, "{name}")]
#[case(Os::MacOS, "{name}")]
fn test_executable_name(#[case] os: Os, #[case] expected: &str) {
    let platform = Platform {
        os,
        arch: Arch::X86_64,
    };
    let name = {Name}UrlBuilder::get_executable_name(&platform);
    assert_eq!(name, expected);
}

#[test]
fn test_download_url_format() {
    let platform = Platform {
        os: Os::Linux,
        arch: Arch::X86_64,
    };
    let url = {Name}UrlBuilder::download_url("1.0.0", &platform).unwrap();
    assert!(url.contains("github.com"));
    assert!(url.contains("1.0.0"));
    assert!(url.ends_with(".tar.gz"));
}

/// Test executable_relative_path uses the new layered API correctly
#[test]
fn test_executable_relative_path() {
    let runtime = {Name}Runtime::new();

    let linux_platform = Platform {
        os: Os::Linux,
        arch: Arch::X86_64,
    };
    assert_eq!(
        runtime.executable_relative_path("1.0.0", &linux_platform),
        "{name}"
    );

    let windows_platform = Platform {
        os: Os::Windows,
        arch: Arch::X86_64,
    };
    assert_eq!(
        runtime.executable_relative_path("1.0.0", &windows_platform),
        "{name}.exe"
    );
}

/// Test executable_extensions returns correct extensions
#[test]
fn test_executable_extensions() {
    let runtime = {Name}Runtime::new();
    // Default is [".exe"], override for .cmd tools
    assert_eq!(runtime.executable_extensions(), &[".exe"]);
}

/// Test executable_name returns correct base name
#[test]
fn test_executable_name_method() {
    let runtime = {Name}Runtime::new();
    // Default is same as name()
    assert_eq!(runtime.executable_name(), runtime.name());
}
```

## Alternative: Non-GitHub Version Fetching

For tools not hosted on GitHub, implement manual version fetching:

```rust
async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
    let url = "https://api.example.com/releases";
    let response: serde_json::Value = ctx.http.get_json_value(url).await?;
    
    let versions = response
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Invalid response"))?
        .iter()
        .filter_map(|release| {
            let version = release["version"].as_str()?;
            let date = release["published_at"].as_str();
            Some(
                VersionInfo::new(version)
                    .with_lts(false)
                    .with_prerelease(false)
                    .with_release_date(date.unwrap_or_default().to_string())
            )
        })
        .collect();
    
    Ok(versions)
}
```

## Alternative: Archive with Subdirectory

If the archive extracts to a subdirectory (e.g., `{name}-{version}/`), use the new layered API:

```rust
// Simple case: just override executable_dir_path
fn executable_dir_path(&self, version: &str, _platform: &Platform) -> Option<String> {
    Some(format!("{name}-{}", version))
}

// The framework will automatically construct:
// - Linux/macOS: {name}-{version}/{name}
// - Windows: {name}-{version}/{name}.exe
```

For more complex cases (like Node.js with different structures per platform):

```rust
fn executable_dir_path(&self, version: &str, platform: &Platform) -> Option<String> {
    let dir_name = format!("{name}-v{}-{}", version, platform.as_str());
    if platform.is_windows() {
        Some(dir_name)  // Windows: no bin subdirectory
    } else {
        Some(format!("{}/bin", dir_name))  // Unix: has bin subdirectory
    }
}
```

For tools using .cmd on Windows (npm, yarn, npx):

```rust
fn executable_extensions(&self) -> &[&str] {
    &[".cmd", ".exe"]  // .cmd takes priority on Windows
}

fn executable_dir_path(&self, version: &str, platform: &Platform) -> Option<String> {
    let dir_name = format!("node-v{}-{}", version, get_platform_string(platform));
    if platform.is_windows() {
        Some(dir_name)
    } else {
        Some(format!("{}/bin", dir_name))
    }
}
// Result on Windows: node-v22.0.0-win-x64/npm.cmd
// Result on Linux: node-v22.0.0-linux-x64/bin/npm
```

## Alternative: Multiple Runtimes per Provider

For providers with multiple runtimes (like node with node, npm, npx):

```rust
// provider.rs
impl Provider for {Name}Provider {
    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![
            Arc::new({Name}Runtime::new()),
            Arc::new({Name}ToolRuntime::new()),
        ]
    }

    fn supports(&self, name: &str) -> bool {
        matches!(name, "{name}" | "{name}-tool")
    }

    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        match name {
            "{name}" => Some(Arc::new({Name}Runtime::new())),
            "{name}-tool" => Some(Arc::new({Name}ToolRuntime::new())),
            _ => None,
        }
    }
}
```

---

# Project Analyzer Templates (Optional)

Use these templates when adding project analyzer support for a language/ecosystem.

## languages/{lang}/mod.rs

```rust
//! {Lang} project analyzer
//!
//! This module provides analysis for {Lang} projects, including:
//! - Dependency detection from {config_file}
//! - Script detection from {config_file} and common tools
//! - Required tool detection

mod analyzer;
mod dependencies;
mod rules;
mod scripts;

pub use analyzer::{Lang}Analyzer;
```

## languages/{lang}/rules.rs

```rust
//! {Lang} script detection rules
//!
//! Defines rules for detecting common {Lang} scripts based on file presence.

use crate::languages::rules::ScriptRule;

/// All {Lang} script detection rules
///
/// Rules are evaluated by priority (highest first).
/// For each script name, only the highest priority matching rule is used.
pub const {LANG}_RULES: &[ScriptRule] = &[
    // =========================================================================
    // Task runners (highest priority)
    // =========================================================================
    ScriptRule::new("{task_runner}", "{task_runner_cmd}", "Run {task_runner}")
        .triggers(&["{task_runner_config}"])
        .priority(100),
    
    // =========================================================================
    // Test runners
    // =========================================================================
    // Task runner-based testing (highest priority)
    ScriptRule::new("test", "{task_runner_cmd} test", "Run tests via {task_runner}")
        .triggers(&["{task_runner_config}"])
        .priority(100),
    
    // Default test runner
    ScriptRule::new("test", "{test_cmd}", "Run tests with {test_tool}")
        .triggers(&["{test_config}", "tests", "test"])
        .excludes(&["{task_runner_config}"])
        .priority(50),
    
    // =========================================================================
    // Linting
    // =========================================================================
    ScriptRule::new("lint", "{lint_cmd}", "Run linter")
        .triggers(&["{lint_config}", "{config_file}"])
        .excludes(&["{task_runner_config}"])
        .priority(50),
    
    // =========================================================================
    // Formatting
    // =========================================================================
    ScriptRule::new("format", "{format_cmd}", "Format code")
        .triggers(&["{format_config}", "{config_file}"])
        .excludes(&["{task_runner_config}"])
        .priority(50),
    
    // =========================================================================
    // Building
    // =========================================================================
    ScriptRule::new("build", "{build_cmd}", "Build project")
        .triggers(&["{config_file}"])
        .priority(50),
];
```

## languages/{lang}/dependencies.rs

```rust
//! {Lang} dependency parsing
//!
//! Parses dependencies from {config_file}

use crate::dependency::{Dependency, DependencySource};
use crate::ecosystem::Ecosystem;
use crate::error::AnalyzerResult;
use std::path::Path;

/// Parse dependencies from {config_file}
pub fn parse_{lang}_dependencies(content: &str, path: &Path) -> AnalyzerResult<Vec<Dependency>> {
    let mut deps = Vec::new();

    // Parse config file format (TOML, JSON, YAML, etc.)
    // Example for TOML:
    // let doc: toml::Value = toml::from_str(content)?;

    // Parse main dependencies
    // if let Some(dependencies) = doc.get("dependencies") {
    //     for (name, version) in dependencies... {
    //         let dep = Dependency::new(
    //             name,
    //             Ecosystem::{Ecosystem},
    //             DependencySource::ConfigFile {
    //                 path: path.to_path_buf(),
    //                 section: "dependencies".to_string(),
    //             },
    //         );
    //         deps.push(dep);
    //     }
    // }

    // Parse dev dependencies
    // Similar pattern with dep.is_dev = true

    Ok(deps)
}
```

## languages/{lang}/scripts.rs

```rust
//! {Lang} script parsing
//!
//! Parses explicit scripts from {config_file}

use crate::error::AnalyzerResult;
use crate::script_parser::ScriptParser;
use crate::types::{Script, ScriptSource};

/// Parse scripts from {config_file}
pub fn parse_{lang}_scripts(content: &str, parser: &ScriptParser) -> AnalyzerResult<Vec<Script>> {
    let mut scripts = Vec::new();

    // Parse config file format
    // let doc = parse_config(content)?;

    // Parse scripts section
    // if let Some(scripts_section) = doc.get("scripts") {
    //     for (name, cmd) in scripts_section {
    //         let mut script = Script::new(
    //             name,
    //             cmd,
    //             ScriptSource::{ConfigSource} {
    //                 section: "scripts".to_string(),
    //             },
    //         );
    //         script.tools = parser.parse(&cmd);
    //         scripts.push(script);
    //     }
    // }

    Ok(scripts)
}
```

## languages/{lang}/analyzer.rs

```rust
//! {Lang} project analyzer implementation

use super::dependencies::parse_{lang}_dependencies;
use super::rules::{LANG}_RULES;
use super::scripts::parse_{lang}_scripts;
use crate::dependency::{Dependency, InstallMethod};
use crate::ecosystem::Ecosystem;
use crate::error::AnalyzerResult;
use crate::languages::rules::{apply_rules, merge_scripts};
use crate::languages::LanguageAnalyzer;
use crate::script_parser::ScriptParser;
use crate::types::{RequiredTool, Script};
use async_trait::async_trait;
use std::path::Path;
use tracing::debug;

/// {Lang} project analyzer
pub struct {Lang}Analyzer {
    script_parser: ScriptParser,
}

impl {Lang}Analyzer {
    /// Create a new {Lang} analyzer
    pub fn new() -> Self {
        Self {
            script_parser: ScriptParser::new(),
        }
    }
}

impl Default for {Lang}Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LanguageAnalyzer for {Lang}Analyzer {
    fn detect(&self, root: &Path) -> bool {
        root.join("{config_file}").exists()
            // Add other detection patterns
            // || root.join("{alt_config}").exists()
    }

    fn name(&self) -> &'static str {
        "{Lang}"
    }

    async fn analyze_dependencies(&self, root: &Path) -> AnalyzerResult<Vec<Dependency>> {
        let mut deps = Vec::new();

        // Analyze main config file
        let config_path = root.join("{config_file}");
        if config_path.exists() {
            debug!("Analyzing {config_file}");
            let content = tokio::fs::read_to_string(&config_path).await?;
            deps.extend(parse_{lang}_dependencies(&content, &config_path)?);
        }

        // Check if dependencies are installed (via lock file or vendor dir)
        let has_lock = root.join("{lock_file}").exists();
        let has_vendor = root.join("{vendor_dir}").exists();

        if has_lock || has_vendor {
            for dep in &mut deps {
                dep.is_installed = true;
            }
        }

        Ok(deps)
    }

    async fn analyze_scripts(&self, root: &Path) -> AnalyzerResult<Vec<Script>> {
        // 1. Parse explicit scripts from config file (highest priority)
        let mut explicit_scripts = Vec::new();
        let config_path = root.join("{config_file}");
        if config_path.exists() {
            let content = tokio::fs::read_to_string(&config_path).await?;
            explicit_scripts = parse_{lang}_scripts(&content, &self.script_parser)?;
        }

        // 2. Apply detection rules for common scripts
        let detected_scripts = apply_rules(root, {LANG}_RULES, &self.script_parser);

        // 3. Merge: explicit scripts take priority over detected ones
        Ok(merge_scripts(explicit_scripts, detected_scripts))
    }

    fn required_tools(&self, _deps: &[Dependency], scripts: &[Script]) -> Vec<RequiredTool> {
        let mut tools = Vec::new();

        // Always need the main runtime for this language
        tools.push(RequiredTool::new(
            "{runtime}",
            Ecosystem::{Ecosystem},
            "{Lang} runtime",
            InstallMethod::vx("{runtime}"),
        ));

        // Check scripts for tool requirements
        for script in scripts {
            for tool in &script.tools {
                if !tool.is_available {
                    let install_method = InstallMethod::{install_method}(&tool.name);
                    tools.push(RequiredTool::new(
                        &tool.name,
                        Ecosystem::{Ecosystem},
                        format!("Required by script '{}'", script.name),
                        install_method,
                    ));
                }
            }
        }

        // Deduplicate by name
        tools.sort_by(|a, b| a.name.cmp(&b.name));
        tools.dedup_by(|a, b| a.name == b.name);

        tools
    }

    fn install_command(&self, dep: &Dependency) -> Option<String> {
        if dep.is_dev {
            Some(format!("{pkg_manager} add --dev {}", dep.name))
        } else {
            Some(format!("{pkg_manager} add {}", dep.name))
        }
    }
}
```

## Analyzer Test Template

```rust
//! Tests for {Lang} analyzer

use rstest::rstest;
use tempfile::TempDir;
use vx_project_analyzer::languages::{Lang}Analyzer;
use vx_project_analyzer::languages::LanguageAnalyzer;

#[test]
fn test_{lang}_project_detection() {
    let temp = TempDir::new().unwrap();
    std::fs::write(temp.path().join("{config_file}"), "{minimal_config}").unwrap();

    let analyzer = {Lang}Analyzer::new();
    assert!(analyzer.detect(temp.path()));
}

#[test]
fn test_{lang}_not_detected_without_config() {
    let temp = TempDir::new().unwrap();

    let analyzer = {Lang}Analyzer::new();
    assert!(!analyzer.detect(temp.path()));
}

#[tokio::test]
async fn test_{lang}_dependencies() {
    let temp = TempDir::new().unwrap();
    std::fs::write(
        temp.path().join("{config_file}"),
        r#"
        {config_with_deps}
        "#,
    )
    .unwrap();

    let analyzer = {Lang}Analyzer::new();
    let deps = analyzer.analyze_dependencies(temp.path()).await.unwrap();

    assert!(!deps.is_empty());
    assert!(deps.iter().any(|d| d.name == "{expected_dep}"));
}

#[tokio::test]
async fn test_{lang}_scripts() {
    let temp = TempDir::new().unwrap();
    std::fs::write(temp.path().join("{config_file}"), "{minimal_config}").unwrap();
    std::fs::create_dir(temp.path().join("tests")).unwrap();

    let analyzer = {Lang}Analyzer::new();
    let scripts = analyzer.analyze_scripts(temp.path()).await.unwrap();

    // Should detect test script
    assert!(scripts.iter().any(|s| s.name == "test"));
}

#[tokio::test]
async fn test_{lang}_script_priority() {
    let temp = TempDir::new().unwrap();
    // Create both task runner config and regular test config
    std::fs::write(temp.path().join("{task_runner_config}"), "").unwrap();
    std::fs::write(temp.path().join("{config_file}"), "{minimal_config}").unwrap();

    let analyzer = {Lang}Analyzer::new();
    let scripts = analyzer.analyze_scripts(temp.path()).await.unwrap();

    // Task runner should take priority
    let test_script = scripts.iter().find(|s| s.name == "test").unwrap();
    assert!(test_script.command.contains("{task_runner}"));
}

#[test]
fn test_{lang}_required_tools() {
    let analyzer = {Lang}Analyzer::new();
    let tools = analyzer.required_tools(&[], &[]);

    assert!(tools.iter().any(|t| t.name == "{runtime}"));
}
```
