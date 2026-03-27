# vx — AI Agent Skills

This directory contains AI agent skills for **[vx](https://github.com/loonghao/vx)** — the universal development tool manager.

These skills are the **single source of truth** shared across:
- `vx ai setup` — embeds skills into the vx binary at compile time, distributes to 13+ AI agents
- **ClawHub** — published automatically via CI when changes merge to main
- **Agent config directories** — `.codebuddy/skills/`, `.claude/skills/`, `.cursor/skills/`, etc.

## Available Skills

| Skill | Description |
|-------|-------------|
| **vx-usage** | Core usage guide — commands, vx.toml, providers, GitHub Actions, containers |
| **vx-commands** | CLI command reference — all flags, output formats (`--json`, `--format toon`) |
| **vx-project** | Project management — init, sync, setup, vx.toml configuration, monorepo |
| **vx-best-practices** | Best practices — version strategy, cross-platform, security, migration |
| **vx-troubleshooting** | Troubleshooting — installation failures, PATH issues, diagnostics, recovery |

## Structure

```
skills/
├── README.md                         # This file
├── vx-usage/SKILL.md                 # Core usage guide (~14 KB)
├── vx-commands/SKILL.md              # CLI command reference (~6 KB)
├── vx-project/SKILL.md               # Project management (~6 KB)
├── vx-best-practices/SKILL.md        # Best practices (~9 KB)
└── vx-troubleshooting/SKILL.md       # Troubleshooting (~7 KB)
```

## Install

```bash
# Via vx (distributes to all AI agents)
vx ai setup

# Via ClawHub CLI
clawhub install loonghao/vx

# Or copy skills/ directory to your AI agent's skills directory
```

## When Skills Activate

The skills trigger when:
- The project contains `vx.toml` or `.vx/` directory
- The user mentions `vx`, tool version management, or cross-platform setup
- The user needs to manage development tool versions

## Links

- **vx GitHub**: https://github.com/loonghao/vx
- **ClawHub**: https://clawhub.ai/loonghao/vx
