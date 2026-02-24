---
name: vx-provider-creator
description: |
  This skill should be used when creating a new runtime provider for the vx tool manager.
  It provides complete templates, code generation, and step-by-step guidance for implementing
  Provider and Runtime traits, including URL builders, platform configuration, test files,
  provider.toml manifest, system package manager fallback, ecosystem-managed tools via
  package_alias (RFC 0033: vx meson = vx uvx:meson, vx vite = vx npx:vite), and optionally
  project analyzer integration for language-specific tools.
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
- **Adding PyPI/npm tools that run in isolated environments** (e.g., `vx meson` = `vx uvx:meson`)

## Workflow Overview

1. **Check license compatibility** (MUST DO FIRST)
2. Create a feature branch from remote main
3. **Determine installation type** (direct download / system package manager / ecosystem package)
4. Generate provider directory structure (including `provider.toml`)
5. Implement core files (lib.rs, provider.rs, runtime.rs, config.rs)
6. **Add system package manager fallback if needed**
7. **Add `package_alias` if tool is a PyPI/npm package** (RFC 0033)
8. Register the provider in workspace and CLI
9. **(Optional)** Add project analyzer integration for language-specific tools
10. Update snapshot tests
11. Verify and test

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
    ├─ No portable binaries → Is it a PyPI/npm package?
    │   ├─ Yes (PyPI) → package_alias = {"ecosystem": "uvx", "package": "..."}
    │   │   └─ Examples: meson, ruff, black, mypy, nox, pre-commit
    │   ├─ Yes (npm)  → package_alias = {"ecosystem": "npx", "package": "..."}
    │   │   └─ Examples: vite, eslint, prettier, create-react-app
    │   └─ No → System Package Manager Only
    │       └─ Examples: make, git (on non-Windows), curl, openssl
    └─ System-installed only → Detection-only
        └─ Examples: msbuild, xcodebuild, systemctl
```

### Provider Types Summary

| Type | Direct Download | Package Manager Fallback | Examples |
|------|-----------------|-------------------------|----------|
| **Standard** | ✅ All platforms | ❌ Not needed | terraform, just, go, node |
| **Hybrid** | ✅ Some platforms | ✅ For others | imagemagick, ffmpeg, docker |
| **Ecosystem (uvx)** | ❌ None | ❌ Runs via `uvx` | meson, ruff, black, mypy |
| **Ecosystem (npx)** | ❌ None | ❌ Runs via `npx` | vite, eslint, prettier |
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
├── build.rs        # REQUIRED: watches provider.star for changes
├── provider.toml   # Provider manifest (metadata, runtimes, constraints)
├── provider.star   # Starlark logic (fetch_versions, download_url, install_layout)
├── src/
│   ├── lib.rs          # Module exports + PROVIDER_STAR + star_metadata() + create_provider()
│   ├── provider.rs     # Provider trait implementation (optional for Starlark-only)
│   ├── runtime.rs      # Runtime trait implementation (optional for Starlark-only)
│   └── config.rs       # URL builder and platform configuration (optional)
└── tests/
    └── runtime_tests.rs  # Unit tests (using rstest)
```

### Required: build.rs

Every provider crate **must** have a `build.rs` that watches `provider.star`:

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

### Required: lib.rs with PROVIDER_STAR and star_metadata()

Every provider crate's `lib.rs` **must** embed `provider.star` and expose `star_metadata()`:

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

// ... rest of lib.rs (module declarations, create_provider, etc.)
```

### Required: Cargo.toml with vx-starlark

```toml
[dependencies]
vx-runtime = { workspace = true }
vx-starlark = { workspace = true }   # REQUIRED for star_metadata()
# ... other deps
```

## Step 2.1: Create provider.toml Manifest

The `provider.toml` file is the declarative manifest for the provider. It defines:
- Provider metadata (name, description, homepage, ecosystem, license)
- Runtime definitions (executable, aliases, bundled tools)
- Version source configuration (optional — can also use `make_fetch_versions` in provider.star)
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

See `references/templates.md` for complete provider.toml template.

## Step 2.2: Create provider.star (Starlark Script)

**provider.star is the preferred way to implement providers.** It replaces the need for Rust code (`runtime.rs`, `config.rs`) for most providers. The Starlark script is pure computation — all real I/O (HTTP, filesystem, msiexec) is performed by the Rust runtime based on descriptor dicts returned by the script.

### File Location

```
crates/vx-providers/{name}/
├── provider.toml   # Metadata only (name, description, ecosystem, license)
└── provider.star   # Logic: fetch_versions, download_url, install_layout, etc.
```

> **When to use provider.star vs provider.toml layout:**
> - `provider.star` — for any custom logic: platform-specific URLs, MSI installs, system package manager fallback, complex version parsing
> - `provider.toml` layout fields — only for simple standard archive/binary downloads with no custom logic

### Starlark Standard Library

Load helpers from `@vx//stdlib:`:

