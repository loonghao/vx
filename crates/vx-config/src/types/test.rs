//! Test pipeline configuration

#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Test configuration (Phase 4)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct TestConfig {
    /// Test framework (auto, jest, pytest, cargo-test, go-test)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,

    /// Run tests in parallel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel: Option<bool>,

    /// Number of parallel workers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workers: Option<u32>,

    /// Coverage configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage: Option<CoverageConfig>,

    /// Test hooks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<TestHooksConfig>,

    /// Test environments
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub environments: HashMap<String, TestEnvironment>,

    /// Test timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,

    /// Retry failed tests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<u32>,
}

/// Coverage configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct CoverageConfig {
    /// Enable coverage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Coverage threshold (percentage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<u32>,

    /// Coverage tool (auto, lcov, cobertura, jacoco)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,

    /// Output format (html, lcov, json, cobertura)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub formats: Vec<String>,

    /// Output directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,

    /// Paths to exclude from coverage
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,

    /// Fail if coverage drops
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_on_decrease: Option<bool>,
}

/// Test hooks configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct TestHooksConfig {
    /// Before all tests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_all: Option<String>,

    /// After all tests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_all: Option<String>,

    /// Before each test file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_each: Option<String>,

    /// After each test file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_each: Option<String>,
}

/// Test environment
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct TestEnvironment {
    /// Environment variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,

    /// Services to start
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub services: Vec<String>,

    /// Setup command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setup: Option<String>,

    /// Teardown command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teardown: Option<String>,
}
