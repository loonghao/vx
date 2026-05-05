# CLAUDE.md — vx Project Instructions for Claude Code

> This file is read by Claude Code at the start of every conversation.
> All project instructions are in [AGENTS.md](AGENTS.md) — this file only adds Claude Code-specific notes.

## Claude Code Specifics

- Follow `AGENTS.md` exactly.
- Use Conventional Commits: `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`
- Run `vx just quick` before submitting PR.
- PRs target `main` branch.

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