| Module | Key Functions | Use Case |
|--------|--------------|----------|
| `github.star` | `make_fetch_versions`, `make_download_url`, `make_github_provider`, `github_asset_url` | GitHub releases |
| `http.star` | `github_releases`, `releases_to_versions`, `parse_github_tag` | HTTP descriptors |
| `platform.star` | `is_windows`, `is_macos`, `is_linux`, `is_x64`, `is_arm64`, `platform_triple`, `platform_ext`, `exe_ext`, `arch_to_gnu`, `arch_to_go`, `os_to_go` | Platform detection |
| `install.star` | `msi_install`, `archive_install`, `binary_install`, `platform_install` | Install descriptors |
| `semver.star` | `semver_compare`, `semver_gt`, `semver_lt`, `semver_parse`, `semver_sort`, `semver_strip_v` | Version comparison |

### provider.star Structure

A complete `provider.star` has these top-level symbols:

```python
# ── Metadata (required, top-level variables) ─────────────────────────────
name        = "mytool"
description = "My awesome tool"
homepage    = "https://example.com"
repository  = "https://github.com/owner/repo"
license     = "MIT"          # SPDX identifier (REQUIRED)
ecosystem   = "devtools"     # nodejs/python/rust/go/devtools/system/...
aliases     = ["mt"]         # optional

# ── Platform constraint (optional, provider-level) ────────────────────────
platforms = {"os": ["windows"]}         # omit if cross-platform

# ── Runtime definitions (required) ───────────────────────────────────────
runtimes = [
    {
        "name":        "mytool",
        "executable":  "mytool",
        "description": "My tool CLI",
        "aliases":     ["mt"],
        "priority":    100,
        # optional: "platform_constraint": {"os": ["windows"]},
        # optional: "bundled_with": "other-runtime",
        # optional: "system_paths": ["C:/Program Files/MyTool"],
        # optional: "test_commands": [
        #     {"command": "{executable} --version", "name": "version_check", "expected_output": "\\d+\\.\\d+"},
        # ],
    },
]

# ── Permissions (sandbox declaration) ────────────────────────────────────
permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": [],
}

# ── fetch_versions (required) ─────────────────────────────────────────────
# Option A: inherit from github.star (zero code)
fetch_versions = make_fetch_versions("owner", "repo")

# Option B: custom logic (uses ctx.platform.os object-style access)
def fetch_versions(ctx):
    releases = ctx.http.get_json("https://api.github.com/repos/owner/repo/releases?per_page=30")
    versions = []
    for release in releases:
        if release.get("draft") or release.get("prerelease"):
            continue
        tag = release.get("tag_name", "")
        v = tag.lstrip("v")
        if v:
            versions.append({"version": v, "lts": True, "prerelease": False})
    return versions

# ── download_url (required) ───────────────────────────────────────────────
# Option A: inherit from github.star
download_url = make_download_url("owner", "repo", "mytool-{vversion}-{triple}.{ext}")

# Option B: custom logic (uses ctx.platform.os object-style access)
def download_url(ctx, version):
    os   = ctx.platform.os
    arch = ctx.platform.arch
    # ... build URL ...
    return url_string  # or None if unsupported

# ── install_layout (required for non-trivial installs) ────────────────────
def install_layout(ctx, version):
    # Returns a descriptor dict; Rust runtime performs actual extraction
    return {
        "type":             "archive",   # or "binary", "msi"
        "strip_prefix":     "mytool-{}".format(version),
        "executable_paths": ["bin/mytool.exe", "bin/mytool"],
    }

# ── environment (optional) ────────────────────────────────────────────────
# Returns a list of env ops (use env.star helpers)
load("@vx//stdlib:env.star", "env_prepend")

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

# ── Path queries (RFC 0037, required) ─────────────────────────────────────
def store_root(ctx):
    return ctx.vx_home + "/store/mytool"

def get_execute_path(ctx, version):
    os = ctx.platform.os
    exe = "mytool.exe" if os == "windows" else "mytool"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None  # or list of post-install descriptors

# ── system_install (optional, for package manager fallback) ───────────────
def system_install(ctx):
    os = ctx.platform.os
    if os == "windows":
        return {"strategies": [{"manager": "winget", "package": "Publisher.MyTool", "priority": 95}]}
    elif os == "macos":
        return {"strategies": [{"manager": "brew", "package": "mytool", "priority": 90}]}
    return {}

# ── constraints (optional) ────────────────────────────────────────────────
constraints = [
    {
        "when": "*",
        "recommends": [{"runtime": "git", "version": ">=2.0", "reason": "Used as backend"}],
    },
]

# ── deps (optional) ───────────────────────────────────────────────────────
def deps(ctx, version):
    return []  # list of {"runtime": "node", "version": ">=18"}

# ── uninstall (optional) ──────────────────────────────────────────────────
# Return False to let vx remove the store directory (default).
# Return a system_uninstall descriptor for system-installed tools.
def uninstall(ctx, _version):
    os = ctx.platform.os
    if os == "windows":
        return {
            "type": "system_uninstall",
            "strategies": [{"manager": "winget", "package": "Publisher.MyTool", "priority": 95}],
        }
    return False  # let vx remove the store directory

# ── package_alias (optional, RFC 0033) ────────────────────────────────────
# Routes `vx <name>` → `vx <ecosystem>:<package>` for ecosystem-managed tools.
# Example: `vx meson` → `vx uvx:meson` (isolated Python env via uv)
#          `vx vite`  → `vx npx:vite`  (npm package via npx)
#
# Supported ecosystems: "uvx" (uv tool), "npx" (npm package)
package_alias = {"ecosystem": "uvx", "package": "meson"}
```

