---
name: vx-provider-updater
description: |
  Update existing VX providers to RFC 0018 + RFC 0019 standards, adding layout configuration
  for binary and archive downloads, and system package manager fallback for hybrid providers.
  Use this skill when updating provider.toml files, migrating from custom post_extract hooks
  to declarative layout configuration, or adding package manager fallback for tools without
  portable binaries on all platforms.
---

# VX Provider Updater

Update existing VX providers to RFC 0018 + RFC 0019 standards with layout configuration and system package manager integration.

## When to Use

- Updating existing provider.toml to add layout configuration
- Migrating from custom `post_extract` hooks to RFC 0019
- **Migrating from Rust runtime.rs to Starlark provider.star**
- **Adding MSI install support for Windows using Starlark**
- **Adding system package manager fallback for tools without portable binaries**
- **Adding missing `license` field to provider.toml** (all providers MUST have it)
- Standardizing provider manifests
- Fixing download/installation issues
- Fixing "No download URL" or "Executable not found" errors
- Batch updating multiple providers

## License Field Requirement

When updating any provider.toml, ensure the `license` field exists under `[provider]`:

```toml
[provider]
name = "example"
license = "MIT"          # SPDX identifier (REQUIRED)
# license_note = "..."   # Optional notes
```

**Blocked licenses** (AGPL-3.0, SSPL, CC BY-NC) must NOT be integrated as providers.
See the vx-provider-creator skill for the full license compatibility guide.

## Quick Reference

### Tool Categories

| Category | Layout Type | Examples |
|----------|-------------|----------|
| Single Binary | `binary` | kubectl, ninja, rust |
| Standard Archive (bin/) | `archive` + strip | node, go, cmake |
| Root Directory | `archive` (no strip) | terraform, just, deno |
| Platform Directory | `archive` + platform strip | helm, bun |
| Binary + Registry Clone | `binary` + git clone | vcpkg (downloads binary, clones registry) |
| **Hybrid (Download + PM)** | `binary/archive` + `system_install` | **imagemagick, ffmpeg, docker** |
| npm/pip Packages | No layout needed | vite, pre-commit |
| System Tools | Detection only | git, docker, openssl |
| **System PM Only** | `system_install` only | **make, curl, openssl** |

## Update Templates

### Template 1: Single File Binary

**适用于**: kubectl, ninja, rustup-init 等单文件下载

```toml
# RFC 0019: Executable Layout Configuration
[runtimes.layout]
download_type = "binary"

[runtimes.layout.binary."windows-x86_64"]
source_name = "tool.exe"
target_name = "tool.exe"
target_dir = "bin"

[runtimes.layout.binary."macos-x86_64"]
source_name = "tool"
target_name = "tool"
target_dir = "bin"
target_permissions = "755"

[runtimes.layout.binary."macos-aarch64"]
source_name = "tool"
target_name = "tool"
target_dir = "bin"
target_permissions = "755"

[runtimes.layout.binary."linux-x86_64"]
source_name = "tool"
target_name = "tool"
target_dir = "bin"
target_permissions = "755"

[runtimes.layout.binary."linux-aarch64"]
source_name = "tool"
target_name = "tool"
target_dir = "bin"
target_permissions = "755"
```

**插入位置**: 在 `[runtimes.versions]` 之后，`[runtimes.platforms]` 之前

### Template 2: Standard Archive with bin/

**适用于**: node, go, python, cmake 等标准压缩包

```toml
# RFC 0019: Executable Layout Configuration
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = "{name}-{version}"  # 或其他模式
executable_paths = [
    "bin/{name}.exe",  # Windows
    "bin/{name}"       # Unix
]
```

**常见 strip_prefix 模式**:
- Node.js: `node-v{version}-{os}-{arch}`
- Go: `go`
- CMake: `cmake-{version}-{os}-{arch}`
- Python: `python`

### Template 3: Root Directory Executable

**适用于**: terraform, just, task, deno 等根目录可执行文件

