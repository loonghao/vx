//! VX Benchmark Management System
//!
//! Unified benchmark data management, baseline comparison, and CI integration.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Benchmark result from performance tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub operation: String,
    pub tool: String,
    pub version: Option<String>,
    pub duration_ms: u64,
    pub success: bool,
    pub timestamp: DateTime<Utc>,
    pub commit_hash: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Performance baseline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub baselines: HashMap<String, OperationBaseline>,
}

/// Baseline for a specific operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationBaseline {
    pub operation: String,
    pub max_duration_ms: u64,
    pub percentile_95_ms: u64,
    pub average_ms: u64,
    pub sample_count: u32,
    pub last_updated: DateTime<Utc>,
}

/// Benchmark comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub operation: String,
    pub tool: String,
    pub current_duration_ms: u64,
    pub baseline_max_ms: u64,
    pub status: ComparisonStatus,
    pub deviation_percent: f64,
    pub message: String,
}

/// Status of benchmark comparison
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComparisonStatus {
    Pass,
    Warning,
    Fail,
    NoBaseline,
}

/// Benchmark manager for unified benchmark operations
pub struct BenchmarkManager {
    baseline_path: PathBuf,
    results_dir: PathBuf,
    current_baseline: Option<PerformanceBaseline>,
}

impl BenchmarkManager {
    /// Create a new benchmark manager
    pub fn new<P: AsRef<Path>>(baseline_path: P, results_dir: P) -> Result<Self> {
        let baseline_path = baseline_path.as_ref().to_path_buf();
        let results_dir = results_dir.as_ref().to_path_buf();

        // Ensure results directory exists
        std::fs::create_dir_all(&results_dir)
            .context("Failed to create benchmark results directory")?;

        let mut manager = Self {
            baseline_path,
            results_dir,
            current_baseline: None,
        };

        // Load existing baseline if available
        if let Err(e) = manager.load_baseline() {
            eprintln!("Warning: Failed to load baseline: {}", e);
        }

        Ok(manager)
    }

    /// Load performance baseline from file
    pub fn load_baseline(&mut self) -> Result<()> {
        if !self.baseline_path.exists() {
            return Ok(()); // No baseline file yet
        }

        let content =
            std::fs::read_to_string(&self.baseline_path).context("Failed to read baseline file")?;

        self.current_baseline =
            Some(serde_json::from_str(&content).context("Failed to parse baseline JSON")?);

        Ok(())
    }

    /// Save performance baseline to file
    pub fn save_baseline(&self, baseline: &PerformanceBaseline) -> Result<()> {
        let content =
            serde_json::to_string_pretty(baseline).context("Failed to serialize baseline")?;

        std::fs::write(&self.baseline_path, content).context("Failed to write baseline file")?;

        Ok(())
    }

    /// Load benchmark results from JSON file
    pub fn load_results<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<BenchmarkResult>> {
        let content = std::fs::read_to_string(file_path.as_ref())
            .context("Failed to read benchmark results file")?;

        let results: Vec<BenchmarkResult> =
            serde_json::from_str(&content).context("Failed to parse benchmark results JSON")?;

        Ok(results)
    }

    /// Save benchmark results to timestamped file
    pub fn save_results(&self, results: &[BenchmarkResult]) -> Result<PathBuf> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("benchmark_results_{}.json", timestamp);
        let file_path = self.results_dir.join(filename);

        let content = serde_json::to_string_pretty(results)
            .context("Failed to serialize benchmark results")?;

        std::fs::write(&file_path, content).context("Failed to write benchmark results file")?;

