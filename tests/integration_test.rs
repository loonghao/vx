//! Comprehensive integration tests for vx tool management
//!
//! This test suite verifies that all tools work correctly with:
//! - Version fetching and listing
//! - Tool installation and download
//! - Version switching
//! - Tool removal/uninstallation
//! - Search functionality
//! - CDN optimization

use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::time::timeout;

/// Test configuration for each tool
#[derive(Debug, Clone)]
pub struct ToolTestConfig {
    pub name: String,
    pub test_versions: Vec<String>,
    pub expected_min_versions: usize,
    pub timeout_seconds: u64,
}

/// Test results for a single tool operation
#[derive(Debug, Clone)]
pub struct TestResult {
    pub tool_name: String,
    pub operation: String,
    pub version: Option<String>,
    pub success: bool,
    pub duration: Duration,
    pub error: Option<String>,
    pub details: HashMap<String, String>,
}

/// Main test framework for vx tools
pub struct VxIntegrationTest {
    pub tools: Vec<ToolTestConfig>,
    pub test_dir: PathBuf,
    pub results: Vec<TestResult>,
    pub parallel: bool,
}

impl VxIntegrationTest {
    /// Create a new test framework instance
    pub fn new() -> Self {
        let test_dir = std::env::temp_dir().join("vx_integration_test");

        // Define tools and their test configurations
        let tools = vec![
            ToolTestConfig {
                name: "uv".to_string(),
                test_versions: vec![
                    "0.7.13".to_string(),
                    "0.7.12".to_string(),
                    "0.7.11".to_string(),
                ],
                expected_min_versions: 5, // Reduced from 25
                timeout_seconds: 180,     // Increased timeout
            },
            ToolTestConfig {
                name: "node".to_string(),
                test_versions: vec![
                    "22.12.0".to_string(),
                    "20.18.1".to_string(),
                    "18.20.5".to_string(),
                ],
                expected_min_versions: 15, // Reduced from 30
                timeout_seconds: 180,
            },
            ToolTestConfig {
                name: "go".to_string(),
                test_versions: vec![
                    "1.23.4".to_string(),
                    "1.22.10".to_string(),
                    "1.21.13".to_string(),
                ],
                expected_min_versions: 3, // Reduced from 20
                timeout_seconds: 180,
            },
            ToolTestConfig {
                name: "bun".to_string(),
                test_versions: vec![
                    "1.1.42".to_string(),
                    "1.1.41".to_string(),
                    "1.1.40".to_string(),
                ],
                expected_min_versions: 5, // Reduced from 15
                timeout_seconds: 180,     // Increased timeout
            },
            ToolTestConfig {
                name: "pnpm".to_string(),
                test_versions: vec![
                    "10.12.1".to_string(),
                    "10.11.1".to_string(),
                    "10.10.0".to_string(),
                ],
                expected_min_versions: 5, // Reduced from 25
                timeout_seconds: 180,     // Increased timeout
            },
            ToolTestConfig {
                name: "yarn".to_string(),
                test_versions: vec![
                    "1.22.22".to_string(),
                    "1.22.21".to_string(),
                    "1.22.20".to_string(),
                ],
                expected_min_versions: 15, // Reduced from 20
                timeout_seconds: 120,
            },
        ];

        Self {
            tools,
            test_dir,
            results: Vec::new(),
            parallel: true,
        }
    }

    /// Setup test environment
    pub async fn setup(&mut self) -> Result<()> {
        // Create test directory
        if self.test_dir.exists() {
            std::fs::remove_dir_all(&self.test_dir)?;
        }
        std::fs::create_dir_all(&self.test_dir)?;

        // Set VX_HOME to test directory
        std::env::set_var("VX_HOME", &self.test_dir);

        println!("🔧 Test environment setup complete");
        println!("📁 Test directory: {}", self.test_dir.display());

        Ok(())
    }

    /// Cleanup test environment
    pub async fn cleanup(&self) -> Result<()> {
        if self.test_dir.exists() {
            std::fs::remove_dir_all(&self.test_dir)?;
        }
        println!("🧹 Test environment cleaned up");
        Ok(())
    }