```toml
# RFC 0019: Executable Layout Configuration
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = ""  # 无前缀
executable_paths = [
    "{name}.exe",  # Windows (root directory)
    "{name}"       # Unix (root directory)
]
```

### Template 4: Platform-Specific Directory

**适用于**: helm, bun 等按平台分目录的压缩包

```toml
# RFC 0019: Executable Layout Configuration
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = "{os}-{arch}"  # 或 "bun-{os}-{arch}"
executable_paths = [
    "{name}.exe",  # Windows
    "{name}"       # Unix
]
```

### Template 5: Complex Nested Structure

**适用于**: java, ffmpeg 等复杂结构

```toml
# RFC 0019: Executable Layout Configuration
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = "jdk-{version}+{build}"  # 根据实际情况调整
executable_paths = [
    "bin/java.exe",  # Windows
    "bin/java"       # Unix
]
```

### Template 6: npm/pip Packages

**适用于**: vite, pre-commit, release-please 等包管理器安装

```toml
[runtimes.versions]
source = "npm"  # 或 "pypi"
package = "{package-name}"

# Note: npm/pip packages don't need layout configuration
# They are installed via package manager and have standard locations
```

### Template 7: System Tools (Detection Only)

**适用于**: git, docker, curl, openssl 等系统工具

```toml
# Note: System tools typically installed by OS package manager
# Use detection to find system-installed versions

[runtimes.detection]
command = "{executable} --version"
pattern = "{name} version ([\\d.]+)"
system_paths = [
    "/usr/bin/{name}",
    "/usr/local/bin/{name}",
    "C:\\Program Files\\{Name}\\bin\\{name}.exe"
]
env_hints = ["{NAME}_HOME"]
```

### Template 8: Hybrid Provider (Direct Download + Package Manager Fallback)

**适用于**: imagemagick, ffmpeg, docker 等部分平台有二进制、部分平台需要包管理器的工具

```toml
# Direct download for platforms with portable binaries (e.g., Linux AppImage)
[runtimes.layout]
download_type = "binary"

[runtimes.layout.binary."linux-x86_64"]
source_name = "tool-{version}-linux-x64.AppImage"
target_name = "tool"
target_dir = "bin"
target_permissions = "755"

# No Windows/macOS configs = download_url returns None, triggers PM fallback

# System dependencies (package managers required for installation)
# macOS requires Homebrew
[[runtimes.system_deps.pre_depends]]
type = "runtime"
id = "brew"
platforms = ["macos"]
reason = "Required to install tool on macOS (no portable binary available)"
optional = false

# Windows: winget (preferred), choco, or scoop (any one is sufficient)
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
package = "tool"
platforms = ["macos"]
priority = 90

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "winget"
package = "Publisher.Tool"  # winget uses Publisher.Package format
platforms = ["windows"]
priority = 95  # Highest on Windows (built-in on Win11)

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "choco"
package = "tool"
platforms = ["windows"]
priority = 80

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "scoop"
package = "tool"
platforms = ["windows"]
priority = 60
```

### Template 9: System Package Manager Only

**适用于**: make, curl 等所有平台都没有可移植二进制的工具

```toml
[[runtimes]]
name = "tool"
description = "Tool description"
executable = "tool"

# No layout configuration (no direct download)

# All platforms need package managers
[[runtimes.system_deps.pre_depends]]
type = "runtime"
id = "brew"
platforms = ["macos"]
reason = "Required to install tool on macOS"
optional = false

[[runtimes.system_deps.pre_depends]]
type = "runtime"
id = "winget"
platforms = ["windows"]
reason = "Preferred package manager for Windows"
optional = true

[[runtimes.system_deps.pre_depends]]
type = "runtime"
id = "choco"
platforms = ["windows"]
optional = true

# System installation strategies for all platforms
[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "brew"
package = "tool"
platforms = ["macos", "linux"]
priority = 90

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "apt"
package = "tool"
platforms = ["linux"]
priority = 90

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "dnf"
package = "tool"
platforms = ["linux"]
priority = 85

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "winget"
package = "Publisher.Tool"
platforms = ["windows"]
priority = 95

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "choco"
package = "tool"
platforms = ["windows"]
priority = 80
```

