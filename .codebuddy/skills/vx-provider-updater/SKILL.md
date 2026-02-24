---
name: vx-provider-updater
description: |
  Update existing VX providers to the latest standards: provider.star as single source of truth,
  top-level variables for metadata (name = "..."), object-style ctx access (ctx.platform.os),
  stdlib helpers (make_fetch_versions, github_asset_url, env_prepend), and required path query
  functions (store_root, get_execute_path, post_install). Add package_alias for PyPI/npm tools
  (RFC 0033: vx meson = vx uvx:meson, vx vite = vx npx:vite). All providers must follow the
  current provider.star format with proper system_install {"strategies": [...]} structure.
---

# VX Provider Updater

Migrate all VX providers to the latest standards: `provider.star` as the **single source of truth**,
replacing `provider.toml` entirely.

## When to Use

- **Migrating from old function-based metadata to top-level variables** (`def name(): return "..."` → `name = "..."`)
- **Migrating from `ctx["platform"]["os"]` to `ctx.platform.os`** (object-style access)
- **Migrating from `make_github_provider` to `make_fetch_versions` + `github_asset_url`**
- **Migrating `environment()` from dict return to list of `env_prepend()`/`env_set()` calls**
- **Adding required path query functions** (`store_root`, `get_execute_path`, `post_install`)
- **Adding `package_alias` for PyPI/npm tools** (RFC 0033: `vx meson` = `vx uvx:meson`)
- **Fixing `system_install` format** to use `{"strategies": [...]}` structure
- Standardizing provider manifests
- Fixing download/installation issues
- Batch updating multiple providers

## Core Changes Summary

### Change 1: Metadata as Top-Level Variables (NOT functions)

**OLD (forbidden):**
```python
def name():
    return "mytool"

def description():
    return "My awesome tool"

def ecosystem():
    return "custom"
```

**NEW (required):**
```python
name        = "mytool"
description = "My awesome tool"
ecosystem   = "custom"
```

### Change 2: ctx Object Access (NOT dict access)

**OLD (forbidden):**
```python
os   = ctx["platform"]["os"]
arch = ctx["platform"]["arch"]
releases = ctx["http"]["get_json"]("https://...")
```

**NEW (required):**
```python
os   = ctx.platform.os
arch = ctx.platform.arch
releases = ctx.http.get_json("https://...")
```

### Change 3: New stdlib Helpers

**OLD (less preferred):**
```python
_p = make_github_provider("owner", "repo", "tool-{triple}.{ext}")
fetch_versions = _p["fetch_versions"]
download_url   = _p["download_url"]
```

**NEW (preferred):**
```python
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

fetch_versions = make_fetch_versions("owner", "repo")

def download_url(ctx, version):
    # Custom per-platform logic using github_asset_url
    asset = "tool-{}-{}.{}".format(version, triple, ext)
    return github_asset_url("owner", "repo", "v" + version, asset)
```

### Change 4: Remove Redundant Old Functions

**Remove these old functions (replaced by new API):**
- `post_extract(ctx, version, install_dir)` — merge into `post_install`

**Required / recommended functions in current format:**
- `fetch_versions(ctx)` — required (or use `make_fetch_versions`)
- `download_url(ctx, version)` — strongly recommended
- `install_layout(ctx, version)` — optional (has defaults)
- `environment(ctx, version)` — optional, returns list of `env_prepend()`/`env_set()` calls
- `post_install(ctx, version)` — optional, return `None` if nothing to do
- `store_root(ctx)` — optional, return path to store root
- `get_execute_path(ctx, version)` — optional, return path to executable
- `system_install(ctx)` — optional, return `{"strategies": [...]}` for PM fallback
- `uninstall(ctx, version)` — optional, return uninstall descriptor or `False`
- `deps(ctx, version)` — optional, return list of dependency dicts

### Change 5: pre_run Signature Change

**OLD:**
```python
def pre_run(ctx, args, executable):
    ...
```

**NEW:**
```python
def pre_run(ctx, args):
    ...
```

### Change 6: environment() Returns List (NOT dict)

**OLD (forbidden):**
```python
def environment(ctx, version, install_dir):
    return {"PATH": install_dir}  # dict format
```