        Ok(file_path)
    }

    /// Compare benchmark results against baseline
    pub fn compare_against_baseline(
        &self,
        results: &[BenchmarkResult],
    ) -> Result<Vec<ComparisonResult>> {
        let baseline = match &self.current_baseline {
            Some(b) => b,
            None => {
                return Ok(results
                    .iter()
                    .map(|r| ComparisonResult {
                        operation: r.operation.clone(),
                        tool: r.tool.clone(),
                        current_duration_ms: r.duration_ms,
                        baseline_max_ms: 0,
                        status: ComparisonStatus::NoBaseline,
                        deviation_percent: 0.0,
                        message: "No baseline available".to_string(),
                    })
                    .collect());
            }
        };

        let mut comparisons = Vec::new();

        for result in results {
            let key = format!("{}_{}", result.operation, result.tool);

            let comparison = if let Some(baseline_op) = baseline.baselines.get(&key) {
                let deviation_percent = if baseline_op.max_duration_ms > 0 {
                    ((result.duration_ms as f64 - baseline_op.max_duration_ms as f64)
                        / baseline_op.max_duration_ms as f64)
                        * 100.0
                } else {
                    0.0
                };

                let (status, message) = if !result.success {
                    (ComparisonStatus::Fail, "Operation failed".to_string())
                } else if result.duration_ms > baseline_op.max_duration_ms {
                    (
                        ComparisonStatus::Fail,
                        format!("Exceeded baseline by {:.1}%", deviation_percent),
                    )
                } else if result.duration_ms > baseline_op.percentile_95_ms {
                    (
                        ComparisonStatus::Warning,
                        format!("Above 95th percentile by {:.1}%", deviation_percent),
                    )
                } else {
                    (
                        ComparisonStatus::Pass,
                        format!("Within baseline ({:.1}% of max)", deviation_percent),
                    )
                };

                ComparisonResult {
                    operation: result.operation.clone(),
                    tool: result.tool.clone(),
                    current_duration_ms: result.duration_ms,
                    baseline_max_ms: baseline_op.max_duration_ms,
                    status,
                    deviation_percent,
                    message,
                }
            } else {
                ComparisonResult {
                    operation: result.operation.clone(),
                    tool: result.tool.clone(),
                    current_duration_ms: result.duration_ms,
                    baseline_max_ms: 0,
                    status: ComparisonStatus::NoBaseline,
                    deviation_percent: 0.0,
                    message: "No baseline for this operation".to_string(),
                }
            };

            comparisons.push(comparison);
        }

        Ok(comparisons)
    }

    /// Update baseline with new benchmark results
    pub fn update_baseline(&mut self, results: &[BenchmarkResult]) -> Result<()> {
        let mut baseline = self
            .current_baseline
            .clone()
            .unwrap_or_else(|| PerformanceBaseline {
                version: "1.0.0".to_string(),
                created_at: Utc::now(),
                last_updated: Utc::now(),
                baselines: HashMap::new(),
            });

        // Group results by operation and tool
        let mut operation_data: HashMap<String, Vec<&BenchmarkResult>> = HashMap::new();
        for result in results.iter().filter(|r| r.success) {
            let key = format!("{}_{}", result.operation, result.tool);
            operation_data
                .entry(key)
                .or_insert_with(Vec::new)
                .push(result);
        }

        // Calculate new baselines
        for (key, data) in operation_data {
            if data.is_empty() {
                continue;
            }

            let mut durations: Vec<u64> = data.iter().map(|r| r.duration_ms).collect();
            durations.sort_unstable();

            let average_ms = durations.iter().sum::<u64>() / durations.len() as u64;
            let percentile_95_ms = durations[(durations.len() as f64 * 0.95) as usize];
            let max_duration_ms = *durations.last().ok_or_else(|| {
                anyhow::anyhow!("No durations found for operation: {}", data[0].operation)
            })?;

            let operation_baseline = OperationBaseline {
                operation: data[0].operation.clone(),
                max_duration_ms: max_duration_ms * 120 / 100, // 20% buffer
                percentile_95_ms,
                average_ms,
                sample_count: durations.len() as u32,
                last_updated: Utc::now(),
            };

            baseline.baselines.insert(key, operation_baseline);
        }

        baseline.last_updated = Utc::now();
        self.current_baseline = Some(baseline.clone());
        self.save_baseline(&baseline)?;

        Ok(())
    }

    /// Check if benchmark results pass CI requirements
    pub fn check_ci_requirements(&self, results: &[BenchmarkResult]) -> Result<CICheckResult> {
        let comparisons = self.compare_against_baseline(results)?;

        let total_tests = comparisons.len();
        let failed_tests = comparisons
            .iter()
            .filter(|c| c.status == ComparisonStatus::Fail)
            .count();
        let warning_tests = comparisons
            .iter()
            .filter(|c| c.status == ComparisonStatus::Warning)
            .count();
        let no_baseline_tests = comparisons
            .iter()
            .filter(|c| c.status == ComparisonStatus::NoBaseline)
            .count();

        let should_fail = failed_tests > 0;
        let should_warn = warning_tests > 0 || no_baseline_tests > 0;

        Ok(CICheckResult {
            passed: !should_fail,
            should_warn,
            total_tests,
            failed_tests,
            warning_tests,
            no_baseline_tests,
            comparisons,
        })
    }

    /// Generate benchmark report
    pub fn generate_report(&self, results: &[BenchmarkResult]) -> Result<String> {
        let comparisons = self.compare_against_baseline(results)?;
        let ci_result = self.check_ci_requirements(results)?;

        let mut report = String::new();
        report.push_str("# 📊 Benchmark Performance Report\n\n");

        // Summary
        report.push_str(&format!("**Total Tests**: {}\n", ci_result.total_tests));
        report.push_str(&format!(
            "**Passed**: {}\n",
            ci_result.total_tests - ci_result.failed_tests
        ));
        report.push_str(&format!("**Failed**: {}\n", ci_result.failed_tests));
        report.push_str(&format!("**Warnings**: {}\n", ci_result.warning_tests));
        report.push_str(&format!(
            "**No Baseline**: {}\n\n",
            ci_result.no_baseline_tests
        ));

        // CI Status
        let status_emoji = if ci_result.passed { "✅" } else { "❌" };
        report.push_str(&format!(
            "{} **CI Status**: {}\n\n",
            status_emoji,
            if ci_result.passed { "PASS" } else { "FAIL" }
        ));

        // Detailed results
        report.push_str("## 📋 Detailed Results\n\n");
        report.push_str("| Tool | Operation | Current | Baseline | Status | Deviation |\n");
        report.push_str("|------|-----------|---------|----------|--------|----------|\n");

        for comp in &comparisons {
            let status_emoji = match comp.status {
                ComparisonStatus::Pass => "✅",
                ComparisonStatus::Warning => "⚠️",
                ComparisonStatus::Fail => "❌",
                ComparisonStatus::NoBaseline => "❓",
            };

            report.push_str(&format!(
                "| {} | {} | {}ms | {}ms | {} | {:.1}% |\n",
                comp.tool,
                comp.operation,
                comp.current_duration_ms,
                comp.baseline_max_ms,
                status_emoji,
                comp.deviation_percent
            ));
        }

        Ok(report)
    }
}

