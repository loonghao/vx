# provider.star — Install Layouts, Version Strategies & Hooks

> ← [Back to Overview](./provider-star-reference.md)

This document covers install layout types, version fetching strategies, and hook system (post-extract and pre-run hooks).

---

## Table of Contents

- [7. Install Layout Types](#7-install-layout-types)
- [8. Version Fetching Strategies](#8-version-fetching-strategies)
- [9. Hooks](#9-hooks)

---

## 7. Install Layout Types

The `install_layout()` function returns a descriptor dict. The `__type` (or `type`) field determines the strategy:

| Type | Required Fields | Optional Fields | Use Case |
|------|----------------|-----------------|----------|
| `"archive"` | `type` | `strip_prefix`, `executable_paths` | tar.gz, zip archives |
| `"binary"` | `type` | `executable_name`, `permissions` | Direct executable download |
| `"msi"` | `type`, `url` | `executable_paths`, `strip_prefix`, `extra_args` | Windows MSI installer |
| `"system_find"` | `type`, `executable` | `system_paths`, `hint` | System-installed tool lookup |

### Archive Layout

```python
def install_layout(ctx, version):
    return {
        "type":             "archive",
        "strip_prefix":     "mytool-v{}".format(version),
        "executable_paths": ["bin/mytool", "mytool"],
    }
```

### Binary Layout

```python
def install_layout(ctx, version):
    exe = "mytool.exe" if ctx.platform.os == "windows" else "mytool"
    return {
        "type":            "binary",
        "executable_name": exe,
        "permissions":     "755",
    }
```

### MSI Layout (Windows)

```python
def install_layout(ctx, version):
    return {
        "type":             "msi",
        "url":              download_url(ctx, version),
        "executable_paths": ["bin/tool.exe", "tool.exe"],
        "extra_args":       ["/quiet", "/norestart"],
    }
```

### System Find Layout

```python
def install_layout(ctx, version):
    return {
        "type":         "system_find",
        "executable":   "cmake",
        "system_paths": ["/usr/local/bin/cmake", "C:\\Program Files\\CMake\\bin\\cmake.exe"],
        "hint":         "Install via 'brew install cmake' or 'winget install Kitware.CMake'",
    }
```

---

## 8. Version Fetching Strategies

| Strategy | Function | When to Use |
|----------|----------|-------------|
| **GitHub releases (template)** | `make_fetch_versions(owner, repo)` | Most GitHub-hosted tools |
| **GitHub releases (raw)** | `github_releases(ctx, owner, repo)` | When you need custom filtering |
| **Non-standard tag prefix** | `fetch_versions_with_tag_prefix(owner, repo, "bun-v")` | Tags like `bun-v1.2.3` |
| **Official API** | `fetch_versions_from_api(url, transform)` | Node.js, Go, Java, etc. |
| **Custom** | Write `fetch_versions(ctx)` manually | Unusual version sources |

### Selection Guide

```
                  ┌─ GitHub releases? ──┐
                  │                      │
              ┌─ Yes ─┐            ┌── No ──┐
              │        │           │         │
        Standard tag?  Non-standard    Official API?
        (v1.2.3)       (bun-v1.2.3)       │
              │              │         ┌─ Yes ──┐
     make_fetch_versions   fetch_versions_   fetch_versions_
                           with_tag_prefix   from_api
                                            │
                                        ┌─ No ──┐
                                        │        │
                                   Custom fetch_versions()
```

---

## 9. Hooks

### Post-Extract Hooks

Executed after archive extraction. Used for:
- Flattening nested directories
- Creating shim scripts
- Setting Unix file permissions

```python
# Flatten JDK directory (jdk-21.0.1+12/ → contents moved to root)
post_extract = post_extract_flatten(pattern="jdk-*")

# Create shim: `bunx` → `bun x`
post_extract = post_extract_shim("bunx", "bun", args=["x"])

# Set permissions on multiple files
post_extract = post_extract_permissions(["bin/node", "bin/npm", "bin/npx"])

# Combine
post_extract = post_extract_combine([
    post_extract_flatten(pattern="jdk-*"),
    post_extract_permissions(["bin/java", "bin/javac"]),
])
```

### Pre-Run Hooks

Executed before the runtime command runs. Used for auto-installing project dependencies:

```python
# Before `npm run ...`, ensure node_modules exists
pre_run = pre_run_ensure_deps("npm",
    trigger_args = ["run", "run-script"],
    check_file   = "package.json",
    install_dir  = "node_modules",
)
```

---

## See Also

- [Core API Reference](./provider-star-core-api.md) — Execution model, file structure, provider functions, `ctx` object
- [Standard Library](./provider-star-stdlib.md) — All 14 stdlib modules (layout builders, hook builders)
- [Language & Conventions](./provider-star-language.md) — Starlark subset, coding style, new provider checklist
- [Back to Overview](./provider-star-reference.md)