**NEW (required):**
```python
load("@vx//stdlib:env.star", "env_prepend", "env_set")

def environment(ctx, _version):
    return [
        env_prepend("PATH", ctx.install_dir),
        # env_set("TOOL_HOME", ctx.install_dir),  # optional
    ]
```

### Change 7: system_install Returns {"strategies": [...]}

**OLD (forbidden):**
```python
"system_install": [
    {"manager": "brew", "package": "mytool"},
]
```

**NEW (required):**
```python
def system_install(ctx):
    os = ctx.platform.os
    if os == "windows":
        return {
            "strategies": [
                {"manager": "winget", "package": "Publisher.MyTool", "priority": 95},
                {"manager": "choco",  "package": "mytool",           "priority": 80},
            ],
        }
    elif os == "macos":
        return {
            "strategies": [
                {"manager": "brew", "package": "mytool", "priority": 90},
            ],
        }
    return {}
```

### Change 8: Add Required Path Query Functions

All providers must implement these functions:

```python
def store_root(ctx):
    return ctx.vx_home + "/store/{name}"

def get_execute_path(ctx, version):
    os = ctx.platform.os
    exe = "{name}.exe" if os == "windows" else "{name}"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None  # Return None if nothing to do
```

### Change 9: deps() Returns List of Dicts

```python
def deps(_ctx, _version):
    return [
        {"runtime": "node", "version": ">=18",
         "reason": "Requires Node.js runtime"},
    ]
```

## License Field Requirement

All providers MUST have `license` as a top-level variable:

```python
license = "MIT"          # SPDX identifier (REQUIRED)
```

**Blocked licenses** (AGPL-3.0, SSPL, CC BY-NC) must NOT be integrated as providers.

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

All provider logic now lives in `provider.star`. See `references/update-templates.md` for
complete migration templates covering:

- **Standard GitHub Provider** — `make_fetch_versions` + custom `download_url`
- **Hybrid Provider** — direct download on some platforms + `system_install` fallback
- **PyPI/npm Package Alias** — `package_alias` for ecosystem-managed tools
- **MSI on Windows** — `msi_install()` + `archive_install()` per platform
- **Non-GitHub Version Source** — custom `fetch_versions` with `fetch_json_versions`

The `provider.toml` now only contains metadata (name, description, ecosystem, license).
All install logic (download URLs, archive layout, system_install, environment) belongs in `provider.star`.

## Update Workflow

### Step 1: Identify Tool Type

```bash
# Check current provider.star
cat crates/vx-providers/{name}/provider.star
```

Questions to answer:
1. Does it use old function-based metadata (`def name(): return "..."`)?
2. Does it use old dict-style ctx access (`ctx["platform"]["os"]`)?
3. Does `environment()` return a dict instead of a list?
4. Are `store_root`, `get_execute_path`, `post_install` missing?
5. Does `system_install` return a flat list instead of `{"strategies": [...]}`?

### Step 2: Choose Migration Path

```
Is provider.star using old format?
├─ Old function metadata → Change 1: top-level variables
├─ ctx["platform"]["os"] → Change 2: ctx.platform.os
├─ make_github_provider → Change 3: make_fetch_versions + github_asset_url
├─ environment() returns dict → Change 6: return list with env_prepend()
├─ system_install flat list → Change 7: {"strategies": [...]}
└─ Missing store_root/get_execute_path/post_install → Change 8: add them
```

See `references/update-templates.md` for complete before/after examples.

### Step 3: Apply Changes

1. Open `crates/vx-providers/{name}/provider.star`
2. Apply the relevant changes from the Core Changes Summary above
3. Ensure `provider.toml` only contains metadata (no layout fields)
4. Save files

### Step 4: Verify Format

Checklist:
- [ ] All metadata as top-level variables (no `def name():` functions)
- [ ] All `ctx["..."]["..."]` replaced with `ctx.platform.os` / `ctx.platform.arch`
- [ ] `environment()` returns list: `[env_prepend("PATH", ctx.install_dir)]`
- [ ] `store_root()`, `get_execute_path()`, `post_install()` all present
- [ ] `runtimes` includes `test_commands`
- [ ] `system_install()` returns `{"strategies": [...]}` (not flat list)
- [ ] `license` field present (SPDX identifier)

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