/// Result of CI benchmark check
#[derive(Debug, Clone)]
pub struct CICheckResult {
    pub passed: bool,
    pub should_warn: bool,
    pub total_tests: usize,
    pub failed_tests: usize,
    pub warning_tests: usize,
    pub no_baseline_tests: usize,
    pub comparisons: Vec<ComparisonResult>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_benchmark_manager_creation() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let baseline_path = temp_dir.path().join("baseline.json");
        let results_dir = temp_dir.path().join("results");

        let manager = BenchmarkManager::new(&baseline_path, &results_dir)
            .expect("Failed to create benchmark manager");
        assert!(results_dir.exists());
    }

    #[test]
    fn test_baseline_comparison() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let baseline_path = temp_dir.path().join("baseline.json");
        let results_dir = temp_dir.path().join("results");

        let manager = BenchmarkManager::new(&baseline_path, &results_dir)
            .expect("Failed to create benchmark manager");

        let results = vec![BenchmarkResult {
            operation: "test".to_string(),
            tool: "tool1".to_string(),
            version: None,
            duration_ms: 1000,
            success: true,
            timestamp: Utc::now(),
            commit_hash: None,
            metadata: HashMap::new(),
        }];

        let comparisons = manager
            .compare_against_baseline(&results)
            .expect("Failed to compare against baseline");
        assert_eq!(comparisons.len(), 1);
        assert_eq!(comparisons[0].status, ComparisonStatus::NoBaseline);
    }
}
