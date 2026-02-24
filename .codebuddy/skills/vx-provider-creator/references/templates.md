# VX Provider Code Templates

Complete code templates for creating a new vx provider.
Replace `{name}` with the tool name (lowercase), `{Name}` with PascalCase, `{owner}` with GitHub owner, `{repo}` with GitHub repo.

---

# provider.star Templates (Starlark — Primary)

Use these templates for the preferred Starlark-based provider approach.

## Template A: Standard GitHub Provider (Most Common)

For tools distributed as GitHub releases with standard archive format.

```python
# provider.star - {name} provider
#
# Source: https://github.com/{owner}/{repo}/releases

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata (top-level variables — required)
# ---------------------------------------------------------------------------
name        = "{name}"
description = "{Description of the tool}"
homepage    = "https://github.com/{owner}/{repo}"
repository  = "https://github.com/{owner}/{repo}"
license     = "MIT"          # SPDX identifier (REQUIRED — check upstream license)
ecosystem   = "devtools"     # nodejs/python/rust/go/devtools/system/...
aliases     = []             # e.g. ["rg"] for ripgrep

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------
runtimes = [
    {
        "name":        "{name}",
        "executable":  "{name}",
        "description": "{Brief description}",
        "aliases":     [],
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "\\d+\\.\\d+"},
        ],
    },
]

# ---------------------------------------------------------------------------
# Permissions (sandbox declaration)
# ---------------------------------------------------------------------------
permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — fully inherited from github.star
# ---------------------------------------------------------------------------
fetch_versions = make_fetch_versions("{owner}", "{repo}")

# ---------------------------------------------------------------------------
# download_url — custom per-platform logic
# ---------------------------------------------------------------------------
def _{name}_triple(ctx):
    """Map platform to Rust target triple."""
    os   = ctx.platform.os
    arch = ctx.platform.arch
    triples = {
        "windows/x64":  "x86_64-pc-windows-msvc",
        "macos/x64":    "x86_64-apple-darwin",
        "macos/arm64":  "aarch64-apple-darwin",
        "linux/x64":    "x86_64-unknown-linux-musl",
        "linux/arm64":  "aarch64-unknown-linux-musl",
    }
    return triples.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    triple = _{name}_triple(ctx)
    if not triple:
        return None
    os  = ctx.platform.os
    ext = "zip" if os == "windows" else "tar.gz"
    # Adjust asset name pattern to match actual release assets:
    asset = "{name}-v{}-{}.{}".format(version, triple, ext)
    return github_asset_url("{owner}", "{repo}", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------
def install_layout(ctx, version):
    triple = _{name}_triple(ctx)
    os  = ctx.platform.os
    exe = "{name}.exe" if os == "windows" else "{name}"
    # Adjust strip_prefix to match the archive's top-level directory:
    strip_prefix = "{name}-{}-{}".format(version, triple) if triple else ""
    return {
        "type":             "archive",
        "strip_prefix":     strip_prefix,
        "executable_paths": [exe, "{name}"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------
def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------
def store_root(ctx):
    return ctx.vx_home + "/store/{name}"

def get_execute_path(ctx, version):
    os = ctx.platform.os
    exe = "{name}.exe" if os == "windows" else "{name}"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------
def deps(_ctx, _version):
    return []
```

## Template B: Fully Inherited (Zero-Code GitHub Provider)

For tools with perfectly standard GitHub release asset naming.

```python
# provider.star - {name} provider
#
# Asset format: {name}-v{version}-{triple}.{ext}

load("@vx//stdlib:github.star", "make_github_provider")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "{name}"
description = "{Description}"
homepage    = "https://github.com/{owner}/{repo}"
repository  = "https://github.com/{owner}/{repo}"
license     = "MIT"
ecosystem   = "devtools"
aliases     = []

runtimes = [
    {
        "name":        "{name}",
        "executable":  "{name}",
        "description": "{Brief description}",
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "\\d+"},
        ],
    },
]

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# Fully inherited: fetch_versions + download_url
# ---------------------------------------------------------------------------
_p             = make_github_provider("{owner}", "{repo}", "{name}-v{version}-{triple}.{ext}")
fetch_versions = _p["fetch_versions"]
download_url   = _p["download_url"]

def install_layout(ctx, version):
    os  = ctx.platform.os
    exe = "{name}.exe" if os == "windows" else "{name}"
    return {
        "type":             "archive",
        "strip_prefix":     "{name}-v{}".format(version),
        "executable_paths": [exe],
    }

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def store_root(ctx):
    return ctx.vx_home + "/store/{name}"

def get_execute_path(ctx, version):
    os = ctx.platform.os
    exe = "{name}.exe" if os == "windows" else "{name}"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
```

