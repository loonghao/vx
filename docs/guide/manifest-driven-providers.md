# Manifest-Driven Providers

vx uses **`provider.star`** (Starlark) as the single source of truth for all provider logic.
Instead of writing Rust code for each tool, you create a `provider.star` file that describes
everything vx needs: metadata, version fetching, download URLs, install layout, environment
variables, and system package manager fallback.

## Overview

A manifest-driven provider uses a single file:

| File | Purpose |
|------|---------|
| `provider.star` | **All logic and metadata** — name, description, download URLs, install layout, environment, system_install |

This approach makes it easy to:
- Add new tools without writing Rust code
- Customize tool behavior through Starlark scripting
- Share tool definitions across teams
- Maintain consistent tool management

## Quick Start

### Using Built-in Providers

vx comes with 60+ built-in providers for popular tools:

```bash
vx node --version      # Node.js
vx go version          # Go
vx jq --help           # jq JSON processor
vx ffmpeg -version     # FFmpeg media toolkit
vx rg --version        # ripgrep
```

### Creating a Custom Provider

Create a `provider.star` file in `~/.vx/providers/mytool/`:

```bash
mkdir -p ~/.vx/providers/mytool
```

```python
# ~/.vx/providers/mytool/provider.star
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Metadata
# ---------------------------------------------------------------------------
name        = "mytool"
description = "My awesome tool"
homepage    = "https://github.com/myorg/mytool"
repository  = "https://github.com/myorg/mytool"
license     = "MIT"
ecosystem   = "devtools"

runtimes = [
    {
        "name":        "mytool",
        "executable":  "mytool",
        "description": "My tool runtime",
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    },
]

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": [],
}

# ---------------------------------------------------------------------------
# Version fetching
# ---------------------------------------------------------------------------
fetch_versions = make_fetch_versions("myorg", "mytool")

# ---------------------------------------------------------------------------
# Download URL
# ---------------------------------------------------------------------------
def download_url(ctx, version):
    os   = ctx.platform.os
    arch = ctx.platform.arch
    triples = {
        "windows/x64":  "x86_64-pc-windows-msvc",
        "macos/x64":    "x86_64-apple-darwin",
        "macos/arm64":  "aarch64-apple-darwin",
        "linux/x64":    "x86_64-unknown-linux-musl",
        "linux/arm64":  "aarch64-unknown-linux-gnu",
    }
    triple = triples.get("{}/{}".format(os, arch))
    if not triple:
        return None
    ext   = "zip" if os == "windows" else "tar.gz"
    asset = "mytool-{}-{}.{}".format(version, triple, ext)
    return github_asset_url("myorg", "mytool", "v" + version, asset)

# ---------------------------------------------------------------------------
# Install layout
# ---------------------------------------------------------------------------
def install_layout(ctx, version):
    os  = ctx.platform.os
    exe = "mytool.exe" if os == "windows" else "mytool"
    return {
        "type":             "archive",
        "strip_prefix":     "mytool-{}".format(version),
        "executable_paths": [exe, "mytool"],
    }

# ---------------------------------------------------------------------------
# Path queries
# ---------------------------------------------------------------------------
def store_root(ctx):
    return ctx.vx_home + "/store/mytool"

def get_execute_path(ctx, version):
    os  = ctx.platform.os
    exe = "mytool.exe" if os == "windows" else "mytool"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# Environment
# ---------------------------------------------------------------------------
def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]
```

Now use it:

```bash
vx mytool --version
```

## provider.star Structure

### Metadata Variables (Top-Level)

All metadata is declared as **top-level variables** (not functions):

```python
name        = "ripgrep"                              # Required
description = "Fast regex search tool"               # Required
homepage    = "https://github.com/BurntSushi/ripgrep"
repository  = "https://github.com/BurntSushi/ripgrep"
license     = "MIT OR Unlicense"                     # Required (SPDX identifier)
ecosystem   = "devtools"                             # Required
aliases     = ["rg"]                                 # Optional
```

**Ecosystem values:**
`nodejs`, `python`, `rust`, `go`, `ruby`, `java`, `dotnet`, `devtools`,
`container`, `cloud`, `ai`, `cpp`, `zig`, `system`

### runtimes List

Define executables provided by this provider:

```python
runtimes = [
    {
        "name":        "ripgrep",      # Runtime name
        "executable":  "rg",           # Actual executable filename
        "description": "Fast regex search tool",
        "aliases":     ["rg"],         # Alternative names
        "priority":    100,
        "test_commands": [
            {
                "command":         "{executable} --version",
                "name":            "version_check",
                "expected_output": "ripgrep \\d+",
            },
        ],
    },
]
```

### permissions

Declare what the provider is allowed to access:

```python
permissions = {
    "http": ["api.github.com", "github.com"],  # Allowed HTTP hosts
    "fs":   [],                                 # Allowed filesystem paths
    "exec": [],                                 # Allowed executables to spawn
}
```

### ctx Object Reference

The `ctx` object is injected by the vx runtime (object-style access):

```python
ctx.platform.os      # "windows" | "macos" | "linux"
ctx.platform.arch    # "x64" | "arm64" | "x86"
ctx.platform.target  # "x86_64-pc-windows-msvc" | "aarch64-apple-darwin" | ...
ctx.install_dir      # "/path/to/install/dir"
ctx.vx_home          # "~/.vx" (VX_HOME)
ctx.version          # current version being installed
```

## Standard Library

Load helpers from the vx stdlib:

```python
load("@vx//stdlib:github.star",   "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:platform.star", "is_windows", "is_macos", "platform_triple", "exe_ext")
load("@vx//stdlib:install.star",  "msi_install", "archive_install", "binary_install")
load("@vx//stdlib:env.star",      "env_prepend", "env_set", "env_append")
load("@vx//stdlib:http.star",     "fetch_json_versions")
load("@vx//stdlib:semver.star",   "semver_sort")
```

### github.star

| Function | Description |
|----------|-------------|
| `make_fetch_versions(owner, repo)` | Returns a `fetch_versions` function for GitHub releases |
| `github_asset_url(owner, repo, tag, asset)` | Build a GitHub release asset URL |
| `make_download_url(owner, repo, template)` | Returns a `download_url` function from a URL template |

### platform.star

| Function | Description |
|----------|-------------|
| `is_windows(ctx)` | `True` if running on Windows |
| `is_macos(ctx)` | `True` if running on macOS |
| `is_linux(ctx)` | `True` if running on Linux |
| `platform_triple(ctx)` | Returns Rust target triple string |
| `exe_ext(ctx)` | Returns `".exe"` on Windows, `""` elsewhere |
| `arch_to_gnu(arch)` | Converts arch to GNU triple component |

### install.star

| Function | Description |
|----------|-------------|
| `archive_install(url, strip_prefix, executable_paths)` | Archive install descriptor |
| `binary_install(url, executable_name)` | Single binary install descriptor |
| `msi_install(url, executable_paths)` | MSI install descriptor (Windows) |
| `platform_install(ctx, windows_url, macos_url, linux_url, ...)` | Per-platform install |

### env.star

| Function | Description |
|----------|-------------|
| `env_prepend(key, value)` | Prepend value to PATH-like variable |
| `env_set(key, value)` | Set environment variable |
| `env_append(key, value)` | Append value to variable |
| `env_unset(key)` | Unset environment variable |

## Provider Functions Reference

### fetch_versions(ctx)

Returns a list of available versions. Usually inherited from `make_fetch_versions`:

```python
# Simplest: fully inherited
fetch_versions = make_fetch_versions("owner", "repo")

# Custom: non-GitHub source
load("@vx//stdlib:http.star", "fetch_json_versions")

def fetch_versions(ctx):
    return fetch_json_versions(
        ctx,
        "https://go.dev/dl/?mode=json",
        lambda releases: [r["version"].lstrip("go") for r in releases],
    )
```

### download_url(ctx, version)

Returns the download URL for a given version and platform:

```python
def download_url(ctx, version):
    os   = ctx.platform.os
    arch = ctx.platform.arch
    triples = {
        "windows/x64":  "x86_64-pc-windows-msvc",
        "macos/x64":    "x86_64-apple-darwin",
        "macos/arm64":  "aarch64-apple-darwin",
        "linux/x64":    "x86_64-unknown-linux-musl",
        "linux/arm64":  "aarch64-unknown-linux-gnu",
    }
    triple = triples.get("{}/{}".format(os, arch))
    if not triple:
        return None
    ext   = "zip" if os == "windows" else "tar.gz"
    asset = "tool-{}-{}.{}".format(version, triple, ext)
    return github_asset_url("owner", "repo", "v" + version, asset)
```