### package_alias — Ecosystem-Managed Tools (RFC 0033)

Use `package_alias` when a tool is **distributed as a package** in an ecosystem (PyPI, npm)
rather than as a standalone binary. This routes `vx <name>` to `vx <ecosystem>:<package>`,
giving each version its own isolated environment.

#### When to Use

| Tool Type | Example | package_alias |
|-----------|---------|---------------|
| Python CLI tool (PyPI) | meson, ruff, black, mypy | `{"ecosystem": "uvx", "package": "..."}` |
| npm CLI tool | vite, eslint, prettier | `{"ecosystem": "npx", "package": "..."}` |

#### How It Works

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

#### provider.star Example (Python/PyPI tool)

```python
# provider.star - Meson provider
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
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "^\\d+\\.\\d+"},
        ],
    },
]

permissions = {
    "http": ["pypi.org"],
    "fs":   [],
    "exec": ["uvx", "uv"],
}

def download_url(_ctx, _version):
    return None  # Not applicable; runs via uvx

def store_root(ctx):
    return ctx.vx_home + "/store/meson"

def get_execute_path(_ctx, _version):
    return None

def post_install(_ctx, _version):
    return None

def deps(_ctx, version):
    return [
        {"runtime": "uv", "version": "*",
         "reason": "Tool is installed and run via uv"},
    ]
```

#### provider.star Example (npm tool)

```python
# provider.star - Vite provider
name        = "vite"
description = "Next generation frontend tooling"
homepage    = "https://vitejs.dev"
repository  = "https://github.com/vitejs/vite"
license     = "MIT"
ecosystem   = "nodejs"

# RFC 0033: route `vx vite` → `vx npx:vite`
package_alias = {"ecosystem": "npx", "package": "vite"}

runtimes = [
    {
        "name":        "vite",
        "executable":  "vite",
        "description": "Vite build tool",
        "priority":    100,
    },
]

permissions = {
    "http": ["registry.npmjs.org"],
    "fs":   [],
    "exec": ["npx", "node"],
}

def download_url(_ctx, _version):
    return None  # Not applicable; runs via npx

def store_root(ctx):
    return ctx.vx_home + "/store/vite"

def get_execute_path(_ctx, _version):
    return None

def post_install(_ctx, _version):
    return None

def deps(_ctx, version):
    return [
        {"runtime": "node", "version": ">=18",
         "reason": "Tool is installed and run via npx"},
    ]
```

#### Ecosystem Comparison

| Syntax | Equivalent | Installer | Runtime Dep | Isolation |
|--------|-----------|-----------|-------------|-----------|
| `vx meson@1.5.0` | `vx uvx:meson@1.5.0` | `UvxInstaller` | `uv` | Per-version Python env |
| `vx ruff@0.9.0` | `vx uvx:ruff@0.9.0` | `UvxInstaller` | `uv` | Per-version Python env |
| `vx vite@5.0` | `vx npx:vite@5.0` | `NpmInstaller` | `node` | npm cache |
| `vx yarn@1.22` | `vx npm:yarn@1.22` | `NpmInstaller` | `node` | npm cache |

