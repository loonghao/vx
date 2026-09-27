# Claude Code notes

> Content preserved from the deleted root `CLAUDE.md`.
> Project-wide rules live in [`AGENTS.md`](https://github.com/loonghao/vx/blob/main/AGENTS.md) and
> [`docs/CONVENTIONS.md`](../CONVENTIONS.md). Only Claude Code-specific notes
> belong here.

## Setup

Claude Code reads `AGENTS.md` automatically. Point it at the LLM indices for
deeper context:

- [`llms.txt`](../../llms.txt) — concise LLM-friendly project index.
- [`llms-full.txt`](../../llms-full.txt) — full LLM documentation.

## Claude Code specifics

- **MCP server config** — in `~/.vscode/mcp.json` or `.vscode/mcp.json`, use `vx`
  as the command:

  ```json
  { "command": "vx", "args": ["npx", "-y", "@scope/package@latest"] }
  ```

  See [MCP integration](../advanced/ai-agent-guide.md#mcp-integration) for the
  full migration table.

- **Claude CLI** — `vx claude <prompt>` for CLI interaction (when available).
- **Token optimization** — prefer `vx list --format toon` or
  `vx list --output-format toon` for token-optimized output (saves 40–60% tokens
  versus plain tables).
- **Worktrees** — use `vx wt` for parallel agent worktrees
  ([guide](../advanced/ai-agent-guide.md#multi-agent-development-vx-wt)).
- **Diagnostics** — run `vx doctor` first when something fails.

## Pre-PR checklist

- Follow [`AGENTS.md`](https://github.com/loonghao/vx/blob/main/AGENTS.md) exactly — it is the single source of truth.
- Conventional Commits: `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`.
- Run `vx just quick` (format → lint → test → build).
- PRs target the `main` branch.

> The original `CLAUDE.md` quick-reference table listed `vx just fmt`. That
> recipe does not exist in the `justfile`; use `vx just format` instead.
