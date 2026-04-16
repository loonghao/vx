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

## See Also

- [Core API Reference](./provider-star-core-api.md) — Execution model, file structure, provider functions, `ctx` object
- [Standard Library](./provider-star-stdlib.md) — All 14 stdlib modules (env, platform, layout, templates…)
- [Layouts & Strategies](./provider-star-layouts.md) — Install layouts, version fetching, hooks
- [Language & Conventions](./provider-star-language.md) — Starlark subset, coding style, new provider checklist
- [Manifest-Driven Providers](./manifest-driven-providers.md) — Getting-started guide
- [Starlark Providers – Advanced Guide](./starlark-providers.md) — Multi-runtime providers, custom version sources
- [vx.toml Reference](../config/vx-toml.md) — Project configuration
- [vx.toml Syntax Guide](./vx-toml-syntax.md) — Patterns and recipes