## Migration: From Rust runtime.rs to Starlark provider.star

Starlark (`provider.star`) is the **single source of truth** for all provider metadata and
install logic. Every provider crate — whether it keeps custom Rust code or is fully
manifest-driven — **must** embed `provider.star` at compile time and expose its metadata
through `star_metadata()`.

### Architecture Overview

```
provider.star  (single source of truth)
    │
    │  include_str!("../provider.star")   ← compile-time embed
    │  build.rs watches for changes
    ▼
lib.rs
    ├── pub const PROVIDER_STAR: &str = include_str!("../provider.star")
    └── pub fn star_metadata() -> &'static StarMetadata   ← OnceLock lazy parse
            │
            ▼
    provider.rs / runtime.rs
        ├── name()        → star_metadata().name_or("tool")
        ├── description() → star_metadata().description (OnceLock &'static str)
        ├── aliases()     → star_metadata().runtimes[0].aliases
        ├── metadata()    → star_metadata().homepage / repository / license
        └── supports()    → star_metadata().runtimes[*].aliases
```

### When to Migrate

- Provider has custom `download_url()` logic in `config.rs`
- Provider has `post_extract()` or `install()` overrides in `runtime.rs`
- Provider needs MSI install support on Windows
- Provider needs system package manager fallback
- Any new provider being created
- **Any existing provider that still hardcodes name/description in Rust**

### Migration Steps

#### Step 0: Add build.rs (ALL providers)

Every provider crate must have a `build.rs` that watches `provider.star`:

```rust
// crates/vx-providers/{name}/build.rs
fn main() {
    // Re-run this build script whenever provider.star changes.
    // This ensures that `include_str!("../provider.star")` in lib.rs always
    // reflects the latest content and that Cargo rebuilds the crate when the
    // Starlark provider definition is updated.
    println!("cargo:rerun-if-changed=provider.star");
}
```

#### Step 0b: Update lib.rs (ALL providers)

Every provider crate's `lib.rs` must embed `provider.star` and expose `star_metadata()`:

```rust
// crates/vx-providers/{name}/src/lib.rs

/// The raw content of `provider.star`, embedded at compile time.
///
/// This is the single source of truth for provider metadata (name, description,
/// aliases, platform constraints, etc.).  The `build.rs` script ensures Cargo
/// re-compiles this crate whenever `provider.star` changes.
pub const PROVIDER_STAR: &str = include_str!("../provider.star");

/// Lazily-parsed metadata from `provider.star`.
///
/// Use this to access provider/runtime metadata without spinning up the full
/// Starlark engine.  The metadata is parsed once on first access.
pub fn star_metadata() -> &'static vx_starlark::StarMetadata {
    use std::sync::OnceLock;
    static META: OnceLock<vx_starlark::StarMetadata> = OnceLock::new();
    META.get_or_init(|| vx_starlark::StarMetadata::parse(PROVIDER_STAR))
}
```

Also add `vx-starlark` to `Cargo.toml`:

```toml
[dependencies]
vx-runtime = { workspace = true }
vx-starlark = { workspace = true }   # ← add this
```

#### Step 0c: Update provider.rs / runtime.rs (ALL providers with custom Rust)

Replace hardcoded strings with calls to `star_metadata()`:

