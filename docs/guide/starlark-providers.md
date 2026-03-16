# Starlark Providers - Advanced Guide

This guide covers advanced features of Starlark providers, including multi-runtime providers, custom version sources, system integration, and extension patterns.

## Overview

Starlark providers offer powerful capabilities beyond basic tool installation:

| Feature | Description |
|---------|-------------|
| **Multi-Runtime Providers** | One provider managing multiple tools |
| **Custom Version Sources** | Fetch versions from any API |
| **System Integration** | Detect and use system-installed tools |
| **Post-Install Hooks** | Run custom setup after installation |
| **Dynamic Dependencies** | Version-aware dependency resolution |
| **Shell Integrations** | Define shell-specific behaviors |

## Multi-Runtime Providers

### What is a Multi-Runtime Provider?

A multi-runtime provider manages multiple related tools under a single provider. For example, `shell-tools` provider includes `starship`, `atuin`, `yazi`, and other shell utilities.

### Structure

```python
# provider.star - Multi-runtime provider example
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

name        = "shell-tools"
description = "Collection of modern shell tools"
homepage    = "https://github.com/vx-dev/shell-tools"
license     = "MIT"
ecosystem   = "devtools"

# Define multiple runtimes
runtimes = [
    {
        "name":        "starship",
        "executable":  "starship",
        "description": "Cross-shell prompt",
        "aliases":     [],
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    },
    {
        "name":        "atuin",
        "executable":  "atuin",
        "description": "Magical shell history",
        "aliases":     [],
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    },
    {
        "name":        "yazi",
        "executable":  "yazi",
        "description": "Blazing fast terminal file manager",
        "aliases":     ["ya"],  # ya is the CLI helper
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

# Repos for each runtime
_RUNTIME_REPOS = {
    "starship": ("starship", "starship"),
    "atuin":    ("atuinsh",  "atuin"),
    "yazi":     ("sxyazi",   "yazi"),
}
```

### Dispatch by Runtime Name

The key to multi-runtime providers is dispatching based on `ctx.runtime_name`:

```python
def fetch_versions(ctx):
    """Fetch versions for the specific runtime."""
    runtime = ctx.runtime_name  # "starship", "atuin", or "yazi"
    owner, repo = _RUNTIME_REPOS.get(runtime, (None, None))
    if not owner:
        return []

    # Use the stdlib helper
    return make_fetch_versions(owner, repo)(ctx)

def download_url(ctx, version):
    """Get download URL for the specific runtime."""
    runtime = ctx.runtime_name
    owner, repo = _RUNTIME_REPOS.get(runtime, (None, None))
    if not owner:
        return None

    os, arch = ctx.platform.os, ctx.platform.arch

    # Build platform-specific asset name
    if runtime == "starship":
        triples = {
            "windows/x64":  "x86_64-pc-windows-msvc",
            "macos/x64":    "x86_64-apple-darwin",
            "macos/arm64":  "aarch64-apple-darwin",
            "linux/x64":    "x86_64-unknown-linux-gnu",
            "linux/arm64":  "aarch64-unknown-linux-gnu",
        }
    elif runtime == "atuin":
        triples = {
            "windows/x64":  "x86_64-pc-windows-msvc",
            "macos/x64":    "x86_64-apple-darwin",
            "macos/arm64":  "aarch64-apple-darwin",
            "linux/x64":    "x86_64-unknown-linux-gnu",
            "linux/arm64":  "aarch64-unknown-linux-gnu",
        }
    elif runtime == "yazi":
        triples = {
            "windows/x64":  "x86_64-pc-windows-msvc",
            "macos/x64":    "x86_64-apple-darwin",
            "macos/arm64":  "aarch64-apple-darwin",
            "linux/x64":    "x86_64-unknown-linux-gnu",
            "linux/arm64":  "aarch64-unknown-linux-gnu",
        }
    else:
        return None

    triple = triples.get(f"{os}/{arch}")
    if not triple:
        return None

    ext = "zip" if os == "windows" else "tar.gz"
    asset = f"{runtime}-{version}-{triple}.{ext}"

    return github_asset_url(owner, repo, f"v{version}", asset)

def install_layout(ctx, version):
    """Install layout for the specific runtime."""
    runtime = ctx.runtime_name
    os = ctx.platform.os
    exe = f"{runtime}.exe" if os == "windows" else runtime

    return {
        "type":             "archive",
        "strip_prefix":     f"{runtime}-{version}",
        "executable_paths": [exe, runtime],
    }

def store_root(ctx):
    """Each runtime gets its own store directory."""
    runtime = ctx.runtime_name
    return f"{ctx.vx_home}/store/{runtime}"

def get_execute_path(ctx, version):
    """Path to the executable."""
    os = ctx.platform.os
    runtime = ctx.runtime_name
    exe = f"{runtime}.exe" if os == "windows" else runtime
    return f"{ctx.install_dir}/{exe}"

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]
```

