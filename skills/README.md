# vx — AI Agent Skills

This directory contains AI agent skills for **[vx](https://github.com/vx-org/vx)** — the universal development tool manager (v0.9.4).

> **Core concept**: vx = prefix any dev tool command with `vx` → it auto-installs the tool and runs it.

These skills are the **single source of truth** shared across:
- `vx ai setup` — embeds skills into the vx binary at compile time, distributes to 17+ AI agents
- **ClawHub** — published automatically via CI when changes merge to main
- **Agent config directories** — `.codebuddy/skills/`, `.claude/skills/`, `.cursor/skills/`, etc.

## Management Model

Edit `skills/` first. Treat project-level agent directories (`.agents/`, `.claude/`,
`.cursor/`, and friends) as compatibility snapshots generated from the canonical
skills, not as independently maintained sources.

When a skill changes:

1. Update the canonical file under `skills/<name>/SKILL.md`.
2. Re-run or refresh the `vx ai setup` distribution path for agent-specific copies.
3. Keep embedded skills and ClawHub publishing aligned through the existing
   `crates/vx-cli/src/commands/ai.rs` embedding and `.github/workflows/sync-skills.yml`.

This keeps the repository from becoming a maze of divergent skill copies while
still supporting agents that require project-local skill folders.

## Skill Authoring Principles

vx skills should teach agents to be precise, scoped, and token-aware:

- Prefer the smallest maintainable change that solves the actual request.
- Read the narrowest relevant file, symbol, diff, log, or test output first.
- Scope command output before printing it; use `vx rg`, `vx fd`, `vx gh --json`,
  `vx gh --jq`, `vx jq`, `vx --compact`, and `vx metrics tokens` to keep context useful.
- Prefer semantic reduction first (`--json`, selected fields, `--jq`, `--toon`);
  use explicit `--compact` for broad subprocess logs after structured and grep-style
  views are insufficient.
- Avoid broad repo dumps, full logs, unrelated cleanup, and single-use wrappers.
- Validate according to risk with the cheapest useful focused check first.
- Treat `skills/` as the canonical source and project-level skill directories as
  generated compatibility snapshots.

## Available Skills

| Skill | Description | Size | Best for |
|-------|-------------|------|----------|
| **vx-usage** | Core usage guide — commands, vx.toml, providers, GitHub Actions, MCP integration | ~15 KB | First-time users, general questions |
| **vx-commands** | CLI command reference — flags, forwarding, and output formats (`--json`, `--toon`, `--compact`) | ~6 KB | Looking up specific command syntax |
| **vx-project** | Project management — init, sync, setup, vx.toml configuration, monorepo | ~6 KB | Setting up or configuring projects |
| **vx-repo-contract** | Repository layout contract — root allowlist, lowercase `justfile`, `vx.toml` keeps `[tools]`, `AGENTS.md` as single source of truth, no artifacts at the root | ~7 KB | Entering a new repo, cleaning up repo structure |
| **vx-best-practices** | Best practices — version strategy, cross-platform, security, provider development | ~10 KB | Team workflows, provider creation |
| **vx-troubleshooting** | Troubleshooting — installation failures, PATH issues, diagnostics, recovery | ~8 KB | Fixing errors, diagnosing issues |
| **vx-agent-workflow** | Token-efficient command execution — cross-platform filtering with `vx rg`, output reduction patterns | ~8 KB | Agents running builds/tests/lints |
| **worktrunk** | Git worktree manager for parallel agent workflows (`vx wt`) | ~8 KB | Running several agents on one repo |

## Structure

```
skills/
├── README.md                          # This file
├── vx-usage/SKILL.md                  # Core usage guide (~15 KB)
├── vx-commands/SKILL.md               # CLI command reference (~6 KB)
├── vx-project/SKILL.md                # Project management (~6 KB)
├── vx-repo-contract/SKILL.md          # Repository layout contract (~7 KB)
├── vx-best-practices/SKILL.md         # Best practices (~10 KB)
├── vx-troubleshooting/SKILL.md        # Troubleshooting (~8 KB)
├── vx-agent-workflow/SKILL.md         # Token-efficient execution (~8 KB)
└── worktrunk/SKILL.md                 # Git worktree manager (~8 KB)
```

## Global vs Project Scope

Skills are **global by default**. `vx ai setup` installs them once per machine
into each agent's home-directory skills folder and records the hash in
`~/.vx/ai-skills.toml`. A repository should not carry copies of them.

| Scope | Command | Skills land in | Hash recorded in |
|-------|---------|----------------|------------------|
| Global (default) | `vx ai setup` | `~/.<agent>/skills/` | `~/.vx/ai-skills.toml` |
| Project (opt-in) | `vx ai setup --project` | `./.<agent>/skills/` | `vx.toml` → `[ai].skills_hash` |

Verify both scopes and converge a repository onto the global install:

```bash
vx ai check          # report drift in both scopes
vx ai check --fix    # refresh stale copies, drop redundant project copies
```

`--fix` only removes project-local copies that are **byte-identical** to the
embedded skill, so no unique content can be lost. Copies that were modified
locally are reported as `divergent` and left alone for manual review.

Repositories that need their own skills should ship **incremental** skills under
a namespace that does not collide with the global `vx-*` set — domain knowledge
belongs in the repo, shared tooling conventions do not.

## Install

```bash
# Via vx (distributes to all AI agents)
vx ai setup

# Via ClawHub CLI
clawhub install loonghao/vx

# Or copy skills/ directory to your AI agent's skills directory
```

## CI Publishing to ClawHub

The repository publishes `skills/` to ClawHub through `.github/workflows/sync-skills.yml`.

- Pushes to `main` that modify `skills/**` trigger an automatic publish
- Maintainers can also trigger the workflow manually with `workflow_dispatch`
- The repository secret `CLAWHUB_TOKEN` must be configured for the publish to succeed
- Failed ClawHub publishes are treated as workflow failures so main-branch sync issues are visible immediately

## When Skills Activate

The skills trigger when:
- The project contains `vx.toml` or `.vx/` directory
- The user mentions `vx`, tool version management, or cross-platform setup
- The user needs to manage development tool versions


### Skill Routing Guide

Use this decision tree to pick the right skill:

```
User's question:
├─ "How do I use vx?" / general usage
│  → vx-usage
├─ "What's the command for...?" / specific flag or syntax
│  → vx-commands
├─ "Set up my project" / vx.toml / monorepo
│  → vx-project
├─ "Where does this file belong?" / repo layout / vx.toml vs justfile
│  → vx-repo-contract
├─ "Best way to..." / team workflow / provider development
│  → vx-best-practices
├─ "Error: ..." / "not working" / "failed"
│  → vx-troubleshooting
├─ "How to filter build/test output?" / "save tokens" / "cross-platform command"
│  → vx-agent-workflow
├─ "MCP integration" / "GitHub Actions"
│  → vx-usage (has dedicated sections)
└─ "Add a new tool to vx"
   → vx-best-practices (provider development section)
```

| User's Question | Recommended Skill |
|-----------------|-------------------|
| "How do I use vx?" | vx-usage |
| "What's the command for...?" | vx-commands |
| "Set up my project with vx" | vx-project |
| "Where should this file go?" / repo cleanup | vx-repo-contract |
| "Should I add [scripts] or a just recipe?" | vx-repo-contract |
| "How do I pin the Rust version?" | vx-project (rust-toolchain section) |
| "What's the best way to...?" | vx-best-practices |
| "vx install failed" / "command not found" | vx-troubleshooting |
| "How do I filter test output?" / "save tokens" | vx-agent-workflow |
| "Cross-platform command syntax" | vx-agent-workflow |
| "How do I add a new tool to vx?" | vx-best-practices (provider dev section) |
| "Set up MCP with vx" | vx-usage (MCP integration section) |
| "Use vx in GitHub Actions" | vx-usage (GitHub Actions section) |

## Links

- **vx GitHub**: https://github.com/vx-org/vx
- **ClawHub**: https://clawhub.ai/loonghao/vx
- **AGENTS.md**: https://github.com/vx-org/vx/blob/main/AGENTS.md
- **llms.txt**: https://github.com/vx-org/vx/blob/main/llms.txt