```rust
// provider.rs / runtime.rs
impl Provider for MyProvider {
    fn name(&self) -> &str {
        // Sourced from provider.star: `def name(): return "mytool"`
        crate::star_metadata().name_or("mytool")
    }

    fn description(&self) -> &str {
        // Sourced from provider.star: `def description(): return "..."`
        // We need a &'static str; use OnceLock to leak once.
        use std::sync::OnceLock;
        static DESC: OnceLock<&'static str> = OnceLock::new();
        DESC.get_or_init(|| {
            let s = crate::star_metadata()
                .description
                .as_deref()
                .unwrap_or("My tool description");
            Box::leak(s.to_string().into_boxed_str())
        })
    }

    fn supports(&self, name: &str) -> bool {
        // Check primary name and all aliases from provider.star
        if name == self.name() { return true; }
        crate::star_metadata()
            .runtimes
            .iter()
            .any(|r| r.name.as_deref() == Some(name) || r.aliases.iter().any(|a| a == name))
    }
}

// In Runtime impl:
impl Runtime for MyRuntime {
    fn aliases(&self) -> &[&str] {
        // Sourced from provider.star runtimes[0].aliases
        use std::sync::OnceLock;
        static ALIASES: OnceLock<Vec<&'static str>> = OnceLock::new();
        ALIASES.get_or_init(|| {
            let meta = crate::star_metadata();
            if let Some(rt) = meta.runtimes.iter().find(|r| r.name.as_deref() == Some("mytool")) {
                rt.aliases
                    .iter()
                    .map(|a| Box::leak(a.clone().into_boxed_str()) as &'static str)
                    .collect()
            } else {
                vec![]
            }
        })
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        let star = crate::star_metadata();
        if let Some(hp) = star.homepage.as_deref() {
            meta.insert("homepage".to_string(), hp.to_string());
        }
        if let Some(repo) = star.repository.as_deref() {
            meta.insert("repository".to_string(), repo.to_string());
        }
        if let Some(license) = star.license.as_deref() {
            meta.insert("license".to_string(), license.to_string());
        }
        meta
    }
}
```

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
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

fetch_versions = make_fetch_versions("owner", "repo")

def _triple(ctx):
    os   = ctx.platform.os
    arch = ctx.platform.arch
    return {
        "windows/x64":  "x86_64-pc-windows-msvc",
        "macos/arm64":  "aarch64-apple-darwin",
        "linux/x64":    "x86_64-unknown-linux-musl",
    }.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    triple = _triple(ctx)
    if not triple:
        return None
    os  = ctx.platform.os
    ext = "zip" if os == "windows" else "tar.gz"
    asset = "tool-v{}-{}.{}".format(version, triple, ext)
    return github_asset_url("owner", "repo", "v" + version, asset)

def install_layout(ctx, version):
    os  = ctx.platform.os
    exe = "tool.exe" if os == "windows" else "tool"
    return {
        "type":             "archive",
        "strip_prefix":     "tool-v{}-{}".format(version, _triple(ctx) or ""),
        "executable_paths": [exe, "tool"],
    }

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def store_root(ctx):
    return ctx.vx_home + "/store/tool"

def get_execute_path(ctx, version):
    os = ctx.platform.os
    exe = "tool.exe" if os == "windows" else "tool"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
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

For providers that use `ManifestDrivenRuntime` (i.e., `provider.rs` delegates entirely to
the manifest), delete the now-redundant files:

```bash
# Safe to delete when provider.rs uses ManifestDrivenRuntime
rm crates/vx-providers/{name}/src/runtime.rs
rm crates/vx-providers/{name}/src/config.rs
```

For providers with custom Rust logic (brew, choco, make, msbuild, msvc, winget), keep
`runtime.rs` and `config.rs` but update them to read metadata from `star_metadata()` as
shown in Step 0c above.

**Current status** (as of this migration):
- ✅ 49 providers: fully manifest-driven (`ManifestDrivenRuntime`), `runtime.rs`/`config.rs` deleted
- ✅ 6 providers: custom Rust kept, metadata sourced from `provider.star` via `star_metadata()`
- ✅ All 55 providers: have `build.rs`, `PROVIDER_STAR` constant, and `star_metadata()` function

### Starlark Standard Library Quick Reference

| Module | Load Path | Key Functions |
|--------|-----------|---------------|
| GitHub | `@vx//stdlib:github.star` | `make_fetch_versions(owner, repo)`, `make_download_url(owner, repo, template)`, `make_github_provider(owner, repo, template)`, `github_asset_url(owner, repo, tag, asset)` |
| Platform | `@vx//stdlib:platform.star` | `is_windows(ctx)`, `is_macos(ctx)`, `is_linux(ctx)`, `is_x64(ctx)`, `is_arm64(ctx)`, `platform_triple(ctx)`, `platform_ext(ctx)`, `exe_ext(ctx)`, `arch_to_gnu(arch)`, `arch_to_go(arch)`, `os_to_go(os)` |
| Install | `@vx//stdlib:install.star` | `msi_install(url, ...)`, `archive_install(url, ...)`, `binary_install(url, ...)`, `platform_install(ctx, ...)` |
| Env | `@vx//stdlib:env.star` | `env_set(key, value)`, `env_prepend(key, value)`, `env_append(key, value)`, `env_unset(key)` |
| HTTP | `@vx//stdlib:http.star` | `github_releases(ctx, owner, repo)`, `fetch_json_versions(ctx, url, transform)`, `releases_to_versions(releases)` |
| Semver | `@vx//stdlib:semver.star` | `semver_compare(a, b)`, `semver_gt/lt/gte/lte/eq(a, b)`, `semver_sort(versions)`, `semver_strip_v(v)` |

