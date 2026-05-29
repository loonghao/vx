================================================================================
                    VX METRICS IMPLEMENTATION ANALYSIS
================================================================================

PART 1: METRICS COMMAND IMPLEMENTATION

Entry: crates/vx-cli/src/commands/metrics.rs (142 lines)

Function handle(last, json, html, clean) does:
  1. Load metrics from ~/.vx/metrics/
  2. Parse JSON CommandMetrics
  3. Render based on flags:
     - Default: render_summary() or render_comparison()
     - --json: generate_ai_summary()
     - --html: generate_html_report()

PART 2: TIMING DATA COLLECTION

Init: crates/vx-metrics/src/init.rs

MetricsGuard::init():
  1. Creates JsonFileExporter (in-memory spans)
  2. Sets up OpenTelemetry SdkTracerProvider
  3. Registers tracing layers
  4. Captures Instant::now() at LINE 152

Report Write (on Drop):
  - provider.force_flush()
  - exporter.take_spans()
  - total_duration_ms = elapsed.as_secs_f64() * 1000.0 (LINE 90)
  - CRITICAL: This is WALL-CLOCK TIME, NOT sum of stages

PART 3: STATISTICS CALCULATIONS

3.1 Stage Extraction: report.rs:88-113
  - Extracts from first matching OpenTelemetry span per stage
  - Does not verify sequential/overlapping

3.2 Overhead Calculation: visualize.rs:104-119
  overhead = total_duration_ms - sum(stage_durations)
  overhead_pct = (overhead / total_duration_ms) * 100.0

  Example: 511ms total, 269ms stages = 242ms overhead
  242 / 511 * 100 = 47.3% CORRECT

3.3 Percentile: visualize.rs:566-583
  - Linear interpolation between sorted values
  - [10,20]: p50=15.0, p95=19.5

3.4 Multi-run Stats: visualize.rs:186-206
  - avg, min, max, p50, p95 on total_duration_ms

3.5 Stage Averages: visualize.rs:225
  - pct_of_total = (avg_ms / avg_total_ms) * 100.0
  - ONLY uses runs where stage exists (filtered)

PART 4: HELP OUTPUT

Location: crates/vx-cli/src/cli.rs:107-182

Clap parser attributes define:
  - about: "Universal version executor..."
  - long_about: 9 usage modes, ecosystems, examples
  - after_help: Subcommand guidance
  - version: From Cargo.toml

Follows AGENTS.md contract.

PART 5: ISSUES EXPLAINED

ISSUE 1: Total != Sum of Stages
  Observation: 511ms total but stages sum to 269ms
  Root cause: WORKING AS DESIGNED
  - total = wall-clock Instant elapsed time
  - stages = only time inside spans
  - gap = unspanned overhead (init, provider loading, config)
  Evidence: init.rs:90, overhead shown in render_summary

ISSUE 2: Overhead Percentage Calculation
  Observation: "242ms (47.3%)"
  Calculation: (242 / 511) * 100 = 47.3%
  Formula: visualize.rs:108
  Meaning: 47.3% of wall-clock time is unspanned overhead
  Result: CORRECT - no issue here

ISSUE 3: Stage Percentage Filtering
  Verified by visualize_tests.rs:178-191
  Intentional: avoids skewing with missing data

CONCLUSION

All metrics calculations are CORRECT and WORKING AS DESIGNED.

Key locations:
- Total time: init.rs:90
- Overhead calc: visualize.rs:106-108
- Stage extraction: report.rs:88-113
- Percentiles: visualize.rs:566-583
- Help contract: cli.rs:107-182, AGENTS.md 109-178

The overhead (47.3%) represents unspanned time during CLI execution.
This is expected and documented behavior.
