//! Path validation tests for UV tools
//!
//! This module tests the path calculation and validation logic for UV tools
//! across different platforms to ensure consistent behavior.

use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use vx_paths::PathManager;
use vx_tool_uv::{UvCommand, UvxTool};

/// Test fixture for UV path validation tests
struct UvPathTestFixture {
    temp_dir: TempDir,
    path_manager: PathManager,
    uv_tool: UvCommand,
    uvx_tool: UvxTool,
}

impl UvPathTestFixture {
    fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let base_dir = temp_dir.path().join(".vx");
        let path_manager = PathManager::with_base_dir(&base_dir)?;

        Ok(Self {
            temp_dir,
            path_manager,
            uv_tool: UvCommand::new(),
            uvx_tool: UvxTool::new(),
        })
    }

    /// Create a mock UV installation with proper directory structure
    fn create_mock_uv_installation(&self, version: &str) -> Result<PathBuf> {
        let version_dir = self.path_manager.tool_version_dir("uv", version);
        fs::create_dir_all(&version_dir)?;

        // UV has a flat directory structure after FlattenDirectory processing
        let uv_exe = if cfg!(windows) {
            version_dir.join("uv.exe")
        } else {
            version_dir.join("uv")
        };
        fs::write(&uv_exe, "fake uv executable")?;

        // UV also includes uvx executable
        let uvx_exe = if cfg!(windows) {
            version_dir.join("uvx.exe")
        } else {
            version_dir.join("uvx")
        };
        fs::write(&uvx_exe, "fake uvx executable")?;

        // Set executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&uv_exe)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&uv_exe, perms)?;

            let mut perms = fs::metadata(&uvx_exe)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&uvx_exe, perms)?;
        }

        Ok(version_dir)
    }
}

#[test]
fn test_uv_path_structure() {
    let fixture = UvPathTestFixture::new().unwrap();
    let version = "0.7.13";

    // Create mock installation
    let version_dir = fixture.create_mock_uv_installation(version).unwrap();

    // Test UV path structure - should be flat after FlattenDirectory
    let expected_uv_exe = if cfg!(windows) {
        version_dir.join("uv.exe")
    } else {
        version_dir.join("uv")
    };

    assert!(
        expected_uv_exe.exists(),
        "UV executable should exist at {}",
        expected_uv_exe.display()
    );
    assert!(expected_uv_exe.is_file(), "UV executable should be a file");
}

#[test]
fn test_uvx_path_structure() {
    let fixture = UvPathTestFixture::new().unwrap();
    let version = "0.7.13";

    // Create mock installation
    let version_dir = fixture.create_mock_uv_installation(version).unwrap();

    // Test uvx path structure - should be in same directory as uv
    let expected_uvx_exe = if cfg!(windows) {
        version_dir.join("uvx.exe")
    } else {
        version_dir.join("uvx")
    };

    assert!(
        expected_uvx_exe.exists(),
        "uvx executable should exist at {}",
        expected_uvx_exe.display()
    );
    assert!(
        expected_uvx_exe.is_file(),
        "uvx executable should be a file"
    );
}

#[test]
fn test_uv_flat_directory_structure() {
    let fixture = UvPathTestFixture::new().unwrap();
    let version = "0.7.13";

    // Create mock installation
    let version_dir = fixture.create_mock_uv_installation(version).unwrap();

    // Test that UV executable is directly in version directory (flat structure)
    let uv_exe = if cfg!(windows) {
        version_dir.join("uv.exe")
    } else {
        version_dir.join("uv")
    };

    assert!(
        uv_exe.exists(),
        "UV executable should be directly in version directory"
    );
    assert!(uv_exe.is_file(), "UV executable should be a file");

    // Verify no nested directories exist (should be flattened)
    let nested_dir = version_dir.join("uv");
    assert!(
        !nested_dir.exists() || !nested_dir.is_dir(),
        "No nested uv directory should exist after flattening"
    );
}

#[test]
fn test_uv_path_manager_resolution() {
    let fixture = UvPathTestFixture::new().unwrap();
    let version = "0.7.13";

    // Create mock installation
    fixture.create_mock_uv_installation(version).unwrap();

    // Test that PathManager correctly resolves UV executable path
    let uv_path = fixture.path_manager.tool_executable_path("uv", version);

    // UV executable should be found directly in version directory
    let expected_path = fixture
        .path_manager
        .tool_version_dir("uv", version)
        .join(if cfg!(windows) { "uv.exe" } else { "uv" });

    assert!(
        expected_path.exists(),
        "UV executable should exist at expected path"
    );

    // Verify executable name is platform-appropriate
    let exe_name = expected_path.file_name().unwrap().to_string_lossy();
    if cfg!(windows) {
        assert_eq!(exe_name, "uv.exe");
    } else {
        assert_eq!(exe_name, "uv");
    }
}

#[test]
fn test_cross_platform_uv_path_consistency() {
    let fixture = UvPathTestFixture::new().unwrap();
    let version = "0.7.13";

    // Create mock installation
    fixture.create_mock_uv_installation(version).unwrap();

    // Test that path manager returns consistent paths
    let uv_path = fixture.path_manager.tool_executable_path("uv", version);
    let version_dir = fixture.path_manager.tool_version_dir("uv", version);

    // Verify path structure - UV should be directly in version directory
    assert_eq!(uv_path.parent().unwrap(), version_dir);

    // Verify executable name is platform-appropriate
    let exe_name = uv_path.file_name().unwrap().to_string_lossy();
    if cfg!(windows) {
        assert_eq!(exe_name, "uv.exe");
    } else {
        assert_eq!(exe_name, "uv");
    }
}