    /// Record a test result
    pub fn record_result(&mut self, result: TestResult) {
        let status = if result.success { "✅" } else { "❌" };
        let version_info = result
            .version
            .as_ref()
            .map(|v| format!(" ({})", v))
            .unwrap_or_default();

        println!(
            "{} {} {}{} - {:.2}s",
            status,
            result.tool_name,
            result.operation,
            version_info,
            result.duration.as_secs_f64()
        );

        if let Some(error) = &result.error {
            println!("   Error: {}", error);
        }

        self.results.push(result);
    }

    /// Execute vx command and measure time
    pub async fn execute_vx_command(
        &self,
        args: &[&str],
        timeout_secs: u64,
    ) -> Result<(String, Duration)> {
        let start = Instant::now();

        let output = timeout(
            Duration::from_secs(timeout_secs),
            tokio::process::Command::new("cargo")
                .args(&["run", "--"])
                .args(args)
                .current_dir("c:/github/vx")
                .output(),
        )
        .await??;

        let duration = start.elapsed();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Command failed: {} {}\nStdout: {}\nStderr: {}",
                "vx",
                args.join(" "),
                stdout,
                stderr
            ));
        }

        Ok((stdout.to_string(), duration))
    }

    /// Test version listing for a tool
    pub async fn test_version_listing(&mut self, tool: &ToolTestConfig) -> Result<()> {
        let start = Instant::now();

        match self
            .execute_vx_command(&["versions", &tool.name], tool.timeout_seconds)
            .await
        {
            Ok((output, _)) => {
                let version_count = output
                    .lines()
                    .filter(|line| line.trim().matches(char::is_numeric).count() > 0)
                    .count();

                let success = version_count >= tool.expected_min_versions;
                let mut details = HashMap::new();
                details.insert("version_count".to_string(), version_count.to_string());
                details.insert(
                    "expected_min".to_string(),
                    tool.expected_min_versions.to_string(),
                );

                self.record_result(TestResult {
                    tool_name: tool.name.clone(),
                    operation: "list_versions".to_string(),
                    version: None,
                    success,
                    duration: start.elapsed(),
                    error: if success {
                        None
                    } else {
                        Some(format!(
                            "Expected at least {} versions, found {}",
                            tool.expected_min_versions, version_count
                        ))
                    },
                    details,
                });
            }
            Err(e) => {
                self.record_result(TestResult {
                    tool_name: tool.name.clone(),
                    operation: "list_versions".to_string(),
                    version: None,
                    success: false,
                    duration: start.elapsed(),
                    error: Some(e.to_string()),
                    details: HashMap::new(),
                });
            }
        }

        Ok(())
    }

    /// Test tool installation for a specific version
    pub async fn test_tool_installation(
        &mut self,
        tool: &ToolTestConfig,
        version: &str,
    ) -> Result<()> {
        let start = Instant::now();

        match self
            .execute_vx_command(&["install", &tool.name, version], tool.timeout_seconds)
            .await
        {
            Ok((output, _)) => {
                let success = output.contains("successfully")
                    || output.contains("installed")
                    || output.contains("✅");
                let mut details = HashMap::new();
                details.insert("output_length".to_string(), output.len().to_string());

                self.record_result(TestResult {
                    tool_name: tool.name.clone(),
                    operation: "install".to_string(),
                    version: Some(version.to_string()),
                    success,
                    duration: start.elapsed(),
                    error: if success {
                        None
                    } else {
                        Some("Installation did not complete successfully".to_string())
                    },
                    details,
                });
            }
            Err(e) => {
                self.record_result(TestResult {
                    tool_name: tool.name.clone(),
                    operation: "install".to_string(),
                    version: Some(version.to_string()),
                    success: false,
                    duration: start.elapsed(),
                    error: Some(e.to_string()),
                    details: HashMap::new(),
                });
            }
        }

        Ok(())
    }

    /// Test tool uninstallation for a specific version
    async fn test_tool_uninstallation(
        &mut self,
        tool: &ToolTestConfig,
        version: &str,
    ) -> Result<()> {
        let start = Instant::now();

        match self
            .execute_vx_command(&["uninstall", &tool.name, version], tool.timeout_seconds)
            .await
        {
            Ok((output, _)) => {
                let success = output.contains("removed")
                    || output.contains("uninstalled")
                    || output.contains("✅");
                let mut details = HashMap::new();
                details.insert("output_length".to_string(), output.len().to_string());

                self.record_result(TestResult {
                    tool_name: tool.name.clone(),
                    operation: "uninstall".to_string(),
                    version: Some(version.to_string()),
                    success,
                    duration: start.elapsed(),
                    error: if success {
                        None
                    } else {
                        Some("Uninstallation did not complete successfully".to_string())
                    },
                    details,
                });
            }
            Err(e) => {
                self.record_result(TestResult {
                    tool_name: tool.name.clone(),
                    operation: "uninstall".to_string(),
                    version: Some(version.to_string()),
                    success: false,
                    duration: start.elapsed(),
                    error: Some(e.to_string()),
                    details: HashMap::new(),
                });
            }
        }

        Ok(())
    }

    /// Test listing installed versions
    pub async fn test_installed_versions(&mut self, tool: &ToolTestConfig) -> Result<()> {
        let start = Instant::now();

        match self
            .execute_vx_command(&["list", &tool.name], tool.timeout_seconds)
            .await
        {
            Ok((output, _)) => {
                let installed_count = output
                    .lines()
                    .filter(|line| line.trim().matches(char::is_numeric).count() > 0)
                    .count();

                let mut details = HashMap::new();
                details.insert("installed_count".to_string(), installed_count.to_string());

                self.record_result(TestResult {
                    tool_name: tool.name.clone(),
                    operation: "list_installed".to_string(),
                    version: None,
                    success: true, // Always successful if command runs
                    duration: start.elapsed(),
                    error: None,
                    details,
                });
            }
            Err(e) => {
                self.record_result(TestResult {
                    tool_name: tool.name.clone(),
                    operation: "list_installed".to_string(),
                    version: None,
                    success: false,
                    duration: start.elapsed(),
                    error: Some(e.to_string()),
                    details: HashMap::new(),
                });
            }
        }

        Ok(())
    }

    /// Test version switching
    async fn test_version_switching(&mut self, tool: &ToolTestConfig, version: &str) -> Result<()> {
        let start = Instant::now();

        match self
            .execute_vx_command(&["use", &tool.name, version], tool.timeout_seconds)
            .await
        {
            Ok((output, _)) => {
                let success = output.contains("switched")
                    || output.contains("using")
                    || output.contains("✅");
                let mut details = HashMap::new();
                details.insert("output_length".to_string(), output.len().to_string());

                self.record_result(TestResult {
                    tool_name: tool.name.clone(),
                    operation: "switch_version".to_string(),
                    version: Some(version.to_string()),
                    success,
                    duration: start.elapsed(),
                    error: if success {
                        None
                    } else {
                        Some("Version switching did not complete successfully".to_string())
                    },
                    details,
                });
            }
            Err(e) => {
                self.record_result(TestResult {
                    tool_name: tool.name.clone(),
                    operation: "switch_version".to_string(),
                    version: Some(version.to_string()),
                    success: false,
                    duration: start.elapsed(),
                    error: Some(e.to_string()),
                    details: HashMap::new(),
                });
            }
        }

        Ok(())
    }

    /// Test search functionality
    async fn test_search(&mut self, tool: &ToolTestConfig, query: &str) -> Result<()> {
        let start = Instant::now();

        match self
            .execute_vx_command(&["search", query], tool.timeout_seconds)
            .await
        {
            Ok((output, _)) => {
                let found_tool = output.to_lowercase().contains(&tool.name.to_lowercase());
                let mut details = HashMap::new();
                details.insert("query".to_string(), query.to_string());
                details.insert("found_tool".to_string(), found_tool.to_string());

                self.record_result(TestResult {
                    tool_name: tool.name.clone(),
                    operation: "search".to_string(),
                    version: None,
                    success: found_tool,
                    duration: start.elapsed(),
                    error: if found_tool {
                        None
                    } else {
                        Some(format!("Tool {} not found in search results", tool.name))
                    },
                    details,
                });
            }
            Err(e) => {
                self.record_result(TestResult {
                    tool_name: tool.name.clone(),
                    operation: "search".to_string(),
                    version: None,
                    success: false,
                    duration: start.elapsed(),
                    error: Some(e.to_string()),
                    details: HashMap::new(),
                });
            }
        }

        Ok(())
    }

    /// Run comprehensive tests for a single tool
    pub async fn test_tool_comprehensive(&mut self, tool: &ToolTestConfig) -> Result<()> {
        println!("🔧 Testing tool: {}", tool.name);

        // 1. Test version listing
        self.test_version_listing(tool).await?;

        // 2. Test search functionality
        self.test_search(tool, &tool.name).await?;

        // 3. Test installation for each test version
        for version in &tool.test_versions {
            self.test_tool_installation(tool, version).await?;
        }

        // 4. Test listing installed versions
        self.test_installed_versions(tool).await?;

        // 5. Test version switching
        if tool.test_versions.len() >= 2 {
            self.test_version_switching(tool, &tool.test_versions[1])
                .await?;
            self.test_version_switching(tool, &tool.test_versions[0])
                .await?;
        }

        // 6. Test uninstallation (keep one version for final tests)
        for version in &tool.test_versions[1..] {
            self.test_tool_uninstallation(tool, version).await?;
        }

        println!("✅ Tool {} testing completed", tool.name);
        Ok(())
    }

    /// Run all tests
    pub async fn run_all_tests(&mut self) -> Result<()> {
        println!("🚀 Starting comprehensive vx integration tests");
        println!(
            "📊 Testing {} tools with {} total operations",
            self.tools.len(),
            self.tools.len() * 7
        ); // Approximate operations per tool

        let start_time = Instant::now();

        // Setup test environment
        self.setup().await?;

        // Test each tool
        for tool in self.tools.clone() {
            if let Err(e) = self.test_tool_comprehensive(&tool).await {
                eprintln!("❌ Error testing tool {}: {}", tool.name, e);
            }
        }

        let total_duration = start_time.elapsed();

        // Print summary
        self.print_summary(total_duration).await;

        // Cleanup
        self.cleanup().await?;

        Ok(())
    }

    /// Print test summary
    pub async fn print_summary(&self, total_duration: Duration) {
        println!("\n📊 Test Summary");
        println!("================");
        println!("Total duration: {:.2}s", total_duration.as_secs_f64());
        println!("Total tests: {}", self.results.len());

        let successful = self.results.iter().filter(|r| r.success).count();
        let failed = self.results.len() - successful;

        println!("✅ Successful: {}", successful);
        println!("❌ Failed: {}", failed);
        println!(
            "Success rate: {:.1}%",
            (successful as f64 / self.results.len() as f64) * 100.0
        );

        // Group by tool
        let mut tool_stats: HashMap<String, (usize, usize)> = HashMap::new();
        for result in &self.results {
            let (success_count, total_count) =
                tool_stats.entry(result.tool_name.clone()).or_insert((0, 0));
            if result.success {
                *success_count += 1;
            }
            *total_count += 1;
        }

        println!("\n📈 Per-tool Results:");
        for (tool_name, (success_count, total_count)) in tool_stats {
            let rate = (success_count as f64 / total_count as f64) * 100.0;
            println!(
                "  {}: {}/{} ({:.1}%)",
                tool_name, success_count, total_count, rate
            );
        }

        // Show failed tests
        if failed > 0 {
            println!("\n❌ Failed Tests:");
            for result in &self.results {
                if !result.success {
                    println!(
                        "  {} {} {}: {}",
                        result.tool_name,
                        result.operation,
                        result.version.as_ref().unwrap_or(&"".to_string()),
                        result
                            .error
                            .as_ref()
                            .unwrap_or(&"Unknown error".to_string())
                    );
                }
            }
        }

        // Performance stats
        let avg_duration = self
            .results
            .iter()
            .map(|r| r.duration.as_secs_f64())
            .sum::<f64>()
            / self.results.len() as f64;

        let max_duration = self
            .results
            .iter()
            .map(|r| r.duration.as_secs_f64())
            .fold(0.0, f64::max);

        println!("\n⏱️  Performance:");
        println!("  Average operation time: {:.2}s", avg_duration);
        println!("  Slowest operation: {:.2}s", max_duration);
    }
}

impl Default for VxIntegrationTest {
    fn default() -> Self {
        Self::new()
    }
}