### Using Multi-Runtime Providers

```bash
# Each runtime is accessible directly
vx starship --version
vx atuin --version
vx yazi --version

# List all runtimes in a provider
vx list shell-tools

# Install specific runtime
vx install starship@1.20.0
```

## Custom Version Sources

### Non-GitHub Version Sources

For tools not hosted on GitHub, implement custom `fetch_versions`:

```python
load("@vx//stdlib:http.star", "http_get")

def fetch_versions(ctx):
    """Fetch versions from a custom API."""
    # Example: Go's official API
    url = "https://go.dev/dl/?mode=json"
    response = http_get(ctx, url)

    versions = []
    for release in response:
        version = release.get("version", "").lstrip("go")
        if version:
            versions.append({
                "version": version,
                "stable":  release.get("stable", True),
                "date":    release.get("published", ""),
                "lts":     False,
            })

    return versions
```

### PyPI Version Source

```python
load("@vx//stdlib:http.star", "http_get")

def fetch_versions(ctx):
    """Fetch versions from PyPI."""
    package = "ruff"  # Your package name
    url = f"https://pypi.org/pypi/{package}/json"

    response = http_get(ctx, url)
    releases = response.get("releases", {})

    versions = []
    for version, files in releases.items():
        # Skip pre-releases
        is_prerelease = any(c.isdigit() and i > 0 and version[i-1] in 'abrc'
                           for i, c in enumerate(version))

        versions.append({
            "version": version,
            "stable":  not is_prerelease,
            "date":    files[0].get("upload_time", "") if files else "",
            "lts":     False,
        })

    # Sort by semver (newest first)
    return sorted(versions, key=lambda v: v["version"], reverse=True)
```

### Node.js Official API

```python
load("@vx//stdlib:http.star", "http_get")

def fetch_versions(ctx):
    """Fetch Node.js versions from official API."""
    url = "https://nodejs.org/dist/index.json"
    response = http_get(ctx, url)

    versions = []
    for release in response:
        version = release.get("version", "").lstrip("v")
        versions.append({
            "version": version,
            "stable":  release.get("lts", False) is False,
            "date":    release.get("date", ""),
            "lts":     release.get("lts", False) is not False,
        })

    return versions
```

## System Integration

### Detecting System-Installed Tools

Use `system_install` to define fallback strategies:

```python
def system_install(ctx):
    """Define how to install via system package managers."""
    os = ctx.platform.os

    if os == "windows":
        return {
            "strategies": [
                {"manager": "winget", "package": "Microsoft.MyTool", "priority": 95},
                {"manager": "choco",  "package": "mytool",            "priority": 80},
                {"manager": "scoop",  "package": "mytool",            "priority": 70},
            ],
        }
    elif os == "macos":
        return {
            "strategies": [
                {"manager": "brew", "package": "mytool", "priority": 90},
            ],
        }
    elif os == "linux":
        return {
            "strategies": [
                {"manager": "apt",  "package": "mytool", "priority": 85},
                {"manager": "dnf",  "package": "mytool", "priority": 80},
                {"manager": "snap", "package": "mytool", "priority": 75},
            ],
        }

    return {}
```