## Update Workflow

### Step 1: Identify Tool Type

```bash
# Check current provider.toml
cat crates/vx-providers/{name}/provider.toml
```

Questions to answer:
1. Is it a single binary or archive?
2. What's the download URL format?
3. What's the internal structure after extraction?
4. Are there platform-specific differences?

### Step 2: Choose Template

Use decision tree:

```
Does the tool provide portable binaries for ALL platforms?
├─ Yes → Is it a binary download (single file)?
│   ├─ Yes → Template 1 (Binary)
│   └─ No → Is it an archive?
│       ├─ Has bin/ directory? → Template 2 (Standard Archive)
│       ├─ Executable in root? → Template 3 (Root Directory)
│       ├─ Platform subdirs? → Template 4 (Platform Directory)
│       └─ Complex? → Template 5 (Complex)
├─ Partial (some platforms have binaries) → Template 8 (Hybrid)
│   └─ Examples: imagemagick (Linux AppImage, macOS/Windows via PM)
│   └─ Examples: ffmpeg (Windows binary, macOS/Linux via brew/apt)
└─ No (no portable binaries) → Check installation method
    ├─ npm/pip package? → Template 6 (Package Manager)
    ├─ System tool (curl, openssl)? → Template 7 (Detection Only)
    └─ Can be installed via PM? → Template 9 (System PM Only)
```
        ├─ Yes → Template 6 (Package Manager)
        └─ No → Template 7 (System Tool)
```

### Step 3: Add Configuration

1. Open `crates/vx-providers/{name}/provider.toml`
2. Locate `[runtimes.versions]` section
3. Add layout configuration after versions, before platforms
4. Save file

### Step 4: Verify Format

Checklist:
- [ ] `download_type` is `"binary"`, `"archive"`, or `"git_clone"`
- [ ] **Important**: Use `snake_case` for values (e.g., `git_clone` NOT `git-clone`)
- [ ] For binary: All platforms have configuration
- [ ] For binary: Unix platforms have `target_permissions = "755"`
- [ ] For archive: `strip_prefix` matches actual structure
- [ ] For archive: `executable_paths` includes Windows and Unix
- [ ] Paths use forward slashes `/`, not backslashes
- [ ] Variables like `{version}`, `{os}`, `{arch}` are correct

### Step 5: Test

```bash
# Build
cargo build --release

# Test installation
vx install {name}@{version}

# Verify
vx which {name}
vx {name} --version
```

## Batch Update Script

For updating multiple providers at once:

```rust
// Create a batch update plan
let providers_to_update = vec![
    ("kubectl", LayoutType::Binary),
    ("terraform", LayoutType::ArchiveRoot),
    ("helm", LayoutType::ArchivePlatform),
    ("just", LayoutType::ArchiveRoot),
    ("task", LayoutType::ArchiveRoot),
];

for (name, layout_type) in providers_to_update {
    update_provider(name, layout_type)?;
}
```

## Common Patterns

### Pattern: Version in Binary Name

```toml
[runtimes.layout.binary."windows-x86_64"]
source_name = "yasm-{version}-win64.exe"  # {version} auto-replaced
target_name = "yasm.exe"
target_dir = "bin"
```

### Pattern: Platform Variations

```toml
[runtimes.layout.archive]
strip_prefix = "node-v{version}-{os}-{arch}"  # All replaced
# {os} → windows, linux, darwin
# {arch} → x86_64, aarch64
```

### Pattern: JavaScript Executables

```toml
[runtimes.layout.archive]
strip_prefix = "yarn-v{version}"
executable_paths = [
    "bin/yarn.js"  # JavaScript file, not native binary
]
```

### Pattern: Windows Only

```toml
[[runtimes]]
name = "rcedit"
description = "Windows resource editor"
executable = "rcedit"

