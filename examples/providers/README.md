# Provider Examples

This directory contains example providers demonstrating vx's `provider.star` format.

> **Note**: The old `provider.toml` format has been deprecated. All providers now use
> `provider.star` (Starlark) as the single source of truth for metadata and logic.

## Structure

```
~/.vx/providers/<name>/
└── provider.star    # All logic and metadata
```

## Example

```python
# ~/.vx/providers/mytool/provider.star
load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")

# Metadata
name        = "mytool"
description = "My awesome tool"
homepage    = "https://github.com/org/mytool"
repository  = "https://github.com/org/mytool"
license     = "MIT"
ecosystem   = "devtools"

# Runtime definitions
runtimes = [
    runtime_def("mytool"),
]

# Permissions
permissions = github_permissions()

# Version fetching
fetch_versions = make_fetch_versions("org", "mytool")

# Download URL
def download_url(ctx, version, platform):
    return github_asset_url(ctx, "org", "mytool", version, platform)
```

## See Also

- [Manifest-Driven Providers Guide](/guide/manifest-driven-providers)
- [Provider Development Documentation](/advanced/plugin-development)
- [Built-in Providers](https://github.com/loonghao/vx/tree/main/crates/vx-providers)
