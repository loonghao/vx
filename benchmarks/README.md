# VX Benchmarks

This directory contains benchmark results and tools for performance tracking of the VX project.

## Directory Structure

```
benchmarks/
├── README.md           # This file
├── manage.py          # Benchmark management tool
├── baseline.json      # Current performance baseline
└── results/           # Benchmark result files
    ├── benchmark_results_YYYYMMDD_HHMMSS.json
    └── ...
```

## Benchmark Management Tool

The `manage.py` script provides comprehensive benchmark management capabilities:

### Usage

```bash
# List all benchmark results
python benchmarks/manage.py list

# Generate summary of latest results
python benchmarks/manage.py summary

# Generate summary of specific file
python benchmarks/manage.py summary --file results/benchmark_results_20250622_114241.json

# Set baseline from latest results
python benchmarks/manage.py set-baseline

# Set baseline from specific file
python benchmarks/manage.py set-baseline --file results/benchmark_results_20250622_114241.json

# Compare latest results with baseline
python benchmarks/manage.py compare

# Generate detailed performance report
python benchmarks/manage.py report

# Save report to file
python benchmarks/manage.py report --output performance_report.md
```

### Features

- **Performance Tracking**: Track operation durations, success rates, and tool performance
- **Baseline Comparison**: Compare current performance against established baselines
- **Statistical Analysis**: Calculate averages, medians, standard deviations
- **Trend Analysis**: Identify performance improvements and regressions
- **Report Generation**: Create detailed markdown reports

## Benchmark Result Format

Each benchmark result file contains an array of operation records:

```json
[
  {
    "operation": "version_fetch",
    "tool": "node",
    "version": null,
    "duration_ms": 57886,
    "success": true,
    "timestamp": "2025-06-22T11:42:14.234581800+00:00",
    "commit_hash": null,
    "metadata": {
      "version_count": "22"
    }
  }
]
```

### Fields

- `operation`: Type of operation (version_fetch, installation, cdn_optimization, etc.)
- `tool`: Tool name (node, go, uv, etc.)
- `version`: Specific version (for installation operations)
- `duration_ms`: Operation duration in milliseconds
- `success`: Whether the operation succeeded
- `timestamp`: ISO 8601 timestamp
- `commit_hash`: Git commit hash (if available)
- `metadata`: Additional operation-specific data

## Performance Metrics

### Operation Types

- **version_fetch**: Time to fetch available versions for a tool
- **installation**: Time to install a specific tool version
- **cdn_optimization**: Time to optimize download URLs using CDN

### Key Metrics

- **Duration**: Operation execution time
- **Success Rate**: Percentage of successful operations
- **Throughput**: Operations per unit time
- **Tool Performance**: Per-tool performance statistics

## Continuous Integration

### Automated Benchmarking

Benchmarks should be run automatically in CI/CD pipelines:

1. **On Pull Requests**: Compare performance against baseline
2. **On Main Branch**: Update baseline if performance improves
3. **Nightly Builds**: Full performance regression testing

### Performance Gates

Consider failing builds if:
- Performance regresses by more than 20%
- Success rate drops below 95%
- Critical operations exceed timeout thresholds

## Best Practices

### Running Benchmarks

1. **Consistent Environment**: Run benchmarks in consistent environments
2. **Multiple Runs**: Average results across multiple runs for stability
3. **Baseline Updates**: Update baselines when making performance improvements
4. **Documentation**: Document significant performance changes

### Analyzing Results

1. **Look for Trends**: Monitor performance over time
2. **Investigate Regressions**: Quickly identify and fix performance issues
3. **Celebrate Improvements**: Track and document performance gains
4. **Tool-Specific Analysis**: Analyze per-tool performance patterns

## Example Workflow

```bash
# Run comprehensive benchmarks (from project root)
cargo test test_performance_benchmarks

# Move results to benchmarks directory
mv benchmark_results_*.json benchmarks/results/

# Generate summary
python benchmarks/manage.py summary

# Compare with baseline
python benchmarks/manage.py compare

# If performance improved, update baseline
python benchmarks/manage.py set-baseline

# Generate report for documentation
python benchmarks/manage.py report --output docs/performance_report.md
```

## Integration with VX

The benchmark system integrates with VX's testing framework:

- **Comprehensive Tests**: `tests/comprehensive_test.rs` generates benchmark data
- **Performance Tests**: Dedicated performance test suites
- **Tool Integration**: Each tool can contribute performance metrics
- **CDN Optimization**: Track CDN performance improvements

## Contributing

When adding new benchmark capabilities:

1. Follow the established JSON format
2. Include relevant metadata
3. Update this documentation
4. Consider impact on CI/CD performance
5. Test the benchmark management tools

## Troubleshooting

### Common Issues

- **Missing Baseline**: Run `set-baseline` to establish initial baseline
- **No Results**: Ensure benchmark tests are generating JSON output
- **Permission Errors**: Check file permissions in benchmarks directory
- **Python Dependencies**: Ensure Python 3.6+ is available

### Getting Help

- Check the comprehensive test logs for benchmark generation issues
- Review the manage.py script for tool-specific problems
- Consult the VX documentation for integration questions