# VX Testing Scripts

This directory contains helper scripts used around repository validation. For provider coverage, the **current source of truth** is the `vx test --ci` flow plus the recipes in `justfile`.

## Recommended Provider Test Entry Points

### Local development

Use the `justfile` recipes first:

```bash
# Static provider checks (fastest)
vx just test-providers-static

# Full provider CI flow in a temporary store
vx just test-providers

# Full provider CI flow with JSON output
vx just test-providers-json

# Focus on a runtime subset
vx just test-runtimes "node,go,uv"

# Match the quick CI smoke set
vx just test-ci-quick
```

These commands map directly to the runtime-aware `vx test --ci` implementation and stay aligned with the repository workflow.

### GitHub Actions

The repository provider workflow lives in:

- `.github/workflows/test-providers.yml`
- `.github/scripts/discover-providers.sh`
- `.github/scripts/summarize-test-results.sh`

That workflow dynamically discovers CI-testable runtimes, chunks them per platform, and runs `vx test --ci` across Linux, macOS, and Windows.

## Legacy Helper Scripts

The repository still includes:

- `test-all-providers.sh`
- `test-all-providers.ps1`

They are convenience helpers for ad-hoc local experimentation, but they are **not** the authoritative CI path. When you need parity with the main workflow, prefer `vx just test-providers`, `vx just test-runtimes`, or the GitHub Actions workflow above.

## Common Workflows

### Fast validation before a PR

```bash
vx just test-providers-static
vx just test-ci-quick
```

### Investigate a specific runtime failure

```bash
vx just test-runtimes "python"
vx just test-runtimes "node,go"
```

### Reproduce the full provider matrix locally

```bash
vx just test-providers
```

## Notes

- Prefer `vx just ...` over calling tools directly.
- Prefer `vx test --ci` over older manifest-specific assumptions.
- For cacheable CI runs, use `--vx-root` instead of `--temp-root`.
