//! Error handling and edge case tests for vx
//!
//! This test suite verifies that vx handles error conditions gracefully.

use crate::integration_test::{TestResult, ToolTestConfig, VxIntegrationTest};
use std::time::{Duration, Instant};

/// Test error handling and edge cases
pub struct ErrorHandlingTest {
    pub test_suite: VxIntegrationTest,
}

impl ErrorHandlingTest {
    pub fn new() -> Self {
        Self {
            test_suite: VxIntegrationTest::new(),
        }
    }

    /// Test invalid tool names
    pub async fn test_invalid_tool_names(&mut self) -> anyhow::Result<()> {
        println!("🧪 Testing invalid tool names");

        let invalid_tools = vec![
            "nonexistent-tool",
            "invalid_tool_name",
            "",
            "tool-with-special-chars!@#",
        ];

        for tool_name in invalid_tools {
            let start = Instant::now();

            match self
                .test_suite
                .execute_vx_command(&["versions", tool_name], 30)
                .await
            {
                Ok((output, _)) => {
                    // Should not succeed for invalid tools
                    self.test_suite.record_result(TestResult {
                        tool_name: tool_name.to_string(),
                        operation: "invalid_tool_test".to_string(),
                        version: None,
                        success: false,
                        duration: start.elapsed(),
                        error: Some(format!("Expected error but got success: {}", output)),
                        details: std::collections::HashMap::new(),
                    });
                }
                Err(e) => {
                    // Expected to fail
                    self.test_suite.record_result(TestResult {
                        tool_name: tool_name.to_string(),
                        operation: "invalid_tool_test".to_string(),
                        version: None,
                        success: true, // Success means it correctly failed
                        duration: start.elapsed(),
                        error: None,
                        details: {
                            let mut details = std::collections::HashMap::new();
                            details.insert("error_message".to_string(), e.to_string());
                            details
                        },
                    });
                }
            }
        }

        Ok(())
    }

    /// Test invalid version numbers
    pub async fn test_invalid_versions(&mut self) -> anyhow::Result<()> {
        println!("🧪 Testing invalid version numbers");

        let test_tool = "node"; // Use a known working tool
        let invalid_versions = vec![
            "999.999.999",
            "invalid-version",
            "",
            "v1.2.3.4.5",
            "latest-invalid",
        ];

        for version in invalid_versions {
            let start = Instant::now();

            match self
                .test_suite
                .execute_vx_command(&["install", test_tool, version], 60)
                .await
            {
                Ok((output, _)) => {
                    // Should not succeed for invalid versions
                    self.test_suite.record_result(TestResult {
                        tool_name: test_tool.to_string(),
                        operation: "invalid_version_test".to_string(),
                        version: Some(version.to_string()),
                        success: false,
                        duration: start.elapsed(),
                        error: Some(format!("Expected error but got success: {}", output)),
                        details: std::collections::HashMap::new(),
                    });
                }
                Err(e) => {
                    // Expected to fail
                    self.test_suite.record_result(TestResult {
                        tool_name: test_tool.to_string(),
                        operation: "invalid_version_test".to_string(),
                        version: Some(version.to_string()),
                        success: true, // Success means it correctly failed
                        duration: start.elapsed(),
                        error: None,
                        details: {
                            let mut details = std::collections::HashMap::new();
                            details.insert("error_message".to_string(), e.to_string());
                            details
                        },
                    });
                }
            }
        }

        Ok(())
    }

    /// Test network timeout scenarios
    pub async fn test_network_timeouts(&mut self) -> anyhow::Result<()> {
        println!("🧪 Testing network timeout scenarios");

        // Test with very short timeout
        let test_tool = ToolTestConfig {
            name: "node".to_string(),
            test_versions: vec!["22.12.0".to_string()],
            expected_min_versions: 1,
            timeout_seconds: 1, // Very short timeout
        };

        let start = Instant::now();

        // This should either succeed quickly or timeout
        match self
            .test_suite
            .execute_vx_command(&["versions", &test_tool.name], 1)
            .await
        {
            Ok((output, _)) => {
                let version_count = output
                    .lines()
                    .filter(|line| line.trim().matches(char::is_numeric).count() > 0)
                    .count();

                self.test_suite.record_result(TestResult {
                    tool_name: test_tool.name.clone(),
                    operation: "timeout_test".to_string(),
                    version: None,
                    success: true,
                    duration: start.elapsed(),
                    error: None,
                    details: {
                        let mut details = std::collections::HashMap::new();
                        details.insert("version_count".to_string(), version_count.to_string());
                        details.insert("completed_quickly".to_string(), "true".to_string());
                        details
                    },
                });
            }
            Err(e) => {
                // Timeout is expected and acceptable
                self.test_suite.record_result(TestResult {
                    tool_name: test_tool.name.clone(),
                    operation: "timeout_test".to_string(),
                    version: None,
                    success: true, // Timeout is an expected behavior
                    duration: start.elapsed(),
                    error: None,
                    details: {
                        let mut details = std::collections::HashMap::new();
                        details.insert("timeout_occurred".to_string(), "true".to_string());
                        details.insert("error_message".to_string(), e.to_string());
                        details
                    },
                });
            }
        }

        Ok(())
    }

