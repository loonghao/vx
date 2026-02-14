//! Test pipeline configuration
//!
//! This module provides test-related functionality:
//! - Multi-framework support
//! - Coverage reporting
//! - Test hooks
//! - Test environments

use crate::{CoverageConfig, TestConfig, TestEnvironment};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Test runner
pub struct TestRunner {
    config: TestConfig,
}

/// Test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Total tests
    pub total: u32,
    /// Passed tests
    pub passed: u32,
    /// Failed tests
    pub failed: u32,
    /// Skipped tests
    pub skipped: u32,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Coverage percentage (if enabled)
    pub coverage: Option<f64>,
    /// Test output
    pub output: String,
}

impl TestResult {
    /// Check if all tests passed
    pub fn is_success(&self) -> bool {
        self.failed == 0
    }

    /// Get pass rate as percentage
    pub fn pass_rate(&self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            (self.passed as f64 / self.total as f64) * 100.0
        }
    }
}

/// Test framework detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestFramework {
    Jest,
    Pytest,
    CargoTest,
    GoTest,
    Vitest,
    Mocha,
    Unknown,
}

impl TestFramework {
    /// Detect test framework from project files
    pub fn detect(project_root: &Path) -> Self {
        // Check for Cargo.toml (Rust)
        if project_root.join("Cargo.toml").exists() {
            return TestFramework::CargoTest;
        }

        // Check for go.mod (Go)
        if project_root.join("go.mod").exists() {
            return TestFramework::GoTest;
        }

        // Check for package.json (Node.js)
        if project_root.join("package.json").exists() {
            // Try to detect specific framework
            if let Ok(content) = std::fs::read_to_string(project_root.join("package.json")) {
                if content.contains("\"vitest\"") {
                    return TestFramework::Vitest;
                }
                if content.contains("\"jest\"") {
                    return TestFramework::Jest;
                }
                if content.contains("\"mocha\"") {
                    return TestFramework::Mocha;
                }
            }
            return TestFramework::Jest; // Default for Node.js
        }

        // Check for pytest
        if project_root.join("pytest.ini").exists()
            || project_root.join("pyproject.toml").exists()
            || project_root.join("setup.py").exists()
        {
            return TestFramework::Pytest;
        }

        TestFramework::Unknown
    }

    /// Get test command for framework
    pub fn test_command(&self) -> &'static str {
        match self {
            TestFramework::Jest => "npx jest",
            TestFramework::Pytest => "pytest",
            TestFramework::CargoTest => "cargo test",
            TestFramework::GoTest => "go test ./...",
            TestFramework::Vitest => "npx vitest run",
            TestFramework::Mocha => "npx mocha",
            TestFramework::Unknown => "echo 'No test framework detected'",
        }
    }

    /// Get coverage command for framework
    pub fn coverage_command(&self) -> &'static str {
        match self {
            TestFramework::Jest => "npx jest --coverage",
            TestFramework::Pytest => "pytest --cov",
            TestFramework::CargoTest => "cargo llvm-cov",
            TestFramework::GoTest => "go test -cover ./...",
            TestFramework::Vitest => "npx vitest run --coverage",
            TestFramework::Mocha => "npx nyc mocha",
            TestFramework::Unknown => "echo 'No coverage tool detected'",
        }
    }
}

impl TestRunner {
    /// Create a new test runner
    pub fn new(config: TestConfig) -> Self {
        Self { config }
    }

    /// Get configured framework or detect
    pub fn get_framework(&self, project_root: &Path) -> TestFramework {
        if let Some(framework) = &self.config.framework {
            match framework.to_lowercase().as_str() {
                "jest" => TestFramework::Jest,
                "pytest" => TestFramework::Pytest,
                "cargo-test" | "cargo" => TestFramework::CargoTest,
                "go-test" | "go" => TestFramework::GoTest,
                "vitest" => TestFramework::Vitest,
                "mocha" => TestFramework::Mocha,
                _ => TestFramework::detect(project_root),
            }
        } else {
            TestFramework::detect(project_root)
        }
    }

