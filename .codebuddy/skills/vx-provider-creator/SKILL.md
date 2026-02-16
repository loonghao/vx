---
name: vx-provider-creator
description: |
  This skill should be used when creating a new runtime provider for the vx tool manager.
  It provides complete templates, code generation, and step-by-step guidance for implementing
  Provider and Runtime traits, including URL builders, platform configuration, test files,
  provider.toml manifest, system package manager fallback, and optionally project analyzer
  integration for language-specific tools.
  Use this skill when the user asks to add support for a new tool/runtime in vx.
---

# VX Provider Creator

This skill guides the creation of new runtime providers for the vx universal tool manager.

## When to Use

- Creating a new provider for a tool (e.g., "add support for ripgrep")
- Implementing a new runtime in vx
- Adding a new tool to the vx ecosystem
- Adding project analyzer support for a language/ecosystem
- Adding tools that require system package manager installation

## Workflow Overview

1. **Check license compatibility** (MUST DO FIRST)
2. Create a feature branch from remote main
3. **Determine installation type** (direct download vs system package manager)
4. Generate provider directory structure (including `provider.toml`)
5. Implement core files (lib.rs, provider.rs, runtime.rs, config.rs)
6. **Add system package manager fallback if needed**
7. Register the provider in workspace and CLI
8. **(Optional)** Add project analyzer integration for language-specific tools
9. Update snapshot tests
10. Verify and test

## ⚠️ License Compliance (MANDATORY - Step 0)

**Before creating ANY provider, you MUST check the upstream tool's license.**

### Blocked Licenses (DO NOT integrate)

These licenses have "copyleft infection" that would require vx itself to change license:

| License | Risk | Example |
|---------|------|---------|
| **AGPL-3.0** | Entire project must be AGPL | x-cmd |
| **SSPL** | Server-side copyleft | MongoDB |
| **CC BY-NC** | No commercial use | - |
| **Proprietary (no redistribution)** | Cannot bundle/distribute | - |

### Allowed Licenses (Safe to integrate)

| License | Type | Notes |
|---------|------|-------|
| **MIT** | Permissive | ✅ No restrictions |
| **Apache-2.0** | Permissive | ✅ Patent grant included |
| **BSD-2/BSD-3** | Permissive | ✅ Minimal restrictions |
| **ISC** | Permissive | ✅ Similar to MIT |
| **MPL-2.0** | Weak copyleft | ✅ File-level copyleft only |
| **Unlicense/CC0** | Public domain | ✅ No restrictions |

### Caution Licenses (Allowed with notes)

| License | Type | Notes |
|---------|------|-------|
| **GPL-2.0/GPL-3.0** | Strong copyleft | ⚠️ OK for vx since we only **download and execute** the tool (not link to it). Add `license_note` in provider.toml |
| **LGPL-2.1/LGPL-3.0** | Weak copyleft | ⚠️ Same as GPL - OK for download/execute. Document in provider.toml |
| **BSL-1.1** | Source-available | ⚠️ HashiCorp tools (terraform, vault). OK for version management. Document restriction |
| **Proprietary (free to use)** | Proprietary | ⚠️ OK if tool is free to download/use (e.g., dotnet, msvc). Add note |

### How to Check

1. Visit the tool's GitHub repository
2. Check the LICENSE file or repository metadata
3. Search for `license` in the repo's About section
4. If no license found, treat as **proprietary** and document

### provider.toml License Fields

Every `provider.toml` MUST include:

```toml
[provider]
name = "example"
license = "MIT"              # SPDX identifier of upstream tool's license
# license_note = "..."       # Optional: any special notes about license implications
```

**If the license is in the "Blocked" category, DO NOT create the provider. Inform the user:**

> ⚠️ Cannot integrate {tool}: it uses {license} which has copyleft infection
> that would require the entire vx project to adopt the same license.
> Consider using it via system package manager instead.

## Installation Type Decision Tree

Before creating a provider, determine the installation method:

```
Does the tool provide portable binaries for all platforms?
├─ Yes → Standard Download Provider
│   └─ Examples: terraform, just, kubectl, helm, go, node
└─ No → Check platform availability
    ├─ Some platforms have binaries → Hybrid Provider (download + package manager)
    │   └─ Examples: imagemagick (Linux AppImage, macOS/Windows via brew/winget)
    │   └─ Examples: ffmpeg (Windows binary, macOS/Linux via brew/apt)
    └─ No portable binaries → System Package Manager Only
        └─ Examples: make, git (on non-Windows), curl, openssl
```

### Provider Types Summary

| Type | Direct Download | Package Manager Fallback | Examples |
|------|-----------------|-------------------------|----------|
| **Standard** | ✅ All platforms | ❌ Not needed | terraform, just, go, node |
| **Hybrid** | ✅ Some platforms | ✅ For others | imagemagick, ffmpeg, docker |
| **System-only** | ❌ None | ✅ All platforms | make, curl, openssl |
| **Detection-only** | ❌ None | ❌ System-installed | msbuild, xcodebuild, systemctl |

## Step 1: Create Feature Branch

```bash
git fetch origin main
git checkout -b feature/{name}-provider origin/main
```

Replace `{name}` with the tool name (lowercase, e.g., `ripgrep`, `fd`).

## Step 2: Create Provider Directory Structure

Create the following structure under `crates/vx-providers/{name}/`:

```
crates/vx-providers/{name}/
├── Cargo.toml
├── provider.toml       # Provider manifest (metadata, runtimes, constraints)
├── src/
│   ├── lib.rs          # Module exports + create_provider() factory
│   ├── provider.rs     # Provider trait implementation
│   ├── runtime.rs      # Runtime trait implementation
│   └── config.rs       # URL builder and platform configuration
└── tests/
    └── runtime_tests.rs  # Unit tests (using rstest)
```

## Step 2.1: Create provider.toml Manifest

The `provider.toml` file is the declarative manifest for the provider. It defines:
- Provider metadata (name, description, homepage, ecosystem)
- Runtime definitions (executable, aliases, bundled tools)
- Version source configuration
- **RFC 0019: Layout configuration** (for binary/archive downloads)
- Platform-specific settings
- Dependency constraints

**Ecosystems available:** `nodejs`, `python`, `rust`, `go`, `ruby`, `java`, `dotnet`, `devtools`, `container`, `cloud`, `ai`, `cpp`, `zig`, `system`

**Version sources:**
- `github-releases` - GitHub Release API (most common)
- `github-tags` - GitHub Tags API
- `nodejs-org` - Node.js official releases
- `python-build-standalone` - Python standalone builds
- `go-dev` - Go official downloads
- `zig-download` - Zig official downloads

**RFC 0019 Layout Types:**
- `binary` - Single file download (needs renaming/placement)
- `archive` - Compressed archive (tar.gz, zip, tar.xz)
- `git_clone` - Git repository clone (for tools like vcpkg that install via git clone)

**Note on download_type values:** Use `snake_case` (e.g., `git_clone`), NOT `kebab-case` (e.g., ~~`git-clone`~~). The manifest parser uses `#[serde(rename_all = "snake_case")]`.

See `references/templates.md` for complete provider.toml template.
See `references/rfc-0019-layout.md` for RFC 0019 layout configuration guide.

## Step 3: Implement Core Files

Refer to `references/templates.md` for complete code templates.

### Key Implementation Points

**Cargo.toml**: Use workspace dependencies, package name `vx-provider-{name}`

**lib.rs**: Export types and provide `create_provider()` factory function

**provider.rs**: Implement Provider trait with:
- `name()` - Provider name (lowercase)
- `description()` - Human-readable description
- `runtimes()` - Return all Runtime instances
- `supports(name)` - Check if runtime name is supported
- `get_runtime(name)` - Get Runtime by name

