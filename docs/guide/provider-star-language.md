# provider.star — Starlark Language Reference & Conventions

> ← [Back to Overview](./provider-star-reference.md)

This document covers the Starlark language subset supported in `provider.star`, coding conventions, and the complete checklist for creating a new provider.

---

## Table of Contents

- [10. Starlark Language Subset](#10-starlark-language-subset)
- [11. Coding Conventions](#11-coding-conventions)
- [12. Checklist: New Provider](#12-checklist-new-provider)

---

## 10. Starlark Language Subset

provider.star uses [Starlark](https://github.com/bazelbuild/starlark), a Python-like language with deliberate restrictions:

### Supported

| Feature | Example |
|---------|---------|
| Variables | `x = 42` |
| Strings | `"hello"`, `'hello'`, `"""multi-line"""` |
| String formatting | `"v{}".format(version)` |
| Lists | `[1, 2, 3]`, list comprehensions |
| Dicts | `{"key": "value"}`, dict comprehensions |
| Functions | `def my_func(arg1, arg2="default"):` |
| Conditionals | `if/elif/else` |
| Loops | `for x in collection:` |
| Boolean logic | `and`, `or`, `not` |
| None | `None` |
| String methods | `.format()`, `.get()`, `.startswith()`, etc. |
| `load()` | `load("@vx//stdlib:module.star", "symbol")` |
| `fail()` | `fail("error message")` — abort with error |

### NOT Supported

| Feature | Reason |
|---------|--------|
| `import` | Use `load()` instead |
| `class` | Not available in Starlark |
| `try/except` | No exception handling |
| `with` | No context managers |
| `lambda` | Not supported |
| `*args, **kwargs` | Not supported |
| Mutation after freeze | Top-level values are frozen after module load |
| Side effects | No I/O, networking, or filesystem access |

### Key Differences from Python

1. **No mutation of frozen values** — Once a module is loaded, its top-level data structures are immutable
2. **No `set` type** — Use `dict` with dummy values or list deduplication
3. **Integer division** — `//` is integer division, `/` is not available
4. **String concatenation** — `"a" + "b"` works, but `str.format()` is preferred
5. **No global state** — Functions cannot modify module-level variables

---

## 11. Coding Conventions

### Naming

| Category | Convention | Example |
|----------|-----------|---------|
| Module variables | `snake_case` | `name`, `fetch_versions` |
| Functions | `snake_case` | `download_url()`, `install_layout()` |
| Private functions | `_` prefix | `_my_platform()`, `_triple()` |
| Constants | `UPPER_SNAKE_CASE` or `_` prefix | `_PLATFORMS`, `RUST_TRIPLES_MUSL` |
| Template variables | `_p` | `_p = github_rust_provider(...)` |

### File Organization

```python
# 1. load() statements
load("@vx//stdlib:provider.star", ...)

# 2. Metadata variables
name        = "..."
description = "..."

# 3. Runtime definitions
runtimes = [...]

# 4. Permissions
permissions = ...

# 5. Private helpers
def _my_platform(ctx): ...
_PLATFORMS = {...}

# 6. Provider functions (or template unpacking)
fetch_versions   = ...
download_url     = ...
install_layout   = ...
store_root       = ...
get_execute_path = ...
environment      = ...
```

### Platform Handling

```python
# ✅ GOOD — Return None for unsupported platforms
def download_url(ctx, version):
    triple = platform_map(ctx, _PLATFORMS)
    if not triple:
        return None
    # ...

# ❌ BAD — fail() for unsupported platform
def download_url(ctx, version):
    triple = platform_map(ctx, _PLATFORMS)
    if not triple:
        fail("Unsupported platform")  # Don't do this!
```

### String Formatting

```python
# ✅ GOOD — Use .format()
url = "https://example.com/v{}/tool-{}.tar.gz".format(version, triple)

# ❌ BAD — Use f-strings (not supported in Starlark)
url = f"https://example.com/v{version}/tool-{triple}.tar.gz"

# ❌ BAD — Use % formatting (not reliable in Starlark)
url = "https://example.com/v%s/tool-%s.tar.gz" % (version, triple)
```

### Unused Parameters

```python
# ✅ GOOD — Prefix with underscore
def deps(_ctx, _version):
    return []

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]
```

---

## 12. Checklist: New Provider

Use this checklist when creating a new provider:

- [ ] Create `crates/vx-providers/<name>/provider.star`
- [ ] Set metadata: `name`, `description`, `ecosystem`, `license`
- [ ] Define `runtimes` with `runtime_def()` (add `bundled_runtime_def()` for bundled tools)
- [ ] Declare `permissions` with `github_permissions()` or `system_permissions()`
- [ ] Choose strategy:
  - [ ] **Template** — `github_rust_provider()`, `github_go_provider()`, `github_binary_provider()`
  - [ ] **Custom functions** — Write `fetch_versions`, `download_url`, `install_layout` manually
- [ ] Define `environment()` (at minimum, prepend install dir to PATH)
- [ ] Add hooks if needed:
  - [ ] `post_extract` — permissions, shims, directory flattening
  - [ ] `pre_run` — dependency auto-install
- [ ] Declare `deps()` if the tool depends on other runtimes
- [ ] Add `system_install` for system package manager fallback
- [ ] Add `test_commands` in runtime definition
- [ ] Classify the provider archetype before writing tests:
  - [ ] `system`
  - [ ] `package_alias`
  - [ ] `binary_direct`
  - [ ] `archive_extract`
  - [ ] `redirect_api`
- [ ] Add `starlark_logic_tests.rs` with semantic assertions that match the chosen archetype
- [ ] Use the shared lint helper instead of a provider-local `known_globals` list:
  - [ ] `vx_starlark::provider_test_support::assert_provider_star_lint_clean(PROVIDER_STAR)`
- [ ] Run static provider checks locally: `vx just test-providers-static`
- [ ] Test: `vx <runtime> --version`
- [ ] Test on all supported platforms (Windows, macOS, Linux)

---

## See Also

- [Core API Reference](./provider-star-core-api.md) — Execution model, file structure, provider functions, `ctx` object
- [Standard Library](./provider-star-stdlib.md) — All 14 stdlib modules
- [Layouts & Strategies](./provider-star-layouts.md) — Install layouts, version fetching, hooks
- [Back to Overview](./provider-star-reference.md)
