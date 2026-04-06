---
applyTo: "crates/vx-providers/*/provider.star"
---

# Starlark Provider Instructions

When editing or creating `provider.star` files, follow these rules:

## Required Fields

Every `provider.star` must define:
- `name` (string) — Provider name
- `runtimes` (list) — At least one `runtime_def()`
- `permissions` (dict) — Use `github_permissions()` or `system_permissions()`

## Prefer Templates

Use templates for 90% of cases — don't hand-write download logic unless necessary:

```starlark
load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:provider_templates.star", "github_rust_provider")

name        = "<name>"
description = "<description>"
ecosystem   = "custom"  # nodejs, python, rust, go, system, custom
runtimes    = [runtime_def("<runtime>", aliases = ["<alias>"])]
permissions = github_permissions()

_p = github_rust_provider("owner", "repo",
    asset = "tool-{vversion}-{triple}.{ext}")
fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]
```

## Template Selection

| Pattern | Template | Example |
|---------|----------|---------|
| Rust target triple naming | `github_rust_provider` | ripgrep, just, uv, fd |
| Go goreleaser style | `github_go_provider` | gh, golangci-lint |
| Single binary (no archive) | `github_binary_provider` | kubectl |
| System package manager only | `system_provider` | 7zip, make |

## Platform Constraints

Return `None` from `download_url()` for unsupported platforms — never raise an error.

## Testing

After creating/editing a provider: `vx <runtime> --version`
