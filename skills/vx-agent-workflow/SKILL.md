---
name: vx-agent-workflow
description: "Token-efficient command execution patterns for AI agents using vx. Use when running builds, tests, linting, GitHub operations, or any command that produces verbose output. Teaches agents to filter output cross-platform using vx-managed tools (vx rg, vx jq) instead of platform-specific syntax (Select-String, grep, findstr). Includes token measurement, savings tracking, and deep recipes for cargo, gh, pytest, and more."
---

# VX Agent Workflow — Token-Efficient Command Execution

> **Core principle**: Use `vx rg` as the universal cross-platform filter. Never use
> platform-specific tools like `Select-String` (PowerShell), `grep` (Unix), or
> `findstr` (cmd). One command, every OS.

## Why This Matters

AI agents pay per-token for both input and output. A full `cargo test` run can
produce 500+ lines when you only need the 5 lines showing failures. Agents that
dump raw output waste 95% of their context window on noise.

**The vx advantage**: vx auto-installs `rg` (ripgrep), `jq`, `fd`, and other
filtering tools on every platform. Use them instead of shell-specific builtins.

## Token Savings Per Operation (Measured)

Based on medium-sized Rust/Python/TypeScript projects:

| Operation | Frequency/30min | Raw tokens | Filtered | Savings |
|-----------|----------------|------------|----------|---------|
| `vx cargo test` | 5x | ~25,000 | ~2,500 | **-90%** |
| `vx cargo build` | 5x | ~15,000 | ~1,500 | **-90%** |
| `vx cargo clippy` | 3x | ~8,000 | ~800 | **-90%** |
| `vx uv run pytest` | 4x | ~8,000 | ~800 | **-90%** |
| `vx gh pr view` | 5x | ~3,000 | ~300 | **-90%** |
| `vx gh run view --log` | 3x | ~30,000 | ~1,500 | **-95%** |
| `vx git status` | 10x | ~3,000 | ~600 | **-80%** |
| `vx git diff` | 5x | ~10,000 | ~2,500 | **-75%** |
| `vx npm test` | 5x | ~20,000 | ~2,000 | **-90%** |
| `vx just quick` | 3x | ~40,000 | ~2,000 | **-95%** |
| **Session total** | | **~162,000** | **~12,500** | **-92%** |

## The Universal Filter Pattern

```bash
vx <command> 2>&1 | vx rg "<pattern>"
```

This works identically on:
- Windows PowerShell 5.1+ / 7+ (pwsh)
- Windows cmd.exe
- Linux Bash/Zsh
- macOS Bash/Zsh

### Why NOT platform-specific tools

| Platform | Native filter | Problem |
|----------|--------------|---------|
| PowerShell | `Select-String -Pattern "..." \| Select-Object -Last 30` | Verbose syntax, outputs objects not text, breaks on Unix |
| Bash/Zsh | `grep -E "..." \| tail -30` | Not available on Windows without WSL |
| cmd.exe | `findstr /R "..."` | Limited regex, no tail equivalent, no context lines |

**With vx, one line replaces all of them:**

```bash
vx <command> 2>&1 | vx rg -m 30 "pattern"
```

---

## Deep Recipe: `vx cargo` (Rust Build & Test)

### Build — Progressive Disclosure

```bash
# Level 1: Just pass/fail (2 tokens)
vx cargo build 2>&1 && echo "OK" || echo "FAIL"

# Level 2: Error count only (5 tokens)
vx cargo build 2>&1 | vx rg -c "^error"

# Level 3: Error messages only (~50-200 tokens)
vx cargo build 2>&1 | vx rg "^error\[" 

# Level 4: Errors + location (~200-500 tokens)
vx cargo build 2>&1 | vx rg "^error|^\s*--> "

# Level 5: Errors with context (~500-1000 tokens)
vx cargo build 2>&1 | vx rg -B 1 -A 3 "^error\["

# NEVER: Full raw output (5000-20000 tokens)
# vx cargo build  ← don't do this unless debugging a novel issue
```

### Test — Optimal Patterns

