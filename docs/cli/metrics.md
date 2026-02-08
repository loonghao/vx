# vx metrics

View performance metrics and diagnostics for vx commands.

## Overview

Every `vx` command execution automatically collects performance data (via OpenTelemetry) and writes it to `~/.vx/metrics/`. The `vx metrics` command lets you view and analyze this data.

## Usage

```bash
# Show the latest run's metrics
vx metrics

# Show the last N runs
vx metrics --last 10

# Export as JSON (AI-friendly)
vx metrics --json

# Generate an interactive HTML report
vx metrics --html report.html

# Clean up old metrics files
vx metrics --clean
```

## Options

| Option | Description |
|--------|-------------|
| `--last N` | Show only the last N runs |
| `--json` | Output metrics as structured JSON |
| `--html <path>` | Generate an interactive HTML report with Chart.js |
| `--clean` | Remove all metrics files |

## Pipeline Stages

vx tracks four execution pipeline stages for each command:

| Stage | Description |
|-------|-------------|
| `resolve` | Resolve runtime version and dependencies |
| `ensure` | Ensure the runtime is installed |
| `prepare` | Prepare environment variables and PATH |
| `execute` | Execute the actual command |

## Per-Layer Tracing Filters

vx uses per-layer filtering to separate console output from metrics collection:

- **Normal mode**: Only warnings and errors are shown on stderr. All `vx=trace` spans are still captured by the OpenTelemetry layer for metrics.
- **`--verbose` mode** (`-v`): Shows `vx` debug messages and info from other crates on stderr.
- **`--debug` mode**: Shows all debug-level messages on stderr.
- **`RUST_LOG` env**: Overrides both the console and OTel filters with the user-specified directive.

This means `vx node --version` will produce clean output (no debug spam), while `vx metrics` can still analyze the full execution trace.

## Output Formats

### Terminal (default)

Displays a waterfall chart of pipeline stages with timing information:

```
╭─ vx node --version ──────────────────────────╮
│ resolve  ███                           50ms   │
│ ensure   ████████████████████████████  800ms   │
│ prepare  █                             10ms   │
│ execute  ███████████                   374ms   │
│                                               │
│ Total: 1234ms  Exit: 0                        │
╰───────────────────────────────────────────────╯
```

### JSON (`--json`)

Structured output suitable for AI analysis and CI integration:

```json
{
  "runs_analyzed": 5,
  "total_ms": { "avg": 1234, "min": 800, "max": 2000, "p50": 1100, "p95": 1900 },
  "stages": {
    "resolve": { "avg_ms": 50 },
    "ensure": { "avg_ms": 800 },
    "prepare": { "avg_ms": 10 },
    "execute": { "avg_ms": 374 }
  },
  "bottleneck": "ensure"
}
```

### HTML (`--html`)

Generates an interactive report with:
- Line charts showing performance trends over time
- Stacked area charts for stage breakdown
- Pie charts for time distribution
- Run history table

## Metrics Storage

Metrics files are stored as JSON in `~/.vx/metrics/`:

```
~/.vx/metrics/
├── 20260208_103000_123.json
├── 20260208_103500_456.json
└── ...
```

Only the most recent 50 files are kept (older files are automatically cleaned up).

## CI Benchmark Integration

vx includes E2E benchmark tests that can be run in CI to detect performance regressions. See the `benchmark.yml` GitHub workflow which runs on every PR across Linux, Windows, and macOS.

Performance thresholds are defined per-platform:

| Test | Linux/macOS | Windows |
|------|------------|---------|
| CLI help | < 350ms | < 500ms |
| CLI version | < 350ms | < 500ms |
| CLI startup | < 3000ms | < 3000ms |
| Config parse (small) | < 1000ms | < 1500ms |
| Config parse (large) | < 3000ms | < 3000ms |
| Setup dry-run (small) | < 1000ms | < 1000ms |
| Setup dry-run (large) | < 3000ms | < 3000ms |
| Script list | < 1000ms | < 1000ms |
| Config validate | < 1000ms | < 1500ms |
