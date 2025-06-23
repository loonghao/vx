//! Performance benchmark tests for vx
//!
//! This test suite establishes performance baselines and monitors for regressions.

#![allow(clippy::duplicate_mod)]

// Re-export integration test utilities
#[path = "integration_test.rs"]
mod integration_test_shared;
use integration_test_shared::VxIntegrationTest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Performance benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub operation: String,
    pub tool: String,
    pub version: Option<String>,
    pub duration_ms: u64,
    pub success: bool,
    pub timestamp: String,
    pub commit_hash: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Performance baseline thresholds (in milliseconds)
#[derive(Debug, Clone)]
pub struct PerformanceBaseline {
    pub version_fetch_max: u64,
    pub installation_max: u64,
    #[allow(dead_code)]
    pub version_switch_max: u64,
    #[allow(dead_code)]
    pub uninstall_max: u64,
}

impl Default for PerformanceBaseline {
    fn default() -> Self {
        Self {
            version_fetch_max: 60_000,  // 60 seconds
            installation_max: 300_000,  // 5 minutes
            version_switch_max: 30_000, // 30 seconds
            uninstall_max: 30_000,      // 30 seconds
        }
    }
}

/// Performance benchmark test suite
pub struct PerformanceBenchmark {
    pub test_suite: VxIntegrationTest,
    pub baseline: PerformanceBaseline,
    pub results: Vec<BenchmarkResult>,
}

impl PerformanceBenchmark {
    pub fn new() -> Self {
        Self {
            test_suite: VxIntegrationTest::new(),
            baseline: PerformanceBaseline::default(),
            results: Vec::new(),
        }
    }

    /// Record a benchmark result
    pub fn record_benchmark(&mut self, result: BenchmarkResult) {
        let status = if result.success { "✅" } else { "❌" };
        let version_info = result
            .version
            .as_ref()
            .map(|v| format!(" ({})", v))
            .unwrap_or_default();

        println!(
            "{} {} {}{} - {}ms",
            status, result.tool, result.operation, version_info, result.duration_ms
        );

        self.results.push(result);
    }

    /// Benchmark version fetching performance
    pub async fn benchmark_version_fetching(&mut self) -> anyhow::Result<()> {
        println!("📊 Benchmarking version fetching performance");

        let tools = vec!["node", "go", "uv", "pnpm", "yarn"];

        for tool_name in tools {
            let start = Instant::now();

            match self
                .test_suite
                .execute_vx_command(&["versions", tool_name], 60)
                .await
            {
                Ok((output, _)) => {
                    let duration = start.elapsed();
                    let version_count = output
                        .lines()
                        .filter(|line| line.trim().matches(char::is_numeric).count() > 0)
                        .count();

                    let mut metadata = HashMap::new();
                    metadata.insert("version_count".to_string(), version_count.to_string());

                    let success = duration.as_millis() <= self.baseline.version_fetch_max as u128;

                    self.record_benchmark(BenchmarkResult {
                        operation: "version_fetch".to_string(),
                        tool: tool_name.to_string(),
                        version: None,
                        duration_ms: duration.as_millis() as u64,
                        success,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        commit_hash: std::env::var("GITHUB_SHA").ok(),
                        metadata,
                    });
                }
                Err(e) => {
                    self.record_benchmark(BenchmarkResult {
                        operation: "version_fetch".to_string(),
                        tool: tool_name.to_string(),
                        version: None,
                        duration_ms: start.elapsed().as_millis() as u64,
                        success: false,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        commit_hash: std::env::var("GITHUB_SHA").ok(),
                        metadata: {
                            let mut metadata = HashMap::new();
                            metadata.insert("error".to_string(), e.to_string());
                            metadata
                        },
                    });
                }
            }
        }

        Ok(())
    }

    /// Benchmark installation performance
    pub async fn benchmark_installation(&mut self) -> anyhow::Result<()> {
        println!("📦 Benchmarking installation performance");

        let test_cases = vec![("node", "22.12.0"), ("go", "1.23.4"), ("uv", "0.7.13")];

        for (tool, version) in test_cases {
            let start = Instant::now();

            match self
                .test_suite
                .execute_vx_command(&["install", tool, version], 300)
                .await
            {
                Ok((output, _)) => {
                    let duration = start.elapsed();
                    let success = duration.as_millis() <= self.baseline.installation_max as u128
                        && (output.contains("successfully")
                            || output.contains("already installed"));

                    let mut metadata = HashMap::new();
                    metadata.insert("output_length".to_string(), output.len().to_string());

                    self.record_benchmark(BenchmarkResult {
                        operation: "installation".to_string(),
                        tool: tool.to_string(),
                        version: Some(version.to_string()),
                        duration_ms: duration.as_millis() as u64,
                        success,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        commit_hash: std::env::var("GITHUB_SHA").ok(),
                        metadata,
                    });
                }
                Err(e) => {
                    self.record_benchmark(BenchmarkResult {
                        operation: "installation".to_string(),
                        tool: tool.to_string(),
                        version: Some(version.to_string()),
                        duration_ms: start.elapsed().as_millis() as u64,
                        success: false,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        commit_hash: std::env::var("GITHUB_SHA").ok(),
                        metadata: {
                            let mut metadata = HashMap::new();
                            metadata.insert("error".to_string(), e.to_string());
                            metadata
                        },
                    });
                }
            }
        }

        Ok(())
    }

