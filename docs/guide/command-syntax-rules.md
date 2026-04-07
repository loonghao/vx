# Unified Command Syntax Rules

This document defines a **single syntax contract** for vx command execution across all ecosystems and runtimes.  
It is the canonical reference for future CLI evolution and documentation consistency.

## Goals

- Reduce learning cost by using one grammar model everywhere.
- Keep common workflows short (`vx <runtime> ...`) while preserving explicit forms.
- Make parsing deterministic and avoid ambiguous patterns.
- Enable progressive command consolidation without breaking existing users.

## Core Principles

- **One prefix**: all managed execution starts with `vx`.
- **One canonical form per intent**: each user intent maps to one recommended syntax.
- **Compatibility first**: old forms remain aliases during migration windows.
- **Deterministic parsing**: no hidden reinterpretation of tokens.

## Syntax Rule Table

| Intent | Canonical Syntax | Example | Compatibility Notes |
|---|---|---|---|
| Runtime execution | `vx <runtime>[@runtime_version] [args...]` | `vx node@22 --version` | Keep existing direct runtime forms. |
| Bundled runtime execution | `vx <bundled_runtime>[@parent_version] [args...]` | `vx npx@20 create-react-app my-app` | For bundled runtimes, `@version` refers to the parent runtime's version. |
| Runtime executable override | `vx <runtime>[@runtime_version]::<executable> [args...]` | `vx msvc@14.42::cl main.cpp` | Accept `runtime::exe@version` temporarily; canonical is version-before-`::`. |
| Package execution | `vx <ecosystem>[@runtime_version]:<package>[@package_version][::executable] [args...]` | `vx uvx:pyinstaller::pyinstaller --version` | This is the only package grammar. |
| Multi-runtime composition | `vx --with <runtime>[@runtime_version] [--with <runtime>[@runtime_version] ...] <target_command>` | `vx --with bun@1.1.0 --with deno node app.js` | `--with` only injects companion runtimes for this invocation. |
| Shell launch | `vx shell launch <runtime>[@runtime_version] [shell]` | `vx shell launch node@22 powershell` | Keep `vx node::powershell` as compatibility alias. |
| Environment assembly | `vx env <create|use|list|show|add|remove|sync|shell|delete> ...` | `vx env sync` | `vx dev` is still valid for project-focused interactive sessions. |
| Project script execution | `vx run <script> [-- <args...>]` | `vx run test -- --nocapture` | Script definitions come from `vx.toml`. |
| Project state sync | `vx sync [--check]` + `vx lock [--check]` | `vx sync --check` | `sync` and `lock` are paired contracts for reproducibility. |
| Global package management | `vx pkg <add|rm|ls|info|shim-update> ...` | `vx pkg add npm:typescript@5.3` | `vx global ...` remains compatibility alias. |
| Project toolchain management | `vx project <init|add|rm|sync|lock|check> ...` | `vx project sync` | Existing top-level commands remain available as aliases. |

## Reserved Tokens

| Token | Meaning | Rule |
|---|---|---|
| `@` | Version selector | Runtime version appears before `:`, package version after package name. |
| `:` | Ecosystem/package separator | Used only for package execution grammar. |
| `::` | Executable selector | Used only to select executable/shell explicitly. |

## Parsing Priority (Deterministic)

1. If first arg is a declared CLI subcommand, route to subcommand parser.
2. Else if token matches package grammar (`<eco>...:<package>...`), parse as package execution.
3. Else if token contains `::`, parse as runtime executable/shell form.
4. Else parse as runtime or installed shim execution.

## Version Resolution Policy (Unified)

For all execution paths (runtime/package/shim):

1. CLI explicit version
2. `vx.lock`
3. `vx.toml`
4. latest compatible

## Global Execution Options Contract (Output + Cache)

These flags are cross-cutting syntax contracts and apply consistently to runtime/package/project execution:

| Concern | Canonical Syntax | Rule |
|---|---|---|
| Structured output (JSON) | `--json` | Shortcut for `--output-format json`; overrides `--output-format` when both are provided. |
| LLM-friendly output (TOON) | `--toon` | Shortcut for `--output-format toon`; overrides `--output-format` when both are provided. |
| Explicit output mode | `--output-format <text|json|toon>` | Canonical explicit form when shortcut flags are not used. |
| Cache strategy | `--cache-mode <normal|refresh|offline|no-cache>` | Unified cache control for all execution paths. |

Resolution notes:

- Output option precedence: shortcut flags (`--json` / `--toon`) > `--output-format` > environment defaults.
- Cache mode must be interpreted uniformly by parser/executor and documented examples.

## Capability Coverage Matrix (Core Scenarios)