```bash
# Cheapest: Just the summary line
vx cargo test 2>&1 | vx rg "^test result:"
# Output: test result: ok. 47 passed; 0 failed; 0 ignored

# Failed tests only (names + messages)
vx cargo test 2>&1 | vx rg "FAILED|panicked|thread .+ panicked|assertion"

# Specific crate (80% faster, 80% fewer tokens)
vx cargo test -p vx-cli 2>&1 | vx rg "test result:|FAILED"

# Specific test by name
vx cargo test -p vx-cli test_parse_args 2>&1 | vx rg "test result:|FAILED|panicked"

# JSON format for structured parsing (nightly or with --format)
vx cargo test -- -Z unstable-options --format json 2>&1 | vx jq -c 'select(.event == "failed")'
```

### Clippy — Warnings & Errors

```bash
# Count issues (1 token answer)
vx cargo clippy 2>&1 | vx rg -c "^(warning|error)\["

# Issue types only
vx cargo clippy 2>&1 | vx rg -o "(warning|error)\[\w+\]" | sort | uniq -c

# Full warnings with file location
vx cargo clippy 2>&1 | vx rg "^(warning|error)\[|^\s*--> "

# Specific lint only
vx cargo clippy 2>&1 | vx rg "unused_variable|dead_code"
```

### Build + Test Combined (`vx just quick`)

```bash
# Optimal: capture only final results from the full pipeline
vx just quick 2>&1 | vx rg "^(error|warning)\[|test result:|FAILED|Finished|could not compile"

# If expecting success (confirmation only)
vx just quick 2>&1 | vx rg -m 5 "Finished|test result: ok|All checks passed"
```

### Tee Pattern — Save Full Output for Failure Recovery