## Template C: Hybrid Provider (Direct Download + System Package Manager Fallback)

For tools with direct download on some platforms but requiring package managers on others.

```python
# provider.star - {name} provider
#
# Linux: direct binary/archive download
# Windows/macOS: system package manager fallback

load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "{name}"
description = "{Description}"
homepage    = "https://example.com"
repository  = "https://github.com/{owner}/{repo}"
license     = "MIT"
ecosystem   = "system"
aliases     = []

runtimes = [
    {
        "name":        "{name}",
        "executable":  "{name}",
        "description": "{Brief description}",
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "\\d+"},
        ],
    },
]

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": [],
}

fetch_versions = make_fetch_versions("{owner}", "{repo}")

# ---------------------------------------------------------------------------
# download_url — Linux only; Windows/macOS use system_install
# ---------------------------------------------------------------------------
def download_url(ctx, version):
    os   = ctx.platform.os
    arch = ctx.platform.arch
    if os == "linux" and arch == "x64":
        asset = "{name}-{}-linux-x64.tar.gz".format(version)
        return github_asset_url("{owner}", "{repo}", "v" + version, asset)
    # Windows/macOS: no portable archive → triggers system_install
    return None

# ---------------------------------------------------------------------------
# install_layout — Linux only
# ---------------------------------------------------------------------------
def install_layout(_ctx, _version):
    return {
        "type":             "archive",
        "strip_prefix":     "",
        "executable_paths": ["{name}"],
    }

# ---------------------------------------------------------------------------
# environment
# ---------------------------------------------------------------------------
def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

# ---------------------------------------------------------------------------
# system_install — Windows and macOS
# ---------------------------------------------------------------------------
def system_install(ctx):
    os = ctx.platform.os
    if os == "windows":
        return {
            "strategies": [
                {"manager": "winget", "package": "Publisher.{Name}", "priority": 95},
                {"manager": "choco",  "package": "{name}",           "priority": 80},
                {"manager": "scoop",  "package": "{name}",           "priority": 60},
            ],
        }
    elif os == "macos":
        return {
            "strategies": [
                {"manager": "brew", "package": "{name}", "priority": 90},
            ],
        }
    elif os == "linux":
        return {
            "strategies": [
                {"manager": "apt", "package": "{name}", "priority": 80},
                {"manager": "dnf", "package": "{name}", "priority": 80},
            ],
        }
    return {}

# ---------------------------------------------------------------------------
# uninstall — delegate to system package manager on Windows/macOS
# ---------------------------------------------------------------------------
def uninstall(ctx, _version):
    os = ctx.platform.os
    if os == "windows":
        return {
            "type": "system_uninstall",
            "strategies": [
                {"manager": "winget", "package": "Publisher.{Name}", "priority": 95},
                {"manager": "choco",  "package": "{name}",           "priority": 80},
            ],
        }
    elif os == "macos":
        return {
            "type": "system_uninstall",
            "strategies": [{"manager": "brew", "package": "{name}", "priority": 90}],
        }
    return False  # Linux: let vx remove the store directory

# ---------------------------------------------------------------------------
# Path queries (RFC 0037)
# ---------------------------------------------------------------------------
def store_root(ctx):
    return ctx.vx_home + "/store/{name}"

def get_execute_path(ctx, version):
    os = ctx.platform.os
    exe = "{name}.exe" if os == "windows" else "{name}"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
```

## Template D: PyPI / npm Package Alias (Ecosystem-Managed Tool)

For tools distributed as Python packages (PyPI) or npm packages.