#[test]
fn test_uv_version_directory_structure() {
    let fixture = UvPathTestFixture::new().unwrap();
    let versions = ["0.6.0", "0.7.0", "0.7.13"];

    // Create multiple versions
    for version in &versions {
        fixture.create_mock_uv_installation(version).unwrap();
    }

    // Test that all versions are properly structured
    for version in &versions {
        let version_dir = fixture.path_manager.tool_version_dir("uv", version);
        assert!(
            version_dir.exists(),
            "Version directory should exist for {}",
            version
        );
        assert!(
            version_dir.is_dir(),
            "Version path should be a directory for {}",
            version
        );

        // Check that the version directory is directly under tools/uv/
        let expected_path = fixture.path_manager.tool_dir("uv").join(version);
        assert_eq!(
            version_dir, expected_path,
            "Version directory path should be correct for {}",
            version
        );

        // Check that executable exists directly in version directory
        let uv_exe = if cfg!(windows) {
            version_dir.join("uv.exe")
        } else {
            version_dir.join("uv")
        };
        assert!(
            uv_exe.exists(),
            "UV executable should exist for version {}",
            version
        );
    }
}

#[test]
fn test_uv_tool_version_listing() {
    let fixture = UvPathTestFixture::new().unwrap();
    let versions = ["0.6.0", "0.7.0", "0.7.13"];

    // Initially no versions
    let listed_versions = fixture.path_manager.list_tool_versions("uv").unwrap();
    assert!(
        listed_versions.is_empty(),
        "Should have no versions initially"
    );

    // Create versions
    for version in &versions {
        fixture.create_mock_uv_installation(version).unwrap();
    }

    // Test version listing
    let listed_versions = fixture.path_manager.list_tool_versions("uv").unwrap();
    assert_eq!(
        listed_versions.len(),
        versions.len(),
        "Should list all created versions"
    );

    // Verify all versions are listed
    for version in &versions {
        assert!(
            listed_versions.contains(&version.to_string()),
            "Should list version {}",
            version
        );
    }
}

#[test]
fn test_uv_installation_detection() {
    let fixture = UvPathTestFixture::new().unwrap();
    let version = "0.7.13";

    // Initially not installed
    assert!(!fixture
        .path_manager
        .is_tool_version_installed("uv", version));

    // Create installation
    fixture.create_mock_uv_installation(version).unwrap();

    // Should now be detected as installed
    assert!(fixture
        .path_manager
        .is_tool_version_installed("uv", version));
}

#[test]
fn test_uv_executable_permissions() {
    let fixture = UvPathTestFixture::new().unwrap();
    let version = "0.7.13";

    // Create mock installation
    fixture.create_mock_uv_installation(version).unwrap();

    let version_dir = fixture.path_manager.tool_version_dir("uv", version);
    let uv_exe = if cfg!(windows) {
        version_dir.join("uv.exe")
    } else {
        version_dir.join("uv")
    };

    // Test that executable exists and is a file
    assert!(uv_exe.exists(), "Executable should exist");
    assert!(uv_exe.is_file(), "Executable should be a file");

    // On Unix, test that executable permissions are set
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&uv_exe).unwrap();
        let permissions = metadata.permissions();
        assert!(
            permissions.mode() & 0o111 != 0,
            "Executable should have execute permissions"
        );
    }
}

#[test]
fn test_uv_path_separator_handling() {
    let fixture = UvPathTestFixture::new().unwrap();
    let version = "0.7.13";

    // Create mock installation
    fixture.create_mock_uv_installation(version).unwrap();

    let version_dir = fixture.path_manager.tool_version_dir("uv", version);
    let uv_path = if cfg!(windows) {
        version_dir.join("uv.exe")
    } else {
        version_dir.join("uv")
    };
    let path_str = uv_path.to_string_lossy();

    // Verify path uses correct separators for platform
    if cfg!(windows) {
        assert!(
            path_str.contains('\\'),
            "Windows paths should use backslashes"
        );
        assert!(
            !path_str.contains('/'),
            "Windows paths should not contain forward slashes"
        );
    } else {
        assert!(
            path_str.contains('/'),
            "Unix paths should use forward slashes"
        );
        assert!(
            !path_str.contains('\\'),
            "Unix paths should not contain backslashes"
        );
    }
}

#[test]
fn test_uv_multiple_executables() {
    let fixture = UvPathTestFixture::new().unwrap();
    let version = "0.7.13";

    // Create mock installation
    let version_dir = fixture.create_mock_uv_installation(version).unwrap();

    // Test that multiple UV tools exist in the same directory
    let uv_exe = if cfg!(windows) {
        version_dir.join("uv.exe")
    } else {
        version_dir.join("uv")
    };

    let uvx_exe = if cfg!(windows) {
        version_dir.join("uvx.exe")
    } else {
        version_dir.join("uvx")
    };

    assert!(uv_exe.exists(), "uv executable should exist");
    assert!(uvx_exe.exists(), "uvx executable should exist");
    assert!(uv_exe.is_file(), "uv should be a file");
    assert!(uvx_exe.is_file(), "uvx should be a file");
}
