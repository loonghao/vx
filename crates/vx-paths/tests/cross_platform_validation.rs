//! Cross-platform path validation tests
//!
//! This module provides comprehensive tests for path handling across different platforms
//! to ensure consistent behavior for all vx tools.

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use vx_paths::{executable_extension, with_executable_extension, PathManager};

/// Test fixture for cross-platform path validation
pub struct CrossPlatformPathTestFixture {
    pub temp_dir: TempDir,
    pub path_manager: PathManager,
}

impl CrossPlatformPathTestFixture {
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let base_dir = temp_dir.path().join(".vx");
        let path_manager = PathManager::with_base_dir(&base_dir)?;

        Ok(Self {
            temp_dir,
            path_manager,
        })
    }

    /// Create a mock tool installation with specified structure
    pub fn create_mock_tool_installation(
        &self,
        tool_name: &str,
        version: &str,
        structure: ToolStructure,
    ) -> Result<PathBuf> {
        let version_dir = self.path_manager.tool_version_dir(tool_name, version);
        fs::create_dir_all(&version_dir)?;

        match structure {
            ToolStructure::Flat => {
                // Tool executable directly in version directory
                let exe_path = version_dir.join(with_executable_extension(tool_name));
                fs::write(&exe_path, format!("fake {} executable", tool_name))?;
                self.set_executable_permissions(&exe_path)?;
            }
            ToolStructure::BinSubdirectory => {
                // Tool executable in bin subdirectory (like Go)
                let bin_dir = version_dir.join("bin");
                fs::create_dir_all(&bin_dir)?;
                let exe_path = bin_dir.join(with_executable_extension(tool_name));
                fs::write(&exe_path, format!("fake {} executable", tool_name))?;
                self.set_executable_permissions(&exe_path)?;
            }
            ToolStructure::CustomSubdirectory(subdir) => {
                // Tool executable in custom subdirectory
                let custom_dir = version_dir.join(subdir);
                fs::create_dir_all(&custom_dir)?;
                let exe_path = custom_dir.join(with_executable_extension(tool_name));
                fs::write(&exe_path, format!("fake {} executable", tool_name))?;
                self.set_executable_permissions(&exe_path)?;
            }
            ToolStructure::NestedPlatform {
                version_prefix,
                platform_suffix,
            } => {
                // Nested structure like bun-v1.2.9/bun-windows-x64/
                let nested_dir = version_dir.join(format!("{}{}", version_prefix, version));
                let platform_dir = nested_dir.join(platform_suffix);
                fs::create_dir_all(&platform_dir)?;
                let exe_path = platform_dir.join(with_executable_extension(tool_name));
                fs::write(&exe_path, format!("fake {} executable", tool_name))?;
                self.set_executable_permissions(&exe_path)?;
            }
        }

        Ok(version_dir)
    }

    /// Set executable permissions on Unix systems
    fn set_executable_permissions(&self, path: &Path) -> Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(path, perms)?;
        }
        Ok(())
    }

    /// Verify that a tool installation follows expected path structure
    pub fn verify_tool_installation(
        &self,
        tool_name: &str,
        version: &str,
        expected_structure: ToolStructure,
    ) -> Result<()> {
        let version_dir = self.path_manager.tool_version_dir(tool_name, version);

        // Verify version directory exists
        assert!(
            version_dir.exists(),
            "Version directory should exist for {} {}",
            tool_name,
            version
        );
        assert!(
            version_dir.is_dir(),
            "Version path should be a directory for {} {}",
            tool_name,
            version
        );

        // Verify executable exists in expected location
        let expected_exe_path = match expected_structure {
            ToolStructure::Flat => version_dir.join(with_executable_extension(tool_name)),
            ToolStructure::BinSubdirectory => version_dir
                .join("bin")
                .join(with_executable_extension(tool_name)),
            ToolStructure::CustomSubdirectory(subdir) => version_dir
                .join(subdir)
                .join(with_executable_extension(tool_name)),
            ToolStructure::NestedPlatform {
                version_prefix,
                platform_suffix,
            } => {
                let nested_dir = version_dir.join(format!("{}{}", version_prefix, version));
                nested_dir
                    .join(platform_suffix)
                    .join(with_executable_extension(tool_name))
            }
        };

        assert!(
            expected_exe_path.exists(),
            "Executable should exist at {}",
            expected_exe_path.display()
        );
        assert!(expected_exe_path.is_file(), "Executable should be a file");

        // Verify executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&expected_exe_path)?;
            let permissions = metadata.permissions();
            assert!(
                permissions.mode() & 0o111 != 0,
                "Executable should have execute permissions"
            );
        }

        Ok(())
    }
}

/// Different tool directory structures
#[derive(Debug, Clone)]
pub enum ToolStructure {
    /// Executable directly in version directory (e.g., bun, uv)
    Flat,
    /// Executable in bin subdirectory (e.g., go)
    BinSubdirectory,
    /// Executable in custom subdirectory
    CustomSubdirectory(String),
    /// Nested platform-specific structure (e.g., old bun structure)
    NestedPlatform {
        version_prefix: String,
        platform_suffix: String,
    },
}

#[test]
fn test_executable_extension_cross_platform() {
    let extension = executable_extension();

    if cfg!(windows) {
        assert_eq!(extension, ".exe");
    } else {
        assert_eq!(extension, "");
    }
}

#[test]
fn test_with_executable_extension_cross_platform() {
    let node_exe = with_executable_extension("node");
    let go_exe = with_executable_extension("go");
    let bun_exe = with_executable_extension("bun");

    if cfg!(windows) {
        assert_eq!(node_exe, "node.exe");
        assert_eq!(go_exe, "go.exe");
        assert_eq!(bun_exe, "bun.exe");
    } else {
        assert_eq!(node_exe, "node");
        assert_eq!(go_exe, "go");
        assert_eq!(bun_exe, "bun");
    }
}