```python
# provider.star - {name} provider (PyPI tool via uvx)
#
# RFC 0033: vx {name} → vx uvx:{name}

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "{name}"
description = "{Description}"
homepage    = "https://pypi.org/project/{name}/"
repository  = "https://github.com/{owner}/{repo}"
license     = "MIT"
ecosystem   = "python"
aliases     = []

# RFC 0033: route `vx {name}` → `vx uvx:{name}`
package_alias = {"ecosystem": "uvx", "package": "{name}"}

# For npm tools, use:
# package_alias = {"ecosystem": "npx", "package": "{name}"}

runtimes = [
    {
        "name":        "{name}",
        "executable":  "{name}",
        "description": "{Brief description}",
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "^\\d+\\.\\d+"},
        ],
    },
]

permissions = {
    "http": ["pypi.org"],   # or ["registry.npmjs.org"] for npm
    "fs":   [],
    "exec": ["uvx", "uv"],  # or ["npx", "node"] for npm
}

# ---------------------------------------------------------------------------
# Not applicable — runs via uvx/npx
# ---------------------------------------------------------------------------
def download_url(_ctx, _version):
    return None

def store_root(ctx):
    return ctx.vx_home + "/store/{name}"

def get_execute_path(_ctx, _version):
    return None

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return [
        {"runtime": "uv", "version": "*",
         "reason": "{name} is installed and run via uv"},
        # For npm tools:
        # {"runtime": "node", "version": ">=18", "reason": "{name} is run via npx"},
    ]
```

## Template E: MSI on Windows + Archive on Other Platforms

```python
# provider.star - {name} provider (MSI on Windows)

load("@vx//stdlib:install.star", "msi_install", "archive_install")
load("@vx//stdlib:github.star",  "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",     "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "{name}"
description = "{Description}"
homepage    = "https://example.com"
repository  = "https://github.com/{owner}/{repo}"
license     = "MIT"
ecosystem   = "devtools"
aliases     = []

runtimes = [
    {
        "name":        "{name}",
        "executable":  "{name}",
        "description": "{Brief description}",
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "\\d+"},
        ],
    },
]

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": [],
}

fetch_versions = make_fetch_versions("{owner}", "{repo}")

def download_url(ctx, version):
    os = ctx.platform.os
    if os == "windows":
        return "https://github.com/{owner}/{repo}/releases/download/v{}/{name}-{}-x64.msi".format(version, version)
    elif os == "macos":
        return github_asset_url("{owner}", "{repo}", "v" + version, "{name}-{}-macos.tar.gz".format(version))
    elif os == "linux":
        return github_asset_url("{owner}", "{repo}", "v" + version, "{name}-{}-linux.tar.gz".format(version))
    return None

def install_layout(ctx, version):
    os  = ctx.platform.os
    url = download_url(ctx, version)
    if os == "windows":
        return msi_install(
            url,
            executable_paths = ["bin/{name}.exe", "{name}.exe"],
        )
    else:
        return archive_install(
            url,
            strip_prefix = "{name}-{}".format(version),
            executable_paths = ["bin/{name}"],
        )

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def store_root(ctx):
    return ctx.vx_home + "/store/{name}"

def get_execute_path(ctx, version):
    os = ctx.platform.os
    exe = "{name}.exe" if os == "windows" else "{name}"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
```

## Template F: Non-GitHub Version Source

For tools with custom version APIs (go.dev, nodejs.org, PyPI, etc.).