When a command fails, save full output to a file so you can dig deeper without
re-running (inspired by rtk's tee feature):

```bash
# Run once, save everything, show only errors
vx cargo test 2>&1 | vx tee .vx-last-output.log | vx rg "FAILED|error|panicked"

# If filtered output is insufficient, read the saved log
vx rg -C 5 "FAILED" .vx-last-output.log

# Alternative: redirect to file + filter separately
vx cargo test > .vx-last-output.log 2>&1; vx rg "test result:|FAILED" .vx-last-output.log
```

---

## Deep Recipe: `vx gh` (GitHub CLI)

### PR Operations — JSON Field Selection

The single most effective token-saving technique for `gh` is `--json` with field selection:

```bash
# BAD: Full PR view (500-2000 tokens of formatting)
vx gh pr view 123

# GOOD: Selected fields only (~50-100 tokens)
vx gh pr view 123 --json title,state,mergeable,reviewDecision,headRefName

# With inline jq filtering
vx gh pr view 123 --json files --jq '.files[].path'

# PR list — compact
vx gh pr list --json number,title,state,headRefName --jq '.[] | "\(.number) \(.state) \(.title)"'
```

### CI Checks — Status Only

```bash
# BAD: Full checks output (1000+ tokens)
vx gh pr checks 123

# GOOD: Just name + conclusion (~100 tokens)
vx gh pr checks 123 --json name,state,conclusion --jq '.[] | "\(.name): \(.conclusion)"'

# Only failed checks
vx gh pr checks 123 --json name,state,conclusion --jq '.[] | select(.conclusion == "FAILURE") | .name'

# Quick pass/fail
vx gh pr checks 123 --json conclusion --jq 'all(.conclusion == "SUCCESS")'
```

### CI Run Logs — Filtered

CI logs are the #1 token-waster (10,000-100,000+ tokens). Always filter:

```bash
# NEVER: Full log dump
# vx gh run view 789 --log  ← 10000+ tokens

# GOOD: Only errors from the log
vx gh run view 789 --log 2>&1 | vx rg -m 50 "^error|FAILED|panic|fatal|Error:"

# Failed step only
vx gh run view 789 --log-failed 2>&1 | vx rg -m 50 "error|FAILED|panic"

# Just the conclusion
vx gh run view 789 --json conclusion,status --jq '.conclusion'

# List recent runs with status
vx gh run list --json databaseId,status,conclusion,headBranch -L 5 --jq '.[] | "\(.databaseId) \(.conclusion) \(.headBranch)"'
```

### Issues — Selective Fields

```bash
# Compact issue view
vx gh issue view 456 --json title,state,labels,body --jq '{title,state,labels: [.labels[].name]}'

# Issue list filtered
vx gh issue list --json number,title,state -L 10 --jq '.[] | "\(.number) \(.title)"'
```

### API Calls — Always Use --jq

```bash
# Direct API with jq projection
vx gh api repos/{owner}/{repo}/actions/runs --jq '.workflow_runs[:3] | .[] | {id,conclusion,head_branch}'

# PR files changed
vx gh api repos/{owner}/{repo}/pulls/123/files --jq '.[].filename'

# Commit status
vx gh api repos/{owner}/{repo}/commits/{sha}/status --jq '.state'
```

---

## Deep Recipe: Python Testing (`pytest`)

### Basic Patterns

```bash
# Cheapest: Just the summary line
vx uv run pytest 2>&1 | vx rg "passed|failed|error" | vx rg "==="
# Output: === 42 passed, 1 failed in 3.21s ===

# Failed test names only
vx uv run pytest 2>&1 | vx rg "^FAILED "

# Short test summary (pytest's own -r flag)
vx uv run pytest --tb=no -q 2>&1
# Output: 42 passed, 1 failed in 3.21s

# Failed with short traceback
vx uv run pytest --tb=short 2>&1 | vx rg -A 5 "^FAILED|^E "

# Only run tests matching a pattern (fastest)
vx uv run pytest -k "test_parse" --tb=short -q 2>&1
```

### pytest Native Flags That Save Tokens

Use pytest's own flags before piping — they reduce output at source:

```bash
# --tb=no: No tracebacks (just pass/fail)
vx uv run pytest --tb=no -q 2>&1

# --tb=line: One-line per failure
vx uv run pytest --tb=line 2>&1

# --tb=short: Short traceback
vx uv run pytest --tb=short 2>&1

# -q / --quiet: Minimal output
vx uv run pytest -q 2>&1

# --no-header: Skip the pytest header
vx uv run pytest --no-header -q 2>&1

# -x: Stop at first failure (don't waste time on cascade)
vx uv run pytest -x --tb=short 2>&1

# Combine for maximum efficiency
vx uv run pytest --no-header --tb=line -q 2>&1
# Output: FAILED tests/test_auth.py::test_login - AssertionError
#         1 failed, 41 passed in 2.1s
```

### pytest JSON Output

```bash
# With pytest-json-report plugin
vx uv run pytest --json-report --json-report-file=- 2>&1 | vx jq '{summary: .summary, failed: [.tests[] | select(.outcome == "failed") | .nodeid]}'

# With JUnit XML
vx uv run pytest --junitxml=- 2>&1 | vx rg "failures=|errors="
```

### Coverage — Summary Only

```bash
# BAD: Full coverage report (1000+ tokens)
# vx uv run pytest --cov

# GOOD: Just the total
vx uv run pytest --cov --cov-report=term-missing 2>&1 | vx rg "^TOTAL|^FAILED|passed"

# Even better: Just the percentage
vx uv run pytest --cov 2>&1 | vx rg "^TOTAL" | vx rg -o "\d+%"
```

---

## Deep Recipe: `vx git` (Version Control)

### Progressive Disclosure for Diffs

```bash
# Level 0: Any changes? (1 token)
vx git diff --quiet && echo "clean" || echo "dirty"

# Level 1: File names only (~10-30 tokens)
vx git diff --name-only

# Level 2: Stats (insertions/deletions per file, ~50-100 tokens)
vx git diff --stat

# Level 3: Specific file diff
vx git diff -- src/specific_file.rs

# Level 4: Full diff (only when you truly need it)
vx git diff
```

### Status — Compact Forms

```bash
# Compact status
vx git status --short --branch
# Output: ## main...origin/main
#          M src/lib.rs
#         ?? new_file.rs

# Count of changes only
vx git status --short | vx rg -c "."
```

### Log — Bounded

```bash
# One-line format, limited
vx git log --oneline -10

# Diff stat for recent commits
vx git log --oneline --stat -3

# Files changed in a range
vx git diff --name-only origin/main...HEAD

# Commit messages in a range
vx git log --oneline origin/main...HEAD
```

---

## vx Built-in Compact Mode (`--compact` / `-u`)

vx has a **built-in RTK-style compact mode** that automatically filters subprocess
output without manual piping. This is the easiest way to save tokens:

```bash
# Compact mode — vx automatically filters build/test/git output
vx --compact cargo test          # Shows only: test result + failures
vx --compact cargo build         # Shows only: errors + final status
vx --compact git status          # Shows only: short status
vx --compact gh pr view 123      # Shows only: essential fields
vx -u cargo test                 # -u is shorthand for --compact

# Filter aggressiveness levels
vx --compact --filter-level light cargo test      # Light filtering
vx --compact --filter-level normal cargo test     # Default
vx --compact --filter-level aggressive cargo test # Maximum compression

# Set globally for entire session
export VX_OUTPUT=compact
vx cargo test                    # Now always compact
vx cargo build                   # Always compact
vx git status                    # Always compact
```

### Compact vs Manual Pipe — When to Use Which

| Approach | Use when | Example |
|----------|----------|---------|
| `vx --compact <cmd>` | Quick runs, standard filters are good enough | `vx --compact cargo test` |
| `<cmd> 2>&1 \| vx rg "pattern"` | Need custom filter pattern | `vx cargo test 2>&1 \| vx rg "my_test\|FAILED"` |
| `--json` / `--jq` | Command has native JSON support | `vx gh pr view N --json title,state` |

### The `--compact` Mode Tracks Token Savings Automatically

Every command run with `--compact`, `--output-format toon`, or `--json` is
automatically measured. Check cumulative savings with:

```bash
vx metrics tokens
vx metrics tokens --json    # For machine consumption
```

---

## Token Measurement & Statistics

### Built-in: `vx metrics tokens`

vx has **built-in token savings tracking** (similar to `rtk gain`). Every time you
use `--output-format toon`, `--compact`, or `--json`, vx automatically records
baseline vs actual token counts. Query the data with:

```bash
# Terminal table — shows savings per command
vx metrics tokens

# Output:
# Token savings summary
# runs:12 records:15 baseline:4500 actual:1200 net_saved:3300 (73.3%)
#
# Command                              Runs   Before    After  Net saved  Saved%
# vx list                                 5     2000      400      1600   80.0%
# vx cargo test                           3     1500      300      1200   80.0%
# vx check                                4     1000      500       500   50.0%

# JSON format — for dashboards or further processing
vx metrics tokens --json

# Last N runs only
vx metrics tokens --last 20
```

### How Tracking Works

vx records a `TokenSavingsRecord` for every command that uses a structured output
format (toon, json, compact). Each record captures:

| Field | Description |
|-------|-------------|
| `baseline_tokens` | Estimated tokens if output were unformatted text |
| `actual_tokens` | Actual tokens in the structured/filtered output |
| `token_delta` | Positive = tokens saved |
| `savings_ratio` | Fraction saved (0.0–1.0) |

Token estimation: **1 token ≈ 4 UTF-8 bytes** (heuristic, matches GPT/Claude tokenizers within ~10%).

### Enabling Token Tracking

Token savings are tracked automatically when you use vx's built-in output modes:

```bash
# These automatically track savings
vx list --output-format toon           # TOON format (40-60% savings)
vx check --json                         # JSON format
vx --compact cargo test                 # Compact wrapper

# Set globally to track all commands
export VX_OUTPUT=toon
```

### Manual Measurement (for piped commands)

For commands piped through `vx rg`, track savings manually:

```bash
# Quick comparison: raw vs filtered
vx cargo test > /tmp/vx-raw.log 2>&1
vx rg "test result:|FAILED" /tmp/vx-raw.log > /tmp/vx-filtered.log
echo "Raw: $(wc -c < /tmp/vx-raw.log) bytes → Filtered: $(wc -c < /tmp/vx-filtered.log) bytes"

# Or one-liner (runs command twice — use for benchmarking only)
echo "Saved: $(echo "scale=1; 100 - $(vx cargo test 2>&1 | vx rg 'test result:|FAILED' | wc -c) * 100 / $(vx cargo test 2>&1 | wc -c)" | bc)%"
```

### AI Summary (For Agent Introspection)

```bash
# Full metrics with AI-friendly JSON summary
vx metrics --json

# Includes timing, stages, and token savings
# Useful for agents to self-optimize their command patterns
```

### HTML Report (For Human Review)

```bash
# Generate visual report of all metrics including token savings
vx metrics --html report.html
```

### Expected Savings by Command Category

| Category | Raw output | Optimal filter | Tokens saved | Technique |
|----------|-----------|----------------|-------------|-----------|
| **Build (cargo/go/tsc)** | 5K-20K | 50-500 | 90-99% | `vx rg "^error"` |
| **Test (cargo/pytest/jest)** | 2K-25K | 50-200 | 90-99% | `vx rg "test result:\|FAILED"` |
| **Lint (clippy/ruff/eslint)** | 2K-10K | 100-500 | 85-95% | `vx rg "^(error\|warning)\["` |
| **Git operations** | 1K-10K | 100-500 | 75-90% | `--stat`, `--name-only`, `--short` |
| **GitHub CLI** | 500-5K | 50-200 | 80-95% | `--json field1,field2 --jq` |
| **CI logs** | 10K-100K | 200-1000 | 95-99% | `--log-failed \| vx rg -m 50` |
| **File listing** | 500-5K | 50-200 | 80-95% | `vx fd` with type filters |

---

## Advanced Patterns

### The Tee-on-Failure Pattern

Save full output only when commands fail (zero cost on success):

```bash
# Bash/Zsh/PowerShell compatible
vx cargo test 2>&1 | vx rg "test result:|FAILED|panicked"; if [ $? -ne 0 ]; then vx cargo test 2>&1 > .vx-debug.log && echo "[full log: .vx-debug.log]"; fi

# Simpler: always tee, only read on failure
vx cargo test > .vx-last.log 2>&1; EXIT=$?; vx rg "test result:|FAILED" .vx-last.log; if [ $EXIT -ne 0 ]; then echo "[debug: .vx-last.log]"; fi
```

### Deduplication — Collapse Repeated Lines

```bash
# Sort + unique count (great for repeated warnings)
vx cargo clippy 2>&1 | vx rg "^warning\[" | sort | uniq -c | sort -rn

# Output:
#   12 warning[dead_code]
#    5 warning[unused_import]
#    2 warning[unused_variable]
```

### Cascading Filters — Cheap to Expensive

Always start with the cheapest check and only expand if needed:

```bash
# 1. Pass/fail? (2 tokens)
vx cargo test 2>&1 > /dev/null && echo "PASS" || echo "FAIL"

# 2. If FAIL → how many failures? (5 tokens)
vx rg -c "FAILED" .vx-last.log

# 3. If > 0 → which tests? (~50 tokens)
vx rg "FAILED" .vx-last.log

# 4. If unclear → context around failures (~200 tokens)
vx rg -B 2 -A 5 "FAILED" .vx-last.log

# 5. Only if truly stuck → full section
vx rg -A 30 "failures:" .vx-last.log
```

### JSON Projection for Structured Commands

Many tools support JSON output natively — use it with `vx jq`:

```bash
# cargo metadata — just workspace members
vx cargo metadata --format-version=1 2>&1 | vx jq '[.workspace_members[]]'

# npm — just outdated packages
vx npm outdated --json 2>&1 | vx jq 'to_entries | .[] | "\(.key): \(.value.current) → \(.value.latest)"'

# pytest json report
vx uv run pytest --json-report --json-report-file=- 2>&1 | vx jq '.summary'
```

---

## Platform-Specific Notes

### Stderr Redirection (`2>&1`)

The `2>&1` syntax works in all shells (PowerShell, Bash, cmd). It merges stderr
into stdout so `vx rg` can filter both streams. Always include it for build/test
commands that may emit errors on stderr.

### PowerShell Pipe Encoding

PowerShell may convert pipe output to UTF-16. If `vx rg` shows garbled output:

```powershell
$env:PYTHONIOENCODING="utf-8"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
```

Or use vx's subprocess environment which handles encoding automatically.

### Long Commands on Windows

Windows cmd.exe has a ~8191 character limit. For complex filter patterns:

```bash
# Put pattern in a file
echo "error|FAILED|panicked|could not compile" > .vx-filter.txt
vx cargo test 2>&1 | vx rg -f .vx-filter.txt
```

---

## Decision Tree for Agents

```
Need to run a command?
│
├─ Is this a vx native command (list, check, versions)?
│  └─ Use --json or --output-format toon
│
├─ Is this `vx gh` (GitHub CLI)?
│  └─ ALWAYS use --json field1,field2 --jq 'projection'
│
├─ Is this a test command?
│  ├─ cargo test → vx cargo test -p <crate> 2>&1 | vx rg "test result:|FAILED"
│  ├─ pytest     → vx uv run pytest --no-header --tb=line -q 2>&1
│  ├─ jest       → vx npm test 2>&1 | vx rg "Tests:|FAIL"
│  └─ go test    → vx go test ./... 2>&1 | vx rg "^(ok|FAIL)"
│
├─ Is this a build command?
│  └─ vx cargo build 2>&1 | vx rg "^error|could not compile"
│
├─ Is this a lint command?
│  └─ vx cargo clippy 2>&1 | vx rg "^(error|warning)\["
│
├─ Will output be small (<20 lines)?
│  └─ Run directly: vx <command>
│
├─ Will output be large or unknown?
│  ├─ Just pass/fail? → && echo "PASS" || echo "FAIL"
│  ├─ Error details?  → 2>&1 | vx rg "error|FAILED|panic"
│  ├─ Count only?     → 2>&1 | vx rg -c "pattern"
│  └─ Context?        → 2>&1 | vx rg -C 3 "error|FAILED"
│
└─ CI log?
   └─ vx gh run view N --log-failed 2>&1 | vx rg -m 50 "error|FAIL"
```

---

## Anti-Patterns

```bash
# BAD: Platform-specific filtering
vx just test 2>&1 | Select-String -Pattern "FAILED"          # PowerShell only
vx just test 2>&1 | grep -E "FAILED"                         # Unix only
vx just test 2>&1 | findstr /R "FAILED"                      # cmd only

# GOOD: Universal with vx
vx just test 2>&1 | vx rg "FAILED"                           # Everywhere

# BAD: Dumping full verbose output
vx cargo test                                                  # 500+ lines
vx gh run view 789 --log                                      # 10000+ lines
vx gh pr view 123                                             # 500+ tokens of formatting
vx uv run pytest                                              # 200+ lines

# GOOD: Filtered to actionable information
vx cargo test 2>&1 | vx rg "test result:|FAILED"             # 2-5 lines
vx gh run view 789 --log-failed 2>&1 | vx rg -m 30 "error"  # ≤30 lines
vx gh pr view 123 --json title,state,mergeable               # ~30 tokens
vx uv run pytest --no-header --tb=line -q 2>&1               # 2-10 lines

# BAD: Full coverage / dependency trees
vx uv run pytest --cov                                        # 100+ lines table
vx npm list                                                    # huge tree

# GOOD: Summary extraction
vx uv run pytest --cov 2>&1 | vx rg "^TOTAL"                 # 1 line
vx npm list --depth=0 2>&1                                    # top-level only

# BAD: Multiple platform-aware branches
if ($IsWindows) { ... | Select-String ... } else { ... | grep ... }

# GOOD: One command everywhere
vx <cmd> 2>&1 | vx rg "pattern"
```

---

## Quick Reference Card

| Task | Token-efficient command | ~Tokens |
|------|----------------------|---------|
| Rust test pass/fail | `vx cargo test 2>&1 && echo OK \|\| echo FAIL` | 2 |
| Rust test summary | `vx cargo test 2>&1 \| vx rg "test result:"` | 10 |
| Rust test failures | `vx cargo test 2>&1 \| vx rg "FAILED\|panicked"` | 20-100 |
| Rust build errors | `vx cargo build 2>&1 \| vx rg "^error"` | 20-200 |
| Clippy count | `vx cargo clippy 2>&1 \| vx rg -c "^(error\|warning)\["` | 2 |
| Clippy issues | `vx cargo clippy 2>&1 \| vx rg "^(error\|warning)\["` | 50-200 |
| pytest summary | `vx uv run pytest --no-header --tb=line -q 2>&1` | 10-50 |
| pytest failures | `vx uv run pytest --tb=short -q 2>&1 \| vx rg "FAILED\|^E "` | 50-200 |
| Node test summary | `vx npm test 2>&1 \| vx rg "Tests:\|FAIL"` | 10-50 |
| Go test summary | `vx go test ./... 2>&1 \| vx rg "^(ok\|FAIL)"` | 10-50 |
| Just quick-check | `vx just quick 2>&1 \| vx rg "error\|FAILED\|Finished"` | 20-100 |
| Git changes | `vx git diff --name-only` | 10-30 |
| Git status | `vx git status --short --branch` | 10-50 |
| PR status | `vx gh pr view N --json title,state,mergeable` | 30-50 |
| PR files | `vx gh pr view N --json files --jq '.files[].path'` | 20-50 |
| CI status | `vx gh pr checks N --json name,conclusion --jq '...'` | 30-100 |
| CI errors | `vx gh run view N --log-failed 2>&1 \| vx rg -m 30 "error"` | 100-500 |
| Search code | `vx rg -n -m 20 "pattern" src/` | 50-200 |
| Find files | `vx fd "pattern" --type f` | 10-50 |
