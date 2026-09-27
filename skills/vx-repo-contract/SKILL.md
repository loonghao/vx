---
name: vx-repo-contract
description: "Repository layout contract for vx-managed repos. Use when an agent enters a new repository, adds files to the repo root, edits vx.toml or justfile, or is asked to clean up / normalize repository structure. Five mechanically checkable rules: root file allowlist, lowercase justfile, vx.toml keeps [tools] only, AGENTS.md as single source of truth, no build artifacts at the root."
---

# VX Repository Contract

> **One-sentence summary**: The repository root is a contract, not a scratch
> directory. Five rules below; every rule ships with a command that decides
> pass/fail, so the contract is enforceable in CI rather than argued about.

Use this skill when you:

- Enter an unfamiliar repository and need to know what belongs where.
- Are about to add a file to the repository root.
- Edit `vx.toml`, `justfile`, `rust-toolchain.toml`, or agent instruction files.
- Are asked to "clean up" or "normalize" repository structure.

## Rule 1 — Root directory file allowlist

A repository root holds **contract files only**. Everything else belongs in a
subdirectory (`docs/`, `scripts/`, `crates/`, `tests/`, …).

Allowed at the root:

| Category | Files |
|----------|-------|
| Agent contract | `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `.cursorrules`, `.windsurfrules`, `.clinerules`, `.github/copilot-instructions.md` |
| Tool config | `vx.toml`, `vx.lock`, `justfile`, `rust-toolchain.toml`, `Cargo.toml`, `package.json`, `pyproject.toml`, `.pre-commit-config.yaml` |
| Lint/format config | `.typos.toml`, `clippy.toml`, `.editorconfig`, `renovate.json`, `codecov.yml` |
| Release config | `release-please-config.json`, `.release-please-manifest.json`, `CHANGELOG.md` |
| VCS config | `.gitattributes`, `.gitignore`, `.gitmessage`, `.gitmodules` |
| Docs | `README.md`, `README_<lang>.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, `LICENSE`, `llms.txt`, `llms-full.txt` |
| Install/packaging | `install.sh`, `install.ps1`, `Dockerfile`, `action.yml`, `distribution.toml`, `Cross.toml` |

**Mechanical check** — any tracked root file outside this table fails:

```bash
vx git ls-files | vx rg -v '[/]' | vx rg -vix \
  '(AGENTS|CLAUDE|GEMINI)\.md|\.cursorrules|\.windsurfrules|\.clinerules|vx\.(toml|lock)|justfile|rust-toolchain\.toml|Cargo\.(toml|lock)|package(-lock)?\.json|pyproject\.toml|\.pre-commit-config\.yaml|\.typos\.toml|clippy\.toml|\.editorconfig|renovate\.json|codecov\.yml|release-please-config\.json|\.release-please-manifest\.json|CHANGELOG\.md|README.*\.md|CONTRIBUTING\.md|SECURITY\.md|CODE_OF_CONDUCT\.md|LICENSE|llms(-full)?\.txt|install.*\.(sh|ps1)|Dockerfile|action\.yml|distribution\.toml|Cross\.toml|\.gitattributes|\.gitignore|\.gitmessage|\.gitmodules'
```

Empty output = pass. Non-empty output = each printed line is a violation.

> Two flag traps, both of which silently turn the check into a no-op:
> - **`rg -E` is not "extended regex"** — it selects a text encoding and fails
>   with `unknown encoding: <pattern>`. `rg` already uses Rust regex syntax; pass
>   the pattern as a positional argument (or with `-e`).
> - **Never pass a bare `/` as a pattern under Git Bash on Windows.** MSYS path
>   conversion rewrites it to a Windows path, so `rg -v '/'` silently stops
>   filtering and the check "passes" on a repo full of violations. Use `[/]`.

**Why it matters**: handoff notes, review transcripts, and one-off reports
(`handoff-*.md`, `pr1013_review.md`, `subissue-*.md`) drift out of date the day
after they are written and are then read as authoritative by the next agent.
Put them in an issue comment or under `docs/`, never at the root.

## Rule 2 — `justfile` is lowercase, and it is the only task runner

`just` accepts `justfile`, `Justfile`, and `.justfile`. Pick **lowercase
`justfile`** and reject the others — case differences break recipe discovery on
case-sensitive filesystems and split conventions across platforms.

**Mechanical check** — exactly one match, and it is `justfile`:

```bash
vx git ls-files | vx rg -i '^\.?justfile$'   # must print exactly: justfile
vx rg -n '^\[scripts\]' vx.toml              # must print nothing (see Rule 3)
```

## Rule 3 — `vx.toml` declares tool versions; `justfile` declares tasks

`vx.toml` answers *which version of a tool*. `justfile` answers *how work gets
run*. Do not let them overlap.

```toml
# vx.toml — keep it about versions
[tools]
node = "22.11.0"
python = "3.11"

[settings]
auto_install = true
```

**Do not add `[scripts]` when a `justfile` exists.** A `[scripts]` entry that
just forwards to `just <recipe>` is a second source of truth for the same task:
change the recipe, and the script silently runs the old behaviour.

**Mechanical check** — fail if both exist:

```bash
test -f justfile && vx rg -q '^\[scripts\]' vx.toml && echo "FAIL: [scripts] duplicates justfile"
```

Pick one per repository:

