# provider.star — Standard Library Reference

> ← [Back to Overview](./provider-star-reference.md)

This document covers all 14 standard library modules available in `provider.star`.

All modules are loaded via:

```python
load("@vx//stdlib:<module>.star", "function1", "function2")
```

---

## Table of Contents

- [6.1 provider.star — Unified Entry Point](#61-providerstar--unified-entry-point)
- [6.2 runtime.star — Runtime Definitions](#62-runtimestar--runtime-definitions)
- [6.3 env.star — Environment Variables](#63-envstar--environment-variables)
- [6.4 platform.star — Platform Detection](#64-platformstar--platform-detection)
- [6.5 http.star — HTTP Descriptors](#65-httpstar--http-descriptors)
- [6.6 github.star — GitHub Helpers](#66-githubstar--github-helpers)
- [6.7 install.star — Install Descriptors](#67-installstar--install-descriptors)
- [6.8 layout.star — Layout, Hooks & Path Factories](#68-layoutstar--layout-hooks--path-factories)
- [6.9 permissions.star — Permission Declarations](#69-permissionsstar--permission-declarations)
- [6.10 system_install.star — Package Manager Strategies](#610-system_installstar--package-manager-strategies)
- [6.11 script_install.star — Script-Based Installation](#611-script_installstar--script-based-installation)
- [6.12 semver.star — Version Comparison](#612-semverstar--version-comparison)
- [6.13 test.star — Testing DSL](#613-teststar--testing-dsl)
- [6.14 provider_templates.star — High-Level Templates](#614-provider_templatesstar--high-level-templates)

---

### 6.1 `provider.star` — Unified Entry Point

The `provider.star` module is a **re-export facade** that aggregates all public APIs from sub-modules. You can import everything from here:

```python
load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def", "dep_def",
     "github_permissions", "system_permissions",
     "env_set", "env_prepend", "env_append", "env_unset",
     "platform_map", "platform_select", "rust_triple",
     "archive_layout", "binary_layout", "bin_subdir_layout",
     "bin_subdir_env", "bin_subdir_execute_path", "path_fns",
     "post_extract_flatten", "post_extract_shim",
     "post_extract_permissions", "post_extract_combine",
     "pre_run_ensure_deps",
     "fetch_versions_from_api", "fetch_versions_with_tag_prefix",
     "winget_install", "brew_install", "apt_install",
     "cross_platform_install",
     "github_rust_provider", "github_go_provider",
     "github_binary_provider", "system_provider")
```

Or import from specific sub-modules for clarity.

---

### 6.2 `runtime.star` — Runtime Definitions

#### `runtime_def(name, **kwargs) → dict`

Defines an independent runtime.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `name` | `string` | — | Runtime name (required) |
| `executable` | `string` | `name` | Executable filename |
| `description` | `string` | `""` | Human-readable description |
| `aliases` | `list[string]` | `[]` | Alternative names |
| `priority` | `int` | `100` | Resolution priority (higher wins) |
| `version_cmd` | `string` | `None` | Custom version command |
| `version_pattern` | `string` | `None` | Regex to match version output |
| `test_commands` | `list[dict]` | `[]` | Validation commands |
| `auto_installable` | `bool` | `True` | Whether vx can auto-install |
| `platform_constraint` | `dict` | `None` | Platform restrictions |
| `system_paths` | `list[string]` | `[]` | Known system install locations |
| `bundled_with` | `string` | `None` | (Use `bundled_runtime_def` instead) |

```python
runtimes = [
    runtime_def("node",
        aliases         = ["nodejs"],
        version_pattern = "v\\d+\\.\\d+\\.\\d+",
        test_commands   = [
            {"command": "{executable} --version", "name": "version_check",
             "expected_output": "v\\d+\\.\\d+"},
        ],
    ),
]
```

#### `bundled_runtime_def(name, bundled_with, **kwargs) → dict`

Defines a runtime shipped inside another runtime (e.g. `npm` inside `node`).

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `name` | `string` | — | Runtime name |
| `bundled_with` | `string` | — | Parent runtime name |
| `executable` | `string` | `name` | Executable filename |
| `description` | `string` | `""` | Description |
| `aliases` | `list[string]` | `[]` | Alternative names |
| `command_prefix` | `list[string]` | `None` | Args prepended when invoked (e.g. `["x"]` makes `bunx foo` → `bun x foo`) |
| `test_commands` | `list[dict]` | `[]` | Validation commands |
| `version_pattern` | `string` | `None` | Version output regex |
| `auto_installable` | `bool` | `True` | Auto-install capability |
| `platform_constraint` | `dict` | `None` | Platform restrictions |

```python
runtimes = [
    runtime_def("node"),
    bundled_runtime_def("npm", "node",
        description = "Node Package Manager"),
    bundled_runtime_def("npx", "node",
        description = "Node Package eXecute"),
]
```

#### `dep_def(runtime, **kwargs) → dict`

Declares a runtime dependency.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `runtime` | `string` | — | Required runtime name |
| `version` | `string` | `"*"` | Version constraint (e.g. `">=18"`) |
| `optional` | `bool` | `False` | Whether the dependency is optional |
| `reason` | `string` | `None` | Human-readable reason |

```python
def deps(_ctx, _version):
    return [
        dep_def("git", optional=True,
                reason="Git is required for fetching modules"),
    ]
```

---

### 6.3 `env.star` — Environment Variables

| Function | Signature | Description |
|----------|-----------|-------------|
| `env_set(key, value)` | `→ dict` | Set environment variable |
| `env_prepend(key, value, sep=None)` | `→ dict` | Prepend to PATH-like variable |
| `env_append(key, value, sep=None)` | `→ dict` | Append to PATH-like variable |
| `env_unset(key)` | `→ dict` | Remove environment variable |

**Return format:**

```python
env_set("GOROOT", "/path")
# → {"op": "set", "key": "GOROOT", "value": "/path"}

env_prepend("PATH", "/usr/local/go/bin")
# → {"op": "prepend", "key": "PATH", "value": "/usr/local/go/bin"}
```

**Usage in `environment()`:**

```python
def environment(ctx, _version):
    return [
        env_set("GOROOT", ctx.install_dir),
        env_prepend("PATH", ctx.install_dir + "/bin"),
        env_set("GO111MODULE", "on"),
    ]
```

---

### 6.4 `platform.star` — Platform Detection

#### Boolean Checks

| Function | Description |
|----------|-------------|
| `is_windows(ctx)` | Returns `True` on Windows |
| `is_macos(ctx)` | Returns `True` on macOS |
| `is_linux(ctx)` | Returns `True` on Linux |
| `is_x64(ctx)` | Returns `True` on x64/amd64 |
| `is_arm64(ctx)` | Returns `True` on arm64/aarch64 |

#### Triple & Architecture

| Function | Signature | Description |
|----------|-----------|-------------|
| `platform_triple(ctx)` | `→ string` | Returns `ctx.platform.target` |
| `rust_triple(ctx, linux_libc="musl")` | `→ string \| None` | Full Rust target triple |
| `go_os_arch(ctx)` | `→ (string, string)` | Go-style `(os, arch)` tuple |
| `arch_to_gnu(arch)` | `→ string` | `"x64"` → `"x86_64"`, `"arm64"` → `"aarch64"` |
| `arch_to_go(arch)` | `→ string` | `"x64"` → `"amd64"`, `"arm64"` → `"arm64"` |
| `os_to_go(os)` | `→ string` | `"macos"` → `"darwin"` |

#### Extension Helpers

| Function | Signature | Description |
|----------|-----------|-------------|
| `platform_ext(ctx)` | `→ string` | `".zip"` on Windows, `".tar.gz"` elsewhere |
| `archive_ext(ctx)` | `→ string` | `"zip"` on Windows, `"tar.gz"` elsewhere (no dot) |
| `exe_ext(ctx)` | `→ string` | `".exe"` on Windows, `""` elsewhere |
| `exe_suffix(ctx)` | `→ string` | Same as `exe_ext()` |

#### Platform Dispatch

| Function | Signature | Description |
|----------|-----------|-------------|
| `platform_map(ctx, mapping, fallback=None)` | `→ any` | Look up `"{os}/{arch}"` key in mapping dict |
| `platform_select(ctx, windows, macos, linux, fallback=None)` | `→ any` | Choose value by OS |

```python
# platform_map — dispatch by OS/arch combination
_PLATFORMS = {
    "windows/x64":  "x86_64-pc-windows-msvc",
    "macos/arm64":  "aarch64-apple-darwin",
    "linux/x64":    "x86_64-unknown-linux-musl",
}
triple = platform_map(ctx, _PLATFORMS)  # returns None if unsupported

# platform_select — dispatch by OS only
bin_dir = platform_select(ctx,
    windows = ctx.install_dir,
    macos   = ctx.install_dir + "/bin",
    linux   = ctx.install_dir + "/bin",
)
```

#### Asset Template Expansion

| Function | Signature | Description |
|----------|-----------|-------------|
| `expand_asset(template, ctx, version, ...)` | `→ string` | Replace `{version}`, `{vversion}`, `{triple}`, `{os}`, `{arch}`, `{ext}`, `{exe}` |

```python
url = expand_asset(
    "mytool-{vversion}-{triple}.{ext}",
    ctx, "1.0.0",
)
# → "mytool-v1.0.0-x86_64-unknown-linux-musl.tar.gz"
```

#### Constants

| Constant | Description |
|----------|-------------|
| `RUST_TRIPLES_MUSL` | Dict mapping `"{os}/{arch}"` → musl-linked Rust triple |
| `RUST_TRIPLES_GNU` | Dict mapping `"{os}/{arch}"` → GNU-linked Rust triple |

---

### 6.5 `http.star` — HTTP Descriptors

> **Important:** These functions return **descriptor dicts**, not actual HTTP responses. The Rust runtime interprets and executes them.

| Function | Signature | Description |
|----------|-----------|-------------|
| `github_releases(ctx, owner, repo, include_prereleases=False)` | `→ descriptor` | GitHub releases descriptor |
| `github_latest_release(ctx, owner, repo)` | `→ descriptor` | Latest release descriptor |
| `github_download_url(owner, repo, tag, asset_name)` | `→ string` | Build GitHub asset download URL |
| `parse_github_tag(tag)` | `→ string` | Strip `v`/`release-`/`version-` prefix from tag |
| `fetch_json(ctx, url)` | `→ descriptor` | Generic JSON fetch descriptor |
| `fetch_json_versions(ctx, url, transform, headers={})` | `→ descriptor` | Version fetch with transform strategy |
| `releases_to_versions(releases, tag_key="tag_name")` | `→ list \| descriptor` | Convert releases array to version info |

#### Transform Strategies for `fetch_json_versions`

| Strategy | API Source |
|----------|-----------|
| `"nodejs_org"` | `https://nodejs.org/dist/index.json` |
| `"go_versions"` | `https://go.dev/dl/?mode=json&include=all` |
| `"adoptium"` | Eclipse Adoptium API |
| `"pypi"` | PyPI JSON API |
| `"npm_registry"` | npm registry |
| `"hashicorp_releases"` | HashiCorp releases API |
| `"github_tags"` | GitHub tags API |

```python
# Node.js — official API
fetch_versions = fetch_versions_from_api(
    "https://nodejs.org/dist/index.json",
    "nodejs_org",
)

# Go — official API
fetch_versions = fetch_versions_from_api(
    "https://go.dev/dl/?mode=json&include=all",
    "go_versions",
)
```

---

### 6.6 `github.star` — GitHub Helpers

| Function | Signature | Description |
|----------|-----------|-------------|
| `github_asset_url(owner, repo, tag, asset_name)` | `→ string` | Build asset download URL |
| `make_fetch_versions(owner, repo, include_prereleases=False)` | `→ function` | Returns bound `fetch_versions(ctx)` |
| `make_download_url(owner, repo, asset_template)` | `→ function` | Returns bound `download_url(ctx, version)` |
| `make_github_provider(owner, repo, asset_template=None, include_prereleases=False)` | `→ dict` | Complete provider namespace |

```python
# Simple pattern — bind fetch_versions to a repo
fetch_versions = make_fetch_versions("BurntSushi", "ripgrep")

# Asset URL construction
url = github_asset_url("BurntSushi", "ripgrep", "14.1.1",
                       "ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz")
# → "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-..."
```

**`make_download_url` template placeholders:**

| Placeholder | Expansion |
|-------------|-----------|
| `{version}` | Version string (e.g. `1.0.0`) |
| `{vversion}` | `v`-prefixed version (e.g. `v1.0.0`) |
| `{triple}` | Rust target triple |
| `{os}` | Go-style OS (`linux`, `darwin`, `windows`) |
| `{arch}` | Go-style arch (`amd64`, `arm64`) |
| `{ext}` | Archive extension (`zip` on Windows, `tar.gz` elsewhere) |
| `{exe}` | Executable suffix (`.exe` on Windows, empty elsewhere) |

---

### 6.7 `install.star` — Install Descriptors

| Function | Signature | Description |
|----------|-----------|-------------|
| `archive_install(url, strip_prefix, executable_paths)` | `→ descriptor` | Archive (tar.gz/zip) install |
| `binary_install(url, executable_name, permissions="755")` | `→ descriptor` | Single binary download |
| `msi_install(url, executable_paths, strip_prefix, extra_args)` | `→ descriptor` | MSI installer (Windows) |
| `platform_install(ctx, windows_url, macos_url, linux_url, ...)` | `→ descriptor` | Per-platform URL selection |
| `system_find(executable, system_paths, hint)` | `→ descriptor` | Find system-installed tool |
| `create_shim(name, target_executable, args, shim_dir)` | `→ descriptor` | Create shim script |
| `set_permissions(path, mode="755")` | `→ descriptor` | Set file permissions |
| `ensure_dependencies(package_manager, check_file, lock_file, install_dir)` | `→ descriptor` | Ensure package deps |
| `run_command(executable, args, working_dir, env, on_failure="warn")` | `→ descriptor` | Run arbitrary command |
| `flatten_dir(pattern, keep_subdirs)` | `→ descriptor` | Flatten directory structure |

---

### 6.8 `layout.star` — Layout, Hooks & Path Factories

#### Layout Builders

These return **functions** (not dicts) that can be assigned directly to `install_layout`.

| Function | Signature | Description |
|----------|-----------|-------------|
| `archive_layout(executable, strip_prefix=None)` | `→ fn(ctx, version) → dict` | Archive install layout |
| `binary_layout(executable)` | `→ fn(ctx, version) → dict` | Single binary layout |
| `bin_subdir_layout(executables, strip_prefix=None)` | `→ fn(ctx, version) → dict` | `bin/` subdirectory layout |

```python
# Archive — flat structure
install_layout = archive_layout("mytool")

# Archive — with version directory stripping
install_layout = archive_layout("mytool",
    strip_prefix="mytool-{vversion}-{triple}")

# Binary — direct download
install_layout = binary_layout("kubectl")

# bin/ subdirectory (Node.js, Go, Java pattern)
install_layout = bin_subdir_layout(
    ["node", "npm", "npx"],
    strip_prefix="node-v{version}-{os}-{arch}")
```

#### Post-Extract Hook Builders

| Function | Signature | Description |
|----------|-----------|-------------|
| `post_extract_flatten(pattern=None)` | `→ fn(ctx, ver, dir) → list` | Flatten top-level version directory |
| `post_extract_shim(shim_name, target_executable, args=None)` | `→ fn(ctx, ver, dir) → list` | Create shim script |
| `post_extract_permissions(paths, mode="755", unix_only=True)` | `→ fn(ctx, ver, dir) → list` | Set executable permissions |
| `post_extract_combine(hooks)` | `→ fn(ctx, ver, dir) → list` | Combine multiple hooks |

```python
# Set permissions on Unix
post_extract = post_extract_permissions(["bin/node", "bin/npm", "bin/npx"])

# Create a shim: `bunx foo` → `bun x foo`
post_extract = post_extract_shim("bunx", "bun", args=["x"])

# Combine multiple hooks
post_extract = post_extract_combine([
    post_extract_flatten(pattern="jdk-*"),
    post_extract_permissions(["bin/java"]),
])
```

#### Pre-Run Hook Builder

| Function | Signature | Description |
|----------|-----------|-------------|
| `pre_run_ensure_deps(package_manager, trigger_args, check_file, lock_file=None, install_dir=None)` | `→ fn(ctx, args, exe) → list` | Auto-install project dependencies before run |

```python
# Ensure node_modules before `npm run`
pre_run = pre_run_ensure_deps("npm",
    trigger_args = ["run", "run-script"],
    check_file   = "package.json",
    install_dir  = "node_modules",
)

# Ensure .venv before `uv run`
pre_run = pre_run_ensure_deps("uv",
    trigger_args = ["run"],
    check_file   = "pyproject.toml",
    install_dir  = ".venv",
)
```

#### Path & Environment Factories

| Function | Signature | Description |
|----------|-----------|-------------|
| `path_fns(store_name, executable=None)` | `→ dict` | Returns `{"store_root": fn, "get_execute_path": fn}` |
| `path_env_fns(extra_env=None)` | `→ dict` | Returns `{"environment": fn, "post_install": fn}` |
| `bin_subdir_env(extra_env=None)` | `→ fn(ctx, version) → list` | Auto-detect `bin/` vs root for PATH |
| `bin_subdir_execute_path(executable)` | `→ fn(ctx, version) → string` | Executable path in `bin/` subdirectory |

```python
# Quick setup for store_root + get_execute_path
paths            = path_fns("node")
store_root       = paths["store_root"]
get_execute_path = bin_subdir_execute_path("node")
environment      = bin_subdir_env()
```

#### Version Fetch Helpers

| Function | Signature | Description |
|----------|-----------|-------------|
| `fetch_versions_from_api(url, transform)` | `→ fn(ctx) → descriptor` | Non-GitHub version API |
| `fetch_versions_with_tag_prefix(owner, repo, tag_prefix, prereleases=False)` | `→ fn(ctx) → descriptor` | Non-standard GitHub tag prefix |

```python
# Bun uses "bun-v" tag prefix
fetch_versions = fetch_versions_with_tag_prefix(
    "oven-sh", "bun", tag_prefix="bun-v")

# Node.js official API
fetch_versions = fetch_versions_from_api(
    "https://nodejs.org/dist/index.json", "nodejs_org")
```

---

### 6.9 `permissions.star` — Permission Declarations

| Function | Signature | Description |
|----------|-----------|-------------|
| `github_permissions(extra_hosts=None, exec_cmds=None)` | `→ dict` | Declares GitHub API + download access |
| `system_permissions(exec_cmds=None, extra_hosts=None)` | `→ dict` | No network download, system package manager only |

```python
# Standard GitHub tool
permissions = github_permissions()

# GitHub + extra API hosts
permissions = github_permissions(extra_hosts=["nodejs.org", "go.dev"])

# System-only (no binary download)
permissions = system_permissions()
```

---

### 6.10 `system_install.star` — Package Manager Strategies

#### Single-Strategy Builders

| Function | Signature | Description |
|----------|-----------|-------------|
| `winget_install(package, priority=90, install_args=None)` | `→ dict` | winget (Windows) |
| `choco_install(package, priority=80, install_args=None)` | `→ dict` | Chocolatey (Windows) |
| `scoop_install(package, priority=70)` | `→ dict` | Scoop (Windows) |
| `brew_install(package, priority=90)` | `→ dict` | Homebrew (macOS/Linux) |
| `apt_install(package, priority=80)` | `→ dict` | APT (Debian/Ubuntu) |
| `dnf_install(package, priority=75)` | `→ dict` | DNF (Fedora/RHEL) |
| `pacman_install(package, priority=70)` | `→ dict` | pacman (Arch Linux) |
| `snap_install(package, priority=60, classic=False)` | `→ dict` | Snap (Linux) |

#### Multi-Strategy Builders

| Function | Signature | Description |
|----------|-----------|-------------|
| `pkg_strategy(manager, package, priority, install_args, platforms)` | `→ dict` | Generic strategy |
| `system_install_strategies(strategies)` | `→ dict` | Wrap strategy list |
| `cross_platform_install(windows, macos, linux, ...)` | `→ fn(ctx) → dict` | OS-dispatched install |
| `windows_install(winget, choco, scoop, ...)` | `→ fn(ctx) → dict` | Windows-specific |
| `multi_platform_install(windows_strategies, macos_strategies, linux_strategies)` | `→ fn(ctx) → dict` | Full control |

```python
# Simple cross-platform
system_install = cross_platform_install(
    windows = winget_install("7zip.7zip"),
    macos   = brew_install("sevenzip"),
    linux   = apt_install("p7zip-full"),
)

# Multiple strategies per platform
system_install = multi_platform_install(
    windows_strategies = [
        winget_install("7zip.7zip", priority=90),
        choco_install("7zip", priority=80),
    ],
    macos_strategies = [brew_install("sevenzip")],
    linux_strategies = [
        apt_install("p7zip-full"),
        brew_install("sevenzip", priority=70),
    ],
)
```

---

### 6.11 `script_install.star` — Script-Based Installation

| Function | Signature | Description |
|----------|-----------|-------------|
| `curl_bash_install(url, post_install_cmds=None)` | `→ fn(ctx) → dict` | `curl \| bash` (Unix) |
| `curl_sh_install(url, post_install_cmds=None)` | `→ fn(ctx) → dict` | `curl \| sh` (POSIX) |
| `irm_iex_install(url, env_vars=None, pre_commands=None, post_install_cmds=None)` | `→ fn(ctx) → dict` | PowerShell `iex(irm ...)` (Windows) |
| `irm_install(url, env_vars=None, post_install_cmds=None)` | `→ fn(ctx) → dict` | Modern PowerShell `irm` (Windows) |
| `platform_script_install(unix=None, windows=None)` | `→ fn(ctx) → dict` | OS-dispatched script install |

```python
# Rustup-style install
script_install = platform_script_install(
    unix    = curl_sh_install("https://sh.rustup.rs"),
    windows = irm_iex_install("https://win.rustup.rs"),
)
```

---

### 6.12 `semver.star` — Version Comparison

| Function | Signature | Description |
|----------|-----------|-------------|
| `semver_strip_v(version)` | `→ string` | Strip `v` prefix |
| `semver_parse(version)` | `→ [major, minor, patch]` | Parse into integer list |
| `semver_compare(a, b)` | `→ -1 \| 0 \| 1` | Compare two versions |
| `semver_gt(a, b)` | `→ bool` | Greater than |
| `semver_lt(a, b)` | `→ bool` | Less than |
| `semver_gte(a, b)` | `→ bool` | Greater than or equal |
| `semver_lte(a, b)` | `→ bool` | Less than or equal |
| `semver_eq(a, b)` | `→ bool` | Equal |
| `semver_sort(versions, reverse=False)` | `→ list` | Sort version strings |

```python
from "@vx//stdlib:semver.star" import semver_gt, semver_sort

if semver_gt(version, "2.0.0"):
    # Use new API format
    pass

sorted_versions = semver_sort(["1.2.3", "1.0.0", "2.0.0"])
# → ["1.0.0", "1.2.3", "2.0.0"]
```

---

### 6.13 `test.star` — Testing DSL

| Function | Signature | Description |
|----------|-----------|-------------|
| `cmd(command, name=None, expect_success=True, expected_output=None, timeout_ms=None)` | `→ dict` | Run command and check result |
| `check_path(path, name=None)` | `→ dict` | Assert path exists |
| `check_not_path(path, name=None)` | `→ dict` | Assert path does not exist |
| `check_env(var_name, name=None, expected_output=None)` | `→ dict` | Assert env var is set |
| `check_not_env(var_name, name=None)` | `→ dict` | Assert env var is not set |
| `check_file(path, name=None, expected_output=None)` | `→ dict` | Assert file exists and content matches |

Used in `test_commands` within runtime definitions:

```python
runtimes = [
    runtime_def("node",
        test_commands=[
            cmd("{executable} --version",
                name="version_check",
                expected_output="v\\d+\\.\\d+"),
            check_path("{install_dir}/bin/node",
                name="binary_exists"),
        ],
    ),
]
```

---

### 6.14 `provider_templates.star` — High-Level Templates

Templates return a **dict** with all standard provider functions pre-configured. Unpack into module-level variables.

#### `github_rust_provider(owner, repo, **kwargs) → dict`

For tools using Rust target triple naming in GitHub releases.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `owner` | `string` | — | GitHub owner |
| `repo` | `string` | — | GitHub repository |
| `asset` | `string` | — | Asset name template |
| `executable` | `string` | `repo` | Executable name |
| `store` | `string` | `repo` | Store directory name |
| `tag_prefix` | `string` | `"v"` | Tag prefix to strip |
| `linux_libc` | `string` | `"musl"` | `"musl"` or `"gnu"` |
| `prereleases` | `bool` | `False` | Include pre-releases |
| `strip_prefix` | `string` | `None` | Archive directory prefix to strip |
| `path_env` | `string` | `None` | Custom PATH env |
| `extra_env` | `list` | `None` | Additional env operations |

**Asset template placeholders:** `{version}`, `{vversion}`, `{triple}`, `{ext}`, `{exe}`

```python
_p = github_rust_provider("BurntSushi", "ripgrep",
    asset        = "ripgrep-{version}-{triple}.{ext}",
    executable   = "rg",
    tag_prefix   = "",
    strip_prefix = "ripgrep-{version}-{triple}",
)
# Asset example: ripgrep-14.1.1-x86_64-unknown-linux-musl.tar.gz
```

#### `github_go_provider(owner, repo, **kwargs) → dict`

For tools using Go-style `{os}_{arch}` naming (goreleaser pattern).

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `owner` | `string` | — | GitHub owner |
| `repo` | `string` | — | GitHub repository |
| `asset` | `string` | — | Asset name template |
| `executable` | `string` | `repo` | Executable name |
| `store` | `string` | `repo` | Store directory name |
| `tag_prefix` | `string` | `"v"` | Tag prefix |
| `prereleases` | `bool` | `False` | Include pre-releases |
| `strip_prefix` | `string` | `None` | Directory prefix to strip |
| `path_env` | `string` | `None` | Custom PATH env |
| `extra_env` | `list` | `None` | Additional env operations |

**Asset template placeholders:** `{version}`, `{vversion}`, `{os}`, `{arch}`, `{ext}`, `{exe}`

```python
_p = github_go_provider("cli", "cli",
    asset        = "gh_{version}_{os}_{arch}.{ext}",
    executable   = "gh",
    strip_prefix = "gh_{version}_{os}_{arch}",
)
# Asset example: gh_2.67.0_linux_amd64.tar.gz
```

#### `github_binary_provider(owner, repo, **kwargs) → dict`

For tools that distribute a single executable (no archive).

```python
_p = github_binary_provider("kubernetes", "kubectl",
    asset = "kubectl{exe}",
)
```

#### `system_provider(store_name, **kwargs) → dict`

For tools installed via system package managers only.

```python
_p = system_provider("7zip", executable="7z")
```

#### Template Return Dict Keys

All templates return a dict with these keys:

| Key | Type | Description |
|-----|------|-------------|
| `"fetch_versions"` | `function` | Version fetcher |
| `"download_url"` | `function` | URL builder |
| `"install_layout"` | `function` | Layout descriptor |
| `"store_root"` | `function` | Store root path |
| `"get_execute_path"` | `function` | Executable path |
| `"post_install"` | `function` | Post-install hook |
| `"environment"` | `function` | Environment setup |
| `"deps"` | `function` | Dependencies |

**Unpacking pattern:**

```python
_p = github_rust_provider(...)

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]
```

---

## See Also

- [Core API Reference](./provider-star-core-api.md) — Execution model, file structure, provider functions, `ctx` object
- [Layouts & Strategies](./provider-star-layouts.md) — Install layouts, version fetching, hooks
- [Language & Conventions](./provider-star-language.md) — Starlark subset, coding style, new provider checklist
- [Back to Overview](./provider-star-reference.md)
