//! Terminal and HTML visualization for vx metrics data.
//!
//! Provides:
//! - Terminal table/bar visualization of pipeline stages
//! - Multi-run comparison tables showing trends
//! - Self-contained HTML report with interactive charts

use crate::report::{CommandMetrics, StageMetrics};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Load metrics JSON files from a directory.
///
/// Returns files sorted by timestamp (newest first).
pub fn load_metrics(dir: &Path, limit: usize) -> anyhow::Result<Vec<CommandMetrics>> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "json").unwrap_or(false))
        .collect();

    // Sort by filename descending (timestamps sort lexicographically)
    files.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    files.truncate(limit);

    let mut results = Vec::new();
    for path in &files {
        let content = std::fs::read_to_string(path)?;
        match serde_json::from_str::<CommandMetrics>(&content) {
            Ok(m) => results.push(m),
            Err(e) => {
                eprintln!(
                    "[vx-metrics] Skipping {}: {}",
                    path.file_name().unwrap_or_default().to_string_lossy(),
                    e
                );
            }
        }
    }
    Ok(results)
}

// ============================================================================
// Terminal Visualization
// ============================================================================

const STAGE_ORDER: &[&str] = &["resolve", "ensure", "prepare", "execute"];

/// Render a single metrics run as a terminal-friendly summary.
pub fn render_summary(m: &CommandMetrics) -> String {
    let mut out = String::new();

    // Header
    out.push_str(&format!("  Command:  {}\n", m.command));
    out.push_str(&format!("  Time:     {}\n", &m.timestamp[..19]));
    out.push_str(&format!(
        "  Exit:     {}\n",
        m.exit_code
            .map(|c| if c == 0 {
                "OK".to_string()
            } else {
                format!("{}", c)
            })
            .unwrap_or_else(|| "?".to_string())
    ));
    out.push_str(&format!("  Total:    {:.2}ms\n", m.total_duration_ms));
    out.push('\n');

    // Stage waterfall bar chart
    if !m.stages.is_empty() {
        let max_duration = m
            .stages
            .values()
            .map(|s| s.duration_ms)
            .fold(0.0_f64, f64::max);
        let bar_width = 40;

        out.push_str("  Stage Breakdown:\n");
        out.push_str("  ─────────────────────────────────────────────────────────────\n");

        for &stage in STAGE_ORDER {
            if let Some(s) = m.stages.get(stage) {
                let pct = if m.total_duration_ms > 0.0 {
                    s.duration_ms / m.total_duration_ms * 100.0
                } else {
                    0.0
                };
                let bar_len = if max_duration > 0.0 {
                    (s.duration_ms / max_duration * bar_width as f64) as usize
                } else {
                    0
                };
                let bar: String = "█".repeat(bar_len);
                let status = if s.success { " " } else { "!" };

                out.push_str(&format!(
                    "  {}{:<10} {:>8.2}ms ({:>5.1}%)  {}\n",
                    status, stage, s.duration_ms, pct, bar
                ));
            }
        }

        // Unaccounted time
        let stage_total: f64 = m.stages.values().map(|s| s.duration_ms).sum();
        let overhead = m.total_duration_ms - stage_total;
        if overhead > 0.5 {
            let pct = overhead / m.total_duration_ms * 100.0;
            let bar_len = if max_duration > 0.0 {
                (overhead / max_duration * bar_width as f64) as usize
            } else {
                0
            };
            let bar: String = "░".repeat(bar_len);
            out.push_str(&format!(
                "   {:<10} {:>8.2}ms ({:>5.1}%)  {}\n",
                "overhead", overhead, pct, bar
            ));
        }

        out.push_str("  ─────────────────────────────────────────────────────────────\n");
    }

    out
}