[runtimes.layout]
download_type = "binary"

# Only Windows platform
[runtimes.layout.binary."windows-x86_64"]
source_name = "rcedit-x64.exe"
target_name = "rcedit.exe"
target_dir = "bin"

[runtimes.platforms.windows]
executable_extensions = [".exe"]
```

## Migration: From Rust runtime.rs to Starlark provider.star

Starlark (`provider.star`) is the **preferred** way to implement providers. It replaces `runtime.rs` and `config.rs` with pure-computation scripts that are easier to read, write, and maintain.

### When to Migrate

- Provider has custom `download_url()` logic in `config.rs`
- Provider has `post_extract()` or `install()` overrides in `runtime.rs`
- Provider needs MSI install support on Windows
- Provider needs system package manager fallback
- Any new provider being created

### Migration Steps

#### Step 1: Create provider.star

Create `crates/vx-providers/{name}/provider.star` with the equivalent logic:

**Before (Rust config.rs):**
```rust
pub fn download_url(version: &str, platform: &Platform) -> Option<String> {
    let triple = match (&platform.os, &platform.arch) {
        (Os::Windows, Arch::X86_64) => "x86_64-pc-windows-msvc",
        (Os::MacOS, Arch::Aarch64)  => "aarch64-apple-darwin",
        (Os::Linux, Arch::X86_64)   => "x86_64-unknown-linux-musl",
        _ => return None,
    };
    let ext = if platform.os == Os::Windows { "zip" } else { "tar.gz" };
    Some(format!("https://github.com/owner/repo/releases/download/v{}/tool-v{}-{}.{}",
        version, version, triple, ext))
}
```

**After (Starlark provider.star):**
```python
load("@vx//stdlib:github.star",   "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:platform.star", "is_windows")

fetch_versions = make_fetch_versions("owner", "repo")

def _triple(ctx):
    os   = ctx["platform"]["os"]
    arch = ctx["platform"]["arch"]
    return {
        "windows/x64":  "x86_64-pc-windows-msvc",
        "macos/arm64":  "aarch64-apple-darwin",
        "linux/x64":    "x86_64-unknown-linux-musl",
    }.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    triple = _triple(ctx)
    if not triple:
        return None
    os  = ctx["platform"]["os"]
    ext = "zip" if os == "windows" else "tar.gz"
    asset = "tool-v{}-{}.{}".format(version, triple, ext)
    return github_asset_url("owner", "repo", "v" + version, asset)

def install_layout(ctx, version):
    os  = ctx["platform"]["os"]
    exe = "tool.exe" if os == "windows" else "tool"
    return {
        "type":             "archive",
        "strip_prefix":     "tool-v{}-{}".format(version, _triple(ctx) or ""),
        "executable_paths": [exe, "tool"],
    }

def environment(ctx, version, install_dir):
    return {"PATH": install_dir}

def deps(ctx, version):
    return []
```

#### Step 2: Simplify provider.toml

After creating `provider.star`, the `provider.toml` only needs metadata (remove layout fields):

```toml
[provider]
name = "mytool"
description = "My awesome tool"
homepage = "https://example.com"
repository = "https://github.com/owner/repo"
ecosystem = "devtools"
license = "MIT"
```

#### Step 3: Remove Rust files (if manifest-only)

If the provider was Rust-only (no `provider.toml`), you can keep the Rust factory or convert to manifest-only. For manifest+star providers, the Rust files are optional.

### Starlark Standard Library Quick Reference

| Module | Load Path | Key Functions |
|--------|-----------|---------------|
| GitHub | `@vx//stdlib:github.star` | `make_fetch_versions(owner, repo)`, `make_download_url(owner, repo, template)`, `make_github_provider(owner, repo, template)`, `github_asset_url(owner, repo, tag, asset)` |
| Platform | `@vx//stdlib:platform.star` | `is_windows(ctx)`, `is_macos(ctx)`, `is_linux(ctx)`, `is_x64(ctx)`, `is_arm64(ctx)`, `platform_triple(ctx)`, `platform_ext(ctx)`, `exe_ext(ctx)`, `arch_to_gnu(arch)`, `arch_to_go(arch)`, `os_to_go(os)` |
| Install | `@vx//stdlib:install.star` | `msi_install(url, ...)`, `archive_install(url, ...)`, `binary_install(url, ...)`, `platform_install(ctx, ...)` |
| HTTP | `@vx//stdlib:http.star` | `github_releases(ctx, owner, repo)`, `releases_to_versions(releases)`, `parse_github_tag(tag)` |
| Semver | `@vx//stdlib:semver.star` | `semver_compare(a, b)`, `semver_gt/lt/gte/lte/eq(a, b)`, `semver_sort(versions)`, `semver_strip_v(v)` |