### install_layout(ctx, version)

Returns an install descriptor dict:

```python
# Archive layout
def install_layout(ctx, version):
    return {
        "type":             "archive",
        "strip_prefix":     "tool-{}".format(version),
        "executable_paths": ["bin/tool.exe", "bin/tool"],
    }

# Binary layout
def install_layout(ctx, version):
    os  = ctx.platform.os
    exe = "tool.exe" if os == "windows" else "tool"
    return {
        "type":            "binary",
        "executable_name": exe,
    }
```

| Layout Type | Required Fields | Optional Fields |
|-------------|----------------|-----------------|
| `"archive"` | `type` | `strip_prefix`, `executable_paths` |
| `"binary"` | `type` | `executable_name`, `source_name`, `permissions` |
| `"msi"` | `type`, `url` | `executable_paths`, `strip_prefix` |

### environment(ctx, version)

Returns a **list** of env operations (not a dict):

```python
load("@vx//stdlib:env.star", "env_prepend", "env_set")

def environment(ctx, _version):
    return [
        env_prepend("PATH", ctx.install_dir),
        env_set("TOOL_HOME", ctx.install_dir),  # optional
    ]
```

### store_root(ctx) / get_execute_path(ctx, version) / post_install(ctx, version)

Path query functions required by all providers:

```python
def store_root(ctx):
    return ctx.vx_home + "/store/mytool"

def get_execute_path(ctx, version):
    os  = ctx.platform.os
    exe = "mytool.exe" if os == "windows" else "mytool"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None  # Return None if nothing to do
```

### system_install(ctx)

Returns package manager fallback strategies:

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

### deps(ctx, version)

Returns runtime dependencies:

```python
def deps(_ctx, _version):
    return [
        {"runtime": "node", "version": ">=18",
         "reason": "Requires Node.js runtime"},
    ]
```

## Real-World Examples

### Standard GitHub Binary Tool (ripgrep)

```python
# provider.star
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

name        = "ripgrep"
description = "ripgrep (rg) - recursively searches directories for a regex pattern"
homepage    = "https://github.com/BurntSushi/ripgrep"
repository  = "https://github.com/BurntSushi/ripgrep"
license     = "MIT OR Unlicense"
ecosystem   = "devtools"
aliases     = ["rg"]

runtimes = [
    {
        "name":        "ripgrep",
        "executable":  "rg",
        "description": "Fast regex search tool",
        "aliases":     ["rg"],
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "ripgrep \\d+"},
        ],
    },
]

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": [],
}

fetch_versions = make_fetch_versions("BurntSushi", "ripgrep")

def _triple(ctx):
    os   = ctx.platform.os
    arch = ctx.platform.arch
    return {
        "windows/x64":  "x86_64-pc-windows-msvc",
        "macos/x64":    "x86_64-apple-darwin",
        "macos/arm64":  "aarch64-apple-darwin",
        "linux/x64":    "x86_64-unknown-linux-musl",
        "linux/arm64":  "aarch64-unknown-linux-gnu",
    }.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    triple = _triple(ctx)
    if not triple:
        return None
    os    = ctx.platform.os
    ext   = "zip" if os == "windows" else "tar.gz"
    asset = "ripgrep-{}-{}.{}".format(version, triple, ext)
    return github_asset_url("BurntSushi", "ripgrep", version, asset)  # no 'v' prefix

def install_layout(ctx, version):
    triple = _triple(ctx)
    os  = ctx.platform.os
    exe = "rg.exe" if os == "windows" else "rg"
    return {
        "type":             "archive",
        "strip_prefix":     "ripgrep-{}-{}".format(version, triple) if triple else "",
        "executable_paths": [exe, "rg"],
    }

def store_root(ctx):
    return ctx.vx_home + "/store/ripgrep"

def get_execute_path(ctx, version):
    os  = ctx.platform.os
    exe = "rg.exe" if os == "windows" else "rg"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]
```

### PyPI Package Alias (meson)

For tools distributed via PyPI, use `package_alias` to route through `uvx`:

```python
# provider.star
name        = "meson"
description = "Meson - An extremely fast and user friendly build system"
homepage    = "https://mesonbuild.com"
repository  = "https://github.com/mesonbuild/meson"
license     = "Apache-2.0"
ecosystem   = "python"
aliases     = ["mesonbuild"]

# RFC 0033: route `vx meson` → `vx uvx:meson`
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
    return None  # Runs via uvx, no direct download

def deps(_ctx, _version):
    return [
        {"runtime": "uv", "version": "*",
         "reason": "Tool is installed and run via uv"},
    ]
```