/// Render a comparison table of multiple runs.
pub fn render_comparison(runs: &[CommandMetrics]) -> String {
    if runs.is_empty() {
        return "  No metrics data found.\n".to_string();
    }

    let mut out = String::new();

    out.push_str("  Performance History (newest first):\n");
    out.push_str(
        "  ═══════════════════════════════════════════════════════════════════════════════\n",
    );
    out.push_str(&format!(
        "  {:<22} {:<30} {:>8} {:>8} {:>8} {:>8} {:>8}\n",
        "Timestamp", "Command", "Total", "Resolve", "Ensure", "Prepare", "Execute"
    ));
    out.push_str(
        "  ───────────────────────────────────────────────────────────────────────────────\n",
    );

    for (i, m) in runs.iter().enumerate() {
        let ts = if m.timestamp.len() >= 19 {
            &m.timestamp[..19]
        } else {
            &m.timestamp
        };
        let cmd = truncate_cmd(&m.command, 28);

        let resolve = stage_ms(&m.stages, "resolve");
        let ensure = stage_ms(&m.stages, "ensure");
        let prepare = stage_ms(&m.stages, "prepare");
        let execute = stage_ms(&m.stages, "execute");

        // Show trend arrows comparing to previous run
        let trend = if i + 1 < runs.len() {
            let prev = &runs[i + 1];
            if m.total_duration_ms < prev.total_duration_ms * 0.9 {
                " ↓" // faster
            } else if m.total_duration_ms > prev.total_duration_ms * 1.1 {
                " ↑" // slower
            } else {
                "  " // same
            }
        } else {
            "  "
        };

        out.push_str(&format!(
            "  {:<22} {:<30} {:>6.0}ms{} {:>6}ms {:>6}ms {:>6}ms {:>6}ms\n",
            ts, cmd, m.total_duration_ms, trend, resolve, ensure, prepare, execute
        ));
    }

    out.push_str(
        "  ═══════════════════════════════════════════════════════════════════════════════\n",
    );

    // Summary statistics
    if runs.len() > 1 {
        let totals: Vec<f64> = runs.iter().map(|r| r.total_duration_ms).collect();
        let avg = totals.iter().sum::<f64>() / totals.len() as f64;
        let min = totals.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = totals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // p50 / p95
        let mut sorted = totals.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p50 = percentile(&sorted, 50.0);
        let p95 = percentile(&sorted, 95.0);

        out.push('\n');
        out.push_str(&format!(
            "  Stats ({} runs): avg={:.0}ms  min={:.0}ms  max={:.0}ms  p50={:.0}ms  p95={:.0}ms\n",
            runs.len(),
            avg,
            min,
            max,
            p50,
            p95
        ));

        // Per-stage averages
        out.push_str("\n  Stage Averages:\n");
        for &stage in STAGE_ORDER {
            let vals: Vec<f64> = runs
                .iter()
                .filter_map(|r| r.stages.get(stage).map(|s| s.duration_ms))
                .collect();
            if !vals.is_empty() {
                let stage_avg = vals.iter().sum::<f64>() / vals.len() as f64;
                let stage_pct = stage_avg / avg * 100.0;
                out.push_str(&format!(
                    "    {:<10} avg={:>8.1}ms  ({:>5.1}% of total)\n",
                    stage, stage_avg, stage_pct
                ));
            }
        }
    }

    out
}

/// Render performance insights / bottleneck analysis.
pub fn render_insights(runs: &[CommandMetrics]) -> String {
    if runs.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    out.push_str("\n  Performance Insights:\n");
    out.push_str("  ─────────────────────────────────────────────────────────────\n");

    let latest = &runs[0];

    // Identify bottleneck stage
    if let Some((stage, metrics)) = latest
        .stages
        .iter()
        .max_by(|a, b| a.1.duration_ms.partial_cmp(&b.1.duration_ms).unwrap())
    {
        let pct = metrics.duration_ms / latest.total_duration_ms * 100.0;
        if pct > 50.0 {
            out.push_str(&format!(
                "  ⚡ Bottleneck: '{}' stage takes {:.1}% ({:.0}ms) of total time\n",
                stage, pct, metrics.duration_ms
            ));
        }
    }

    // Check prepare stage overhead (common issue: scanning all installed runtimes)
    if let Some(prepare) = latest.stages.get("prepare") {
        if prepare.duration_ms > 100.0 {
            out.push_str(&format!(
                "  🔍 'prepare' stage is slow ({:.0}ms) — likely scanning installed runtime directories\n",
                prepare.duration_ms
            ));
            // Count path resolution events
            let prepare_span = latest.spans.iter().find(|s| s.name == "prepare");
            if let Some(span) = prepare_span {
                let path_events = span
                    .events
                    .iter()
                    .filter(|e| e.name.contains("Found executable"))
                    .count();
                if path_events > 5 {
                    out.push_str(&format!(
                        "         → Scanned {} runtime executables during PATH construction\n",
                        path_events
                    ));
                    out.push_str("         → Consider caching resolved executable paths\n");
                }
            }
        }
    }

    // Check resolve stage
    if let Some(resolve) = latest.stages.get("resolve") {
        if resolve.duration_ms > 100.0 {
            out.push_str(&format!(
                "  🔍 'resolve' stage is slow ({:.0}ms) — directory traversal for executable\n",
                resolve.duration_ms
            ));
        }
    }

    // Overhead analysis
    let stage_total: f64 = latest.stages.values().map(|s| s.duration_ms).sum();
    let overhead = latest.total_duration_ms - stage_total;
    if overhead > 50.0 {
        let pct = overhead / latest.total_duration_ms * 100.0;
        out.push_str(&format!(
            "  📊 Overhead (CLI init, provider loading): {:.0}ms ({:.1}%)\n",
            overhead, pct
        ));
    }

    // Trend analysis (regression detection)
    if runs.len() >= 3 {
        let recent_avg = runs[..std::cmp::min(3, runs.len())]
            .iter()
            .map(|r| r.total_duration_ms)
            .sum::<f64>()
            / std::cmp::min(3, runs.len()) as f64;

        let older_avg = if runs.len() > 3 {
            runs[3..std::cmp::min(6, runs.len())]
                .iter()
                .map(|r| r.total_duration_ms)
                .sum::<f64>()
                / std::cmp::min(3, runs.len() - 3) as f64
        } else {
            recent_avg
        };

        if recent_avg > older_avg * 1.2 && runs.len() > 3 {
            out.push_str(&format!(
                "  ⚠️  Performance regression: recent avg ({:.0}ms) is {:.0}% slower than before ({:.0}ms)\n",
                recent_avg,
                (recent_avg / older_avg - 1.0) * 100.0,
                older_avg
            ));
        } else if recent_avg < older_avg * 0.8 && runs.len() > 3 {
            out.push_str(&format!(
                "  ✅ Performance improved: recent avg ({:.0}ms) is {:.0}% faster than before ({:.0}ms)\n",
                recent_avg,
                (1.0 - recent_avg / older_avg) * 100.0,
                older_avg
            ));
        }
    }

    out.push_str("  ─────────────────────────────────────────────────────────────\n");
    out
}