**runtime.rs**: Implement Runtime trait with:
- `name()` - Runtime name
- `description()` - Description
- `aliases()` - Alternative names (if any)
- `ecosystem()` - One of: System, NodeJs, Python, Rust, Go
- `metadata()` - Homepage, documentation, category
- `fetch_versions(ctx)` - Fetch available versions
- `download_url(version, platform)` - Build download URL
- **Executable Path Configuration** (layered approach, most providers only need 1-2):
  - `executable_name()` - Base name of executable (default: `name()`)
  - `executable_extensions()` - Windows extensions (default: `[".exe"]`, use `[".cmd", ".exe"]` for npm/yarn)
  - `executable_dir_path(version, platform)` - Directory containing executable (default: install root)
  - `executable_relative_path(version, platform)` - Full path (auto-generated from above, rarely override)
- `verify_installation(version, install_path, platform)` - Verify installation

**config.rs**: Implement URL builder with:
- `download_url(version, platform)` - Full download URL
- `get_target_triple(platform)` - Platform target triple
- `get_archive_extension(platform)` - Archive extension (zip/tar.gz)
- `get_executable_name(platform)` - Executable name with extension

## Step 4: Register Provider

### 4.1 Update Root Cargo.toml

Add to `[workspace]` members:
```toml
"crates/vx-providers/{name}",
```

Add to `[workspace.dependencies]`:
```toml
vx-provider-{name} = { path = "crates/vx-providers/{name}" }
```

### 4.2 Update vx-cli/Cargo.toml

Add dependency:
```toml
vx-provider-{name} = { workspace = true }
```

### 4.3 Update registry.rs

In `crates/vx-cli/src/registry.rs`, add:
```rust
// Register {Name} provider
registry.register(vx_provider_{name}::create_provider());
```

## Step 5: Project Analyzer Integration (Optional)

If the new tool corresponds to a language/ecosystem (e.g., Go, Java, PHP), add project analyzer support.

### 5.1 Create Language Analyzer Directory

```
crates/vx-project-analyzer/src/languages/{lang}/
├── mod.rs          # Module exports
├── analyzer.rs     # {Lang}Analyzer implementation
├── dependencies.rs # Dependency parsing
├── rules.rs        # Script detection rules
└── scripts.rs      # Explicit script parsing
```

### 5.2 Define Script Detection Rules

```rust
// rules.rs
use crate::languages::rules::ScriptRule;

pub const {LANG}_RULES: &[ScriptRule] = &[
    ScriptRule::new("build", "{build_command}", "Build the project")
        .triggers(&["{config_file}"])
        .priority(50),
    ScriptRule::new("test", "{test_command}", "Run tests")
        .triggers(&["{test_config}", "tests"])
        .priority(50),
    ScriptRule::new("lint", "{lint_command}", "Run linter")
        .triggers(&["{lint_config}"])
        .excludes(&["{task_runner_config}"])
        .priority(50),
];
```

### 5.3 Implement LanguageAnalyzer

```rust
// analyzer.rs
use super::rules::{LANG}_RULES;
use crate::languages::rules::{apply_rules, merge_scripts};
use crate::languages::LanguageAnalyzer;

pub struct {Lang}Analyzer {
    script_parser: ScriptParser,
}

#[async_trait]
impl LanguageAnalyzer for {Lang}Analyzer {
    fn detect(&self, root: &Path) -> bool {
        root.join("{config_file}").exists()
    }

    fn name(&self) -> &'static str {
        "{Lang}"
    }

    async fn analyze_dependencies(&self, root: &Path) -> AnalyzerResult<Vec<Dependency>> {
        // Parse {config_file} for dependencies
    }

    async fn analyze_scripts(&self, root: &Path) -> AnalyzerResult<Vec<Script>> {
        // 1. Parse explicit scripts from config
        let explicit = parse_config_scripts(root, &self.script_parser).await?;
        
        // 2. Apply detection rules
        let detected = apply_rules(root, {LANG}_RULES, &self.script_parser);
        
        // 3. Merge (explicit takes priority)
        Ok(merge_scripts(explicit, detected))
    }

    fn required_tools(&self, _deps: &[Dependency], _scripts: &[Script]) -> Vec<RequiredTool> {
        vec![RequiredTool::new(
            "{tool}",
            Ecosystem::{Ecosystem},
            "{Tool} runtime",
            InstallMethod::vx("{tool}"),
        )]
    }

    fn install_command(&self, dep: &Dependency) -> Option<String> {
        Some(format!("{package_manager} add {}", dep.name))
    }
}
```

