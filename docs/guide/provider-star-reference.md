# provider.star — Language & Standard Library Reference

vx uses a **two-phase execution model**: `provider.star` files run as pure Starlark (no I/O), returning descriptor dicts that the Rust runtime interprets for actual downloads, installs, and execution. Providers are defined declaratively — no Rust code required for new tools.

> **Companion docs**
>
> - [Manifest-Driven Providers](./manifest-driven-providers.md) — Getting-started tutorial
> - [Starlark Providers – Advanced Guide](./starlark-providers.md) — Multi-runtime, hooks, system integration

---

## Quick Start — Minimal Provider

```python
load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "github_rust_provider")

name        = "mytool"
description = "My awesome tool"
ecosystem   = "devtools"

runtimes    = [runtime_def("mytool")]
permissions = github_permissions()

_p = github_rust_provider("owner", "repo",
    asset = "mytool-{vversion}-{triple}.{ext}")

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]
```

---

## Navigation

| I need to… | Go here |
|------------|---------|
| Understand the execution model, file structure, top-level variables, provider functions, and the `ctx` object | [Core API →](./provider-star-core-api.md) |
| Look up stdlib functions (env, platform, layout, github, templates…) | [Standard Library →](./provider-star-stdlib.md) |
| Learn install layout types, version fetching strategies, and hooks | [Layouts & Strategies →](./provider-star-layouts.md) |
| Review Starlark syntax rules, coding conventions, and the new provider checklist | [Language & Conventions →](./provider-star-language.md) |

---

## Published asset versions from HTML

Use `fetch_html_versions(ctx, url, href_prefix, filename_prefix, filename_suffix)`
from `@vx//stdlib:http.star` when source tags can precede published binaries:

```python
load("@vx//stdlib:http.star", "fetch_html_versions")

def fetch_versions(ctx):
    return fetch_html_versions(ctx,
        "https://publisher.example/download/",
        "https://download.publisher.example/stable/",
        "editor-",
        "-x86_64.AppImage",
    )
```

This returns a pure descriptor. Rust fetches the page with a 30-second timeout,
requires a successful HTTP status, and matches actual anchor `href` attributes.
The absolute HTTP(S) `href_prefix` must end in `/`; filename literals match the
linked basename. Choose the suffix for `ctx.platform.os` and `ctx.platform.arch`.
Only dotted ASCII numeric versions are returned, deduplicated and sorted newest
first; optional `exclude_version_suffixes` excludes release-specific suffixes.
Results have `stable=True`, `lts=False`, and no release date.

Quoted and unquoted attributes are supported. Comments, raw-text elements,
non-anchor attributes, text, relative links, traversal, encoded paths, queries,
and fragments are excluded. This is literal link discovery, without HTML entity
decoding or JavaScript execution. No matching binary or a failed request returns
an error; the existing expired-cache fallback can reuse only the same Provider
script and OS/architecture scope. Other descriptor cache formats stay unchanged.

## See Also

- [Core API Reference](./provider-star-core-api.md) — Execution model, file structure, provider functions, `ctx` object
- [Standard Library](./provider-star-stdlib.md) — All 14 stdlib modules (env, platform, layout, templates…)
- [Layouts & Strategies](./provider-star-layouts.md) — Install layouts, version fetching, hooks
- [Language & Conventions](./provider-star-language.md) — Starlark subset, coding style, new provider checklist
- [Manifest-Driven Providers](./manifest-driven-providers.md) — Getting-started guide
- [Starlark Providers – Advanced Guide](./starlark-providers.md) — Multi-runtime providers, custom version sources
- [vx.toml Reference](../config/vx-toml.md) — Project configuration
- [vx.toml Syntax Guide](./vx-toml-syntax.md) — Patterns and recipes