### ctx Object Reference

```python
ctx = {
    "platform": {
        "os":     "windows" | "macos" | "linux",
        "arch":   "x64" | "arm64" | "x86",
        "target": "x86_64-pc-windows-msvc" | ...,
    },
    "http": {
        "get_json": lambda url: ...,  # returns parsed JSON
    },
    "paths": {
        "install_dir": "/path/to/install",
        "cache_dir":   "/path/to/cache",
    },
}
```

### install_layout Return Values

| Type | Required Fields | Optional Fields |
|------|----------------|------------------|
| `"archive"` | `type` | `strip_prefix`, `executable_paths` |
| `"binary"` | `type` | `executable_name`, `source_name`, `permissions` |
| `"msi"` | `type`, `url` | `executable_paths`, `strip_prefix`, `extra_args` |

## Migration: Adding MSI Install Support (Windows)

For tools that distribute `.msi` installers on Windows, use `msi_install()` from `install.star`.

### How MSI Install Works

The Rust runtime runs:
```
msiexec /a <file.msi> /qn /norestart TARGETDIR=<install_dir>
```
This extracts the MSI contents to `install_dir` **without modifying the Windows registry**.

### Template: MSI on Windows + Archive on Other Platforms

```python
# provider.star
load("@vx//stdlib:install.star",  "msi_install", "archive_install")
load("@vx//stdlib:github.star",   "make_fetch_versions", "github_asset_url")

fetch_versions = make_fetch_versions("owner", "repo")

def download_url(ctx, version):
    os = ctx["platform"]["os"]
    if os == "windows":
        return "https://github.com/owner/repo/releases/download/v{}/tool-{}-x64.msi".format(version, version)
    elif os == "macos":
        return github_asset_url("owner", "repo", "v" + version, "tool-{}-macos.tar.gz".format(version))
    elif os == "linux":
        return github_asset_url("owner", "repo", "v" + version, "tool-{}-linux.tar.gz".format(version))
    return None

def install_layout(ctx, version):
    os  = ctx["platform"]["os"]
    url = download_url(ctx, version)
    if os == "windows":
        return msi_install(
            url,
            executable_paths = ["bin/tool.exe", "tool.exe"],
            # strip_prefix = "PFiles/Tool",  # if msiexec extracts to a subdir
        )
    else:
        return archive_install(
            url,
            strip_prefix = "tool-{}".format(version),
            executable_paths = ["bin/tool"],
        )

def environment(ctx, version, install_dir):
    return {"PATH": install_dir}

def deps(ctx, version):
    return []
```

### Template: platform_install() Convenience Helper

```python
load("@vx//stdlib:install.star", "platform_install")

def install_layout(ctx, version):
    return platform_install(
        ctx,
        windows_url = "https://example.com/tool-{}.msi".format(version),
        macos_url   = "https://example.com/tool-{}-macos.tar.gz".format(version),
        linux_url   = "https://example.com/tool-{}-linux.tar.gz".format(version),
        windows_msi = True,
        executable_paths = ["bin/tool.exe", "bin/tool"],
        strip_prefix = "tool-{}".format(version),
    )
```