### Platform Constraints

Restrict tools to specific platforms:

```python
# Provider-level platform constraint
platforms = {
    "os": ["windows", "linux"]  # Not available on macOS
}

# Or in runtimes list
runtimes = [
    {
        "name":        "mytool",
        "executable":  "mytool",
        "platform_os": ["windows"],  # Only on Windows
        # ...
    },
]
```

### System Path Detection

Define where to look for system-installed tools:

```python
runtimes = [
    {
        "name":        "mytool",
        "executable":  "mytool",
        "system_paths": [
            "/usr/local/bin/mytool",
            "/usr/bin/mytool",
            "C:\\Program Files\\MyTool\\mytool.exe",
        ],
        "env_hints": ["MYTOOL_HOME"],  # Check MYTOOL_HOME/bin
        # ...
    },
]
```

## Post-Install Hooks

### Running Setup Commands

```python
def post_install(ctx, version):
    """Run setup after installation."""
    return {
        "commands": [
            {
                "executable": f"{ctx.install_dir}/mytool",
                "args":       ["init", "--install"],
                "cwd":        ctx.install_dir,
            },
        ],
    }
```

### Creating Symlinks

```python
def post_install(ctx, version):
    """Create symlinks after installation."""
    os = ctx.platform.os

    if os != "windows":  # Symlinks need admin on Windows
        return {
            "symlinks": [
                {
                    "source": f"{ctx.install_dir}/bin/tool",
                    "target": f"{ctx.vx_home}/bin/tool",  # Global shim
                },
            ],
        }

    return None
```

### Setting Permissions

```python
def post_install(ctx, version):
    """Set executable permissions on Unix."""
    os = ctx.platform.os

    if os != "windows":
        return {
            "permissions": [
                {
                    "path": f"{ctx.install_dir}/mytool",
                    "mode": "755",
                },
            ],
        }

    return None
```

## Dynamic Dependencies

### Version-Aware Dependencies

```python
def deps(ctx, version):
    """Return dependencies based on version."""
    import re

    # Parse major version
    match = re.match(r"(\d+)", version)
    major = int(match.group(1)) if match else 0

    if major >= 20:
        return [
            {"runtime": "node", "version": ">=20", "reason": "Requires Node.js 20+ API"},
        ]
    elif major >= 18:
        return [
            {"runtime": "node", "version": ">=18", "reason": "Requires Node.js 18+ API"},
        ]
    else:
        return [
            {"runtime": "node", "version": ">=16", "reason": "Minimum Node.js 16"},
        ]
```

### Optional Dependencies

```python
def deps(ctx, version):
    """Required and recommended dependencies."""
    return [
        # Required dependency
        {"runtime": "node", "version": ">=18", "reason": "Runtime dependency", "optional": False},

        # Optional but recommended
        {"runtime": "npm", "version": "*", "reason": "For package management", "optional": True},
        {"runtime": "yarn", "version": "*", "reason": "Alternative package manager", "optional": True},
    ]
```

## Shell Integrations

### Defining Shell Behaviors

For tools that integrate with shells (like `starship`, `atuin`):

```python
runtimes = [
    {
        "name":        "myshell",
        "executable":  "myshell",
        "description": "Shell integration tool",
        "shells": {
            "bash":     "~/.bashrc",
            "zsh":      "~/.zshrc",
            "fish":     "~/.config/fish/config.fish",
            "powershell": "$PROFILE",
        },
        # ...
    },
]
```

### Shell Init Script

```python
def shell_init(ctx, shell):
    """Generate shell initialization script."""
    if shell == "bash":
        return 'eval "$(myshell init bash)"'
    elif shell == "zsh":
        return 'eval "$(myshell init zsh)"'
    elif shell == "fish":
        return 'myshell init fish | source'
    elif shell == "powershell":
        return 'Invoke-Expression (&myshell init powershell)'

    return None
```

## Install Layout Types

### Archive (tar.gz, zip)