// ============================================================================
// HTML Report (rust-embed template)
// ============================================================================

use rust_embed::Embed;

#[derive(Embed)]
#[folder = "templates/"]
struct Templates;

/// Generate a self-contained HTML report with interactive charts.
///
/// The HTML template is embedded at compile time via `rust-embed` from `templates/report.html`.
/// Data is injected by replacing `{{PLACEHOLDER}}` tokens in the template.
pub fn generate_html_report(runs: &[CommandMetrics]) -> String {
    let runs_json = serde_json::to_string(runs).unwrap_or_else(|_| "[]".to_string());

    // Stage data for charts
    let mut stage_series: HashMap<&str, Vec<f64>> = HashMap::new();
    let mut labels: Vec<String> = Vec::new();
    let mut totals: Vec<f64> = Vec::new();

    // Iterate in reverse (oldest first) for chronological chart
    for m in runs.iter().rev() {
        let ts = if m.timestamp.len() >= 16 {
            m.timestamp[11..16].to_string() // HH:MM
        } else {
            "?".to_string()
        };
        labels.push(ts);
        totals.push(m.total_duration_ms);

        for &stage in STAGE_ORDER {
            let val = m.stages.get(stage).map(|s| s.duration_ms).unwrap_or(0.0);
            stage_series.entry(stage).or_default().push(val);
        }
    }

    let labels_json = serde_json::to_string(&labels).unwrap_or_else(|_| "[]".to_string());
    let totals_json = serde_json::to_string(&totals).unwrap_or_else(|_| "[]".to_string());
    let resolve_json = serde_json::to_string(stage_series.get("resolve").unwrap_or(&vec![]))
        .unwrap_or_else(|_| "[]".to_string());
    let ensure_json = serde_json::to_string(stage_series.get("ensure").unwrap_or(&vec![]))
        .unwrap_or_else(|_| "[]".to_string());
    let prepare_json = serde_json::to_string(stage_series.get("prepare").unwrap_or(&vec![]))
        .unwrap_or_else(|_| "[]".to_string());
    let execute_json = serde_json::to_string(stage_series.get("execute").unwrap_or(&vec![]))
        .unwrap_or_else(|_| "[]".to_string());

    // Latest run breakdown for pie chart
    let latest_stages = if let Some(latest) = runs.first() {
        let mut data = Vec::new();
        for &stage in STAGE_ORDER {
            let val = latest
                .stages
                .get(stage)
                .map(|s| s.duration_ms)
                .unwrap_or(0.0);
            data.push(format!("{{ name: '{}', value: {:.2} }}", stage, val));
        }
        let stage_total: f64 = latest.stages.values().map(|s| s.duration_ms).sum();
        let overhead = latest.total_duration_ms - stage_total;
        if overhead > 0.5 {
            data.push(format!("{{ name: 'overhead', value: {:.2} }}", overhead));
        }
        data.join(",")
    } else {
        String::new()
    };

    // Load embedded template
    let template = Templates::get("report.html")
        .expect("report.html template must be embedded")
        .data;
    let template_str = std::str::from_utf8(&template).expect("template must be valid UTF-8");

    // Replace placeholders
    template_str
        .replace(
            "{{TIMESTAMP}}",
            &chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string(),
        )
        .replace("{{COUNT}}", &runs.len().to_string())
        .replace("{{LABELS}}", &labels_json)
        .replace("{{TOTALS}}", &totals_json)
        .replace("{{RESOLVE}}", &resolve_json)
        .replace("{{ENSURE}}", &ensure_json)
        .replace("{{PREPARE}}", &prepare_json)
        .replace("{{EXECUTE}}", &execute_json)
        .replace("{{LATEST_STAGES}}", &latest_stages)
        .replace("{{RUNS_JSON}}", &runs_json)
}