| Scenario | Scope | Canonical Entry |
|---|---|---|
| Single runtime/package/shim execution | Daily command execution | `vx <runtime>...` / package grammar |
| Multi-runtime composition | One-shot companion runtime injection | `vx --with ... <target_command>` |
| Project-aware execution | Resolve toolchain from project context | `vx run` / `vx sync` / `vx lock` |
| Environment assembly | Build and reuse named/project environments | `vx env ...` / `vx dev` |
| Parse + state synchronization | Keep parser, docs, and lock state aligned | This document + parser tests + `vx sync/lock` checks |

## Multi-Environment Assembly Rules

- `--with` follows `runtime[@version]` grammar and can be repeated.
- Companion runtimes are **additive** to the current invocation environment only.
- Every `--with` runtime resolves versions using the same unified version policy.
- `--with` does not replace the primary target command; it augments execution prerequisites.

## Bundled Runtime Version Semantics

Bundled runtimes (e.g., `npm`, `npx` bundled with `node`; `cargo`, `rustc` bundled with `rust`) do not have independent version numbers in vx. They share the parent runtime's version space.

When `@version` is used with a bundled runtime, the version refers to the **parent runtime's version**, not the bundled runtime's own version:

```bash
# npx is bundled with node — @20 means "node version 20"
vx npx@20 create-react-app my-app   # Uses npx from Node.js 20

# npm is bundled with node — @22 means "node version 22"
vx npm@22 ci                         # Uses npm from Node.js 22

# cargo is bundled with rust — @1.80 means "rust version 1.80"
vx cargo@1.80 build --release        # Uses cargo from Rust 1.80
```

This is semantically equivalent to using `--with`:

```bash
# These two commands are equivalent:
vx npx@20 create-react-app my-app
vx --with node@20 npx create-react-app my-app
```

The version propagation happens automatically: when a bundled runtime is requested with an explicit version, vx installs the parent runtime at that version and uses the bundled tool from it.

## Project-Aware Execution Contract (`vx.toml` + `vx.lock`)

- Project context is discovered from the current directory upward.
- Runtime/package/shim execution in project context must follow the same version policy.
- `vx sync` is the desired-state reconciler from `vx.toml` to local runtime state.
- `vx lock` materializes deterministic versions, while `vx lock --check`/`vx sync --check` enforce drift detection.
- `vx run <script>` uses the same resolver semantics as direct command execution.

## Parse & Sync Contract (Governance)

Any syntax change MUST be synchronized across:

1. CLI parser behavior and tests (`crates/vx-cli/tests/`)
2. Canonical docs (`docs/guide/command-syntax-rules.md` + `/zh/` counterpart)
3. Agent guardrails (`AGENTS.md`)
4. CLI usage examples (`crates/vx-cli/src/cli.rs` long help)

No silent reinterpretation of ambiguous tokens is allowed. Compatibility aliases require explicit migration hints.

## Disallowed / Deprecated Patterns


- Disallow: `vx uvx::pyinstaller` (invalid package grammar). Use `vx uvx:pyinstaller`.
- Deprecate docs/examples using `vx install <runtime> <version>`; use `vx install <runtime>@<version>`.
- Deprecate `runtime::shell` as the primary documented shell form; prefer `vx shell launch <runtime> [shell]`.
- Deprecate `vx global ...` as primary form; prefer `vx pkg ...`.

## CLI Consolidation Plan

### Phase 1 (Documentation + Alias)

- Introduce documented canonical groups:
  - `vx pkg ...` (global package lifecycle)
  - `vx project ...` (project toolchain lifecycle)
- Keep existing commands as aliases (`global`, `add`, `remove`, `sync`, `lock`, `check`, `init`).

### Phase 2 (UX Warnings)

- For non-canonical invocations, print one-line migration hints.
- Add `--no-hints` for CI/noise-free environments.

### Phase 3 (Hardening)

- Freeze grammar and publish parser tests for all canonical forms.
- Keep aliases indefinitely for high-frequency commands, remove only low-usage legacy forms.

### Phase 4 (Legacy Cleanup)

- Remove non-canonical docs/examples first, then hide low-value aliases behind warnings.
- Prioritize cleanup targets:
  - duplicated shell forms where `vx shell ...` already covers the scenario,
  - deprecated install examples (`vx install <runtime> <version>`),
  - ambiguous or low-usage legacy spellings.
- Require usage telemetry + migration notice window before behavioral removal.


## Documentation Contract

Every new CLI/guide/tool doc must include:

- One canonical syntax line.
- One short examples block.
- If alias exists, mark it as compatibility alias.

## Test Contract (Parser + Docs)

- Add parser matrix tests covering runtime/package/shell/shim collision cases.
- Add docs lint checks for forbidden legacy syntax patterns.
- Ensure `README.md`, `README_zh.md`, and CLI docs share identical canonical forms.
