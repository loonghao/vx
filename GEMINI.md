# GEMINI.md — vx Project Instructions for Google Gemini

> This file is read by Google Gemini (AlphaCoder) at the start of every conversation.
> All project instructions are in [AGENTS.md](AGENTS.md) — this file only adds Gemini-specific notes.

## Gemini Specifics

- Follow [AGENTS.md](AGENTS.md) exactly — it is the single source of truth for vx.
- Also reference [`llms.txt`](llms.txt) for concise LLM-friendly index, [`llms-full.txt`](llms-full.txt) for full documentation.
- Use Conventional Commits: `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`
- Run `vx just quick` before submitting PR.
- PRs target `main` branch.
- Project version: **v0.9.13** with **142 providers**.

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

## Google Gemini Agent-Specific Notes

- **Long context window**: Gemini's 1M+ token context enables full project analysis.
- **Structured reasoning**: Use `vx list --json` or `vx list --output-format toon` for AI-parseable output.
- **MCP integration**: Replace `npx`/`uvx` with `vx` in MCP configs — `"command": "vx", "args": ["npx", ...]`.
- **Worktree workflow**: Use `vx wt switch <branch>` for parallel agent worktrees.
- **When uncertain**: Read `AGENTS.md` first, then consult `docs/` for deeper details.