### Hybrid Provider (imagemagick)

Direct download on Linux, system package manager on Windows/macOS:

```python
# provider.star
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

name        = "imagemagick"
description = "ImageMagick - image manipulation software"
homepage    = "https://imagemagick.org"
repository  = "https://github.com/ImageMagick/ImageMagick"
license     = "ImageMagick"
ecosystem   = "devtools"
aliases     = ["magick", "convert", "mogrify"]

runtimes = [
    {
        "name":        "imagemagick",
        "executable":  "magick",
        "description": "ImageMagick image manipulation",
        "aliases":     ["magick", "convert", "mogrify"],
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    },
]

permissions = {
    "http": ["api.github.com", "github.com", "imagemagick.org"],
    "fs":   [],
    "exec": [],
}

fetch_versions = make_fetch_versions("ImageMagick", "ImageMagick")

def download_url(ctx, version):
    os = ctx.platform.os
    if os == "linux":
        arch = ctx.platform.arch
        suffix = "x86_64" if arch == "x64" else "aarch64"
        asset = "ImageMagick--gcc-{}.AppImage".format(suffix)
        return github_asset_url("ImageMagick", "ImageMagick",
                                "refs/tags/" + version, asset)
    return None  # Windows/macOS use system_install

def install_layout(ctx, version):
    os = ctx.platform.os
    if os == "linux":
        return {
            "type":            "binary",
            "executable_name": "magick",
            "permissions":     "755",
        }
    return None

def system_install(ctx):
    os = ctx.platform.os
    if os == "windows":
        return {
            "strategies": [
                {"manager": "winget", "package": "ImageMagick.ImageMagick", "priority": 95},
                {"manager": "choco",  "package": "imagemagick",             "priority": 80},
            ],
        }
    elif os == "macos":
        return {
            "strategies": [
                {"manager": "brew", "package": "imagemagick", "priority": 90},
            ],
        }
    return {}

def store_root(ctx):
    return ctx.vx_home + "/store/imagemagick"

def get_execute_path(ctx, version):
    os  = ctx.platform.os
    exe = "magick.exe" if os == "windows" else "magick"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]
```

### MSI on Windows (Windows-specific tool)

```python
# provider.star
load("@vx//stdlib:install.star", "msi_install", "archive_install")
load("@vx//stdlib:github.star",  "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",     "env_prepend")

name        = "mytool"
description = "My tool with MSI installer on Windows"
license     = "MIT"
ecosystem   = "devtools"

fetch_versions = make_fetch_versions("owner", "repo")

def download_url(ctx, version):
    os = ctx.platform.os
    if os == "windows":
        return "https://github.com/owner/repo/releases/download/v{}/mytool-{}-x64.msi".format(
            version, version)
    elif os == "macos":
        return github_asset_url("owner", "repo", "v" + version,
                                "mytool-{}-macos.tar.gz".format(version))
    elif os == "linux":
        return github_asset_url("owner", "repo", "v" + version,
                                "mytool-{}-linux.tar.gz".format(version))
    return None

def install_layout(ctx, version):
    os  = ctx.platform.os
    url = download_url(ctx, version)
    if os == "windows":
        return msi_install(url, executable_paths=["bin/mytool.exe", "mytool.exe"])
    else:
        return archive_install(url,
                               strip_prefix="mytool-{}".format(version),
                               executable_paths=["bin/mytool"])

def store_root(ctx):
    return ctx.vx_home + "/store/mytool"

def get_execute_path(ctx, version):
    os  = ctx.platform.os
    exe = "mytool.exe" if os == "windows" else "mytool"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]
```

## provider.toml (Metadata Only)

After creating `provider.star`, the `provider.toml` only needs metadata — **no layout fields**:

```toml
[provider]
name        = "mytool"
description = "My awesome tool"
homepage    = "https://example.com"
repository  = "https://github.com/myorg/mytool"
ecosystem   = "devtools"
license     = "MIT"
```

No `[runtimes.layout]`, no `download_type`, no `strip_prefix` — all install logic lives in
`provider.star`.

## Provider Directory Structure

vx loads providers from multiple locations:

```
~/.vx/providers/          # User-defined providers (highest priority)
├── mytool/
│   ├── provider.star     # All logic (required)
│   └── provider.toml     # Metadata only (optional)
└── custom-node/
    ├── provider.star
    └── provider.toml

$VX_PROVIDERS_PATH/       # Environment variable path
└── team-tools/
    ├── provider.star
    └── provider.toml

Built-in providers        # Lowest priority (crates/vx-providers/*)
```

**Loading Priority:**
1. `~/.vx/providers/*/provider.star` (user local, highest)
2. `$VX_PROVIDERS_PATH/*/provider.star` (environment variable)
3. Built-in providers (lowest)

## Package Alias (npm/PyPI Tools)

For tools distributed as npm or PyPI packages, use `package_alias` to route through the
ecosystem's package runner:

| Syntax | Routes to | Installer | Requires |
|--------|-----------|-----------|---------|
| `vx meson@1.5.0` | `vx uvx:meson@1.5.0` | `UvxInstaller` | `uv` |
| `vx ruff@0.9.0` | `vx uvx:ruff@0.9.0` | `UvxInstaller` | `uv` |
| `vx vite@5.0` | `vx npx:vite@5.0` | `NpmInstaller` | `node` |

```python
# PyPI tool
package_alias = {"ecosystem": "uvx", "package": "ruff"}

# npm tool
package_alias = {"ecosystem": "npx", "package": "vite"}
```

## Best Practices

### 1. Always Declare license

```python
license = "MIT"          # SPDX identifier — REQUIRED
```

Blocked licenses (AGPL-3.0, SSPL, CC BY-NC) must NOT be integrated.

### 2. Cover All Major Platforms

```python
triples = {
    "windows/x64":  "x86_64-pc-windows-msvc",
    "macos/x64":    "x86_64-apple-darwin",
    "macos/arm64":  "aarch64-apple-darwin",
    "linux/x64":    "x86_64-unknown-linux-musl",
    "linux/arm64":  "aarch64-unknown-linux-gnu",
}
```

### 3. Add test_commands

```python
runtimes = [
    {
        "name": "mytool",
        "executable": "mytool",
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "\\d+\\.\\d+"},
        ],
    },
]
```

### 4. Use system_install for System Tools

For tools that are better installed via system package managers:

```python
def system_install(ctx):
    os = ctx.platform.os
    if os == "macos":
        return {"strategies": [{"manager": "brew", "package": "mytool", "priority": 90}]}
    elif os == "windows":
        return {"strategies": [{"manager": "winget", "package": "Org.MyTool", "priority": 95}]}
    return {}
```

### 5. Use Descriptive Names

```python
# Good
name        = "ripgrep"
description = "Fast line-oriented search tool, recursively searches directories"

# Avoid
name        = "rg"
description = "Search tool"
```

## Troubleshooting

### Provider Not Found

```bash
# Check if provider is loaded
vx list

# Verify provider.star location
ls ~/.vx/providers/mytool/provider.star
```

### Version Detection Fails

```bash
# Test manually
mytool --version

# Check fetch_versions in provider.star
grep -A5 "fetch_versions" ~/.vx/providers/mytool/provider.star
```

### Download Fails

1. Check network connectivity
2. Verify `download_url()` returns the correct URL for your platform
3. Test the URL manually: `curl -I <url>`

### "No download URL" on macOS/Windows

Add `system_install()` with package manager fallback:

```python
def system_install(ctx):
    os = ctx.platform.os
    if os == "macos":
        return {"strategies": [{"manager": "brew", "package": "mytool", "priority": 90}]}
    return {}
```

### Executable Not Found After Install

Check `install_layout()` — verify `strip_prefix` and `executable_paths` match the actual
archive structure. Download the archive manually and inspect it:

```bash
tar -tzf tool-1.0.0-linux.tar.gz | head -20
```

## Advanced Topics

For more advanced usage, see:

- **[provider.star Language & Standard Library Reference](./provider-star-reference.md)** — Complete stdlib API, ctx object, templates, coding conventions
- **[Starlark Providers - Advanced Guide](./starlark-providers.md)** — Multi-runtime providers, custom version sources, system integration, and extension patterns

## See Also

- [Provider Development Guide](../advanced/plugin-development.md) — For providers with custom Rust code
- [Configuration Reference](../config/vx-toml.md) — Project configuration
- [CLI Commands](../cli/overview.md) — Command reference