#### Implementation Notes

- `package_alias` is parsed from `provider.star` by `StarMetadata::parse()`
- The routing happens in `vx-cli/src/lib.rs` (RFC 0033 logic)
- `uvx` ecosystem requires `uv` runtime; `npx` ecosystem requires `node` runtime
- Version pinning in `vx.toml` works normally: `meson = "1.5.0"` → `uvx meson==1.5.0`

### Inheritance Levels

Choose the level that fits your provider:

**Level 0 — Fully inherited (2 lines)**
```python
load("@vx//stdlib:github.star", "make_github_provider")
_p = make_github_provider("owner", "repo", "mytool-{vversion}-{triple}.{ext}")
fetch_versions = _p["fetch_versions"]
download_url   = _p["download_url"]
```

**Level 1 — Inherit fetch_versions, custom download_url**
```python
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:platform.star", "is_windows")

fetch_versions = make_fetch_versions("owner", "repo")

def download_url(ctx, version):
    os  = ctx.platform.os
    ext = "zip" if os == "windows" else "tar.gz"
    asset = "mytool-v{}-{}.{}".format(version, os, ext)
    return github_asset_url("owner", "repo", "v" + version, asset)
```

**Level 2 — Fully custom (non-GitHub source)**
```python
def fetch_versions(ctx):
    data = ctx.http.get_json("https://example.com/api/versions")
    return [{"version": v["name"], "lts": True, "prerelease": False} for v in data]

def download_url(ctx, version):
    os = ctx.platform.os
    return "https://example.com/download/{}/{}".format(version, os)
```

### MSI Install (Windows)

For tools that distribute `.msi` installers on Windows, use `msi_install()` from `install.star`:

```python
load("@vx//stdlib:install.star", "msi_install", "archive_install")
load("@vx//stdlib:platform.star", "is_windows")

def download_url(ctx, version):
    os = ctx.platform.os
    if os == "windows":
        return "https://example.com/tool-{}.msi".format(version)
    elif os == "macos":
        return "https://example.com/tool-{}-macos.tar.gz".format(version)
    elif os == "linux":
        return "https://example.com/tool-{}-linux.tar.gz".format(version)
    return None

def install_layout(ctx, version):
    os = ctx.platform.os
    if os == "windows":
        url = download_url(ctx, version)
        # msi_install uses msiexec /a (administrative install, no registry changes)
        return msi_install(
            url,
            executable_paths = ["bin/tool.exe", "tool.exe"],
            strip_prefix = "PFiles/Tool",  # optional: strip msiexec extraction prefix
        )
    else:
        url = download_url(ctx, version)
        return archive_install(
            url,
            strip_prefix = "tool-{}".format(version),
            executable_paths = ["bin/tool"],
        )
```

> **How MSI install works:** `msi_install()` returns a descriptor dict. The Rust runtime runs:
> `msiexec /a <file.msi> /qn /norestart TARGETDIR=<install_dir>`
> This extracts the MSI contents without modifying the Windows registry.

### platform_install() Convenience Helper

For tools with different URLs per platform (including MSI on Windows):

```python
load("@vx//stdlib:install.star", "platform_install")

def install_layout(ctx, version):
    return platform_install(
        ctx,
        windows_url = "https://example.com/tool-{}.msi".format(version),
        macos_url   = "https://example.com/tool-{}-macos.tar.gz".format(version),
        linux_url   = "https://example.com/tool-{}-linux.tar.gz".format(version),
        windows_msi = True,                          # use msi_install on Windows
        executable_paths = ["bin/tool.exe", "bin/tool"],
        strip_prefix = "tool-{}".format(version),
    )
```

### System Package Manager Fallback

For tools without portable binaries on some platforms:

```python
def download_url(ctx, version):
    os = ctx.platform.os
    if os == "linux":
        return "https://github.com/owner/repo/releases/download/v{}/tool-linux.tar.gz".format(version)
    # Windows/macOS: no portable binary → return None → triggers system_install
    return None

def system_install(ctx):
    os = ctx.platform.os
    if os == "windows":
        return {
            "strategies": [
                {"manager": "winget", "package": "Publisher.Tool", "priority": 95},
                {"manager": "choco",  "package": "tool",           "priority": 80},
                {"manager": "scoop",  "package": "tool",           "priority": 60},
            ],
        }
    elif os == "macos":
        return {
            "strategies": [
                {"manager": "brew", "package": "tool", "priority": 90},
            ],
        }
    elif os == "linux":
        return {
            "strategies": [
                {"manager": "apt", "package": "tool", "priority": 80},
                {"manager": "dnf", "package": "tool", "priority": 80},
            ],
        }
    return {}
```

