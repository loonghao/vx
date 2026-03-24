# vx — AI Agent Skill

This skill teaches AI coding agents (OpenClaw, Claude Code, Cursor, Copilot, etc.) how to use **[vx](https://github.com/loonghao/vx)** — the universal development tool manager.

## What is vx?

vx automatically installs and manages 50+ development tools (Node.js, Python/uv, Go, Rust, etc.) with zero configuration. Just prefix any command with `vx`:

```bash
vx node --version      # Auto-installs Node.js if needed
vx cargo build         # Auto-installs Rust if needed
vx npm test            # Uses project-pinned version
```

## What This Skill Provides

| File | Purpose |
|------|---------|
| `SKILL.md` | Main skill — comprehensive vx usage guide for AI agents |
| `vx-commands-reference.md` | Command quick reference with all flags and options |
| `vx-project-setup.md` | Project setup guide with vx.toml configuration examples |

## When This Skill Activates

The skill triggers when:
- The project contains `vx.toml` or `.vx/` directory
- The user mentions `vx`, tool version management, or cross-platform setup
- The user needs to manage development tool versions

## Install

```bash
# Via ClawHub CLI
clawhub install loonghao/vx

# Or manually: copy the skill folder to your AI agent's skills directory
```

## Links

- **vx GitHub**: https://github.com/loonghao/vx
- **ClawHub**: https://clawhub.ai/loonghao/vx