#[test]
fn test_path_separator_consistency() {
    let fixture = CrossPlatformPathTestFixture::new().unwrap();

    let tool_dir = fixture.path_manager.tool_dir("test-tool");
    let version_dir = fixture.path_manager.tool_version_dir("test-tool", "1.0.0");
    let exe_path = fixture
        .path_manager
        .tool_executable_path("test-tool", "1.0.0");

    let tool_str = tool_dir.to_string_lossy();
    let version_str = version_dir.to_string_lossy();
    let exe_str = exe_path.to_string_lossy();

    if cfg!(windows) {
        // Windows should use backslashes
        assert!(
            tool_str.contains('\\'),
            "Windows paths should use backslashes"
        );
        assert!(
            version_str.contains('\\'),
            "Windows paths should use backslashes"
        );
        assert!(
            exe_str.contains('\\'),
            "Windows paths should use backslashes"
        );

        // Should not contain forward slashes
        assert!(
            !tool_str.contains('/'),
            "Windows paths should not contain forward slashes"
        );
        assert!(
            !version_str.contains('/'),
            "Windows paths should not contain forward slashes"
        );
        assert!(
            !exe_str.contains('/'),
            "Windows paths should not contain forward slashes"
        );
    } else {
        // Unix should use forward slashes
        assert!(
            tool_str.contains('/'),
            "Unix paths should use forward slashes"
        );
        assert!(
            version_str.contains('/'),
            "Unix paths should use forward slashes"
        );
        assert!(
            exe_str.contains('/'),
            "Unix paths should use forward slashes"
        );

        // Should not contain backslashes
        assert!(
            !tool_str.contains('\\'),
            "Unix paths should not contain backslashes"
        );
        assert!(
            !version_str.contains('\\'),
            "Unix paths should not contain backslashes"
        );
        assert!(
            !exe_str.contains('\\'),
            "Unix paths should not contain backslashes"
        );
    }
}

#[test]
fn test_flat_structure_tools() {
    let fixture = CrossPlatformPathTestFixture::new().unwrap();
    let tools = ["bun", "uv", "node"];

    for tool in &tools {
        let version = "1.0.0";
        fixture
            .create_mock_tool_installation(tool, version, ToolStructure::Flat)
            .unwrap();
        fixture
            .verify_tool_installation(tool, version, ToolStructure::Flat)
            .unwrap();

        // Verify tool is detected as installed
        assert!(fixture
            .path_manager
            .is_tool_version_installed(tool, version));

        // Verify tool appears in version listing
        let versions = fixture.path_manager.list_tool_versions(tool).unwrap();
        assert!(versions.contains(&version.to_string()));
    }
}

#[test]
fn test_bin_subdirectory_structure_tools() {
    let fixture = CrossPlatformPathTestFixture::new().unwrap();
    let tools = ["go"];

    for tool in &tools {
        let version = "1.21.0";
        fixture
            .create_mock_tool_installation(tool, version, ToolStructure::BinSubdirectory)
            .unwrap();
        fixture
            .verify_tool_installation(tool, version, ToolStructure::BinSubdirectory)
            .unwrap();

        // Verify tool is detected as installed
        assert!(fixture
            .path_manager
            .is_tool_version_installed(tool, version));
    }
}

#[test]
fn test_nested_platform_structure() {
    let fixture = CrossPlatformPathTestFixture::new().unwrap();
    let tool = "bun";
    let version = "1.2.9";

    let platform_suffix = if cfg!(windows) {
        "bun-windows-x64"
    } else if cfg!(target_os = "macos") {
        "bun-darwin-x64"
    } else {
        "bun-linux-x64"
    };

    let structure = ToolStructure::NestedPlatform {
        version_prefix: "bun-v".to_string(),
        platform_suffix: platform_suffix.to_string(),
    };

    fixture
        .create_mock_tool_installation(tool, version, structure.clone())
        .unwrap();
    fixture
        .verify_tool_installation(tool, version, structure)
        .unwrap();
}

#[test]
fn test_multiple_versions_same_tool() {
    let fixture = CrossPlatformPathTestFixture::new().unwrap();
    let tool = "node";
    let versions = ["18.17.0", "20.11.0", "22.0.0"];

    // Create multiple versions
    for version in &versions {
        fixture
            .create_mock_tool_installation(tool, version, ToolStructure::Flat)
            .unwrap();
    }

    // Verify all versions are listed
    let listed_versions = fixture.path_manager.list_tool_versions(tool).unwrap();
    assert_eq!(listed_versions.len(), versions.len());

    for version in &versions {
        assert!(listed_versions.contains(&version.to_string()));
        assert!(fixture
            .path_manager
            .is_tool_version_installed(tool, version));
    }
}

#[test]
fn test_tool_removal_cross_platform() {
    let fixture = CrossPlatformPathTestFixture::new().unwrap();
    let tool = "test-tool";
    let version = "1.0.0";

    // Create installation
    fixture
        .create_mock_tool_installation(tool, version, ToolStructure::Flat)
        .unwrap();
    assert!(fixture
        .path_manager
        .is_tool_version_installed(tool, version));

    // Remove installation
    fixture
        .path_manager
        .remove_tool_version(tool, version)
        .unwrap();
    assert!(!fixture
        .path_manager
        .is_tool_version_installed(tool, version));

    // Verify it's no longer listed
    let versions = fixture.path_manager.list_tool_versions(tool).unwrap();
    assert!(!versions.contains(&version.to_string()));
}
