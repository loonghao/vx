//! Common test utilities for vx-core

use std::env;
use std::path::PathBuf;
use tempfile::TempDir;
use vx_core::{FigmentConfigManager, GlobalToolManager, VenvManager, VxEnvironment};

/// Test fixture for creating isolated test environments
pub struct TestFixture {
    pub temp_dir: TempDir,
    pub original_dir: PathBuf,
}

impl TestFixture {
    /// Create a new test fixture with isolated directory
    pub fn new() -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let original_dir = env::current_dir()?;

        // Change to temp directory for isolation
        env::set_current_dir(temp_dir.path())?;

        Ok(Self {
            temp_dir,
            original_dir,
        })
    }

    /// Get the path to the temporary directory
    pub fn path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    /// Create a file in the test directory
    pub fn create_file(&self, name: &str, content: &str) -> anyhow::Result<()> {
        std::fs::write(self.path().join(name), content)?;
        Ok(())
    }

    /// Create a directory in the test directory
    pub fn create_dir(&self, name: &str) -> anyhow::Result<()> {
        std::fs::create_dir_all(self.path().join(name))?;
        Ok(())
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        // Restore original directory
        let _ = env::set_current_dir(&self.original_dir);
    }
}

/// Create a test configuration manager with minimal setup
pub fn create_test_config_manager() -> anyhow::Result<FigmentConfigManager> {
    Ok(FigmentConfigManager::minimal()?)
}

/// Create a test VX environment
pub fn create_test_vx_environment() -> anyhow::Result<VxEnvironment> {
    Ok(VxEnvironment::new()?)
}

/// Create a test venv manager
pub fn create_test_venv_manager() -> anyhow::Result<(VenvManager, TempDir)> {
    let temp_dir = TempDir::new()?;
    let original_dir = env::current_dir()?;

    // Change to temp directory
    env::set_current_dir(temp_dir.path())?;

    let manager = VenvManager::new()?;

    // Restore directory
    env::set_current_dir(original_dir)?;

    Ok((manager, temp_dir))
}

/// Create a test global tool manager
pub fn create_test_global_tool_manager() -> anyhow::Result<GlobalToolManager> {
    Ok(GlobalToolManager::new()?)
}

/// Sample TOML configurations for testing
pub mod sample_configs {
    pub const VALID_VX_CONFIG: &str = r#"
[tools]
node = "18.17.0"
python = "3.11.5"
go = "1.21.6"

[settings]
auto_install = true
cache_duration = "7d"
"#;

    pub const INVALID_TOML: &str = r#"
[tools
invalid toml syntax
"#;

    pub const CONFIG_WITH_ISSUES: &str = r#"
[tools.node]
version = ""

[tools.python]
version = "3.11.5"

[registries.empty-url]
base_url = ""

[defaults]
update_interval = ""
"#;

    pub const PACKAGE_JSON: &str = r#"
{
  "name": "test-project",
  "version": "1.0.0",
  "engines": {
    "node": "18.17.0"
  }
}
"#;

    pub const PYPROJECT_TOML: &str = r#"
[project]
name = "test-project"
version = "1.0.0"
requires-python = ">=3.11.5"
"#;
}

/// Environment variable helpers for testing
pub mod env_helpers {
    use std::env;

    /// Set environment variable for test duration
    pub struct EnvVar {
        key: String,
        original: Option<String>,
    }

    impl EnvVar {
        pub fn set(key: &str, value: &str) -> Self {
            let original = env::var(key).ok();
            env::set_var(key, value);
            Self {
                key: key.to_string(),
                original,
            }
        }
    }

    impl Drop for EnvVar {
        fn drop(&mut self) {
            match &self.original {
                Some(value) => env::set_var(&self.key, value),
                None => env::remove_var(&self.key),
            }
        }
    }
}