### 5.4 Register Analyzer

In `crates/vx-project-analyzer/src/languages/mod.rs`:

```rust
mod {lang};
pub use {lang}::{Lang}Analyzer;

pub fn all_analyzers() -> Vec<Box<dyn LanguageAnalyzer>> {
    vec![
        // ... existing analyzers
        Box::new({Lang}Analyzer::new()),
    ]
}
```

### 5.5 Add Analyzer Tests

```rust
// crates/vx-project-analyzer/tests/analyzer_tests.rs

#[tokio::test]
async fn test_{lang}_project_detection() {
    let temp = TempDir::new().unwrap();
    std::fs::write(temp.path().join("{config_file}"), "...").unwrap();
    
    let analyzer = {Lang}Analyzer::new();
    assert!(analyzer.detect(temp.path()));
}

#[tokio::test]
async fn test_{lang}_scripts() {
    let temp = TempDir::new().unwrap();
    std::fs::write(temp.path().join("{config_file}"), "...").unwrap();
    
    let analyzer = {Lang}Analyzer::new();
    let scripts = analyzer.analyze_scripts(temp.path()).await.unwrap();
    
    assert!(scripts.iter().any(|s| s.name == "test"));
}
```

## Step 6: Update Snapshot Tests

Update provider/runtime counts in:
- `tests/cmd/plugin/plugin-stats.md` - Increment "Total providers" and "Total runtimes"
- `tests/cmd/search/search.md` - Add the new runtime to the search results

## Step 7: Add Documentation

Add documentation for the new tool in the appropriate category:

### English Documentation (`docs/tools/`)

| Category | File | Tools |
|----------|------|-------|
| DevOps | `devops.md` | terraform, docker, kubectl, helm, git |
| Cloud CLI | `cloud.md` | aws, az, gcloud |
| Build Tools | `build-tools.md` | just, task, cmake, ninja, protoc, vite |
| AI Tools | `ai.md` | ollama |
| Scientific/HPC | `scientific.md` | spack, rez |
| Code Quality | `quality.md` | pre-commit |
| Other | `other.md` | deno, zig, java, vscode, rcedit, choco |

### Chinese Documentation (`docs/zh/tools/`)

Create corresponding Chinese documentation with the same structure.

### Documentation Template

```markdown
## {Tool Name}

{Brief description}

```bash
vx install {name} latest

vx {name} --version
vx {name} {common-command-1}
vx {name} {common-command-2}
```

**Key Features:** (optional)
- Feature 1
- Feature 2

**Platform Support:** (if special)
- Windows: {notes}
- Linux/macOS: {notes}
```

## Step 8: Version Fetching Strategies

### GitHub Releases (Preferred)

```rust
ctx.fetch_github_releases(
    "runtime-name",
    "owner",
    "repo",
    GitHubReleaseOptions::new()
        .strip_v_prefix(false)  // Set true if versions have 'v' prefix
        .skip_prereleases(true),
).await
```

### Manual GitHub API

```rust
let url = "https://api.github.com/repos/{owner}/{repo}/releases";
let response = ctx.http.get_json_value(url).await?;
// Parse response and build VersionInfo
```

## Step 9: Verification and Testing

```bash
# Check compilation
cargo check -p vx-provider-{name}

# Run tests
cargo test -p vx-provider-{name}

# If analyzer was added
cargo test -p vx-project-analyzer

# Verify full workspace
cargo check

# Run snapshot tests
cargo test --test cli_tests
```

## Common Patterns

### VersionInfo Construction