## Migration: From post_extract to Layout

### Before (Custom Rust Code)

```rust
// In runtime.rs
fn post_extract(&self, version: &str, install_path: &PathBuf) -> Result<()> {
    use std::fs;
    
    let platform = Platform::current();
    let original_name = format!("tool-{}-{}.exe", version, platform.arch);
    let original_path = install_path.join(&original_name);
    
    let bin_dir = install_path.join("bin");
    fs::create_dir_all(&bin_dir)?;
    
    let target_path = bin_dir.join("tool.exe");
    fs::rename(&original_path, &target_path)?;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&target_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&target_path, perms)?;
    }
    
    Ok(())
}
```

### After (RFC 0019 TOML)

```toml
# In provider.toml
[runtimes.layout]
download_type = "binary"

[runtimes.layout.binary."windows-x86_64"]
source_name = "tool-{version}-x86_64.exe"
target_name = "tool.exe"
target_dir = "bin"

[runtimes.layout.binary."linux-x86_64"]
source_name = "tool-{version}-x86_64"
target_name = "tool"
target_dir = "bin"
target_permissions = "755"
```

**Benefits**:
- ✅ No Rust code needed
- ✅ Declarative configuration
- ✅ Easy to update
- ✅ Cross-platform handling built-in

## Migration: Adding System Package Manager Fallback

When a tool has "No download URL" errors on certain platforms, add package manager fallback.

### Identify the Problem

Error messages indicating need for package manager fallback:
- `No download URL for {tool} {version}` on macOS/Windows
- `Executable not found: ~/.vx/store/{tool}/{version}/bin/{tool}` after "successful" install
- Tool works on Linux but fails on macOS/Windows

### Step 1: Check Platform Coverage

```bash
# Check which platforms have binary downloads
grep -A 20 "layout.binary" crates/vx-providers/{name}/provider.toml

# If missing platforms (e.g., macos, windows), need package manager fallback
```

### Step 2: Add system_deps.pre_depends

```toml
# Add after [runtimes.versions] or [runtimes.layout]

# macOS requires Homebrew
[[runtimes.system_deps.pre_depends]]
type = "runtime"
id = "brew"
platforms = ["macos"]
reason = "Required to install {tool} on macOS (no portable binary available)"
optional = false

# Windows: any one of winget/choco/scoop
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

[[runtimes.system_deps.pre_depends]]
type = "runtime"
id = "scoop"
platforms = ["windows"]
reason = "Alternative to winget for Windows installation"
optional = true
```

### Step 3: Add system_install.strategies

```toml
# System installation strategies
[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "brew"
package = "{brew_package_name}"
platforms = ["macos"]
priority = 90

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "winget"
package = "Publisher.Package"  # Find via: winget search {tool}
platforms = ["windows"]
priority = 95

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "choco"
package = "{choco_package_name}"  # Find via: choco search {tool}
platforms = ["windows"]
priority = 80

[[runtimes.system_install.strategies]]
type = "package_manager"
manager = "scoop"
package = "{scoop_package_name}"  # Find via: scoop search {tool}
platforms = ["windows"]
priority = 60
```

### Step 4: Update runtime.rs (for custom Runtime implementations)

If the provider has a custom `runtime.rs` (not manifest-driven), add:

```rust
use vx_system_pm::{PackageInstallSpec, PackageManagerRegistry};
use vx_runtime::InstallResult;

// Add to impl Runtime
async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
    let platform = Platform::current();

    // Try direct download first
    if let Some(url) = self.download_url(version, &platform).await? {
        return self.install_via_download(version, &url, ctx).await;
    }

    // No direct download, try package manager
    self.install_via_package_manager(version, ctx).await
}

// Add helper method
async fn install_via_package_manager(
    &self,
    version: &str,
    _ctx: &RuntimeContext,
) -> Result<InstallResult> {
    let registry = PackageManagerRegistry::new();
    let available = registry.get_available().await;

    for pm in &available {
        let package = Self::get_package_name_for_manager(pm.name());
        let spec = PackageInstallSpec {
            package: package.to_string(),
            ..Default::default()
        };

        if pm.install_package(&spec).await.is_ok() {
            let exe_path = which::which("{executable}").ok();
            return Ok(InstallResult::system_installed(
                format!("{} (via {})", version, pm.name()),
                exe_path,
            ));
        }
    }

    Err(anyhow::anyhow!("No package manager available"))
}
```

