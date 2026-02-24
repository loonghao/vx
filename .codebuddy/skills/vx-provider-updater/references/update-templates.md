# Provider Update Templates

Complete templates for updating providers to the latest standards.

---

## Part 1: provider.star Update Templates

Use these templates when migrating from old provider.star format to the current standard.

### Migration: Old → New Format

#### Metadata: Functions → Top-Level Variables

```python
# OLD (forbidden)
def name():        return "mytool"
def description(): return "My tool"
def ecosystem():   return "devtools"
def license():     return "MIT"
def homepage():    return "https://example.com"
def repository():  return "https://github.com/owner/repo"
def aliases():     return []

# NEW (required)
name        = "mytool"
description = "My tool"
ecosystem   = "devtools"
license     = "MIT"
homepage    = "https://example.com"
repository  = "https://github.com/owner/repo"
aliases     = []
```

#### ctx Access: Dict → Object Style

```python
# OLD (forbidden)
os   = ctx["platform"]["os"]
arch = ctx["platform"]["arch"]

# NEW (required)
os   = ctx.platform.os
arch = ctx.platform.arch
```

#### environment(): Dict → List

```python
# OLD (forbidden)
def environment(ctx, version, install_dir):
    return {"PATH": install_dir}

# NEW (required)
load("@vx//stdlib:env.star", "env_prepend")

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]
```

#### Add Required Path Query Functions

```python
# NEW (required — add these if missing)
def store_root(ctx):
    return ctx.vx_home + "/store/mytool"

def get_execute_path(ctx, version):
    os = ctx.platform.os
    exe = "mytool.exe" if os == "windows" else "mytool"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None
```

#### Add test_commands to runtimes

```python
# OLD
runtimes = [
    {"name": "mytool", "executable": "mytool", "description": "My tool", "priority": 100},
]

# NEW
runtimes = [
    {
        "name":        "mytool",
        "executable":  "mytool",
        "description": "My tool",
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check", "expected_output": "\\d+"},
        ],
    },
]
```

#### Fix system_install Format

```python
# OLD (forbidden — flat list)
"system_install": [
    {"manager": "brew", "package": "mytool"},
]

# NEW (required — nested strategies)
def system_install(ctx):
    os = ctx.platform.os
    if os == "windows":
        return {"strategies": [
            {"manager": "winget", "package": "Publisher.MyTool", "priority": 95},
            {"manager": "choco",  "package": "mytool",           "priority": 80},
        ]}
    elif os == "macos":
        return {"strategies": [
            {"manager": "brew", "package": "mytool", "priority": 90},
        ]}
    return {}
```

### Complete Updated provider.star (Standard GitHub Tool)

```python
# provider.star - mytool provider (UPDATED to current standard)
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# ---------------------------------------------------------------------------
# Provider metadata (top-level variables)
# ---------------------------------------------------------------------------
name        = "mytool"
description = "My awesome tool"
homepage    = "https://github.com/owner/repo"
repository  = "https://github.com/owner/repo"
license     = "MIT"
ecosystem   = "devtools"
aliases     = []

# ---------------------------------------------------------------------------
# Runtime definitions
# ---------------------------------------------------------------------------
runtimes = [
    {
        "name":        "mytool",
        "executable":  "mytool",
        "description": "My awesome tool",
        "aliases":     [],
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
# fetch_versions
# ---------------------------------------------------------------------------
fetch_versions = make_fetch_versions("owner", "repo")

# ---------------------------------------------------------------------------
# download_url
# ---------------------------------------------------------------------------
def _mytool_triple(ctx):
    os   = ctx.platform.os
    arch = ctx.platform.arch
    return {
        "windows/x64":  "x86_64-pc-windows-msvc",
        "macos/x64":    "x86_64-apple-darwin",
        "macos/arm64":  "aarch64-apple-darwin",
        "linux/x64":    "x86_64-unknown-linux-musl",
        "linux/arm64":  "aarch64-unknown-linux-musl",
    }.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    triple = _mytool_triple(ctx)
    if not triple:
        return None
    os  = ctx.platform.os
    ext = "zip" if os == "windows" else "tar.gz"
    asset = "mytool-{}-{}.{}".format(version, triple, ext)
    return github_asset_url("owner", "repo", "v" + version, asset)

# ---------------------------------------------------------------------------
# install_layout
# ---------------------------------------------------------------------------
def install_layout(ctx, version):
    triple = _mytool_triple(ctx)
    os  = ctx.platform.os
    exe = "mytool.exe" if os == "windows" else "mytool"
    return {
        "type":             "archive",
        "strip_prefix":     "mytool-{}-{}".format(version, triple) if triple else "",
        "executable_paths": [exe, "mytool"],
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
    return ctx.vx_home + "/store/mytool"

def get_execute_path(ctx, version):
    os = ctx.platform.os
    exe = "mytool.exe" if os == "windows" else "mytool"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

# ---------------------------------------------------------------------------
# deps
# ---------------------------------------------------------------------------
def deps(_ctx, _version):
    return []
```

### Complete Updated provider.star (Hybrid: Direct + System PM)

```python
# provider.star - mytool provider (hybrid: direct download + system PM fallback)
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

name        = "mytool"
description = "My tool"
homepage    = "https://example.com"
repository  = "https://github.com/owner/repo"
license     = "MIT"
ecosystem   = "system"
aliases     = []

runtimes = [
    {
        "name":        "mytool",
        "executable":  "mytool",
        "description": "My tool",
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    },
]

permissions = {"http": ["api.github.com", "github.com"], "fs": [], "exec": []}

fetch_versions = make_fetch_versions("owner", "repo")

def download_url(ctx, version):
    os = ctx.platform.os
    if os == "linux":
        return github_asset_url("owner", "repo", "v" + version,
                                "mytool-{}-linux-x64.tar.gz".format(version))
    return None  # Windows/macOS: use system_install

def install_layout(_ctx, _version):
    return {"type": "archive", "strip_prefix": "", "executable_paths": ["mytool"]}

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def system_install(ctx):
    os = ctx.platform.os
    if os == "windows":
        return {"strategies": [
            {"manager": "winget", "package": "Publisher.MyTool", "priority": 95},
            {"manager": "choco",  "package": "mytool",           "priority": 80},
        ]}
    elif os == "macos":
        return {"strategies": [
            {"manager": "brew", "package": "mytool", "priority": 90},
        ]}
    return {}

def store_root(ctx):      return ctx.vx_home + "/store/mytool"
def get_execute_path(ctx, version):
    exe = "mytool.exe" if ctx.platform.os == "windows" else "mytool"
    return ctx.install_dir + "/" + exe
def post_install(_ctx, _version): return None
def deps(_ctx, _version): return []
```

---

> **Note**: All install logic (download URLs, archive layout, system_install, environment)
> now lives in `provider.star`. The `provider.toml` only contains metadata.
> See Part 1 above for complete provider.star migration templates.