```python
def install_layout(ctx, version):
    return {
        "type":             "archive",
        "url":              "https://example.com/tool.tar.gz",  # Optional if download_url is set
        "strip_prefix":     "tool-v1.0.0",  # Remove this prefix from paths
        "executable_paths": ["bin/tool", "tool"],  # Possible executable locations
    }
```

### Single Binary

```python
def install_layout(ctx, version):
    os = ctx.platform.os
    exe = "tool.exe" if os == "windows" else "tool"

    return {
        "type":            "binary",
        "url":             "https://example.com/tool-binary",  # Direct binary download
        "executable_name": exe,  # Name for the downloaded binary
        "permissions":     "755",  # Unix permissions
    }
```

### MSI (Windows Only)

```python
def install_layout(ctx, version):
    return {
        "type":             "msi",
        "url":              "https://example.com/installer.msi",
        "executable_paths": ["bin/tool.exe", "tool.exe"],
        "strip_prefix":     "Tool",  # MSI product name prefix
        "extra_args":       ["/quiet", "/norestart"],  # Additional MSI args
    }
```

### System Find (Detect System Installation)

```python
def install_layout(ctx, version):
    return {
        "type":         "system_find",
        "executable":   "tool",  # Executable name to find
        "system_paths": [
            "/usr/local/bin/tool",
            "C:\\Program Files\\Tool\\tool.exe",
        ],
        "hint":         "Install via 'winget install Tool' or 'brew install tool'",
    }
```

## Testing Provider Scripts

### Using vx test

```bash
# Test a provider script
vx test provider.star

# Test with verbose output
vx test provider.star --verbose

# Test specific functions
vx test provider.star --function fetch_versions
```

### Linting Provider Scripts

```bash
# Check for issues
vx lint provider.star

# Auto-fix issues
vx lint provider.star --fix
```

## Best Practices

### 1. Use Standard Library Functions

Always prefer stdlib helpers over custom implementations:

```python
# Good: Use stdlib
load("@vx//stdlib:github.star", "make_fetch_versions")
fetch_versions = make_fetch_versions("owner", "repo")

# Avoid: Custom implementation
def fetch_versions(ctx):
    # ... 50 lines of HTTP parsing code
```

### 2. Handle Platform Differences Gracefully

```python
def download_url(ctx, version):
    os, arch = ctx.platform.os, ctx.platform.arch

    # Provide fallbacks
    key = f"{os}/{arch}"
    triple = PLATFORM_TRIPLES.get(key)

    if not triple:
        # Return None to indicate unsupported platform
        return None

    # ... build URL
```

### 3. Use Meaningful Error Messages

```python
def download_url(ctx, version):
    triple = get_triple(ctx)
    if not triple:
        # Log why this platform isn't supported
        return None

    if not version:
        fail("version is required for download_url")

    # ... rest of implementation
```

### 4. Keep Metadata Accurate

```python
name        = "mytool"           # Must match directory name
description = "My awesome tool"  # Clear, concise description
homepage    = "https://..."      # Project website
repository  = "https://..."      # Source code location
license     = "MIT"              # SPDX identifier
ecosystem   = "devtools"         # Category for grouping
```

## Debugging

### Enable Debug Logging

```bash
# Enable verbose output
vx --verbose mytool --version

# See Starlark execution
VX_DEBUG=starlark vx mytool --version
```

### Test Specific Functions

```python
# Add to provider.star for testing
def _test():
    """Self-test function for debugging."""
    ctx = {
        "platform": {"os": "linux", "arch": "x64"},
        "version":  "1.0.0",
    }

    print("Testing fetch_versions...")
    versions = fetch_versions(ctx)
    print(f"Found {len(versions)} versions")

    print("Testing download_url...")
    url = download_url(ctx, "1.0.0")
    print(f"Download URL: {url}")
```

## See Also

- [Manifest-Driven Providers](./manifest-driven-providers.md) — Getting-started guide for provider creation
- [provider.star Language & Standard Library Reference](./provider-star-reference.md) — Complete stdlib API, ctx object, templates, coding conventions
- [vx.toml Syntax Guide](./vx-toml-syntax.md) — Project configuration patterns