### ctx Object Reference

The `ctx` object injected by the vx runtime (object-style attribute access):

```python
# Platform info (object-style access)
ctx.platform.os     # "windows" | "macos" | "linux"
ctx.platform.arch   # "x64" | "arm64" | "x86"
ctx.platform.target # "x86_64-pc-windows-msvc" | ...  (Rust target triple)

# Paths
ctx.install_dir     # "/path/to/install"
ctx.vx_home         # "~/.vx"
ctx.cache_dir       # "/path/to/cache"

# HTTP (descriptor-based, not direct calls)
# Use stdlib functions: github_releases(), fetch_json_versions(), etc.
```

### install_layout Return Values

| Type | Fields | Description |
|------|--------|-------------|
| `"archive"` | `strip_prefix`, `executable_paths` | ZIP/TAR.GZ/TAR.XZ archive |
| `"binary"` | `executable_name`, `source_name` (opt), `permissions` (opt) | Single file download |
| `"msi"` | `url`, `executable_paths` (opt), `strip_prefix` (opt), `extra_args` (opt) | Windows MSI installer |

### Complete Example: Standard GitHub Provider

```python
# provider.star - ripgrep provider
load("@vx//stdlib:github.star",   "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",      "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata (top-level variables)
# ---------------------------------------------------------------------------
name        = "ripgrep"
description = "ripgrep - recursively searches directories for a regex pattern"
homepage    = "https://github.com/BurntSushi/ripgrep"
repository  = "https://github.com/BurntSushi/ripgrep"
license     = "MIT OR Unlicense"
ecosystem   = "devtools"
aliases     = ["rg"]

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------
runtimes = [
    {
        "name":        "ripgrep",
        "executable":  "rg",
        "description": "Fast regex search tool",
        "aliases":     ["rg"],
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "ripgrep \\d+"},
        ],
    },
]

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# fetch_versions — fully inherited
# ---------------------------------------------------------------------------
fetch_versions = make_fetch_versions("BurntSushi", "ripgrep")

# ---------------------------------------------------------------------------
# download_url — custom override (tag has NO 'v' prefix)
# ---------------------------------------------------------------------------
def _rg_triple(ctx):
    os   = ctx.platform.os
    arch = ctx.platform.arch
    triples = {
        "windows/x64":  "x86_64-pc-windows-msvc",
        "macos/x64":    "x86_64-apple-darwin",
        "macos/arm64":  "aarch64-apple-darwin",
        "linux/x64":    "x86_64-unknown-linux-musl",
        "linux/arm64":  "aarch64-unknown-linux-gnu",
    }
    return triples.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    triple = _rg_triple(ctx)
    if not triple:
        return None
    os  = ctx.platform.os
    ext = "zip" if os == "windows" else "tar.gz"
    asset = "ripgrep-{}-{}.{}".format(version, triple, ext)
    tag = version  # ripgrep tags have NO 'v' prefix
    return github_asset_url("BurntSushi", "ripgrep", tag, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------
def install_layout(ctx, version):
    triple = _rg_triple(ctx)
    os  = ctx.platform.os
    exe = "rg.exe" if os == "windows" else "rg"
    strip_prefix = "ripgrep-{}-{}".format(version, triple) if triple else ""
    return {
        "type":             "archive",
        "strip_prefix":     strip_prefix,
        "executable_paths": [exe, "rg"],
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
    return ctx.vx_home + "/store/ripgrep"

def get_execute_path(ctx, version):
    os = ctx.platform.os
    exe = "rg.exe" if os == "windows" else "rg"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def deps(ctx, version):
    return []
```

### Complete Example: MSI on Windows + Archive on Other Platforms

