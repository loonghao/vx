# CLAUDE.md — vx Project Instructions for Claude Code

> This file is read by Claude Code at the start of every conversation.
> All project instructions are in [AGENTS.md](AGENTS.md) — this file only adds Claude Code-specific notes.

## Claude Code Specifics

- Follow [AGENTS.md](AGENTS.md) exactly — it is the single source of truth for vx.
- Also reference [`llms.txt`](llms.txt) for concise LLM-friendly index, [`llms-full.txt`](llms-full.txt) for full documentation.
- Use Conventional Commits: `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`
- Run `vx just quick` before submitting PR.
- PRs target `main` branch.
- Project version: **v0.9.5** with **142 providers**.

## Quick Reference

| Task | Command |
|------|---------|
| Full check | `vx just quick` |
| Format | `vx just fmt` |
| Lint | `vx just lint` |
| Test | `vx just test` |
| Build | `vx just build` |
| Single crate test | `vx cargo test -p <crate-name>` |

## Project Layout

```
vx-cli          → CLI entry point
vx-resolver    → Command resolution & execution
vx-runtime     → Tool installation & management
vx-starlark    → Starlark DSL engine
vx-providers/*  → Tool definitions (provider.star)
```

## Claude Code Agent-Specific Notes

- **Claude Code MCP**: When configuring MCP servers in `~/.vscode/mcp.json` or `.vscode/mcp.json`, use `vx` as the command: `"command": "vx", "args": ["npx", ...]`
- **Claude CLI**: Use `vx claude <prompt>` for CLI interaction (if available).
- **Token optimization**: Use `vx list --format toon` for token-optimized output (saves 40-60% tokens).
- **Worktree workflow**: Use `vx wt` commands for parallel agent worktrees.
- **Diagnostics**: Run `vx doctor` first when encountering errors.