    /// Benchmark CDN optimization performance
    pub async fn benchmark_cdn_optimization(&mut self) -> anyhow::Result<()> {
        println!("⚡ Benchmarking CDN optimization performance");

        // Test CDN vs direct download performance
        let tools = vec!["node", "go"];

        for tool in tools {
            let start = Instant::now();

            // Test version fetching with CDN optimization
            match self
                .test_suite
                .execute_vx_command(&["versions", tool], 60)
                .await
            {
                Ok((output, _)) => {
                    let duration = start.elapsed();
                    let version_count = output
                        .lines()
                        .filter(|line| line.trim().matches(char::is_numeric).count() > 0)
                        .count();

                    let mut metadata = HashMap::new();
                    metadata.insert("version_count".to_string(), version_count.to_string());
                    metadata.insert("cdn_enabled".to_string(), "true".to_string());

                    self.record_benchmark(BenchmarkResult {
                        operation: "cdn_optimization".to_string(),
                        tool: tool.to_string(),
                        version: None,
                        duration_ms: duration.as_millis() as u64,
                        success: version_count > 0,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        commit_hash: std::env::var("GITHUB_SHA").ok(),
                        metadata,
                    });
                }
                Err(e) => {
                    self.record_benchmark(BenchmarkResult {
                        operation: "cdn_optimization".to_string(),
                        tool: tool.to_string(),
                        version: None,
                        duration_ms: start.elapsed().as_millis() as u64,
                        success: false,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        commit_hash: std::env::var("GITHUB_SHA").ok(),
                        metadata: {
                            let mut metadata = HashMap::new();
                            metadata.insert("error".to_string(), e.to_string());
                            metadata
                        },
                    });
                }
            }
        }

        Ok(())
    }

    /// Run all performance benchmarks
    pub async fn run_all_benchmarks(&mut self) -> anyhow::Result<()> {
        println!("🚀 Starting performance benchmarks");

        let start_time = Instant::now();

        // Setup test environment
        self.test_suite.setup().await?;

        // Run benchmarks
        self.benchmark_version_fetching().await?;
        self.benchmark_installation().await?;
        self.benchmark_cdn_optimization().await?;

        let total_duration = start_time.elapsed();

        // Print summary
        self.print_benchmark_summary(total_duration).await;

        // Save results
        self.save_benchmark_results().await?;

        // Cleanup
        self.test_suite.cleanup().await?;

        Ok(())
    }

    /// Print benchmark summary
    pub async fn print_benchmark_summary(&self, total_duration: Duration) {
        println!("\n📊 Performance Benchmark Summary");
        println!("=================================");
        println!("Total duration: {:.2}s", total_duration.as_secs_f64());
        println!("Total benchmarks: {}", self.results.len());

        let successful = self.results.iter().filter(|r| r.success).count();
        let failed = self.results.len() - successful;

        println!("✅ Successful: {}", successful);
        println!("❌ Failed: {}", failed);
        println!(
            "Success rate: {:.1}%",
            (successful as f64 / self.results.len() as f64) * 100.0
        );

        // Performance statistics by operation
        let mut operation_stats: HashMap<String, Vec<u64>> = HashMap::new();
        for result in &self.results {
            if result.success {
                operation_stats
                    .entry(result.operation.clone())
                    .or_default()
                    .push(result.duration_ms);
            }
        }

        println!("\n⏱️  Performance by Operation:");
        for (operation, durations) in operation_stats {
            let avg = durations.iter().sum::<u64>() as f64 / durations.len() as f64;
            let min = *durations.iter().min().unwrap_or(&0);
            let max = *durations.iter().max().unwrap_or(&0);

            println!(
                "  {}: avg={:.0}ms, min={}ms, max={}ms",
                operation, avg, min, max
            );
        }

        // Check for performance regressions
        println!("\n🎯 Performance Baseline Check:");
        let mut regressions = 0;
        for result in &self.results {
            let threshold = match result.operation.as_str() {
                "version_fetch" => self.baseline.version_fetch_max,
                "installation" => self.baseline.installation_max,
                "cdn_optimization" => self.baseline.version_fetch_max,
                _ => continue,
            };

            if result.duration_ms > threshold {
                println!(
                    "  ⚠️  {} {} exceeded threshold: {}ms > {}ms",
                    result.tool, result.operation, result.duration_ms, threshold
                );
                regressions += 1;
            }
        }

        if regressions == 0 {
            println!("  ✅ All operations within performance baselines");
        } else {
            println!(
                "  ⚠️  {} operations exceeded performance baselines",
                regressions
            );
        }
    }

    /// Save benchmark results to file
    pub async fn save_benchmark_results(&self) -> anyhow::Result<()> {
        let results_json = serde_json::to_string_pretty(&self.results)?;

        // Save to file with timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("benchmark_results_{}.json", timestamp);

        tokio::fs::write(&filename, results_json).await?;
        println!("💾 Benchmark results saved to: {}", filename);

        Ok(())
    }
}

impl Default for PerformanceBenchmark {
    fn default() -> Self {
        Self::new()
    }
}
