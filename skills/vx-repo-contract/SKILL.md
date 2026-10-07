---
name: vx-repo-contract
description: "Apply the shared repository layout contract to vx-managed repos. Use when reviewing root files, vx.toml, justfile, agent instructions, or repository cleanup."
---

# VX Repository Contract

Use the shared checker for local and CI decisions. This skill explains the rules;
it does not implement a second contract.

## Where the rules are defined

Rule definitions, profiles, allowlists, and artifact patterns live in
[`contract/repo_contract.json`](https://github.com/dcc-mcp/.github/blob/main/contract/repo_contract.json)
in `dcc-mcp/.github`. Read the JSON from the tooling checkout before changing a
repository. The checker consumes that JSON; the summaries below are guidance.
If they disagree, follow the JSON and update this skill.

The examples and summaries below use tooling revision
`d98ad6e8b9f2444699b0b109b2dbfb1c3000c56e`. When updating it, keep the local
checkout, workflow reference, and workflow `ref` input on the same revision.
Read the complete contract for rules beyond these daily editing concerns.

| This skill | Contract id | Severity | Profiles |
|---|---|---|---|
| Rule 1 — root allowlist | R006 (`root-allowlist`) | warning; denied entries error | strict |
| Rule 2 — lowercase `justfile` | R002 (`justfile-lowercase`) | error | baseline, strict |
| Rule 3 — no duplicate scripts | R007 (`no-scripts-with-justfile`) | warning | strict |
| Rule 4 — agent instructions | R003 (`agents-md-exists`) / R008 (`agents-derived-symlink`) | error / warning | baseline + strict / strict |
| Rule 5 — root artifacts | R001 (`no-root-artifacts`) | error | baseline, strict |

`baseline` omits R006, R007, and R008. `strict` includes them. R007 and R008
findings are warnings by default; R006 also emits errors for `root_deny_globs`.
`--fail-on error` allows warnings. Use `--fail-on warning` to block on warnings,
or add and promote selected rules with `--rule` and `--error-rule`.

## Set up the shared checker locally

A caller workflow does not install the checker in the repository being checked.
From that repository, clone the tooling once into an unused sibling directory:

```bash
vx git clone --no-checkout https://github.com/dcc-mcp/.github.git ../contract-tooling
vx git -C ../contract-tooling checkout --detach d98ad6e8b9f2444699b0b109b2dbfb1c3000c56e
```

Keep the complete checkout: the checker imports other modules from `scripts/`.
The sibling location keeps tooling files out of the target repository's root.
If tooling already exists, verify its revision instead of cloning over it.

Run the default gate from the target repository:

```bash
vx python@3.12 ../contract-tooling/scripts/check_repo_contract.py \
  --root . --contract ../contract-tooling/contract/repo_contract.json \
  --profile baseline --fail-on error --format text
```

To check every rule and fail on warnings too:

```bash
vx python@3.12 ../contract-tooling/scripts/check_repo_contract.py \
  --root . --contract ../contract-tooling/contract/repo_contract.json \
  --profile strict --fail-on warning --format text
```

`--root` selects the repository under review; `--contract` selects the definition
from the tooling checkout. Exit 0 means no finding reaches the chosen threshold,
exit 1 means a finding reaches it, and exit 2 means the check could not run.

## Rule 1 — Root directory allowlist (R006)

Before adding a top-level entry, consult `root_allowlist`, `root_deprecated`, and
`root_deny_globs` in the JSON. R006 covers root files and directories. Entries
such as `.dockerignore`, `Makefile`, `uv.lock`, and `rust-toolchain` are allowed
by the shared contract; a smaller hand-written file list would reject them.

Keep temporary handoff notes, review transcripts, and reports outside the source
root. Check the result with the strict invocation above instead of maintaining
another allowlist regex.

## Rule 2 — Lowercase `justfile` (R002)

Use the contract's lowercase `justfile` convention. R002 reports capitalized
variants such as `Justfile`; the convention prefers `justfile` even where the
Runtime supports other filenames.

Put shared tasks in `justfile`; R007 below determines whether a `[scripts]`
entry duplicates one of those tasks.

## Rule 3 — Avoid duplicate task entry points (R007)

R007 checks each `[scripts]` entry when a `justfile` exists. It warns when the
entry forwards to `just <recipe>` or `vx just <recipe>`, or when its key matches
a recipe name with hyphens and underscores treated as equivalent. Independent
entry points are allowed; the existence of the table alone is not a violation.

For example, with a `test` recipe, `test = "vx just test"` duplicates that task.
An entry such as `serve = "vx node scripts/dev.mjs"` is allowed when `serve` is
not a recipe in the justfile.

R007 is a strict-profile warning, so the default baseline gate does not enforce
it. To enforce only this additional rule alongside baseline errors:

```bash
vx python@3.12 ../contract-tooling/scripts/check_repo_contract.py \
  --root . --contract ../contract-tooling/contract/repo_contract.json \
  --profile baseline --rule R007 --error-rule R007 --fail-on error --format text
```

For Rust version ownership and the `rustup-managed` declaration, see the
**Rust Version Resolution** section of the **vx-project** skill. R004 and R005
check TOML and supported version forms; read `tools_delegation_sentinels` in the
contract before changing a delegated Runtime declaration.

## Rule 4 — Canonical agent instructions (R003, R008)

R003 requires a root `AGENTS.md`; it is a baseline error even when no derived
agent files exist.

R008 permits absent derived files. Existing files listed in the contract's
`agents_derived_files` must be symlinks to `AGENTS.md` or generated files carrying
the configured `agents_provenance_marker`. A hand-edited file that merely
mentions `AGENTS.md` does not satisfy R008. Use the shared checker to verify the
symlink target or generation provenance.

Keep project instructions in `AGENTS.md`. Agent distribution directories such
as `.claude/skills/` are runtime-managed output; vx's canonical skill sources
live under `skills/`. R011 separately checks for tracked agent-directory files.

## Rule 5 — No build artifacts at the root (R001)

R001 applies the JSON's `forbidden_artifact_globs` to top-level files. It covers
coverage reports, shared libraries, object files, and platform metadata such as
`.DS_Store`, as well as named outputs such as `clippy_check.txt` and
`commit_msg.txt`. Nested fixtures are outside this root-only rule.

Keep run output in a temporary directory or the project's designated output
location. The strict root allowlist also checks which output directories may
appear at the root; use the shared checker for both decisions.

## Reuse the checker in CI and just

Adopt the same tooling revision through the reusable workflow:

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
    uses: dcc-mcp/.github/.github/workflows/repo-contract.yml@d98ad6e8b9f2444699b0b109b2dbfb1c3000c56e
    with:
      ref: d98ad6e8b9f2444699b0b109b2dbfb1c3000c56e
```

This runs `baseline` with `fail-on: error`. For all strict warnings to block CI,
set `profile: strict` and `fail-on: warning`. To enforce R007 with baseline,
set `rules: R007` and `error-rules: R007` instead.

After the local tooling setup, a repository can expose the same checker as a
recipe without copying its rules:

```make
# justfile
contract-check tooling="../contract-tooling" profile="baseline" fail_on="error":
    vx python@3.12 "{{tooling}}/scripts/check_repo_contract.py" --root . --contract "{{tooling}}/contract/repo_contract.json" --profile "{{profile}}" --fail-on "{{fail_on}}" --format text
```

Run `vx just contract-check` for baseline, or
`vx just contract-check ../contract-tooling strict warning` for strict enforcement.
The recipe fails when the checker fails; no shell regex gate or
`set -e` interpretation is needed.

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
