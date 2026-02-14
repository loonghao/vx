//! Test utilities for vx-cli
//!
//! This module provides common testing utilities, mocks, and helpers
//! for testing vx-cli functionality.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Output;
use std::sync::Arc;
use tempfile::TempDir;
use vx_runtime::{
    Ecosystem, Provider, ProviderRegistry, Runtime, RuntimeContext, RuntimeDependency, VersionInfo,
    mock_context,
};

/// Mock runtime for testing
#[derive(Debug, Clone)]
pub struct MockRuntime {
    pub name: String,
    pub version: String,
    pub executable_path: Option<PathBuf>,
    pub should_fail: bool,
}

impl MockRuntime {
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

#[async_trait::async_trait]
impl Runtime for MockRuntime {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Mock runtime for testing"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Unknown
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn dependencies(&self) -> &[RuntimeDependency] {
        &[]
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> anyhow::Result<Vec<VersionInfo>> {
        Ok(vec![VersionInfo::new(&self.version)])
    }
}

/// Mock provider for testing
pub struct MockProvider {
    pub name: String,
    pub runtimes: Vec<Arc<dyn Runtime>>,
}

impl std::fmt::Debug for MockProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockProvider")
            .field("name", &self.name)
            .field("runtimes_count", &self.runtimes.len())
            .finish()
    }
}

impl MockProvider {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            runtimes: Vec::new(),
        }
    }

    pub fn with_runtime(mut self, runtime: MockRuntime) -> Self {
        self.runtimes.push(Arc::new(runtime));
        self
    }
}

impl Provider for MockProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Mock provider for testing"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        self.runtimes.clone()
    }
}

/// Test environment setup
pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub registry: ProviderRegistry,
    pub context: RuntimeContext,
    pub mock_runtimes: HashMap<String, MockRuntime>,
}

impl Default for TestEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl TestEnvironment {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let registry = ProviderRegistry::new();
        let context = mock_context();

        Self {
            temp_dir,
            registry,
            context,
            mock_runtimes: HashMap::new(),
        }
    }

    pub fn add_mock_runtime(&mut self, runtime: MockRuntime) {
        self.mock_runtimes.insert(runtime.name.clone(), runtime);
    }

    pub fn setup_mock_provider(&mut self, provider: MockProvider) {
        self.registry.register(Arc::new(provider));
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
    fn test_mock_runtime_creation() {
        let runtime = MockRuntime::new("node", "18.0.0");
        assert_eq!(runtime.name, "node");
        assert_eq!(runtime.version, "18.0.0");
        assert!(runtime.executable_path.is_none());
    }

    #[test]
    fn test_mock_runtime_with_executable() {
        let path = PathBuf::from("/usr/bin/node");
        let runtime = MockRuntime::new("node", "18.0.0").with_executable(path.clone());

        assert_eq!(runtime.name, "node");
        assert_eq!(runtime.executable_path, Some(path));
    }

    #[test]
    fn test_test_environment() {
        let mut env = TestEnvironment::new();
        let runtime = MockRuntime::new("test-runtime", "1.0.0");
        env.add_mock_runtime(runtime);

        assert!(env.temp_path().exists());
        assert!(env.mock_runtimes.contains_key("test-runtime"));
    }
}