```python
# provider.star - {name} provider (custom version API)

load("@vx//stdlib:http.star", "fetch_json_versions")
load("@vx//stdlib:env.star",  "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata
# ---------------------------------------------------------------------------
name        = "{name}"
description = "{Description}"
homepage    = "https://example.com"
repository  = "https://github.com/{owner}/{repo}"
license     = "MIT"
ecosystem   = "devtools"
aliases     = []

runtimes = [
    {
        "name":        "{name}",
        "executable":  "{name}",
        "description": "{Brief description}",
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "\\d+"},
        ],
    },
]

permissions = {
    "http": ["example.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — custom API
# Supported transforms: "go_versions", "nodejs_org", "pypi", "npm_registry",
#                       "hashicorp_releases", "adoptium", "github_tags"
# ---------------------------------------------------------------------------
def fetch_versions(ctx):
    return fetch_json_versions(ctx,
        "https://example.com/api/versions",
        "go_versions",  # replace with appropriate transform
    )

def download_url(ctx, version):
    os   = ctx.platform.os
    arch = ctx.platform.arch
    return "https://example.com/download/{}/{}-{}.tar.gz".format(version, os, arch)

def install_layout(ctx, version):
    os  = ctx.platform.os
    exe = "{name}.exe" if os == "windows" else "{name}"
    return {
        "type":             "archive",
        "strip_prefix":     "{name}-{}".format(version),
        "executable_paths": [exe],
    }

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def store_root(ctx):
    return ctx.vx_home + "/store/{name}"

def get_execute_path(ctx, version):
    os = ctx.platform.os
    exe = "{name}.exe" if os == "windows" else "{name}"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
    return []
```

---

# Starlark Provider Rust Shim Files

The minimal Rust files needed to wire a Starlark provider into the vx crate system.

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
vx-runtime  = { workspace = true }
vx-starlark = { workspace = true }
```

## src/lib.rs

```rust
//! {Name} provider for vx (Starlark-based)

pub const PROVIDER_STAR: &str = include_str!("../provider.star");

pub fn star_metadata() -> &'static vx_starlark::StarMetadata {
    use std::sync::OnceLock;
    static META: OnceLock<vx_starlark::StarMetadata> = OnceLock::new();
    META.get_or_init(|| vx_starlark::StarMetadata::parse(PROVIDER_STAR))
}

use std::sync::Arc;
use vx_runtime::Provider;

pub fn create_provider() -> Arc<dyn Provider> {
    Arc::new(vx_runtime::ManifestDrivenProvider::new(
        PROVIDER_STAR,
        include_str!("../provider.toml"),
    ))
}
```

## build.rs

```rust
fn main() {
    // Re-run if provider.star or provider.toml changes
    println!("cargo:rerun-if-changed=provider.star");
    println!("cargo:rerun-if-changed=provider.toml");
}
```

---

# provider.toml Templates (Supplementary)

The `provider.toml` manifest contains **metadata only**. All install logic
(download URLs, archive layout, system_install, environment) lives in `provider.star`.

## Standard provider.toml (Metadata Only)

```toml
[provider]
name        = "{name}"
description = "{Description of the tool}"
homepage    = "https://github.com/{owner}/{repo}"
repository  = "https://github.com/{owner}/{repo}"
ecosystem   = "devtools"   # nodejs/python/rust/go/devtools/system/...
license     = "MIT"        # SPDX identifier (REQUIRED)
```

## provider.toml with Platform Constraint

For tools only available on specific platforms:

```toml
[provider]
name        = "{name}"
description = "{Description}"
ecosystem   = "system"
license     = "MIT"

# Provider only available on Windows
[provider.platforms]
os = ["windows"]
```

## provider.toml with Bundled Runtimes

For tools bundled with another runtime (e.g., npm bundled with node):

```toml
[provider]
name        = "{name}"
description = "{Description}"
ecosystem   = "nodejs"
license     = "MIT"

[[runtimes]]
name         = "{name}x"
executable   = "{name}x"
bundled_with = "{name}"   # shares installation with main runtime

[[runtimes.constraints]]
when     = "*"
requires = [
    { runtime = "{name}", version = "*", reason = "{name}x is bundled with {name}" }
]
```

## Version Source Options (in provider.toml)

If not using `make_fetch_versions` in provider.star, you can declare the version source in provider.toml:

```toml
# GitHub Releases (most common)
[runtimes.versions]
source         = "github-releases"
owner          = "{owner}"
repo           = "{repo}"
strip_v_prefix = true

# GitHub Tags
[runtimes.versions]
source         = "github-tags"
owner          = "{owner}"
repo           = "{repo}"
strip_v_prefix = true

# Node.js official
[runtimes.versions]
source      = "nodejs-org"
lts_pattern = "lts/*"

