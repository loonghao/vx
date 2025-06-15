//! Common test utilities for vx-cli

use std::env;
use std::path::PathBuf;
use tempfile::TempDir;
use vx_core::{VenvManager, PluginRegistry};

/// Test fixture for CLI tests
pub struct CliTestFixture {
    pub temp_dir: TempDir,
    pub original_dir: PathBuf,
}

impl CliTestFixture {
    /// Create a new CLI test fixture
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
}

impl Drop for CliTestFixture {
    fn drop(&mut self) {
        // Restore original directory
        let _ = env::set_current_dir(&self.original_dir);
    }
}

/// Create a test venv manager for CLI tests
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

/// Create a test plugin registry
pub fn create_test_registry() -> PluginRegistry {
    PluginRegistry::new()
}

/// Clean up test environment
pub fn cleanup_test_env() {
    // Clean up any test artifacts
    // This is called after each test
}

/// Sample configurations for CLI testing
pub mod sample_configs {
    pub const SAMPLE_VX_CONFIG: &str = r#"
[tools]
node = "18.17.0"
python = "3.11.5"

[settings]
auto_install = true
"#;
}
