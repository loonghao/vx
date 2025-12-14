//! Test utilities for vx-cli
//!
//! This module provides common testing utilities, mocks, and helpers
//! for testing vx-cli functionality.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Output;
use tempfile::TempDir;
use vx_plugin::{BundleRegistry, ToolBundle, VxTool};

/// Mock tool for testing
#[derive(Debug, Clone)]
pub struct MockTool {
    pub name: String,
    pub version: String,
    pub executable_path: Option<PathBuf>,
    pub should_fail: bool,
}

impl MockTool {
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            executable_path: None,
            should_fail: false,
        }
    }

    pub fn with_executable(mut self, path: PathBuf) -> Self {
        self.executable_path = Some(path);
        self
    }

    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

/// Mock bundle for testing
#[derive(Debug)]
pub struct MockPlugin {
    pub name: String,
    pub tools: Vec<MockTool>,
}

impl MockPlugin {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tools: Vec::new(),
        }
    }

    pub fn with_tool(mut self, tool: MockTool) -> Self {
        self.tools.push(tool);
        self
    }
}

impl ToolBundle for MockPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Mock plugin for testing"
    }

    fn tools(&self) -> Vec<Box<dyn VxTool>> {
        self.tools
            .iter()
            .map(|tool| Box::new(tool.clone()) as Box<dyn VxTool>)
            .collect()
    }
}

#[async_trait::async_trait]
impl VxTool for MockTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Mock tool for testing"
    }

    async fn fetch_versions(
        &self,
        _include_prerelease: bool,
    ) -> anyhow::Result<Vec<vx_version::VersionInfo>> {
        // Return a mock version for testing
        Ok(vec![vx_version::VersionInfo::new(self.version.clone())])
    }

    async fn get_executable_path(&self, _install_dir: &std::path::Path) -> anyhow::Result<PathBuf> {
        self.executable_path
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Tool not installed"))
    }

    async fn get_installed_versions(&self) -> anyhow::Result<Vec<String>> {
        if self.executable_path.is_some() {
            Ok(vec![self.version.clone()])
        } else {
            Ok(vec![])
        }
    }

    async fn is_version_installed(&self, version: &str) -> anyhow::Result<bool> {
        Ok(self.executable_path.is_some() && self.version == version)
    }
}

/// Test environment setup
pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub registry: BundleRegistry,
    pub mock_tools: HashMap<String, MockTool>,
}

impl Default for TestEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl TestEnvironment {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let registry = BundleRegistry::new();

        Self {
            temp_dir,
            registry,
            mock_tools: HashMap::new(),
        }
    }

    pub fn add_mock_tool(&mut self, tool: MockTool) {
        self.mock_tools.insert(tool.name.clone(), tool);
    }

    pub async fn setup_mock_plugin(&mut self, plugin: MockPlugin) {
        let _ = self.registry.register_bundle(Box::new(plugin)).await;
    }

    pub fn temp_path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }
}

/// Mock command execution for testing
pub struct MockCommandExecutor {
    pub expected_commands: Vec<(String, Vec<String>)>,
    pub responses: Vec<Result<Output, std::io::Error>>,
    pub call_count: usize,
}

impl Default for MockCommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl MockCommandExecutor {
    pub fn new() -> Self {
        Self {
            expected_commands: Vec::new(),
            responses: Vec::new(),
            call_count: 0,
        }
    }

    pub fn expect_command(mut self, command: &str, args: Vec<&str>) -> Self {
        self.expected_commands.push((
            command.to_string(),
            args.iter().map(|s| s.to_string()).collect(),
        ));
        self
    }

    pub fn with_response(mut self, output: Output) -> Self {
        self.responses.push(Ok(output));
        self
    }

    pub fn with_error(mut self, error: std::io::Error) -> Self {
        self.responses.push(Err(error));
        self
    }

    pub fn execute(&mut self, command: &str, args: &[String]) -> Result<Output, std::io::Error> {
        if self.call_count >= self.expected_commands.len() {
            panic!("Unexpected command call: {} {:?}", command, args);
        }

        let (expected_cmd, expected_args) = &self.expected_commands[self.call_count];
        assert_eq!(command, expected_cmd, "Command mismatch");
        assert_eq!(args, expected_args, "Arguments mismatch");

        let response = match &self.responses[self.call_count] {
            Ok(output) => Ok(Output {
                status: output.status,
                stdout: output.stdout.clone(),
                stderr: output.stderr.clone(),
            }),
            Err(e) => Err(std::io::Error::new(e.kind(), e.to_string())),
        };
        self.call_count += 1;

        response
    }
}

/// Create a mock successful command output
pub fn mock_success_output(stdout: &str) -> Output {
    use std::process::Command;

    // Create a real command that will succeed to get a valid ExitStatus
    let output = Command::new("echo")
        .arg("")
        .output()
        .unwrap_or_else(|_| Output {
            status: std::process::ExitStatus::default(),
            stdout: Vec::new(),
            stderr: Vec::new(),
        });

    Output {
        status: output.status,
        stdout: stdout.as_bytes().to_vec(),
        stderr: Vec::new(),
    }
}

/// Create a mock failed command output
pub fn mock_error_output(stderr: &str, _exit_code: i32) -> Output {
    use std::process::Command;

    // Create a real command that will fail to get a valid ExitStatus
    let output = Command::new("nonexistent-command-xyz")
        .output()
        .unwrap_or_else(|_| Output {
            status: std::process::ExitStatus::default(),
            stdout: Vec::new(),
            stderr: Vec::new(),
        });

    Output {
        status: output.status,
        stdout: Vec::new(),
        stderr: stderr.as_bytes().to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_tool_creation() {
        let tool = MockTool::new("node", "18.0.0");
        assert_eq!(tool.name(), "node");
        assert_eq!(tool.version, "18.0.0");
        assert!(tool.executable_path.is_none());
    }

    #[test]
    fn test_mock_tool_with_executable() {
        let path = PathBuf::from("/usr/bin/node");
        let tool = MockTool::new("node", "18.0.0").with_executable(path.clone());

        assert_eq!(tool.name(), "node");
        assert_eq!(tool.executable_path, Some(path));
    }

    #[tokio::test]
    async fn test_test_environment() {
        let mut env = TestEnvironment::new();
        let tool = MockTool::new("test-tool", "1.0.0");
        env.add_mock_tool(tool);

        assert!(env.temp_path().exists());
        assert!(env.mock_tools.contains_key("test-tool"));
    }
}
