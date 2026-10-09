# Windsurf notes

> Content preserved from the deleted root `.windsurfrules`.
> Project-wide rules live in [`AGENTS.md`](https://github.com/vx-org/vx/blob/main/AGENTS.md) and
> [`docs/CONVENTIONS.md`](../CONVENTIONS.md).

## From `.windsurfrules`

- Follow [`AGENTS.md`](https://github.com/vx-org/vx/blob/main/AGENTS.md) exactly — it is the single source of truth.
- Conventional Commits: `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`.
- Run `vx just quick` before submitting a PR.
- PRs target the `main` branch.

The Windsurf rules file was byte-identical to `.clinerules` and `.cursorrules`
apart from the H1 title, so it carried no Windsurf-specific behaviour. The shared
quick-reference table it repeated (full check, format, lint, test, build,
single-crate test) now lives in the *Build & test* section of
[`AGENTS.md`](https://github.com/vx-org/vx/blob/main/AGENTS.md#build--test) and in
[`docs/integrations/cursor.md`](cursor.md).

> The original file stated a provider count of 137 and listed a `vx just fmt`
> recipe. Both are stale: the count drifts as providers are added (check
> `crates/vx-providers/`), and `fmt` does not exist — use `vx just format`.
