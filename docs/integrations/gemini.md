# Google Gemini notes

> Content preserved from the deleted root `GEMINI.md`.
> Project-wide rules live in [`AGENTS.md`](https://github.com/vx-org/vx/blob/main/AGENTS.md) and
> [`docs/CONVENTIONS.md`](../CONVENTIONS.md). Only Gemini-specific notes belong
> here.

## Setup

- Follow [`AGENTS.md`](https://github.com/vx-org/vx/blob/main/AGENTS.md) exactly — it is the single source of truth.
- [`llms.txt`](../../llms.txt) — concise LLM-friendly project index.
- [`llms-full.txt`](../../llms-full.txt) — full LLM documentation.

## Gemini specifics

- **Long context window** — Gemini's 1M+ token context makes a full project read
  practical; start with `AGENTS.md`, then consult `docs/`.
- **Structured reasoning** — prefer machine-readable output: `vx list --json` or
  `vx list --output-format toon`. See
  [token-efficient output](../advanced/ai-agent-guide.md#token-efficient-output).
- **MCP integration** — replace `npx` / `uvx` with `vx` in MCP configs:
  `"command": "vx", "args": ["npx", ...]`.
- **Worktrees** — `vx wt switch <branch>` for parallel agent worktrees.
- **When uncertain** — read `AGENTS.md` first, then `docs/`.

## Pre-PR checklist

- Conventional Commits: `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`.
- Run `vx just quick` (format → lint → test → build).
- PRs target the `main` branch.

> The original `GEMINI.md` quick-reference table listed `vx just fmt`. That
> recipe does not exist in the `justfile`; use `vx just format` instead.