    /// Test disk space and permission scenarios
    pub async fn test_disk_space_scenarios(&mut self) -> anyhow::Result<()> {
        println!("🧪 Testing disk space scenarios");

        // Test installation to a path that might have permission issues
        let start = Instant::now();

        // Try to install to a restricted directory (this should fail gracefully)
        std::env::set_var("VX_HOME", "C:\\Windows\\System32\\vx_test"); // Restricted path

        match self
            .test_suite
            .execute_vx_command(&["install", "node", "22.12.0"], 60)
            .await
        {
            Ok((output, _)) => {
                // Reset environment
                std::env::remove_var("VX_HOME");

                self.test_suite.record_result(TestResult {
                    tool_name: "node".to_string(),
                    operation: "permission_test".to_string(),
                    version: Some("22.12.0".to_string()),
                    success: false, // Should not succeed in restricted directory
                    duration: start.elapsed(),
                    error: Some(format!(
                        "Installation should have failed in restricted directory: {}",
                        output
                    )),
                    details: std::collections::HashMap::new(),
                });
            }
            Err(e) => {
                // Reset environment
                std::env::remove_var("VX_HOME");

                // Expected to fail
                self.test_suite.record_result(TestResult {
                    tool_name: "node".to_string(),
                    operation: "permission_test".to_string(),
                    version: Some("22.12.0".to_string()),
                    success: true, // Success means it correctly failed
                    duration: start.elapsed(),
                    error: None,
                    details: {
                        let mut details = std::collections::HashMap::new();
                        details.insert("error_message".to_string(), e.to_string());
                        details
                    },
                });
            }
        }

        Ok(())
    }

    /// Test concurrent operations
    pub async fn test_concurrent_operations(&mut self) -> anyhow::Result<()> {
        println!("🧪 Testing concurrent operations");

        // Test multiple version fetches simultaneously
        let tools = vec!["node", "go", "yarn"];
        let mut handles = Vec::new();

        for tool in tools {
            let tool_name = tool.to_string();
            let handle = tokio::spawn(async move {
                let start = Instant::now();
                let mut test_suite = VxIntegrationTest::new();

                match test_suite
                    .execute_vx_command(&["versions", &tool_name], 60)
                    .await
                {
                    Ok((output, _)) => {
                        let version_count = output
                            .lines()
                            .filter(|line| line.trim().matches(char::is_numeric).count() > 0)
                            .count();

                        TestResult {
                            tool_name: tool_name.clone(),
                            operation: "concurrent_test".to_string(),
                            version: None,
                            success: version_count > 0,
                            duration: start.elapsed(),
                            error: None,
                            details: {
                                let mut details = std::collections::HashMap::new();
                                details
                                    .insert("version_count".to_string(), version_count.to_string());
                                details
                            },
                        }
                    }
                    Err(e) => TestResult {
                        tool_name: tool_name.clone(),
                        operation: "concurrent_test".to_string(),
                        version: None,
                        success: false,
                        duration: start.elapsed(),
                        error: Some(e.to_string()),
                        details: std::collections::HashMap::new(),
                    },
                }
            });
            handles.push(handle);
        }

        // Wait for all concurrent operations to complete
        for handle in handles {
            match handle.await {
                Ok(result) => {
                    self.test_suite.record_result(result);
                }
                Err(e) => {
                    self.test_suite.record_result(TestResult {
                        tool_name: "unknown".to_string(),
                        operation: "concurrent_test".to_string(),
                        version: None,
                        success: false,
                        duration: Duration::from_secs(0),
                        error: Some(format!("Task join error: {}", e)),
                        details: std::collections::HashMap::new(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Run all error handling tests
    pub async fn run_all_error_tests(&mut self) -> anyhow::Result<()> {
        println!("🚀 Starting error handling and edge case tests");

        let start_time = Instant::now();

        // Setup test environment
        self.test_suite.setup().await?;

        // Run all error tests
        self.test_invalid_tool_names().await?;
        self.test_invalid_versions().await?;
        self.test_network_timeouts().await?;
        self.test_disk_space_scenarios().await?;
        self.test_concurrent_operations().await?;

        let total_duration = start_time.elapsed();

        // Print summary
        self.test_suite.print_summary(total_duration).await;

        // Cleanup
        self.test_suite.cleanup().await?;

        Ok(())
    }
}

impl Default for ErrorHandlingTest {
    fn default() -> Self {
        Self::new()
    }
}