# Python standalone builds
[runtimes.versions]
source          = "python-build-standalone"
stable_pattern  = "3.*"

# Go official
[runtimes.versions]
source         = "go-dev"
stable_pattern = "stable"

# Zig official
[runtimes.versions]
source         = "zig-download"
stable_pattern = "stable"
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

use crate::languages::rules::ScriptRule;

pub const {LANG}_RULES: &[ScriptRule] = &[
    // Task runners (highest priority)
    ScriptRule::new("{task_runner}", "{task_runner_cmd}", "Run {task_runner}")
        .triggers(&["{task_runner_config}"])
        .priority(100),

    // Test runners
    ScriptRule::new("test", "{task_runner_cmd} test", "Run tests via {task_runner}")
        .triggers(&["{task_runner_config}"])
        .priority(100),

    ScriptRule::new("test", "{test_cmd}", "Run tests with {test_tool}")
        .triggers(&["{test_config}", "tests", "test"])
        .excludes(&["{task_runner_config}"])
        .priority(50),

    // Linting
    ScriptRule::new("lint", "{lint_cmd}", "Run linter")
        .triggers(&["{lint_config}", "{config_file}"])
        .excludes(&["{task_runner_config}"])
        .priority(50),

    // Formatting
    ScriptRule::new("format", "{format_cmd}", "Format code")
        .triggers(&["{format_config}", "{config_file}"])
        .excludes(&["{task_runner_config}"])
        .priority(50),

    // Building
    ScriptRule::new("build", "{build_cmd}", "Build project")
        .triggers(&["{config_file}"])
        .priority(50),
];
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

pub struct {Lang}Analyzer {
    script_parser: ScriptParser,
}

impl {Lang}Analyzer {
    pub fn new() -> Self {
        Self { script_parser: ScriptParser::new() }
    }
}

impl Default for {Lang}Analyzer {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl LanguageAnalyzer for {Lang}Analyzer {
    fn detect(&self, root: &Path) -> bool {
        root.join("{config_file}").exists()
    }

    fn name(&self) -> &'static str { "{Lang}" }

    async fn analyze_dependencies(&self, root: &Path) -> AnalyzerResult<Vec<Dependency>> {
        let mut deps = Vec::new();
        let config_path = root.join("{config_file}");
        if config_path.exists() {
            debug!("Analyzing {config_file}");
            let content = tokio::fs::read_to_string(&config_path).await?;
            deps.extend(parse_{lang}_dependencies(&content, &config_path)?);
        }
        Ok(deps)
    }

    async fn analyze_scripts(&self, root: &Path) -> AnalyzerResult<Vec<Script>> {
        let mut explicit_scripts = Vec::new();
        let config_path = root.join("{config_file}");
        if config_path.exists() {
            let content = tokio::fs::read_to_string(&config_path).await?;
            explicit_scripts = parse_{lang}_scripts(&content, &self.script_parser)?;
        }
        let detected_scripts = apply_rules(root, {LANG}_RULES, &self.script_parser);
        Ok(merge_scripts(explicit_scripts, detected_scripts))
    }

    fn required_tools(&self, _deps: &[Dependency], scripts: &[Script]) -> Vec<RequiredTool> {
        let mut tools = vec![
            RequiredTool::new(
                "{runtime}",
                Ecosystem::{Ecosystem},
                "{Lang} runtime",
                InstallMethod::vx("{runtime}"),
            ),
        ];
        for script in scripts {
            for tool in &script.tools {
                if !tool.is_available {
                    tools.push(RequiredTool::new(
                        &tool.name,
                        Ecosystem::{Ecosystem},
                        format!("Required by script '{}'", script.name),
                        InstallMethod::{install_method}(&tool.name),
                    ));
                }
            }
        }
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
        r#"{config_with_deps}"#,
    ).unwrap();
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
    assert!(scripts.iter().any(|s| s.name == "test"));
}

#[test]
fn test_{lang}_required_tools() {
    let analyzer = {Lang}Analyzer::new();
    let tools = analyzer.required_tools(&[], &[]);
    assert!(tools.iter().any(|t| t.name == "{runtime}"));
}
```
