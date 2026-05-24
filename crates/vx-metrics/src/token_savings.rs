//! Token savings accounting for machine-readable command output.

use crate::report::{CommandMetrics, TokenSavingsRecord};
use serde::Serialize;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

static TOKEN_SAVINGS: OnceLock<Mutex<Vec<TokenSavingsRecord>>> = OnceLock::new();

fn records() -> &'static Mutex<Vec<TokenSavingsRecord>> {
    TOKEN_SAVINGS.get_or_init(|| Mutex::new(Vec::new()))
}

/// Estimate token count from UTF-8 text.
///
/// This intentionally uses a small dependency-free heuristic for metrics. It is
/// stable and cheap enough to run for every command, while still making savings
/// trends visible during debugging.
pub fn estimate_tokens(text: &str) -> u64 {
    if text.is_empty() {
        return 0;
    }

    text.chars().count().div_ceil(4) as u64
}

/// Build a token savings record from baseline and actual output strings.
pub fn build_token_savings_record(
    output_type: impl Into<String>,
    output_format: impl Into<String>,
    baseline_format: impl Into<String>,
    baseline: &str,
    actual: &str,
) -> TokenSavingsRecord {
    let baseline_tokens = estimate_tokens(baseline);
    let actual_tokens = estimate_tokens(actual);
    let token_delta = baseline_tokens as i64 - actual_tokens as i64;
    let savings_ratio = if baseline_tokens == 0 {
        0.0
    } else {
        token_delta as f64 / baseline_tokens as f64
    };

    TokenSavingsRecord {
        output_type: output_type.into(),
        output_format: output_format.into(),
        baseline_format: baseline_format.into(),
        baseline_bytes: baseline.len(),
        actual_bytes: actual.len(),
        baseline_tokens,
        actual_tokens,
        token_delta,
        savings_ratio,
    }
}

/// Record token savings for the current command.
pub fn record_token_savings(record: TokenSavingsRecord) {
    if let Ok(mut guard) = records().lock() {
        guard.push(record);
    }
}

/// Drain all token savings records collected for the current command.
pub fn drain_token_savings() -> Vec<TokenSavingsRecord> {
    records()
        .lock()
        .map(|mut guard| guard.drain(..).collect())
        .unwrap_or_default()
}

/// Aggregated token savings across loaded metrics runs.
#[derive(Debug, Clone, Serialize)]
pub struct TokenSavingsSummary {
    /// Number of metrics runs inspected.
    pub runs: usize,
    /// Number of render records that included token savings data.
    pub records: usize,
    /// Total baseline tokens across all records.
    pub baseline_tokens: u64,
    /// Total actual tokens across all records.
    pub actual_tokens: u64,
    /// Net token delta. Positive means saved tokens; negative means extra tokens.
    pub net_saved_tokens: i64,
    /// Fraction saved vs baseline. Negative means the actual output used more tokens.
    pub savings_ratio: f64,
    /// Per-command aggregate, sorted by largest savings first.
    pub commands: Vec<CommandTokenSavings>,
}

/// Aggregated token savings for one command string.
#[derive(Debug, Clone, Serialize)]
pub struct CommandTokenSavings {
    /// Command string as recorded in metrics.
    pub command: String,
    /// Number of metrics runs for this command.
    pub runs: usize,
    /// Number of render records for this command.
    pub records: usize,
    /// Total baseline tokens.
    pub baseline_tokens: u64,
    /// Total actual tokens.
    pub actual_tokens: u64,
    /// Net token delta. Positive means saved tokens; negative means extra tokens.
    pub net_saved_tokens: i64,
    /// Fraction saved vs baseline. Negative means the actual output used more tokens.
    pub savings_ratio: f64,
}

#[derive(Default)]
struct CommandAccumulator {
    runs: usize,
    records: usize,
    baseline_tokens: u64,
    actual_tokens: u64,
    net_saved_tokens: i64,
}

/// Summarize token savings in newest-first metrics runs.
pub fn summarize_token_savings(runs: &[CommandMetrics]) -> TokenSavingsSummary {
    let mut by_command: BTreeMap<String, CommandAccumulator> = BTreeMap::new();
    let mut baseline_tokens = 0_u64;
    let mut actual_tokens = 0_u64;
    let mut net_saved_tokens = 0_i64;
    let mut records = 0_usize;

    for run in runs {
        if run.token_savings.is_empty() {
            continue;
        }

        let entry = by_command.entry(run.command.clone()).or_default();
        entry.runs += 1;

        for record in &run.token_savings {
            records += 1;
            baseline_tokens += record.baseline_tokens;
            actual_tokens += record.actual_tokens;
            net_saved_tokens += record.token_delta;

            entry.records += 1;
            entry.baseline_tokens += record.baseline_tokens;
            entry.actual_tokens += record.actual_tokens;
            entry.net_saved_tokens += record.token_delta;
        }
    }

    let mut commands: Vec<CommandTokenSavings> = by_command
        .into_iter()
        .map(|(command, acc)| CommandTokenSavings {
            command,
            runs: acc.runs,
            records: acc.records,
            baseline_tokens: acc.baseline_tokens,
            actual_tokens: acc.actual_tokens,
            net_saved_tokens: acc.net_saved_tokens,
            savings_ratio: ratio(acc.net_saved_tokens, acc.baseline_tokens),
        })
        .collect();

    commands.sort_by_key(|command| Reverse(command.net_saved_tokens));

    TokenSavingsSummary {
        runs: runs.len(),
        records,
        baseline_tokens,
        actual_tokens,
        net_saved_tokens,
        savings_ratio: ratio(net_saved_tokens, baseline_tokens),
        commands,
    }
}

/// Render token savings summary as a compact terminal table.
pub fn render_token_savings(summary: &TokenSavingsSummary) -> String {
    if summary.records == 0 {
        return "No token savings data found. Run vx commands with --format toon or --compact.\n"
            .to_string();
    }

    let mut out = String::new();
    out.push_str("Token savings summary\n");
    out.push_str(&format!(
        "runs:{} records:{} baseline:{} actual:{} net_saved:{} ({:.1}%)\n\n",
        summary.runs,
        summary.records,
        summary.baseline_tokens,
        summary.actual_tokens,
        summary.net_saved_tokens,
        summary.savings_ratio * 100.0
    ));
    out.push_str(&format!(
        "{:<36} {:>6} {:>8} {:>8} {:>10} {:>8}\n",
        "Command", "Runs", "Before", "After", "Net saved", "Saved%"
    ));
    out.push_str(&format!(
        "{:<36} {:>6} {:>8} {:>8} {:>10} {:>8}\n",
        "-".repeat(36),
        "----",
        "------",
        "-----",
        "---------",
        "------"
    ));

    for command in &summary.commands {
        out.push_str(&format!(
            "{:<36} {:>6} {:>8} {:>8} {:>10} {:>7.1}%\n",
            truncate(&command.command, 36),
            command.runs,
            command.baseline_tokens,
            command.actual_tokens,
            command.net_saved_tokens,
            command.savings_ratio * 100.0
        ));
    }

    out
}

fn ratio(net_saved_tokens: i64, baseline_tokens: u64) -> f64 {
    if baseline_tokens == 0 {
        0.0
    } else {
        net_saved_tokens as f64 / baseline_tokens as f64
    }
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let truncated: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() && max_chars >= 3 {
        let mut shortened: String = truncated.chars().take(max_chars - 3).collect();
        shortened.push_str("...");
        shortened
    } else {
        truncated
    }
}