### ctx Object Reference

```python
# ctx is a struct injected by the vx runtime (object-style access)
ctx.platform.os      # "windows" | "macos" | "linux"
ctx.platform.arch    # "x64" | "arm64" | "x86"
ctx.platform.target  # "x86_64-pc-windows-msvc" | "aarch64-apple-darwin" | ...
ctx.install_dir      # "/path/to/install/dir"
ctx.vx_home          # "~/.vx" (VX_HOME)
ctx.version          # current version being installed
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
    os = ctx.platform.os
    if os == "windows":
        return "https://github.com/owner/repo/releases/download/v{}/tool-{}-x64.msi".format(version, version)
    elif os == "macos":
        return github_asset_url("owner", "repo", "v" + version, "tool-{}-macos.tar.gz".format(version))
    elif os == "linux":
        return github_asset_url("owner", "repo", "v" + version, "tool-{}-linux.tar.gz".format(version))
    return None

def install_layout(ctx, version):
    os  = ctx.platform.os
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

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def store_root(ctx):
    return ctx.vx_home + "/store/tool"

def get_execute_path(ctx, version):
    os = ctx.platform.os
    exe = "tool.exe" if os == "windows" else "tool"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def deps(_ctx, _version):
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

## Migration: Adding System Package Manager Fallback

When a tool has "No download URL" errors on certain platforms, add package manager fallback.

### Identify the Problem

Error messages indicating need for package manager fallback:
- `No download URL for {tool} {version}` on macOS/Windows
- `Executable not found: ~/.vx/store/{tool}/{version}/bin/{tool}` after "successful" install
- Tool works on Linux but fails on macOS/Windows

### Step 1: Check Platform Coverage

```bash
# Check which platforms have binary downloads in provider.star
grep -A 5 "download_url" crates/vx-providers/{name}/provider.star

# If download_url returns None for macos/windows, need system_install fallback
```

### Step 2: Add system_install() to provider.star

```python
# In provider.star — add or update system_install function
def system_install(ctx):
    os = ctx.platform.os
    if os == "windows":
        return {
            "strategies": [
                {"manager": "winget", "package": "Publisher.Package", "priority": 95},
                {"manager": "choco",  "package": "{tool}",            "priority": 80},
                {"manager": "scoop",  "package": "{tool}",            "priority": 60},
            ],
        }
    elif os == "macos":
        return {
            "strategies": [
                {"manager": "brew", "package": "{tool}", "priority": 90},
            ],
        }
    elif os == "linux":
        return {
            "strategies": [
                {"manager": "apt", "package": "{tool}", "priority": 80},
                {"manager": "dnf", "package": "{tool}", "priority": 80},
            ],
        }
    return {}
```

### Step 3: Update runtime.rs (for custom Runtime implementations)

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

## Migration: Adding package_alias for Ecosystem-Managed Tools (RFC 0033)

Use `package_alias` when a tool is **distributed as a package** in an ecosystem (PyPI via `uvx`,
or npm via `npx`) rather than as a standalone binary. This routes `vx <name>` to
`vx <ecosystem>:<package>`, giving each version its own isolated environment.

### When to Use

| Tool Type | Example | package_alias |
|-----------|---------|---------------|
| Python CLI tool (PyPI) | meson, ruff, black, mypy, nox | `{"ecosystem": "uvx", "package": "..."}` |
| npm CLI tool | vite, eslint, prettier, create-react-app | `{"ecosystem": "npx", "package": "..."}` |

### How It Works

```
vx meson@1.5.0
  ↓ RFC 0033: package_alias routing (from provider.star)
vx uvx:meson@1.5.0
  ↓ UvxInstaller.install()