### Step 5: Add Cargo.toml Dependencies

```toml
[dependencies]
vx-system-pm = { workspace = true }
tracing = { workspace = true }
which = { workspace = true }
```

### Package Name Lookup

| Manager | How to Find Package Name |
|---------|--------------------------|
| brew | `brew search {tool}` |
| winget | `winget search {tool}` |
| choco | `choco search {tool}` |
| scoop | `scoop search {tool}` |
| apt | `apt search {tool}` |
| dnf | `dnf search {tool}` |

### Common Package Name Mappings

| Tool | brew | winget | choco | Notes |
|------|------|--------|-------|-------|
| ImageMagick | imagemagick | ImageMagick.ImageMagick | imagemagick | |
| FFmpeg | ffmpeg | Gyan.FFmpeg | ffmpeg | |
| AWS CLI | awscli | Amazon.AWSCLI | awscli | |
| Azure CLI | azure-cli | Microsoft.AzureCLI | azure-cli | |
| Docker | docker | Docker.DockerDesktop | docker-desktop | Desktop on Win/Mac |
| Git | git | Git.Git | git | |
| Make | make | GnuWin32.Make | make | |

## Special Cases

### Case 1: Installer Files (.msi, .pkg)

For tools using installers (not supported yet):

```toml
# Note: Tool uses platform-specific installers (.msi, .pkg, .deb)
# Layout configuration will be added when installer support is implemented

[runtimes.platforms.windows]
executable_extensions = [".exe"]
```

Examples: awscli, azcli, gcloud, ollama, vscode

### Case 2: Multiple Executables

For tools providing multiple executables:

```toml
[runtimes.layout.archive]
strip_prefix = "toolset-{version}"
executable_paths = [
    "bin/tool1.exe",
    "bin/tool1",
    "bin/tool2.exe",
    "bin/tool2"
]
```

### Case 3: Bundled Dependencies

Tools bundled with another runtime:

```toml
[[runtimes]]
name = "npm"
executable = "npm"
bundled_with = "node"  # Comes with Node.js

# No layout configuration needed
# npm is installed as part of node

[[runtimes.constraints]]
when = "*"
requires = [
    { runtime = "node", version = "*", reason = "npm is bundled with Node.js" }
]
```

## Validation Rules

### Rule 1: Platform Coverage

Binary layout must cover all supported platforms:
- ✅ windows-x86_64
- ✅ macos-x86_64, macos-aarch64
- ✅ linux-x86_64, linux-aarch64

### Rule 2: Unix Permissions

Unix platforms must have `target_permissions`:

```toml
# ❌ Missing permissions
[runtimes.layout.binary."linux-x86_64"]
source_name = "tool"
target_name = "tool"
target_dir = "bin"

# ✅ Correct
[runtimes.layout.binary."linux-x86_64"]
source_name = "tool"
target_name = "tool"
target_dir = "bin"
target_permissions = "755"
```

### Rule 3: Path Separators

Always use forward slashes in paths:

```toml
# ❌ Wrong
executable_paths = ["bin\\tool.exe"]

# ✅ Correct
executable_paths = ["bin/tool.exe"]
```

### Rule 4: Variable Syntax

Use correct variable placeholders:

```toml
# ❌ Wrong
strip_prefix = "$version-{os}"

# ✅ Correct
strip_prefix = "{version}-{os}"
```

## Testing Checklist

After updating a provider:

- [ ] `cargo check -p vx-provider-{name}` passes
- [ ] `cargo build --release` succeeds
- [ ] `vx install {name}@latest` works
- [ ] `vx which {name}` shows correct path
- [ ] `vx {name} --version` executes successfully
- [ ] Tested on Windows (if applicable)
- [ ] Tested on Linux (if applicable)
- [ ] Tested on macOS (if applicable)

## Update Documentation

After successful update:

1. Update migration status in `docs/provider-migration-status.md`
2. Add entry to changelog if significant
3. Update tool documentation in `docs/tools/` if needed

## Troubleshooting

### Issue: Executable not found after install

**Cause**: Incorrect `strip_prefix` or `executable_paths`

**Solution**: 
1. Download the archive manually
2. Inspect the actual structure
3. Update configuration to match

### Issue: Permission denied on Unix

**Cause**: Missing `target_permissions`

**Solution**: Add `target_permissions = "755"` to Unix platforms

### Issue: Wrong executable on Windows

**Cause**: Incorrect file extension or order in `executable_paths`

**Solution**: Ensure `.exe` files come before non-extension files

### Issue: Version variable not replaced

**Cause**: Wrong variable syntax or missing variable

**Solution**: Use `{version}` not `$version` or `${version}`

### Issue: "No download URL for {tool}" on macOS/Windows

**Cause**: No layout.binary configuration for that platform, and no package manager fallback

**Solution**: Add system package manager support:
1. Add `system_deps.pre_depends` for brew (macOS) or winget/choco (Windows)
2. Add `system_install.strategies` for each package manager
3. If custom runtime.rs, implement `install()` with package manager fallback

### Issue: "Executable not found" after system package manager install

**Cause**: `install_quiet` returns version string, loses `executable_path` from `InstallResult`

**Solution**: 
1. Ensure `install()` returns `InstallResult::system_installed(version, Some(exe_path))`
2. Use `which::which("{executable}")` to get actual executable path
3. Test handler should use `executable_path` from `InstallResult`, not compute store path

### Issue: Package manager install succeeds but tool not found

**Cause**: Package manager installed to non-standard location, or `which::which()` fails

**Solution**:
1. Check package manager's installation location
2. Ensure PATH includes package manager's bin directory
3. Use explicit path lookup: `which::which("{executable}")`

### Issue: Wrong package name for package manager

**Cause**: Package names differ between package managers

**Solution**: Look up correct package name for each manager:
- brew: `brew search {tool}`
- winget: `winget search {tool}` (uses Publisher.Package format)
- choco: `choco search {tool}`
- scoop: `scoop search {tool}`

### Issue: TOML parse error "unknown variant" for download_type

**Cause**: Using kebab-case (`git-clone`) instead of snake_case (`git_clone`)

**Solution**: Use `snake_case` for all enum values in provider.toml:
- `download_type = "git_clone"` ✅
- `download_type = "git-clone"` ❌
- `ecosystem = "cpp"` ✅ (new in recent update)

The error diagnostic system will suggest the correct format.

### Issue: TOML parse error "unknown variant" for ecosystem

**Cause**: Using an unsupported ecosystem value

**Solution**: Use one of the supported values:
`nodejs`, `python`, `rust`, `go`, `ruby`, `java`, `dotnet`, `devtools`, `container`, `cloud`, `ai`, `cpp`, `zig`, `system`

### Issue: "No factory registered" appears in debug output

**Cause**: Provider has `provider.toml` but no Rust factory implementation

**Solution**: This is expected for manifest-only providers (in development). The build summary now shows:
```
registered 53 lazy providers (0 errors, 9 manifest-only, 0 warnings)
```
- **errors**: Real configuration problems
- **manifest-only**: Expected for providers without Rust implementation yet

## Reference

See also:
- `references/rfc-0019-layout.md` - Complete RFC 0019 specification
- `docs/provider-migration-status.md` - Migration status tracker
- `docs/provider-update-summary.md` - Batch update summary
- `crates/vx-providers/imagemagick/` - Example hybrid provider with PM fallback
