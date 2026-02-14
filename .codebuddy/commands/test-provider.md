## /test-provider

Standardized smoke tests for a provider (canonical runtime), supporting aliases like `cl -> msvc`.

### Usage
- `/test-provider <provider-name> [--skip-install] [--channel <stable|nightly>]`

### Workflow
1) Resolve runtime via registry (accept aliases); derive canonical name + executable.
2) Ensure baseline setup: `vx setup --quiet`.
3) If not installed and not `--skip-install`: `vx install <canonical>` (respect channel if provided).
4) Run smoke commands (collect stdout/stderr + exit code):
   - `vx list <canonical> --status`
   - `vx where <canonical> --all`
   - `vx versions <canonical>`
   - `vx <canonical> --version` (if supported)
   - For each alias (including executable), run `vx <alias> --version` and bare `vx <alias>` to ensure forwarding/help works (e.g., `vx msvc`, `vx cl`).
5) Summarize results (PASS/FAIL per command, note paths/found versions) and store under `.codebuddy/issues/provider-smoke-<canonical>.md`.
6) If failures occur, capture repro steps and suggest fixes (e.g., missing installation, PATH resolution).

### Examples
- `/test-provider msvc`
- `/test-provider node`
- `/test-provider python`
