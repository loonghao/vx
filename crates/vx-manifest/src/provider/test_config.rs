use serde::{Deserialize, Serialize};

use super::defaults::{default_test_timeout, default_true};

/// Test configuration for a runtime
///
/// Defines how to test a runtime installation. Supports:
/// - Functional test commands
/// - Platform-specific tests
/// - Inline test scripts
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TestConfig {
    /// Functional test commands (cross-platform)
    #[serde(default)]
    pub functional_commands: Vec<TestCommand>,

    /// Install verification commands
    #[serde(default)]
    pub install_verification: Vec<TestCommand>,

    /// Test timeout in milliseconds
    #[serde(default = "default_test_timeout")]
    pub timeout_ms: u64,

    /// Conditions to skip tests (e.g., "ci-windows")
    #[serde(default)]
    pub skip_on: Vec<String>,

    /// Platform-specific test configuration
    #[serde(default)]
    pub platforms: Option<TestPlatformConfig>,

    /// Inline test scripts (for complex test logic)
    #[serde(default)]
    pub inline_scripts: Option<InlineTestScripts>,
}

impl TestConfig {
    /// Check if tests should be skipped for the given condition
    pub fn should_skip(&self, condition: &str) -> bool {
        self.skip_on.iter().any(|s| s == condition)
    }

    /// Get functional commands for the current platform
    pub fn get_functional_commands(&self) -> Vec<&TestCommand> {
        let mut commands: Vec<&TestCommand> = self.functional_commands.iter().collect();

        // Add platform-specific commands
        if let Some(ref platforms) = self.platforms {
            #[cfg(windows)]
            if let Some(ref win) = platforms.windows {
                commands.extend(win.functional_commands.iter());
            }

            #[cfg(unix)]
            if let Some(ref unix) = platforms.unix {
                commands.extend(unix.functional_commands.iter());
            }
        }

        commands
    }

    /// Get the inline script for the current platform
    pub fn get_inline_script(&self) -> Option<&str> {
        self.inline_scripts.as_ref().and_then(|scripts| {
            #[cfg(windows)]
            {
                scripts.windows.as_deref()
            }
            #[cfg(unix)]
            {
                scripts.unix.as_deref()
            }
        })
    }

    /// Check if there are any tests configured
    pub fn has_tests(&self) -> bool {
        !self.functional_commands.is_empty()
            || !self.install_verification.is_empty()
            || self.platforms.is_some()
            || self.inline_scripts.is_some()
    }
}

/// Test command definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestCommand {
    /// Command template (supports {executable}, {version}, {install_dir})
    pub command: String,

    /// Expect the command to succeed (exit code 0)
    #[serde(default = "default_true")]
    pub expect_success: bool,

    /// Expected output pattern (regex)
    #[serde(default)]
    pub expected_output: Option<String>,

    /// Expected exit code (overrides expect_success)
    #[serde(default)]
    pub expected_exit_code: Option<i32>,

    /// Test name/description
    #[serde(default)]
    pub name: Option<String>,

    /// Timeout for this specific command (ms)
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

impl TestCommand {
    /// Create a new test command
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            expect_success: true,
            expected_output: None,
            expected_exit_code: None,
            name: None,
            timeout_ms: None,
        }
    }

    /// Set expected output pattern
    pub fn with_expected_output(mut self, pattern: impl Into<String>) -> Self {
        self.expected_output = Some(pattern.into());
        self
    }

    /// Set expected exit code
    pub fn with_exit_code(mut self, code: i32) -> Self {
        self.expected_exit_code = Some(code);
        self
    }

    /// Get the display name for this test
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.command)
    }
}

/// Platform-specific test configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TestPlatformConfig {
    /// Windows-specific tests
    #[serde(default)]
    pub windows: Option<PlatformTestCommands>,

    /// Unix (Linux/macOS) specific tests
    #[serde(default)]
    pub unix: Option<PlatformTestCommands>,

    /// macOS-specific tests
    #[serde(default)]
    pub macos: Option<PlatformTestCommands>,

    /// Linux-specific tests
    #[serde(default)]
    pub linux: Option<PlatformTestCommands>,
}

/// Test commands for a specific platform
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PlatformTestCommands {
    /// Functional test commands for this platform
    #[serde(default)]
    pub functional_commands: Vec<TestCommand>,
}

/// Inline test scripts for different platforms
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct InlineTestScripts {
    /// Windows batch script
    #[serde(default)]
    pub windows: Option<String>,

    /// Unix shell script (for Linux and macOS)
    #[serde(default)]
    pub unix: Option<String>,

    /// macOS-specific script (overrides unix on macOS)
    #[serde(default)]
    pub macos: Option<String>,

    /// Linux-specific script (overrides unix on Linux)
    #[serde(default)]
    pub linux: Option<String>,
}

impl InlineTestScripts {
    /// Get the script for the current platform
    pub fn for_current_platform(&self) -> Option<&str> {
        #[cfg(target_os = "windows")]
        {
            self.windows.as_deref()
        }

        #[cfg(target_os = "macos")]
        {
            self.macos.as_deref().or(self.unix.as_deref())
        }

        #[cfg(target_os = "linux")]
        {
            self.linux.as_deref().or(self.unix.as_deref())
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            self.unix.as_deref()
        }
    }
}