uv tool install meson==1.5.0  (pre-warms uv cache)
+ creates shim: exec uvx meson==1.5.0 "$@"
  ↓ execution
uvx meson==1.5.0 [args...]  ← isolated Python env per version
```

### Step 1: Add package_alias to provider.star

```python
# provider.star - for a Python/PyPI tool (e.g., meson, ruff, black)
name        = "meson"
description = "Meson - An extremely fast and user friendly build system"
homepage    = "https://mesonbuild.com"
repository  = "https://github.com/mesonbuild/meson"
license     = "Apache-2.0"
ecosystem   = "python"
aliases     = ["mesonbuild"]

# RFC 0033: route `vx meson` → `vx uvx:meson`
# Each version runs in its own isolated uv-managed Python environment.
package_alias = {"ecosystem": "uvx", "package": "meson"}

runtimes = [
    {
        "name":        "meson",
        "executable":  "meson",
        "description": "Meson build system",
        "aliases":     ["mesonbuild"],
        "priority":    100,
    },
]

permissions = {
    "http": ["pypi.org"],
    "fs":   [],
    "exec": ["uvx", "uv"],
}

def download_url(_ctx, _version):
    return None  # Not applicable; runs via uvx

def deps(_ctx, version):
    return [
        {"runtime": "uv", "version": "*",
         "reason": "Tool is installed and run via uv"},
    ]
```

### Step 2: Simplify provider.toml (metadata only)

When using `package_alias`, the `provider.toml` only needs metadata — no layout config needed:

```toml
[provider]
name = "meson"
description = "Meson - An extremely fast and user friendly build system"
homepage = "https://mesonbuild.com"
repository = "https://github.com/mesonbuild/meson"
ecosystem = "python"
license = "Apache-2.0"
```

### Step 3: Remove layout configuration

If the provider previously had layout config (binary/archive), remove it — `package_alias`
tools don't download binaries directly.

### Ecosystem Comparison

| Syntax | Equivalent | Installer | Runtime Dep | Isolation |
|--------|-----------|-----------|-------------|-----------|
| `vx meson@1.5.0` | `vx uvx:meson@1.5.0` | `UvxInstaller` | `uv` | Per-version Python env |
| `vx ruff@0.9.0` | `vx uvx:ruff@0.9.0` | `UvxInstaller` | `uv` | Per-version Python env |
| `vx vite@5.0` | `vx npx:vite@5.0` | `NpmInstaller` | `node` | npm cache |
| `vx yarn@1.22` | `vx npm:yarn@1.22` | `NpmInstaller` | `node` | npm cache |

### Troubleshooting: package_alias Not Working

**Issue**: `vx meson` still tries to download a binary instead of routing to `uvx:meson`

**Cause**: `package_alias` field not parsed from `provider.star` into `StarMetadata`

**Check**:
1. Verify `package_alias = {"ecosystem": "uvx", "package": "meson"}` is a **top-level variable** in `provider.star` (not inside a function)
2. Verify `StarMetadata::parse()` reads `package_alias` (check `vx-starlark/src/metadata.rs`)
3. Verify `parse_metadata()` in `vx-starlark/src/provider/mod.rs` maps `star_meta.package_alias` to `ProviderMeta.package_alias`

**Issue**: `vx uvx:ruff@0.9.0` fails with "Unknown runtime 'uvx:ruff'"

**Cause**: `uvx` ecosystem not registered in the installer or runtime dependency maps

**Check** (all three must be present):
1. `vx-ecosystem-pm/src/lib.rs`: `"uvx" => Ok(Box::new(UvxInstaller::new()))`
2. `vx-cli/src/lib.rs` → `get_all_required_runtimes_for_ecosystem("uvx")`: returns `vec!["uv"]`
3. `vx-shim/src/executor.rs` → `infer_all_runtimes_from_ecosystem("uvx")`: returns `vec![RuntimeDependency { runtime: "uv", ... }]`

## Reference

See also:
- `docs/provider-migration-status.md` - Migration status tracker
- `docs/provider-update-summary.md` - Batch update summary
- `crates/vx-providers/imagemagick/` - Example hybrid provider with PM fallback
- `crates/vx-providers/meson/` - Example package_alias provider (uvx ecosystem)
