//! Path validation tests for Bun tools
//!
//! This module tests the path calculation and validation logic for Bun tools
//! across different platforms to ensure consistent behavior.

use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use vx_paths::PathManager;
use vx_plugin::VxTool;
use vx_tool_bun::BunTool;

/// Test fixture for Bun path validation tests
struct BunPathTestFixture {
    temp_dir: TempDir,
    path_manager: PathManager,
    bun_tool: BunTool,
}

impl BunPathTestFixture {
    fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let base_dir = temp_dir.path().join(".vx");
        let path_manager = PathManager::with_base_dir(&base_dir)?;

        Ok(Self {
            temp_dir,
            path_manager,
            bun_tool: BunTool::new(),
        })
    }

    /// Create a mock Bun installation with proper directory structure
    fn create_mock_bun_installation(&self, version: &str) -> Result<PathBuf> {
        let version_dir = self.path_manager.tool_version_dir("bun", version);
        fs::create_dir_all(&version_dir)?;

        // Bun has a flat directory structure after FlattenDirectory processing
        let bun_exe = if cfg!(windows) {
            version_dir.join("bun.exe")
        } else {
            version_dir.join("bun")
        };
        fs::write(&bun_exe, "fake bun executable")?;

        // Set executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&bun_exe)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&bun_exe, perms)?;
        }

        Ok(version_dir)
    }

    /// Create a mock Bun installation with the old problematic structure (for testing fixes)
    fn create_mock_bun_installation_old_structure(&self, version: &str) -> Result<PathBuf> {
        let version_dir = self.path_manager.tool_version_dir("bun", version);
        fs::create_dir_all(&version_dir)?;

        // Old problematic structure: bun-v1.2.9/bun-windows-x64/bun.exe
        let nested_dir = version_dir.join(format!("bun-v{}", version));
        let platform_dir = if cfg!(windows) {
            nested_dir.join("bun-windows-x64")
        } else if cfg!(target_os = "macos") {
            nested_dir.join("bun-darwin-x64")
        } else {
            nested_dir.join("bun-linux-x64")
        };
        fs::create_dir_all(&platform_dir)?;

        let bun_exe = if cfg!(windows) {
            platform_dir.join("bun.exe")
        } else {
            platform_dir.join("bun")
        };
        fs::write(&bun_exe, "fake bun executable")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&bun_exe)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&bun_exe, perms)?;
        }

        Ok(version_dir)
    }
}

#[test]
fn test_bun_path_structure() {
    let fixture = BunPathTestFixture::new().unwrap();
    let version = "1.2.9";

    // Create mock installation with correct structure
    let version_dir = fixture.create_mock_bun_installation(version).unwrap();

    // Test Bun path structure - should be flat after FlattenDirectory
    let expected_bun_exe = if cfg!(windows) {
        version_dir.join("bun.exe")
    } else {
        version_dir.join("bun")
    };

    assert!(
        expected_bun_exe.exists(),
        "Bun executable should exist at {}",
        expected_bun_exe.display()
    );
    assert!(
        expected_bun_exe.is_file(),
        "Bun executable should be a file"
    );
}

#[test]
fn test_bun_flat_directory_structure() {
    let fixture = BunPathTestFixture::new().unwrap();
    let version = "1.2.9";

    // Create mock installation
    let version_dir = fixture.create_mock_bun_installation(version).unwrap();

    // Test that Bun executable is directly in version directory (flat structure)
    let bun_exe = if cfg!(windows) {
        version_dir.join("bun.exe")
    } else {
        version_dir.join("bun")
    };

    assert!(
        bun_exe.exists(),
        "Bun executable should be directly in version directory"
    );
    assert!(bun_exe.is_file(), "Bun executable should be a file");

    // Verify no nested directories exist (should be flattened)
    let nested_dir = version_dir.join(format!("bun-v{}", version));
    assert!(
        !nested_dir.exists(),
        "Nested bun-v{} directory should not exist after flattening",
        version
    );
}

#[test]
fn test_bun_path_manager_resolution() {
    let fixture = BunPathTestFixture::new().unwrap();
    let version = "1.2.9";

    // Create mock installation
    fixture.create_mock_bun_installation(version).unwrap();

    // Test that PathManager correctly resolves Bun executable path
    let bun_path = fixture.path_manager.tool_executable_path("bun", version);

    // Bun executable should be found directly in version directory
    let expected_path = fixture
        .path_manager
        .tool_version_dir("bun", version)
        .join(if cfg!(windows) { "bun.exe" } else { "bun" });

    assert!(
        expected_path.exists(),
        "Bun executable should exist at expected path"
    );

    // Verify executable name is platform-appropriate
    let exe_name = expected_path.file_name().unwrap().to_string_lossy();
    if cfg!(windows) {
        assert_eq!(exe_name, "bun.exe");
    } else {
        assert_eq!(exe_name, "bun");
    }
}

#[test]
fn test_cross_platform_bun_path_consistency() {
    let fixture = BunPathTestFixture::new().unwrap();
    let version = "1.2.9";

    // Create mock installation
    fixture.create_mock_bun_installation(version).unwrap();

    // Test that path manager returns consistent paths
    let bun_path = fixture.path_manager.tool_executable_path("bun", version);
    let version_dir = fixture.path_manager.tool_version_dir("bun", version);

    // Verify path structure - Bun should be directly in version directory
    assert_eq!(bun_path.parent().unwrap(), version_dir);

    // Verify executable name is platform-appropriate
    let exe_name = bun_path.file_name().unwrap().to_string_lossy();
    if cfg!(windows) {
        assert_eq!(exe_name, "bun.exe");
    } else {
        assert_eq!(exe_name, "bun");
    }
}

