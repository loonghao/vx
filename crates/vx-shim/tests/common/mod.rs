//! Common test utilities for vx-shim

use std::env;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test fixture for shim tests
pub struct ShimTestFixture {
    pub temp_dir: TempDir,
    pub original_dir: PathBuf,
}

impl ShimTestFixture {
    /// Create a new shim test fixture
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
    
    /// Create an executable file in the test directory
    #[cfg(unix)]
    pub fn create_executable(&self, name: &str, content: &str) -> anyhow::Result<()> {
        use std::os::unix::fs::PermissionsExt;
        
        let path = self.path().join(name);
        std::fs::write(&path, content)?;
        
        let mut perms = std::fs::metadata(&path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&path, perms)?;
        
        Ok(())
    }
    
    #[cfg(windows)]
    pub fn create_executable(&self, name: &str, content: &str) -> anyhow::Result<()> {
        let exe_name = if name.ends_with(".exe") { name.to_string() } else { format!("{}.exe", name) };
        std::fs::write(self.path().join(exe_name), content)?;
        Ok(())
    }
}

impl Drop for ShimTestFixture {
    fn drop(&mut self) {
        // Restore original directory
        let _ = env::set_current_dir(&self.original_dir);
    }
}

/// Sample shim configurations for testing
pub mod sample_configs {
    pub const TOML_SHIM_CONFIG: &str = r#"
path = "/usr/bin/node"
args = ["--version"]

[env]
NODE_ENV = "development"
PATH_EXTRA = "/extra/path"
"#;

    pub const LEGACY_SCOOP_CONFIG: &str = r#"path = C:\tools\node\node.exe
args = --version"#;

    pub const SHIM_WITH_ENV_VARS: &str = r#"
path = "${NODE_HOME}/bin/node"
args = ["${NODE_ARGS}"]

[env]
NODE_ENV = "${NODE_ENV:-production}"
DEBUG = "${DEBUG}"
"#;
}
