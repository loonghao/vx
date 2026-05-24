# vx — AI Agent Skills

This directory contains AI agent skills for **[vx](https://github.com/loonghao/vx)** — the universal development tool manager (v0.8.25).

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
  `vx jq`, and `vx metrics tokens` to keep context useful.
- Avoid broad repo dumps, full logs, unrelated cleanup, and single-use wrappers.
- Validate according to risk with the cheapest useful focused check first.
- Treat `skills/` as the canonical source and project-level skill directories as
  generated compatibility snapshots.

## Available Skills

| Skill | Description | Size | Best for |
|-------|-------------|------|----------|
| **vx-usage** | Core usage guide — commands, vx.toml, providers, GitHub Actions, MCP integration | ~15 KB | First-time users, general questions |
| **vx-commands** | CLI command reference — all flags, output formats (`--json`, `--output-format toon`) | ~6 KB | Looking up specific command syntax |
| **vx-project** | Project management — init, sync, setup, vx.toml configuration, monorepo | ~6 KB | Setting up or configuring projects |
| **vx-best-practices** | Best practices — version strategy, cross-platform, security, provider development | ~10 KB | Team workflows, provider creation |
| **vx-troubleshooting** | Troubleshooting — installation failures, PATH issues, diagnostics, recovery | ~8 KB | Fixing errors, diagnosing issues |

## Structure

```
skills/
├── README.md                         # This file
├── vx-usage/SKILL.md                 # Core usage guide (~15 KB)
├── vx-commands/SKILL.md              # CLI command reference (~6 KB)
├── vx-project/SKILL.md               # Project management (~6 KB)
├── vx-best-practices/SKILL.md        # Best practices (~10 KB)
└── vx-troubleshooting/SKILL.md       # Troubleshooting (~8 KB)
```

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
├─ "Best way to..." / team workflow / provider development
│  → vx-best-practices
├─ "Error: ..." / "not working" / "failed"
│  → vx-troubleshooting
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
| "What's the best way to...?" | vx-best-practices |
| "vx install failed" / "command not found" | vx-troubleshooting |
| "How do I add a new tool to vx?" | vx-best-practices (provider dev section) |
| "Set up MCP with vx" | vx-usage (MCP integration section) |
| "Use vx in GitHub Actions" | vx-usage (GitHub Actions section) |

## Links

- **vx GitHub**: https://github.com/loonghao/vx
- **ClawHub**: https://clawhub.ai/loonghao/vx
- **AGENTS.md**: https://github.com/loonghao/vx/blob/main/AGENTS.md
- **llms.txt**: https://github.com/loonghao/vx/blob/main/llms.txt
