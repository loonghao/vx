---
name: vx-repo-contract
description: "Repository layout contract for vx-managed repos. Use when an agent enters a new repository, adds files to the repo root, edits vx.toml or justfile, or is asked to clean up / normalize repository structure. Rules: root file allowlist (R006), lowercase justfile (R002), no vx.toml [scripts] when a justfile exists (R007, blocks CI), AGENTS.md as single source of truth (R003/R008), no build artifacts at the root (R001). Severities are defined in dcc-mcp/.github contract/repo_contract.json."
---

# VX Repository Contract

> **One-sentence summary**: The repository root is a contract, not a scratch
> directory. Every rule below ships with a command that decides pass/fail, so
> the contract is enforceable in CI rather than argued about.

## Where the rules are defined

This skill is a **guide to** the contract, not the definition of it. The
machine-readable definitions — rule ids, severities, and profiles — live in
[`contract/repo_contract.json`](https://github.com/dcc-mcp/.github/blob/main/contract/repo_contract.json)
in the `dcc-mcp/.github` repository. The CI gate
(`scripts/check_repo_contract.py`) and this skill read that same file, so the
pass/fail criteria are stated exactly once.

The five rules below are the ones an agent hits while editing a repository, and
each is annotated with its contract id. The contract carries further rules that
are swept nightly and need no day-to-day guidance; when you need the full list,
read the JSON rather than inventing a sixth rule here.

| This skill | Contract id | Severity |
|---|---|---|
| Rule 1 — root file allowlist | R006 (`root-allowlist`) | warning (`strict`) |
| Rule 2 — lowercase `justfile` | R002 (`justfile-lowercase`) | error |
| Rule 3 — no `[scripts]` with a justfile | R007 (`no-scripts-with-justfile`) | **error** |
| Rule 4 — `AGENTS.md` is the source of truth | R003 (`agents-md-exists`), R008 (`agents-derived-symlink`) | error / warning |
| Rule 5 — no build artifacts at the root | R001 (`no-root-artifacts`) | error |

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
| Agent contract | `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `.cursorrules`, `.windsurfrules`, `.clinerules` |
| Tool config | `vx.toml`, `vx.lock`, `justfile`, `rust-toolchain.toml`, `Cargo.toml`, `Cargo.lock`, `package.json`, `pyproject.toml`, `.pre-commit-config.yaml` |
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

> Contract id: **R002** (`justfile-lowercase`), severity `error`, profiles
> `baseline` + `strict`.
>
> The rule text, severity, and profiles above are **defined in
> [`contract/repo_contract.json`](https://github.com/dcc-mcp/.github/blob/main/contract/repo_contract.json)
> in `dcc-mcp/.github`**, not in this file. That JSON is the single source of
> truth: `scripts/check_repo_contract.py` (the CI gate) and this skill read the
> same definitions. Treat the id and the severity quoted here as a pointer —
> when they disagree with the JSON, the JSON wins and this file is stale.

`just` accepts `justfile`, `Justfile`, and `.justfile`. Pick **lowercase
`justfile`** and reject the others — case differences break recipe discovery on
case-sensitive filesystems and split conventions across platforms.

**Mechanical check** — exactly one match, and it is `justfile`:

```bash
vx git ls-files | vx rg -i '^\.?justfile$'   # must print exactly: justfile
vx rg -n '^\[scripts\]' vx.toml              # must print nothing (see Rule 3)
```

## Rule 3 — `vx.toml` declares tool versions; `justfile` declares tasks

> Contract id: **R007** (`no-scripts-with-justfile`), severity **`error`**,
> profiles `baseline` + `strict`. Source: the anti-drift cleanup that removed
> 255 mirrored `[scripts]` entries.
>
> Severity and profiles are **defined in
> [`contract/repo_contract.json`](https://github.com/dcc-mcp/.github/blob/main/contract/repo_contract.json)
> in `dcc-mcp/.github`** — the single source of truth shared with the CI gate.
> This rule is an **error on the default `baseline` profile**: adding a
> `[scripts]` entry to a repository that already has a `justfile` fails the gate
> immediately, with one annotation per script naming the duplicated task. It is
> not a warning you can accumulate.
>
> Related ids in the same contract: R004 (`vx-toml-parses`) and R005
> (`tools-version-format`) govern what the `[tools]` table may contain.

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
if [ -f justfile ] && vx rg -q '^\[scripts\]' vx.toml; then
  echo "FAIL: [scripts] duplicates justfile (R007)"
  exit 1
fi
```

> `test -f justfile && rg -q ... && echo FAIL` is wrong: when `rg` finds nothing
> it exits non-zero, the `&&` chain stops, and — worse — the whole line exits
> non-zero, so under `set -e` a *passing* repository fails the check. Use an
> `if` so that "no `[scripts]`" is a pass rather than a script error.

Pick one per repository:

| Situation | Use |
|-----------|-----|
| `justfile` present | Add recipes there. No `[scripts]` in `vx.toml`. |
| No `justfile`, few tasks | `[scripts]` in `vx.toml` is fine. |
| No `justfile`, many tasks | Create a `justfile` and keep `vx.toml` version-only. |

**Rust is a special case, and it is owned by another rule set.** Rust has two
independent version namespaces: `rust-toolchain.toml` names the *toolchain*
(`rustc 1.98.1`), while `[tools] rust` in `vx.toml` is resolved against *rustup
release* numbers. A pin written in the wrong namespace looks correct and does
nothing. For the full precedence table, the `rustup-managed` opt-out, and the
`RUSTUP_TOOLCHAIN` interaction, see the **Rust Version Resolution** section of
the **vx-project** skill — do not invent a second rule here.

The only mechanically checkable part that belongs in *this* contract: a
repository that pins Rust must declare its intent in exactly one place.

```bash
# both present → verify the vx.toml entry is not a silent duplicate pin
test -f rust-toolchain.toml && vx rg -n '^rust\s*=' vx.toml
```

If that prints a numeric pin (e.g. `rust = "1.98.1"`) next to a toolchain file,
the two are different namespaces and one of them is dead — resolve it per
**vx-project**, then remove the dead entry.

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
  [ -f "$f" ] || continue                       # absent is fine
  vx rg -q 'AGENTS\.md' "$f" || echo "FAIL: $f does not point at AGENTS.md"
done
```

> `test -f "$f" && rg -q ... || echo FAIL` is wrong: when the file is absent the
> `&&` short-circuits and `||` fires anyway, so you get a failure for a file that
> does not exist. Guard with `continue` (or an `if`) instead.

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
vx cargo test 2>&1 | vx tee "$(mktemp -d)/test.log"   # log stays out of the repo
```

## Running the whole contract

The commands above are for a quick local answer. Do not reimplement them as a
second gate: `dcc-mcp/.github` already ships one, driven by the same
`contract/repo_contract.json` and covering all eleven rules rather than the five
an agent touches day to day. Adopt it with one caller file:

```yaml
# .github/workflows/repo-contract.yml
name: Repo contract
on:
  pull_request:
  push:
    branches: [main]
permissions:
  contents: read
jobs:
  contract:
    uses: dcc-mcp/.github/.github/workflows/repo-contract.yml@main
```

That runs the `baseline` profile, which is enough to fail a repository that adds
`[scripts]` next to a justfile (R007). Run it locally against a checkout with:

```bash
python scripts/check_repo_contract.py --root . --profile baseline --format text
```

In a repository that already has a `justfile`, the standalone checks are still
useful as a fast pre-push recipe:

```make
# justfile
contract-check:
    #!/usr/bin/env bash
    set -euo pipefail
    fail=0
    # 1. root allowlist (R006)
    out=$(git ls-files | rg -v '[/]' | rg -vix '<allowlist from Rule 1>') || true
    [ -z "$out" ] || { echo "root allowlist:"; echo "$out"; fail=1; }
    # 2. justfile name (R002)
    [ "$(git ls-files | rg -i '^\.?justfile$')" = "justfile" ] || { echo "justfile casing"; fail=1; }
    # 3. [scripts] must not duplicate the justfile (R007)
    if [ -f justfile ] && rg -q '^\[scripts\]' vx.toml; then
      echo "[scripts] duplicates justfile"; fail=1
    fi
    # 4. per-agent files must point at AGENTS.md (R008)
    for f in CLAUDE.md GEMINI.md .cursorrules .windsurfrules .clinerules; do
      [ -f "$f" ] || continue
      rg -q 'AGENTS\.md' "$f" || { echo "$f does not point at AGENTS.md"; fail=1; }
    done
    # 5. no build artifacts at the root (R001)
    out=$(git ls-files | rg -i '(^|[/])([^/]*\.(log|tmp|out|o|pyc)|coverage\.json|audit-result\.json)$|^(target|dist|build)[/]') || true
    [ -z "$out" ] || { echo "artifacts:"; echo "$out"; fail=1; }
    exit $fail
```

> Check 3 keeps the `if` form. As `[ -f justfile ] && rg -q ... && { ...; }`, a
> repository with *no* `[scripts]` makes `rg` exit 1, the `&&` chain
> short-circuits, and under `set -e` the recipe reports a failure on a clean
> repository — the one case the gate must never get wrong.

A contract nobody enforces is a preference, not a contract.

## Delivery Surface — External Systems Are Evidence

GitHub PRs, CI runs, and dashboards are **supporting evidence**, not the delivery
surface. The conclusion has to land where the work is tracked.

- Record status, branch/commit/PR, validation run, blocker, and next owner in one
  issue comment before ending a turn.
- Never treat a green CI run as proof the work shipped — verify the terminal
  state (merged / released / deployed) and record *that*.
- Keep internal routing, issue IDs, and local absolute paths off public GitHub
  surfaces; PR text stays technical and public-safe.

**Mechanical check** — a turn that touched code must end with exactly one issue
comment carrying status + commit/PR + validation + next owner. PR comments alone
fail the check.