| Situation | Use |
|-----------|-----|
| `justfile` present | Add recipes there. No `[scripts]` in `vx.toml`. |
| No `justfile`, few tasks | `[scripts]` in `vx.toml` is fine. |
| No `justfile`, many tasks | Create a `justfile` and keep `vx.toml` version-only. |

**Rust is a special case.** `rust-toolchain.toml` (read by `rustup`) wins over
`[tools] rust` for anything run through `rustup`, and `vx` resolves the Rust
provider through `rustup` release numbers, not `rustc` versions. Therefore:

- Pin Rust with `rust-toolchain.toml`, and leave `rust` **out** of `[tools]`.
- If you must list it, add a comment saying the pin is advisory only.
- Never expect `vx install rust@1.98.1` to reproduce what `rust-toolchain.toml`
  pins; they are different version namespaces.

```toml
# rust-toolchain.toml  ← authoritative
[toolchain]
channel = "1.98.1"
```

**Mechanical check** — fail if a bare `rust` pin exists next to a toolchain file:

```bash
test -f rust-toolchain.toml && vx rg -q '^rust\s*=' vx.toml && echo "FAIL: [tools] rust conflicts with rust-toolchain.toml"
```

## Rule 4 — `AGENTS.md` is the single source of truth

`AGENTS.md` is the cross-agent standard. Per-agent files (`CLAUDE.md`,
`GEMINI.md`, `.cursorrules`, `.windsurfrules`, `.clinerules`) are **pointers**,
not parallel rulebooks. A rule that lives in two files will diverge, and the
divergence is invisible.

A per-agent file may contain:

1. A pointer to `AGENTS.md` as the canonical contract.
2. Agent-specific *mechanics* only — hook paths, MCP wiring, flag syntax.

It may **not** restate project rules, command conventions, or review gates.

**Mechanical check** — every per-agent root file must reference `AGENTS.md`:

```bash
for f in CLAUDE.md GEMINI.md .cursorrules .windsurfrules .clinerules; do
  test -f "$f" && vx rg -q 'AGENTS\.md' "$f" || echo "FAIL: $f does not point at AGENTS.md"
done
```

**Reality check on agent directories**: `.claude/skills/`, `.cursor/rules/`,
`.codebuddy/skills/` and friends are runtime-managed distribution targets. Treat
them as generated output — never hand-edit them, and never `git add` them. The
canonical source lives elsewhere (for vx's own skills: `skills/`).

## Rule 5 — No build or run artifacts at the root

Logs, coverage reports, audit output, and object files are **run output**, not
source. They bloat diffs, cause merge conflicts, and get quoted as facts long
after they are stale.

Never commit to the root: `*.log`, `*.tmp`, `*.out`, `*.o`, `*.pyc`,
`coverage.json`, `audit-result.json`, `target/`, `dist/`, `build/`.

**Mechanical check** — must print nothing:

```bash
vx git ls-files | vx rg -i '(^|[/])([^/]*\.(log|tmp|out|o|pyc)|coverage\.json|audit-result\.json)$|^(target|dist|build)[/]'
```

Prefer redirecting output into a temp dir over letting tools write to the
working root:

```bash
vx cargo test 2>&1 | vx tee "$TEMP/test.log"   # log stays out of the repo
```

## Running the whole contract

```bash
# 1. root allowlist      2. justfile name    3. no [scripts] + justfile
# 4. AGENTS.md pointers  5. no artifacts
vx just contract-check   # if the repo exposes the recipe; otherwise run 1–5 above
```

In CI, run the five checks as one job and fail on any non-empty output. A
contract nobody enforces is a preference, not a contract.

## Project skills are incremental, never copies

Built-in vx skills (`vx-usage`, `vx-project`, `vx-commands`, …) are installed
**once, globally** by `vx ai setup`. A repository that commits its own copy has
two rulebooks that drift apart.

| Content | Where it lives |
|---------|----------------|
| Built-in vx skills | Global agent directories, installed by `vx ai setup` |
| Project-specific knowledge | `skills/<project>-<topic>/SKILL.md`, tracked in git |
| Agent directories (`.claude/skills/`, `.agents/skills/`, …) | Generated output — never hand-edited, never committed |

Rules for a project-owned skill:

1. Namespace it — never start a project skill with `vx-`, and never reuse a
   built-in name. That prefix is how `vx ai check` tells a copy from an original.
2. Keep it additive — document what the project knows that vx does not (domain
   pitfalls, DCC specifics, internal APIs). Do not restate vx command syntax.
3. Never commit agent directories. They are distribution targets.

**Mechanical check** — must print nothing:

```bash
vx ai check                  # reports local copies of built-in skills as drift
vx ai check --fix            # removes them and refreshes the global install
```

## Delivery surface — external systems are evidence, not the answer

GitHub PRs, CI runs, and dashboards are **supporting evidence**, not the delivery
surface. The conclusion has to land where the work is tracked.

- Record status, branch/commit/PR, validation run, blocker, and next owner in one
  issue or task comment before ending a turn.
- Never treat a green CI run as proof the work shipped — verify the terminal
  state (merged / released / deployed) and record *that*.
- Keep internal routing, issue IDs, and local absolute paths off public GitHub
  surfaces; PR text stays technical and public-safe.

**Mechanical check** — a turn that touched code ends with exactly one task-system
comment carrying status + commit/PR + validation + next owner. A PR comment alone
fails the check.
