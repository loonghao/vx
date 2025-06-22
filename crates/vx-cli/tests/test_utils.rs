//! Test utilities for vx-cli tests

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Output;
use vx_plugin::{PluginRegistry, VxPlugin, VxTool};

/// Test environment setup
pub struct TestEnvironment {
    pub registry: PluginRegistry,
}

impl TestEnvironment {
    pub fn new() -> Self {
        Self {
            registry: PluginRegistry::new(),
        }
    }
}

/// Mock plugin for testing
pub struct MockPlugin {
    name: String,
    tools: Vec<MockTool>,
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

impl VxPlugin for MockPlugin {
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

/// Mock tool for testing
#[derive(Clone)]
pub struct MockTool {
    name: String,
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

#[async_trait]
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
    ) -> Result<Vec<vx_plugin::VersionInfo>, anyhow::Error> {
        // Return mock versions for testing
        Ok(vec![
            vx_plugin::VersionInfo::new("1.0.0"),
            vx_plugin::VersionInfo::new("2.0.0"),
        ])
    }
}

/// Mock command executor for testing
pub struct MockCommandExecutor {
    expectations: Vec<CommandExpectation>,
    current_index: usize,
}

struct CommandExpectation {
    command: String,
    args: Vec<String>,
    response: Result<Output, std::io::Error>,
}

impl MockCommandExecutor {
    pub fn new() -> Self {
        Self {
            expectations: Vec::new(),
            current_index: 0,
        }
    }

    pub fn expect_command(mut self, command: &str, args: Vec<&str>) -> Self {
        self.expectations.push(CommandExpectation {
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            response: Ok(mock_success_output("")),
        });
        self
    }

    pub fn with_response(mut self, output: Output) -> Self {
        if let Some(last) = self.expectations.last_mut() {
            last.response = Ok(output);
        }
        self
    }

    pub fn with_error(mut self, error: std::io::Error) -> Self {
        if let Some(last) = self.expectations.last_mut() {
            last.response = Err(error);
        }
        self
    }

    pub fn execute(&mut self, command: &str, args: &[String]) -> Result<Output, std::io::Error> {
        if self.current_index >= self.expectations.len() {
            panic!("Unexpected command call: {} {:?}", command, args);
        }

        let expectation = &self.expectations[self.current_index];

        if expectation.command != command {
            panic!(
                "Command mismatch: expected '{}', got '{}'",
                expectation.command, command
            );
        }

        if expectation.args != args {
            panic!(
                "Arguments mismatch: expected {:?}, got {:?}",
                expectation.args, args
            );
        }

        self.current_index += 1;
        expectation
            .response
            .as_ref()
            .map(|o| o.clone())
            .map_err(|e| std::io::Error::new(e.kind(), e.to_string()))
    }
}

/// Create a mock successful command output
pub fn mock_success_output(stdout: &str) -> Output {
    use std::process::Command;

    // Create a real successful command to get a proper ExitStatus
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "echo"]).output()
    } else {
        Command::new("true").output()
    };

    let status = output.map(|o| o.status).unwrap_or_else(|_| {
        // Fallback: create a mock status
        std::process::Command::new("echo").status().unwrap()
    });

    Output {
        status,
        stdout: stdout.as_bytes().to_vec(),
        stderr: Vec::new(),
    }
}

/// Create a mock failed command output
pub fn mock_error_output(stderr: &str, _exit_code: i32) -> Output {
    use std::process::Command;

    // Create a real failed command to get a proper ExitStatus
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "exit 1"]).output()
    } else {
        Command::new("false").output()
    };

    let status = output.map(|o| o.status).unwrap_or_else(|_| {
        // Fallback: create a mock status
        std::process::Command::new("false")
            .status()
            .unwrap_or_else(|_| std::process::Command::new("echo").status().unwrap())
    });

    Output {
        status,
        stdout: Vec::new(),
        stderr: stderr.as_bytes().to_vec(),
    }
}

/// Helper to create test configuration
pub fn create_test_config() -> HashMap<String, String> {
    let mut config = HashMap::new();
    config.insert("vx_home".to_string(), "/tmp/vx-test".to_string());
    config.insert("tools_dir".to_string(), "/tmp/vx-test/tools".to_string());
    config.insert("cache_dir".to_string(), "/tmp/vx-test/cache".to_string());
    config
}

/// Helper to create temporary test directory
pub fn create_temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory")
}

/// Helper to create test file with content
pub fn create_test_file(dir: &std::path::Path, name: &str, content: &str) -> std::path::PathBuf {
    let file_path = dir.join(name);
    std::fs::write(&file_path, content).expect("Failed to write test file");
    file_path
}

/// Helper to check if a command exists on the system
pub fn command_exists(command: &str) -> bool {
    which::which(command).is_ok()
}

/// Skip test if command is not available
#[macro_export]
macro_rules! skip_if_command_missing {
    ($command:expr) => {
        if !test_utils::command_exists($command) {
            eprintln!("Skipping test: {} command not found", $command);
            return;
        }
    };
}