```rust
VersionInfo::new(version)
    .with_lts(false)
    .with_prerelease(false)
    .with_release_date(date_string)
```

### VerificationResult

```rust
// Success
VerificationResult::success(exe_path)

// Failure
VerificationResult::failure(
    vec!["Error message".to_string()],
    vec!["Suggested fix".to_string()],
)
```

### Platform Matching

```rust
match (&platform.os, &platform.arch) {
    (Os::Windows, Arch::X86_64) => Some("x86_64-pc-windows-msvc"),
    (Os::Windows, Arch::Aarch64) => Some("aarch64-pc-windows-msvc"),
    (Os::MacOS, Arch::X86_64) => Some("x86_64-apple-darwin"),
    (Os::MacOS, Arch::Aarch64) => Some("aarch64-apple-darwin"),
    (Os::Linux, Arch::X86_64) => Some("x86_64-unknown-linux-musl"),
    (Os::Linux, Arch::Aarch64) => Some("aarch64-unknown-linux-musl"),
    _ => None,
}
```

### Executable Path Configuration (Layered API)

The framework provides a layered approach - most providers only need 1-2 overrides:

```rust
// 1. Simple case: executable in root with standard .exe
// No overrides needed, defaults work

// 2. Tool uses .cmd on Windows (npm, yarn, npx)
fn executable_extensions(&self) -> &[&str] {
    &[".cmd", ".exe"]
}

// 3. Executable in subdirectory
fn executable_dir_path(&self, version: &str, _platform: &Platform) -> Option<String> {
    Some(format!("myapp-{}", version))
}

// 4. Different executable name than runtime name
fn executable_name(&self) -> &str {
    "python3"  // Runtime name is "python"
}

// 5. Complex platform-specific paths (Node.js style)
fn executable_dir_path(&self, version: &str, platform: &Platform) -> Option<String> {
    let dir = format!("node-v{}-{}", version, platform.as_str());
    if platform.is_windows() {
        Some(dir)  // Windows: no bin subdir
    } else {
        Some(format!("{}/bin", dir))  // Unix: has bin subdir
    }
}
```

### ScriptRule Priority Guidelines

| Priority | Use Case |
|----------|----------|
| 100 | Task runners (nox, tox, just, make) |
| 90 | Secondary task runners |
| 50 | Default tools (pytest, ruff, cargo) |

## System Package Manager Integration

For tools without portable binaries on all platforms, implement system package manager fallback.

### When to Use System Package Manager

| Platform | No Direct Download | Package Manager Options |
|----------|-------------------|------------------------|
| **macOS** | No portable binary | brew (priority 90) |
| **Windows** | No portable binary | winget (95), choco (80), scoop (60) |
| **Linux** | No portable binary | apt (90), dnf (85), pacman (80) |

### Step 1: Add system_deps.pre_depends in provider.toml

Declare which package managers are required as dependencies:

```toml
# macOS requires Homebrew
[[runtimes.system_deps.pre_depends]]
type = "runtime"
id = "brew"
platforms = ["macos"]
reason = "Required to install {tool} on macOS (no portable binary available)"
optional = false  # brew is required

# Windows: winget (preferred), choco, or scoop (any one is sufficient)
[[runtimes.system_deps.pre_depends]]
type = "runtime"
id = "winget"
platforms = ["windows"]
reason = "Preferred package manager for Windows (built-in on Windows 11)"
optional = true  # any one of winget/choco/scoop is sufficient

[[runtimes.system_deps.pre_depends]]
type = "runtime"
id = "choco"
platforms = ["windows"]
reason = "Alternative to winget for Windows installation"
optional = true

[[runtimes.system_deps.pre_depends]]
type = "runtime"
id = "scoop"
platforms = ["windows"]
reason = "Alternative to winget for Windows installation"
optional = true
```

### Step 2: Add system_install.strategies in provider.toml

Define how to install via each package manager:

```toml
# System installation strategies for platforms without direct download
[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "brew"
package = "mytool"  # Homebrew package name
platforms = ["macos"]
priority = 90

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "winget"
package = "Publisher.Package"  # winget uses Publisher.Package format
platforms = ["windows"]
priority = 95  # Highest priority on Windows (built-in on Win11)

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "choco"
package = "mytool"
platforms = ["windows"]
priority = 80

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "scoop"
package = "mytool"
platforms = ["windows"]
priority = 60
```

### Step 3: Implement install() Method with Fallback

For hybrid providers, override `install()` to try direct download first, then fall back to package manager:

```rust
use vx_system_pm::{PackageInstallSpec, PackageManagerRegistry};
use vx_runtime::{InstallResult, Runtime, RuntimeContext};

impl MyRuntime {
    /// Get package name for specific package manager
    fn get_package_name_for_manager(manager: &str) -> &'static str {
        match manager {
            "winget" => "Publisher.MyTool",  // winget uses Publisher.Package format
            "brew" | "choco" | "scoop" | "apt" => "mytool",
            "dnf" | "yum" => "MyTool",  // Some use different casing
            _ => "mytool",
        }
    }

    /// Install via system package manager
    async fn install_via_package_manager(
        &self,
        version: &str,
        _ctx: &RuntimeContext,
    ) -> Result<InstallResult> {
        let registry = PackageManagerRegistry::new();
        let available_managers = registry.get_available().await;

        if available_managers.is_empty() {
            return Err(anyhow::anyhow!(
                "No package manager available. Please install brew (macOS) or winget/choco/scoop (Windows)"
            ));
        }

        // Try each available package manager (sorted by priority)
        for pm in &available_managers {
            let package_name = Self::get_package_name_for_manager(pm.name());
            let spec = PackageInstallSpec {
                package: package_name.to_string(),
                ..Default::default()
            };

            match pm.install_package(&spec).await {
                Ok(_) => {
                    // Return system-installed result with actual executable path
                    let exe_path = which::which("mytool").ok();
                    return Ok(InstallResult::system_installed(
                        format!("{} (via {})", version, pm.name()),
                        exe_path,
                    ));
                }
                Err(e) => {
                    tracing::warn!("Failed to install via {}: {}", pm.name(), e);
                    continue;
                }
            }
        }

        Err(anyhow::anyhow!("All package managers failed"))
    }
}

#[async_trait]
impl Runtime for MyRuntime {
    async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        let platform = Platform::current();

        // Try direct download first (if available for this platform)
        if let Some(url) = self.download_url(version, &platform).await? {
            return self.install_via_download(version, &url, ctx).await;
        }

        // Fall back to system package manager
        self.install_via_package_manager(version, ctx).await
    }
}
```

### Step 4: Handle InstallResult Correctly

**Important**: System-installed tools have different paths than store-installed tools:

```rust
// Store-installed: executable in ~/.vx/store/{tool}/{version}/bin/
InstallResult::success(install_path, exe_path, version)

// System-installed: executable in system PATH (e.g., /opt/homebrew/bin/)
InstallResult::system_installed(version, Some(exe_path))
```

The test handler and other code must check `executable_path` from `InstallResult` rather than computing store paths.

### Package Manager Priority Reference

| Manager | Platform | Priority | Notes |
|---------|----------|----------|-------|
| **winget** | Windows | 95 | Built-in on Win11, App Installer on Win10 |
| **brew** | macOS | 90 | De-facto standard for macOS |
| **apt** | Linux (Debian) | 90 | Debian/Ubuntu default |
| **dnf** | Linux (Fedora) | 85 | Fedora/RHEL default |
| **choco** | Windows | 80 | Popular third-party |
| **pacman** | Linux (Arch) | 80 | Arch Linux default |
| **scoop** | Windows | 60 | Developer-focused |

### Common Package Names

