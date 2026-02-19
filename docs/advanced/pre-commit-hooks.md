# Pre-commit Hooks

vx uses [prek](https://prek.j178.dev/) (a Rust-based pre-commit replacement) to enforce code quality checks before every commit. This document describes the hooks configured in `.pre-commit-config.yaml` and how they protect the codebase.

## Overview

Pre-commit hooks run automatically when you execute `git commit`. If any hook fails, the commit is blocked until the issue is resolved. This catches problems early — before they reach CI or break other developers' builds.

```bash
# Install hooks (one-time setup)
vx prek install

# Run all hooks manually on all files
vx prek run --all-files

# Run a specific hook
vx prek run --hook-id cargo-hakari
```

## Configured Hooks

### 1. Spell Checking (`typos`)

Catches common typos in source code and documentation.

```yaml
- repo: https://github.com/crate-ci/typos
  rev: v1.43.4
  hooks:
    - id: typos
```

### 2. Rust Formatting (`cargo-fmt`)

Ensures all Rust code is formatted with `rustfmt`.

```yaml
- id: cargo-fmt
  entry: vx cargo fmt --all --
  types: [rust]
```

### 3. Rust Linting (`cargo-clippy`)

Runs Clippy and treats all warnings as errors.

```yaml
- id: cargo-clippy
  entry: vx cargo clippy --workspace -- -D warnings
  types: [rust]
```

### 4. Test Compilation Check (`cargo-check-tests`)

Compiles all test code to catch errors like `E0061` (wrong number of arguments) that only appear in test files.

```yaml
- id: cargo-check-tests
  entry: vx cargo check --workspace --tests
  types: [rust]
```

### 5. YAML/JSON Formatting (`prettier`)

Formats YAML and JSON files with Prettier.

```yaml
- id: prettier
  entry: vx npx prettier --write --ignore-unknown
  types_or: [yaml, json]
```

### 6. Workspace-Hack Verification (`cargo-hakari`) ⭐ New

Verifies that the `workspace-hack` crate is up-to-date with the current dependency graph.

```yaml
- id: cargo-hakari
  name: cargo hakari check
  entry: vx cargo hakari generate --diff
  language: system
  files: Cargo\.(toml|lock)$
  pass_filenames: false
```

**Why this matters:** vx uses [cargo-hakari](https://docs.rs/cargo-hakari) to optimize build times by unifying feature flags across the workspace. When you add or update a dependency in any `Cargo.toml`, the `workspace-hack` crate must be regenerated. If it drifts out of sync, CI will fail with a diff like:

```diff
--- original
+++ modified
@@ -20,7 +20,7 @@
 regex = { version = "1" }
-regex-automata = { version = "0.4", default-features = false, features = ["dfa-build", ...] }
+regex-automata = { version = "0.4", default-features = false, features = ["dfa", "hybrid", ...] }
```

**How it works:**
- Triggers on any change to `Cargo.toml` or `Cargo.lock`
- Runs `cargo hakari generate --diff` which exits with a non-zero code if the workspace-hack is stale
- Does **not** modify files — it only checks

**When it fails:** Run the following to fix it:

```bash
# Regenerate workspace-hack
vx cargo hakari generate

# Also update dependency declarations
vx cargo hakari manage-deps

# Or use the justfile recipe
just hakari-generate
```

### 7. Justfile Duplicate Recipe Detection (`justfile-no-duplicate-recipes`) ⭐ New

Detects duplicate recipe definitions in the `justfile`.

```yaml
- id: justfile-no-duplicate-recipes
  name: justfile no duplicate recipes
  entry: vx just --list
  language: system
  files: ^justfile$
  pass_filenames: false
```

**Why this matters:** The `just` command runner does not silently ignore duplicate recipe definitions — it exits with an error like:

```
error: Recipe `test-pkgs` first defined on line 74 is redefined on line 155
   ——▶ justfile:155:1
    │
155 │ test-pkgs PKGS:
    │ ^^^^^^^^^
Error: Process completed with exit code 1.
```

This error would cause any `just` command to fail, breaking the entire development workflow and CI pipeline.

**How it works:**
- Triggers only when the `justfile` is modified
- Runs `just --list` which parses the entire justfile and fails immediately on duplicate recipes
- Catches the problem at commit time, before it reaches CI

**When it fails:** Find and remove the duplicate recipe definition:

```bash
# Find duplicate recipe names
grep -n "^[a-zA-Z_-]*:" justfile | sort | uniq -d

# Or use just to show the error location
just --list
```

### 8. Common Safety Checks

Standard checks from `pre-commit-hooks`:

| Hook | Description |
|------|-------------|
| `check-merge-conflict` | Prevents committing unresolved merge conflict markers |
| `check-added-large-files` | Blocks files larger than 500 KB |
| `end-of-file-fixer` | Ensures files end with a newline |
| `check-toml` | Validates TOML syntax |
| `trailing-whitespace` | Removes trailing whitespace |

## Setup

### Install prek

```bash
# Install prek via vx (auto-installs if needed)
vx prek install
```

### Verify Installation

```bash
# Check that hooks are installed
ls .git/hooks/pre-commit

# Run all hooks on all files to verify everything passes
vx prek run --all-files
```

## Bypassing Hooks (Emergency Only)

In rare cases where you need to commit without running hooks:

```bash
# Skip all hooks (use sparingly!)
git commit --no-verify -m "emergency fix"
```

::: warning
Bypassing hooks should be a last resort. The CI pipeline runs the same checks, so skipping hooks locally just delays the failure to CI.
:::

## Troubleshooting

### `cargo-hakari` fails after adding a dependency

```bash
# Regenerate workspace-hack
vx cargo hakari generate
vx cargo hakari manage-deps

# Verify it's now clean
vx cargo hakari generate --diff
```

### `justfile-no-duplicate-recipes` fails

```bash
# Show the error with line numbers
vx just --list

# Search for the duplicate
grep -n "^recipe-name:" justfile
```

### Hook runs slowly

The `cargo-clippy` and `cargo-check-tests` hooks compile Rust code, which can be slow on first run. Subsequent runs use the incremental compilation cache and are much faster.

### Reset all hooks

```bash
# Uninstall and reinstall
vx prek uninstall
vx prek install
```

## Advanced Usage

### Run hooks on specific files

```bash
# Run all hooks on a specific file
vx prek run --files src/main.rs

# Run a specific hook on specific files
vx prek run --hook-id cargo-fmt --files src/lib.rs src/main.rs
```

### Run hooks in CI

The CI pipeline runs the same hooks via `vx prek run --all-files`. This ensures that:

1. Local development and CI use identical checks
2. No "works on my machine" issues with formatting or linting
3. The workspace-hack is always in sync

### Adding a new hook

To add a new pre-commit hook, edit `.pre-commit-config.yaml`:

```yaml
- repo: local
  hooks:
    - id: my-new-check
      name: my new check description
      entry: vx cargo my-check
      language: system
      types: [rust]
      pass_filenames: false
```

Then run `vx prek run --all-files` to verify the new hook works correctly.