    /// Check if parallel execution is enabled
    pub fn is_parallel(&self) -> bool {
        self.config.parallel.unwrap_or(true)
    }

    /// Get number of workers
    pub fn workers(&self) -> u32 {
        self.config.workers.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|p| p.get() as u32)
                .unwrap_or(4)
        })
    }

    /// Get timeout in seconds
    pub fn timeout(&self) -> u32 {
        self.config.timeout.unwrap_or(300) // 5 minutes default
    }

    /// Get retry count
    pub fn retries(&self) -> u32 {
        self.config.retries.unwrap_or(0)
    }

    /// Check if coverage is enabled
    pub fn coverage_enabled(&self) -> bool {
        self.config
            .coverage
            .as_ref()
            .and_then(|c| c.enabled)
            .unwrap_or(false)
    }

    /// Get coverage threshold
    pub fn coverage_threshold(&self) -> Option<u32> {
        self.config.coverage.as_ref().and_then(|c| c.threshold)
    }

    /// Get test environment by name
    pub fn get_environment(&self, name: &str) -> Option<&TestEnvironment> {
        self.config.environments.get(name)
    }

    /// Build test command
    pub fn build_command(&self, project_root: &Path) -> String {
        let framework = self.get_framework(project_root);
        let mut cmd = if self.coverage_enabled() {
            framework.coverage_command().to_string()
        } else {
            framework.test_command().to_string()
        };

        // Add parallel flag if supported
        if self.is_parallel() {
            match framework {
                TestFramework::Jest => cmd.push_str(&format!(" --maxWorkers={}", self.workers())),
                TestFramework::Pytest => cmd.push_str(&format!(" -n {}", self.workers())),
                TestFramework::CargoTest => {
                    cmd.push_str(&format!(" -- --test-threads={}", self.workers()))
                }
                TestFramework::GoTest => cmd.push_str(&format!(" -parallel {}", self.workers())),
                _ => {}
            }
        }

        cmd
    }
}

/// Coverage report generator
pub struct CoverageReporter {
    config: CoverageConfig,
}

impl CoverageReporter {
    /// Create a new coverage reporter
    pub fn new(config: CoverageConfig) -> Self {
        Self { config }
    }

    /// Get output directory
    pub fn output_dir(&self) -> &str {
        self.config.output.as_deref().unwrap_or("coverage")
    }

    /// Get output formats
    pub fn formats(&self) -> &[String] {
        if self.config.formats.is_empty() {
            &[]
        } else {
            &self.config.formats
        }
    }

    /// Check coverage against threshold
    pub fn check_threshold(&self, coverage: f64) -> Result<(), String> {
        if let Some(threshold) = self.config.threshold
            && coverage < threshold as f64
        {
            return Err(format!(
                "Coverage {:.1}% is below threshold {}%",
                coverage, threshold
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_framework_detection() {
        let dir = tempdir().unwrap();

        // Create Cargo.toml
        std::fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();
        assert_eq!(TestFramework::detect(dir.path()), TestFramework::CargoTest);
    }

    #[test]
    fn test_result_pass_rate() {
        let result = TestResult {
            total: 100,
            passed: 95,
            failed: 5,
            skipped: 0,
            duration_ms: 1000,
            coverage: Some(80.0),
            output: String::new(),
        };

        assert_eq!(result.pass_rate(), 95.0);
        assert!(!result.is_success());
    }

    #[test]
    fn test_coverage_threshold() {
        let config = CoverageConfig {
            enabled: Some(true),
            threshold: Some(80),
            ..Default::default()
        };

        let reporter = CoverageReporter::new(config);
        assert!(reporter.check_threshold(85.0).is_ok());
        assert!(reporter.check_threshold(75.0).is_err());
    }
}