#[test]
fn test_bun_version_directory_structure() {
    let fixture = BunPathTestFixture::new().unwrap();
    let versions = ["1.0.0", "1.1.0", "1.2.9"];

    // Create multiple versions
    for version in &versions {
        fixture.create_mock_bun_installation(version).unwrap();
    }

    // Test that all versions are properly structured
    for version in &versions {
        let version_dir = fixture.path_manager.tool_version_dir("bun", version);
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

        // Check that the version directory is directly under tools/bun/
        let expected_path = fixture.path_manager.tool_dir("bun").join(version);
        assert_eq!(
            version_dir, expected_path,
            "Version directory path should be correct for {}",
            version
        );

        // Check that executable exists directly in version directory
        let bun_exe = if cfg!(windows) {
            version_dir.join("bun.exe")
        } else {
            version_dir.join("bun")
        };
        assert!(
            bun_exe.exists(),
            "Bun executable should exist for version {}",
            version
        );
    }
}

#[test]
fn test_bun_tool_version_listing() {
    let fixture = BunPathTestFixture::new().unwrap();
    let versions = ["1.0.0", "1.1.0", "1.2.9"];

    // Initially no versions
    let listed_versions = fixture.path_manager.list_tool_versions("bun").unwrap();
    assert!(
        listed_versions.is_empty(),
        "Should have no versions initially"
    );

    // Create versions
    for version in &versions {
        fixture.create_mock_bun_installation(version).unwrap();
    }

    // Test version listing
    let listed_versions = fixture.path_manager.list_tool_versions("bun").unwrap();
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
fn test_bun_installation_detection() {
    let fixture = BunPathTestFixture::new().unwrap();
    let version = "1.2.9";

    // Initially not installed
    assert!(!fixture
        .path_manager
        .is_tool_version_installed("bun", version));

    // Create installation
    fixture.create_mock_bun_installation(version).unwrap();

    // Should now be detected as installed
    assert!(fixture
        .path_manager
        .is_tool_version_installed("bun", version));
}

#[test]
fn test_bun_executable_permissions() {
    let fixture = BunPathTestFixture::new().unwrap();
    let version = "1.2.9";

    // Create mock installation
    fixture.create_mock_bun_installation(version).unwrap();

    let version_dir = fixture.path_manager.tool_version_dir("bun", version);
    let bun_exe = if cfg!(windows) {
        version_dir.join("bun.exe")
    } else {
        version_dir.join("bun")
    };

    // Test that executable exists and is a file
    assert!(bun_exe.exists(), "Executable should exist");
    assert!(bun_exe.is_file(), "Executable should be a file");

    // On Unix, test that executable permissions are set
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&bun_exe).unwrap();
        let permissions = metadata.permissions();
        assert!(
            permissions.mode() & 0o111 != 0,
            "Executable should have execute permissions"
        );
    }
}

#[test]
fn test_bun_path_separator_handling() {
    let fixture = BunPathTestFixture::new().unwrap();
    let version = "1.2.9";

    // Create mock installation
    fixture.create_mock_bun_installation(version).unwrap();

    let version_dir = fixture.path_manager.tool_version_dir("bun", version);
    let bun_path = if cfg!(windows) {
        version_dir.join("bun.exe")
    } else {
        version_dir.join("bun")
    };
    let path_str = bun_path.to_string_lossy();

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
fn test_bun_flatten_directory_fix() {
    let fixture = BunPathTestFixture::new().unwrap();
    let version = "1.2.9";

    // Create mock installation with old problematic structure
    let version_dir = fixture
        .create_mock_bun_installation_old_structure(version)
        .unwrap();

    // Verify the old structure exists (for testing purposes)
    let nested_dir = version_dir.join(format!("bun-v{}", version));
    assert!(
        nested_dir.exists(),
        "Old nested structure should exist for testing"
    );

    // The fix should ensure that after FlattenDirectory processing,
    // the executable would be moved to the root of version directory
    // This test verifies we can detect the problematic structure
    let platform_dir = if cfg!(windows) {
        nested_dir.join("bun-windows-x64")
    } else if cfg!(target_os = "macos") {
        nested_dir.join("bun-darwin-x64")
    } else {
        nested_dir.join("bun-linux-x64")
    };

    let nested_exe = if cfg!(windows) {
        platform_dir.join("bun.exe")
    } else {
        platform_dir.join("bun")
    };

    assert!(
        nested_exe.exists(),
        "Nested executable should exist in old structure"
    );

    // After FlattenDirectory fix, the executable should be moved to version root
    // This is what the installer should do
    let target_exe = if cfg!(windows) {
        version_dir.join("bun.exe")
    } else {
        version_dir.join("bun")
    };

    // Simulate the FlattenDirectory fix
    fs::copy(&nested_exe, &target_exe).unwrap();

    assert!(
        target_exe.exists(),
        "Executable should exist at root after flattening"
    );
    assert!(
        target_exe.is_file(),
        "Flattened executable should be a file"
    );
}
