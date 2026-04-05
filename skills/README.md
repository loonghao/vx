# vx — AI Agent Skills

This directory contains AI agent skills for **[vx](https://github.com/loonghao/vx)** — the universal development tool manager (v0.8.18).

> **Core concept**: vx = prefix any dev tool command with `vx` → it auto-installs the tool and runs it.

These skills are the **single source of truth** shared across:
- `vx ai setup` — embeds skills into the vx binary at compile time, distributes to 13+ AI agents
- **ClawHub** — published automatically via CI when changes merge to main
- **Agent config directories** — `.codebuddy/skills/`, `.claude/skills/`, `.cursor/skills/`, etc.

## Available Skills

| Skill | Description | Size | Best for |
|-------|-------------|------|----------|
| **vx-usage** | Core usage guide — commands, vx.toml, providers, GitHub Actions, MCP integration | ~15 KB | First-time users, general questions |
| **vx-commands** | CLI command reference — all flags, output formats (`--json`, `--format toon`) | ~6 KB | Looking up specific command syntax |
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