| Tool | brew | winget | choco | scoop | apt |
|------|------|--------|-------|-------|-----|
| ImageMagick | imagemagick | ImageMagick.ImageMagick | imagemagick | imagemagick | imagemagick |
| FFmpeg | ffmpeg | Gyan.FFmpeg | ffmpeg | ffmpeg | ffmpeg |
| Git | git | Git.Git | git | git | git |
| AWS CLI | awscli | Amazon.AWSCLI | awscli | aws | awscli |
| Azure CLI | azure-cli | Microsoft.AzureCLI | azure-cli | - | azure-cli |
| Docker | docker | Docker.DockerDesktop | docker-desktop | - | docker.io |

## provider.toml Quick Reference

### Minimal Example (GitHub Releases with Layout)

```toml
[provider]
name = "mytool"
description = "My awesome tool"
homepage = "https://github.com/owner/repo"
repository = "https://github.com/owner/repo"
ecosystem = "devtools"

[[runtimes]]
name = "mytool"
description = "My tool CLI"
executable = "mytool"

[runtimes.versions]
source = "github-releases"
owner = "owner"
repo = "repo"
strip_v_prefix = true

# RFC 0019: Executable Layout Configuration
[runtimes.layout]
download_type = "archive"  # or "binary"

[runtimes.layout.archive]
strip_prefix = "mytool-{version}"
executable_paths = [
    "bin/mytool.exe",  # Windows
    "bin/mytool"       # Unix
]

[runtimes.platforms.windows]
executable_extensions = [".exe"]

[runtimes.platforms.unix]
executable_extensions = []
```

### Binary Download Example

```toml
[runtimes.layout]
download_type = "binary"

[runtimes.layout.binary."windows-x86_64"]
source_name = "mytool-{version}-win64.exe"
target_name = "mytool.exe"
target_dir = "bin"

[runtimes.layout.binary."linux-x86_64"]
source_name = "mytool-{version}-linux"
target_name = "mytool"
target_dir = "bin"
target_permissions = "755"
```

### Hybrid Provider Example (Direct Download + Package Manager Fallback)

For tools like ImageMagick that have direct download on some platforms but need package managers on others:

```toml
[provider]
name = "mytool"
description = "My awesome tool"
homepage = "https://example.com"
ecosystem = "devtools"

[[runtimes]]
name = "mytool"
description = "My tool CLI"
executable = "mytool"

[runtimes.versions]
source = "github-releases"
owner = "owner"
repo = "repo"

# Linux: Direct download available (AppImage, binary, etc.)
[runtimes.layout]
download_type = "binary"

[runtimes.layout.binary."linux-x86_64"]
source_name = "mytool-{version}-linux-x64"
target_name = "mytool"
target_dir = "bin"
target_permissions = "755"

# Note: No Windows/macOS binary configs = download_url returns None
# Triggers package manager fallback

# macOS requires Homebrew
[[runtimes.system_deps.pre_depends]]
type = "runtime"
id = "brew"
platforms = ["macos"]
reason = "Required to install mytool on macOS (no portable binary available)"
optional = false

# Windows: winget (preferred) or choco/scoop
[[runtimes.system_deps.pre_depends]]
type = "runtime"
id = "winget"
platforms = ["windows"]
reason = "Preferred package manager for Windows (built-in on Windows 11)"
optional = true

[[runtimes.system_deps.pre_depends]]
type = "runtime"
id = "choco"
platforms = ["windows"]
reason = "Alternative to winget for Windows installation"
optional = true

# System installation strategies
[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "brew"
package = "mytool"
platforms = ["macos"]
priority = 90

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "winget"
package = "Publisher.MyTool"
platforms = ["windows"]
priority = 95

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "choco"
package = "mytool"
platforms = ["windows"]
priority = 80
```

### provider.toml Fields Reference