// ============================================================================
// JSON Export (for AI consumption)
// ============================================================================

/// Generate a summary JSON suitable for AI analysis.
pub fn generate_ai_summary(runs: &[CommandMetrics]) -> serde_json::Value {
    let totals: Vec<f64> = runs.iter().map(|r| r.total_duration_ms).collect();
    let avg = if totals.is_empty() {
        0.0
    } else {
        totals.iter().sum::<f64>() / totals.len() as f64
    };

    let mut sorted = totals.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut stage_avgs = HashMap::new();
    for &stage in STAGE_ORDER {
        let vals: Vec<f64> = runs
            .iter()
            .filter_map(|r| r.stages.get(stage).map(|s| s.duration_ms))
            .collect();
        if !vals.is_empty() {
            stage_avgs.insert(
                stage.to_string(),
                serde_json::json!({
                    "avg_ms": vals.iter().sum::<f64>() / vals.len() as f64,
                    "min_ms": vals.iter().cloned().fold(f64::INFINITY, f64::min),
                    "max_ms": vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
                    "pct_of_total": vals.iter().sum::<f64>() / vals.len() as f64 / avg * 100.0,
                }),
            );
        }
    }

    let mut bottlenecks = Vec::new();
    if let Some(latest) = runs.first() {
        if let Some(prepare) = latest.stages.get("prepare") {
            if prepare.duration_ms > 100.0 {
                bottlenecks.push(serde_json::json!({
                    "stage": "prepare",
                    "issue": "Slow runtime directory scanning for PATH construction",
                    "duration_ms": prepare.duration_ms,
                    "suggestion": "Cache resolved executable paths to avoid repeated filesystem traversal"
                }));
            }
        }
        if let Some(resolve) = latest.stages.get("resolve") {
            if resolve.duration_ms > 100.0 {
                bottlenecks.push(serde_json::json!({
                    "stage": "resolve",
                    "issue": "Slow executable directory traversal",
                    "duration_ms": resolve.duration_ms,
                    "suggestion": "Cache the executable path after first resolution"
                }));
            }
        }
    }

    serde_json::json!({
        "runs_analyzed": runs.len(),
        "total_ms": {
            "avg": avg,
            "min": sorted.first().unwrap_or(&0.0),
            "max": sorted.last().unwrap_or(&0.0),
            "p50": percentile(&sorted, 50.0),
            "p95": percentile(&sorted, 95.0),
        },
        "stages": stage_avgs,
        "bottlenecks": bottlenecks,
        "latest_run": runs.first().map(|r| serde_json::json!({
            "command": r.command,
            "total_ms": r.total_duration_ms,
            "exit_code": r.exit_code,
            "stages": r.stages,
        })),
    })
}

// ============================================================================
// Helpers
// ============================================================================

fn stage_ms(stages: &HashMap<String, StageMetrics>, name: &str) -> String {
    stages
        .get(name)
        .map(|s| format!("{:.0}", s.duration_ms))
        .unwrap_or_else(|| "-".to_string())
}

fn truncate_cmd(cmd: &str, max: usize) -> String {
    // Strip path prefix, keep just the binary name and args
    let simplified = if let Some(idx) = cmd.rfind("vx.exe ") {
        format!("vx {}", &cmd[idx + 7..])
    } else if let Some(idx) = cmd.rfind("vx ") {
        cmd[idx..].to_string()
    } else {
        cmd.to_string()
    };

    if simplified.len() > max {
        format!("{}...", &simplified[..max - 3])
    } else {
        simplified
    }
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((p / 100.0) * (sorted.len() as f64 - 1.0)).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}