```python
# provider.star - tool with MSI installer on Windows
load("@vx//stdlib:install.star",  "msi_install", "archive_install")
load("@vx//stdlib:github.star",   "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",      "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata (top-level variables)
# ---------------------------------------------------------------------------
name        = "mytool"
description = "My tool with MSI installer on Windows"
homepage    = "https://example.com"
repository  = "https://github.com/owner/mytool"
license     = "MIT"
ecosystem   = "devtools"

runtimes = [{"name": "mytool", "executable": "mytool", "description": "My tool", "priority": 100}]
permissions = {"http": ["api.github.com", "github.com"], "fs": [], "exec": []}

fetch_versions = make_fetch_versions("owner", "mytool")

def download_url(ctx, version):
    os = ctx.platform.os
    if os == "windows":
        return "https://github.com/owner/mytool/releases/download/v{}/mytool-{}-x64.msi".format(version, version)
    elif os == "macos":
        return github_asset_url("owner", "mytool", "v" + version, "mytool-{}-macos.tar.gz".format(version))
    elif os == "linux":
        return github_asset_url("owner", "mytool", "v" + version, "mytool-{}-linux.tar.gz".format(version))
    return None

def install_layout(ctx, version):
    os = ctx.platform.os
    url = download_url(ctx, version)
    if os == "windows":
        # MSI: msiexec /a extracts to TARGETDIR, no registry changes
        return msi_install(
            url,
            executable_paths = ["bin/mytool.exe", "mytool.exe"],
            # strip_prefix = "PFiles/MyTool",  # uncomment if msiexec extracts to a subdir
        )
    else:
        return archive_install(
            url,
            strip_prefix = "mytool-{}".format(version),
            executable_paths = ["bin/mytool"],
        )

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def store_root(ctx):
    return ctx.vx_home + "/store/mytool"

def get_execute_path(ctx, version):
    os = ctx.platform.os
    exe = "mytool.exe" if os == "windows" else "mytool"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def deps(ctx, version):
    return []
```
```

## Step 3: Implement Core Files

> **Preferred approach:** Use `provider.star` (Starlark) instead of Rust files for most providers.
> Only create Rust files (`runtime.rs`, `config.rs`) when you need capabilities not available in Starlark.

### Option A: Starlark-only Provider (Recommended)

For most providers, you only need:

```
crates/vx-providers/{name}/
├── Cargo.toml      # includes vx-starlark dependency
├── build.rs        # watches provider.star
├── provider.toml   # metadata: name, description, ecosystem, license
├── provider.star   # all logic: fetch_versions, download_url, install_layout
└── src/
    └── lib.rs      # PROVIDER_STAR + star_metadata() + create_provider()
```

The `lib.rs` for a Starlark-only provider:

```rust
pub const PROVIDER_STAR: &str = include_str!("../provider.star");

pub fn star_metadata() -> &'static vx_starlark::StarMetadata {
    use std::sync::OnceLock;
    static META: OnceLock<vx_starlark::StarMetadata> = OnceLock::new();
    META.get_or_init(|| vx_starlark::StarMetadata::parse(PROVIDER_STAR))
}

use std::sync::Arc;
use vx_runtime::Provider;

pub fn create_provider() -> Arc<dyn Provider> {
    // ManifestDrivenRuntime reads from provider.star embedded above
    Arc::new(vx_runtime::ManifestDrivenProvider::new(
        PROVIDER_STAR,
        include_str!("../provider.toml"),
    ))
}
```

The `provider.toml` for a Starlark provider only needs metadata:

```toml
[provider]
name = "mytool"
description = "My awesome tool"
homepage = "https://example.com"
repository = "https://github.com/owner/repo"
ecosystem = "devtools"
license = "MIT"
```

All logic (versions, URLs, install layout, system_install) goes in `provider.star`.
See **Step 2.2** for the complete Starlark guide.

### Option B: Rust Provider (for advanced cases)

Refer to `references/templates.md` for complete code templates.

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

> **Note:** For Starlark providers, use `system_install(ctx)` in `provider.star` instead:
> ```python
> def system_install(ctx):
>     os = ctx.platform.os
>     if os == "windows":
>         return {"strategies": [
>             {"manager": "winget", "package": "Publisher.Package", "priority": 95},
>             {"manager": "choco",  "package": "mytool",            "priority": 80},
>         ]}
>     elif os == "macos":
>         return {"strategies": [{"manager": "brew", "package": "mytool", "priority": 90}]}
>     return {}
> ```

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

### provider.toml Quick Reference

### Minimal Example (Metadata Only — Logic in provider.star)

For Starlark providers, `provider.toml` only needs metadata. All logic goes in `provider.star`:

```toml
[provider]
name = "mytool"
description = "My awesome tool"
homepage = "https://github.com/owner/repo"
repository = "https://github.com/owner/repo"
ecosystem = "devtools"
license = "MIT"
```

See **Step 2.2** and `references/templates.md` for complete `provider.star` templates.
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