| Section | Field | Description |
|---------|-------|-------------|
| `[provider]` | `name` | Provider name (required) |
| | `description` | Human-readable description |
| | `homepage` | Project homepage URL |
| | `repository` | Source repository URL |
| | `ecosystem` | `nodejs`, `python`, `rust`, `go`, `ruby`, `java`, `dotnet`, `devtools`, `container`, `cloud`, `ai`, `cpp`, `zig`, `system` |
| `[provider.platforms]` | `os` | Restrict to platforms: `["windows"]`, `["macos"]`, `["linux"]` |
| `[[runtimes]]` | `name` | Runtime name (required) |
| | `description` | Runtime description |
| | `executable` | Executable file name (required) |
| | `aliases` | Alternative names list |
| | `bundled_with` | If bundled with another runtime |
| `[runtimes.versions]` | `source` | Version source type |
| | `owner` | GitHub owner (for github-releases/tags) |
| | `repo` | GitHub repo name |
| | `strip_v_prefix` | Remove 'v' from version tags |
| **`[runtimes.layout]`** | **`download_type`** | **`"binary"`, `"archive"`, or `"git_clone"` (RFC 0019)** |
| `[runtimes.layout.binary."{platform}"]` | `source_name` | Downloaded file name (supports `{version}`) |
| | `target_name` | Final executable name |
| | `target_dir` | Target directory (e.g., `"bin"`) |
| | `target_permissions` | Unix permissions (e.g., `"755"`) |
| `[runtimes.layout.archive]` | `strip_prefix` | Directory prefix to remove (supports `{version}`, `{os}`, `{arch}`) |
| | `executable_paths` | Paths to executables after stripping |
| `[runtimes.executable_config]` | `dir_pattern` | Directory pattern (e.g., `{name}-{version}`) |
| | `extensions` | Executable extensions list |
| **`[[runtimes.system_deps.pre_depends]]`** | **`type`** | **`"runtime"` (dependency type)** |
| | `id` | Package manager runtime id (brew, winget, choco, scoop) |
| | `platforms` | Array of platforms: `["macos"]`, `["windows"]`, `["linux"]` |
| | `reason` | Human-readable reason for dependency |
| | `optional` | `true` if any one of multiple options is sufficient |
| **`[[runtimes.system_install.strategies]]`** | **`type`** | **`"package_manager"` or `"manual"`** |
| | `manager` | Package manager name (brew, winget, choco, scoop, apt, dnf) |
| | `package` | Package name in that manager |
| | `platforms` | Array of platforms this strategy applies to |
| | `priority` | Priority (higher = preferred). winget=95, brew=90, choco=80, scoop=60 |
| `[[runtimes.constraints]]` | `when` | Version condition (e.g., `*`, `^1`, `>=2`) |
| | `requires` | Required dependencies list |
| | `recommends` | Recommended dependencies list |

## Manifest Error Diagnostics

When developing a new provider, if your `provider.toml` has issues, the vx error system provides structured diagnostics:

### Error Categories

1. **Parse Errors (with context)** - TOML parsing failures with provider name and hints:
   - Unknown enum variants (e.g., wrong `ecosystem` or `download_type` value)
   - Type mismatches (e.g., using `when = { os = "windows" }` instead of `when = "*"`)
   - Missing required fields
   - kebab-case vs snake_case confusion

2. **Build Errors** - Provider registration failures:
   - **`NoFactory` (manifest-only)**: Provider has a `provider.toml` but no Rust implementation yet. This is expected for new providers that only have manifests.
   - **`FactoryFailed`**: The Rust factory function failed to create the provider.

### Common Mistakes and Auto-Hints

| Error Pattern | Auto-Hint |
|---------------|----------|
| `unknown variant "cpp"` for ecosystem | Lists all valid ecosystem values |
| `invalid type: map, expected a string` for `when` | Suggests using `when = "*"` with separate `platform` field |
| `unknown variant "git-clone"` for download_type | Suggests using `git_clone` (snake_case) |
| `missing field "name"` | Points to the required field |
| `invalid type: integer, expected a string` | Suggests quoting version numbers |

### Debug Output

The build summary shows:
```
INFO: registered 53 lazy providers (0 errors, 9 manifest-only, 0 warnings)
```

- **errors**: Real configuration errors that need fixing
- **manifest-only**: Providers with manifests but no Rust factory (expected during development)
- **warnings**: Non-fatal issues

## Reference Files

For complete code templates, see `references/templates.md`.
