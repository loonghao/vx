# Cursor notes

> Content preserved from the deleted root `.cursorrules` and harvested from
> `.cursor/rules/*.mdc`. Project-wide rules live in [`AGENTS.md`](https://github.com/vx-org/vx/blob/main/AGENTS.md)
> and [`docs/CONVENTIONS.md`](../CONVENTIONS.md).
>
> The `.cursor/rules/*.mdc` files themselves are left in place — they live inside
> the `.cursor/` dot-directory and are owned by a separate globalisation task.

## From `.cursorrules`

- Follow [`AGENTS.md`](https://github.com/vx-org/vx/blob/main/AGENTS.md) exactly — it is the single source of truth.
- Conventional Commits: `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`.
- Run `vx just quick` before submitting a PR.
- PRs target the `main` branch.

> The original file stated a provider count of 137 and listed a `vx just fmt`
> recipe. Both are stale: the count drifts as providers are added (check
> `crates/vx-providers/`), and `fmt` does not exist — use `vx just format`.

## From `.cursor/rules/vx-core.mdc`

- Always prefix commands with `vx`: `vx npm install`, `vx cargo build`,
  `vx go run main.go`.
- Never suggest manual tool installation — vx handles it automatically.
- Task runner: `vx just <task>`.
- Pre-commit: `vx just quick` (format → lint → test → build).

Terminology, code style, error handling, logging and architecture layers are
enforced by [`docs/CONVENTIONS.md`](../CONVENTIONS.md). One rule there that is
easy to miss:

- **Import order**: stdlib → external crates → internal crates, each group
  separated by a blank line.

## From `.cursor/rules/vx-providers.mdc`

- New provider skeleton and the five high-level templates: see
  [provider development](../advanced/ai-agent-guide.md#starlark-provider-system).
- Return `None` from `download_url` when the current platform is unsupported —
  do not fabricate a URL.
- Verify with `vx <runtime> --version`.
- Full DSL reference: [`docs/guide/provider-star-reference.md`](../guide/provider-star-reference.md).

## From `.cursor/rules/vx-testing.mdc`

- Tests live in `crates/<name>/tests/` — never inline `#[cfg(test)]` modules.
- Use `rstest` for parameterized tests.
- Name tests `test_<function_name>_<scenario>()`.
- Mock network calls; never use real HTTP in unit tests. Prefer the utilities in
  `vx-runtime::testing`.

See [testing conventions](../advanced/ai-agent-guide.md#testing-conventions).

## From `.cursor/rules/vx-mcp.mdc`

- Always use `vx` instead of `npx` / `uvx` / `node` / `python` in MCP server
  configs.
- Applies to `**/mcp*.json`, `**/.claude/settings.json`,
  `**/claude_desktop_config.json`, `**/.vscode/settings.json`.

See [MCP integration](../advanced/ai-agent-guide.md#mcp-integration).
