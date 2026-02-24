# Quick Provider Migration Guide

Fast-track guide for updating providers to the latest standards.

---

## Part 1: provider.star Migration (5-Minute Checklist)

### What Needs to Change

| Old Format | New Format |
|-----------|-----------|
| `def name(): return "..."` | `name = "..."` (top-level variable) |
| `ctx["platform"]["os"]` | `ctx.platform.os` |
| `ctx["platform"]["arch"]` | `ctx.platform.arch` |
| `environment()` returns `{"PATH": dir}` | `environment()` returns `[env_prepend("PATH", dir)]` |
| `make_github_provider(...)` | `make_fetch_versions(...)` + `github_asset_url(...)` |
| Missing `store_root`, `get_execute_path`, `post_install` | Add all three |

### Step 1: Update Metadata

```python
# BEFORE
def name():        return "mytool"
def description(): return "My tool"
def ecosystem():   return "devtools"
def license():     return "MIT"

# AFTER
name        = "mytool"
description = "My tool"
ecosystem   = "devtools"
license     = "MIT"
```

### Step 2: Update ctx Access

```python
# BEFORE
os   = ctx["platform"]["os"]
arch = ctx["platform"]["arch"]

# AFTER
os   = ctx.platform.os
arch = ctx.platform.arch
```

### Step 3: Update environment()

```python
# BEFORE
def environment(ctx, version, install_dir):
    return {"PATH": install_dir}

# AFTER
load("@vx//stdlib:env.star", "env_prepend")

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]
```

### Step 4: Add Required Path Query Functions

```python
def store_root(ctx):
    return ctx.vx_home + "/store/mytool"

def get_execute_path(ctx, version):
    os = ctx.platform.os
    exe = "mytool.exe" if os == "windows" else "mytool"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None
```

### Step 5: Update runtimes to Include test_commands

```python
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

### Step 6: Verify

```bash
# Syntax check (Starlark)
vx check-star crates/vx-providers/mytool/provider.star

# Build
cargo check -p vx-provider-mytool

# Test
cargo build --release
./target/release/vx install mytool@latest
./target/release/vx mytool --version
```

---

## Part 2: provider.toml (Metadata Only)

The `provider.toml` now only contains metadata. All install logic lives in `provider.star`.

### Minimal provider.toml

```toml
[provider]
name        = "{name}"
description = "{Description}"
homepage    = "https://github.com/{owner}/{repo}"
repository  = "https://github.com/{owner}/{repo}"
ecosystem   = "devtools"
license     = "MIT"
```

No `[runtimes.layout]`, no `download_type`, no `strip_prefix` — all of that is now
handled by `install_layout()` in `provider.star`.

---

## Part 3: Common Patterns

### Pattern: GitHub Release Standard Archive

```python
# provider.star
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

name        = "mytool"
description = "My tool"
homepage    = "https://github.com/owner/repo"
repository  = "https://github.com/owner/repo"
license     = "MIT"
ecosystem   = "devtools"
aliases     = []

runtimes = [{"name": "mytool", "executable": "mytool", "description": "My tool", "priority": 100,
             "test_commands": [{"command": "{executable} --version", "name": "version_check"}]}]
permissions = {"http": ["api.github.com", "github.com"], "fs": [], "exec": []}

fetch_versions = make_fetch_versions("owner", "repo")

def _triple(ctx):
    os, arch = ctx.platform.os, ctx.platform.arch
    return {"windows/x64": "x86_64-pc-windows-msvc",
            "macos/x64":   "x86_64-apple-darwin",
            "macos/arm64": "aarch64-apple-darwin",
            "linux/x64":   "x86_64-unknown-linux-musl",
            "linux/arm64": "aarch64-unknown-linux-musl"}.get("{}/{}".format(os, arch))

def download_url(ctx, version):
    triple = _triple(ctx)
    if not triple: return None
    ext = "zip" if ctx.platform.os == "windows" else "tar.gz"
    return github_asset_url("owner", "repo", "v" + version,
                            "mytool-{}-{}.{}".format(version, triple, ext))

def install_layout(ctx, version):
    exe = "mytool.exe" if ctx.platform.os == "windows" else "mytool"
    return {"type": "archive", "strip_prefix": "mytool-{}-{}".format(version, _triple(ctx) or ""),
            "executable_paths": [exe]}

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]

def store_root(ctx):      return ctx.vx_home + "/store/mytool"
def get_execute_path(ctx, version):
    exe = "mytool.exe" if ctx.platform.os == "windows" else "mytool"
    return ctx.install_dir + "/" + exe
def post_install(_ctx, _version): return None
def deps(_ctx, _version): return []
```

### Pattern: PyPI Tool (package_alias)

```python
name          = "mytool"
description   = "My PyPI tool"
homepage      = "https://pypi.org/project/mytool/"
repository    = "https://github.com/owner/repo"
license       = "MIT"
ecosystem     = "python"
package_alias = {"ecosystem": "uvx", "package": "mytool"}

runtimes    = [{"name": "mytool", "executable": "mytool", "description": "My tool", "priority": 100}]
permissions = {"http": ["pypi.org"], "fs": [], "exec": ["uvx", "uv"]}

def download_url(_ctx, _version): return None
def store_root(ctx):              return ctx.vx_home + "/store/mytool"
def get_execute_path(_ctx, _v):   return None
def post_install(_ctx, _v):       return None
def deps(_ctx, _v):               return [{"runtime": "uv", "version": "*", "reason": "Runs via uv"}]
```

### Pattern: Hybrid (Direct Download + System PM Fallback)

```python
def download_url(ctx, version):
    if ctx.platform.os == "linux":
        return github_asset_url("owner", "repo", "v" + version,
                                "mytool-{}-linux-x64.tar.gz".format(version))
    return None  # Windows/macOS: triggers system_install

def system_install(ctx):
    os = ctx.platform.os
    if os == "windows":
        return {"strategies": [
            {"manager": "winget", "package": "Publisher.MyTool", "priority": 95},
            {"manager": "choco",  "package": "mytool",           "priority": 80},
        ]}
    elif os == "macos":
        return {"strategies": [{"manager": "brew", "package": "mytool", "priority": 90}]}
    return {}
```

---

## Troubleshooting

### "Executable not found"
1. Download archive manually and check actual structure
2. Update `strip_prefix` and `executable_paths` to match

### "Permission denied" on Unix
Add `target_permissions = "755"` to binary layout for Unix platforms.

### ctx access error
Replace all `ctx["platform"]["os"]` with `ctx.platform.os`.

### environment() type error
Replace `return {"PATH": dir}` with `return [env_prepend("PATH", dir)]`.

---

## Validation Checklist

- [ ] All metadata as top-level variables (no `def name():` functions)
- [ ] All `ctx["..."]["..."]` replaced with `ctx.platform.os` / `ctx.platform.arch`
- [ ] `environment()` returns list (not dict)
- [ ] `store_root()`, `get_execute_path()`, `post_install()` all present
- [ ] `runtimes` includes `test_commands`
- [ ] `system_install()` returns `{"strategies": [...]}` (not flat list)
- [ ] `license` field present (SPDX identifier)
- [ ] `cargo check -p vx-provider-{name}` passes
- [ ] `vx install {name}@latest` works
- [ ] `vx {name} --version` works
