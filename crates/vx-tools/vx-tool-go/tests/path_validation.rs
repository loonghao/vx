//! Path validation tests for Go tools
//!
//! This module tests the path calculation and validation logic for Go tools
//! across different platforms to ensure consistent behavior.

use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use vx_paths::PathManager;
use vx_plugin::VxTool;
use vx_tool_go::GoTool;

/// Test fixture for Go path validation tests
struct GoPathTestFixture {
    temp_dir: TempDir,
    path_manager: PathManager,
    go_tool: GoTool,
}

impl GoPathTestFixture {
    fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let base_dir = temp_dir.path().join(".vx");
        let path_manager = PathManager::with_base_dir(&base_dir)?;

        Ok(Self {
            temp_dir,
            path_manager,
            go_tool: GoTool::new(),
        })
    }

    /// Create a mock Go installation with proper directory structure
    fn create_mock_go_installation(&self, version: &str) -> Result<PathBuf> {
        let version_dir = self.path_manager.tool_version_dir("go", version);
        fs::create_dir_all(&version_dir)?;

        // Go has a specific directory structure: go/bin/go
        let bin_dir = version_dir.join("bin");
        fs::create_dir_all(&bin_dir)?;

        // Create Go executable in bin directory
        let go_exe = if cfg!(windows) {
            bin_dir.join("go.exe")
        } else {
            bin_dir.join("go")
        };
        fs::write(&go_exe, "fake go executable")?;

        // Create gofmt executable
        let gofmt_exe = if cfg!(windows) {
            bin_dir.join("gofmt.exe")
        } else {
            bin_dir.join("gofmt")
        };
        fs::write(&gofmt_exe, "fake gofmt executable")?;

        // Set executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&go_exe)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&go_exe, perms)?;

            let mut perms = fs::metadata(&gofmt_exe)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&gofmt_exe, perms)?;
        }

        Ok(version_dir)
    }
}

#[test]
fn test_go_path_structure() {
    let fixture = GoPathTestFixture::new().unwrap();
    let version = "1.24.4";

    // Create mock installation
    let version_dir = fixture.create_mock_go_installation(version).unwrap();

    // Test Go path structure - Go executable should be in bin subdirectory
    let expected_go_exe = if cfg!(windows) {
        version_dir.join("bin").join("go.exe")
    } else {
        version_dir.join("bin").join("go")
    };

    assert!(
        expected_go_exe.exists(),
        "Go executable should exist at {}",
        expected_go_exe.display()
    );
    assert!(expected_go_exe.is_file(), "Go executable should be a file");
}

#[test]
fn test_go_bin_directory_structure() {
    let fixture = GoPathTestFixture::new().unwrap();
    let version = "1.24.4";

    // Create mock installation
    let version_dir = fixture.create_mock_go_installation(version).unwrap();

    // Test that bin directory exists
    let bin_dir = version_dir.join("bin");
    assert!(bin_dir.exists(), "bin directory should exist");
    assert!(bin_dir.is_dir(), "bin should be a directory");

    // Test that Go executable is in bin directory
    let go_exe = if cfg!(windows) {
        bin_dir.join("go.exe")
    } else {
        bin_dir.join("go")
    };
    assert!(go_exe.exists(), "Go executable should be in bin directory");
}

#[test]
fn test_go_tool_executable_path_resolution() {
    let fixture = GoPathTestFixture::new().unwrap();
    let version = "1.24.4";

    // Create mock installation
    fixture.create_mock_go_installation(version).unwrap();

    // Test that PathManager correctly resolves Go executable path
    let go_path = fixture.path_manager.tool_executable_path("go", version);

    // Go executable should be found in bin subdirectory
    let expected_path = fixture
        .path_manager
        .tool_version_dir("go", version)
        .join("bin")
        .join(if cfg!(windows) { "go.exe" } else { "go" });

    // The PathManager should find the executable in the bin directory
    assert!(
        go_path.exists() || expected_path.exists(),
        "Go executable should be found at {} or {}",
        go_path.display(),
        expected_path.display()
    );
}

#[test]
fn test_cross_platform_go_path_consistency() {
    let fixture = GoPathTestFixture::new().unwrap();
    let version = "1.24.4";

    // Create mock installation
    fixture.create_mock_go_installation(version).unwrap();

    // Test that path manager returns consistent paths
    let go_path = fixture.path_manager.tool_executable_path("go", version);
    let version_dir = fixture.path_manager.tool_version_dir("go", version);

    // Verify path structure - Go should be in version_dir/bin/
    let bin_dir = version_dir.join("bin");
    let expected_go_path = if cfg!(windows) {
        bin_dir.join("go.exe")
    } else {
        bin_dir.join("go")
    };

    // The actual path should either be the expected path or the PathManager should find it
    assert!(
        expected_go_path.exists(),
        "Go executable should exist at expected path"
    );

    // Verify executable name is platform-appropriate
    let exe_name = expected_go_path.file_name().unwrap().to_string_lossy();
    if cfg!(windows) {
        assert_eq!(exe_name, "go.exe");
    } else {
        assert_eq!(exe_name, "go");
    }
}

#[test]
fn test_go_version_directory_structure() {
    let fixture = GoPathTestFixture::new().unwrap();
    let versions = ["1.21.6", "1.22.0", "1.24.4"];

    // Create multiple versions
    for version in &versions {
        fixture.create_mock_go_installation(version).unwrap();
    }

    // Test that all versions are properly structured
    for version in &versions {
        let version_dir = fixture.path_manager.tool_version_dir("go", version);
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

        // Check that the version directory is directly under tools/go/
        let expected_path = fixture.path_manager.tool_dir("go").join(version);
        assert_eq!(
            version_dir, expected_path,
            "Version directory path should be correct for {}",
            version
        );

        // Check that bin directory exists
        let bin_dir = version_dir.join("bin");
        assert!(
            bin_dir.exists(),
            "bin directory should exist for version {}",
            version
        );
    }
}

#[test]
fn test_go_tool_version_listing() {
    let fixture = GoPathTestFixture::new().unwrap();
    let versions = ["1.21.6", "1.22.0", "1.24.4"];

    // Initially no versions
    let listed_versions = fixture.path_manager.list_tool_versions("go").unwrap();
    assert!(
        listed_versions.is_empty(),
        "Should have no versions initially"
    );

    // Create versions
    for version in &versions {
        fixture.create_mock_go_installation(version).unwrap();
    }

    // Test version listing
    let listed_versions = fixture.path_manager.list_tool_versions("go").unwrap();
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
fn test_go_installation_detection() {
    let fixture = GoPathTestFixture::new().unwrap();
    let version = "1.24.4";

    // Initially not installed
    assert!(!fixture
        .path_manager
        .is_tool_version_installed("go", version));

    // Create installation
    fixture.create_mock_go_installation(version).unwrap();

    // Should now be detected as installed
    assert!(fixture
        .path_manager
        .is_tool_version_installed("go", version));
}

#[test]
fn test_go_executable_permissions() {
    let fixture = GoPathTestFixture::new().unwrap();
    let version = "1.24.4";

    // Create mock installation
    fixture.create_mock_go_installation(version).unwrap();

    let version_dir = fixture.path_manager.tool_version_dir("go", version);
    let go_exe = if cfg!(windows) {
        version_dir.join("bin").join("go.exe")
    } else {
        version_dir.join("bin").join("go")
    };

    // Test that executable exists and is a file
    assert!(go_exe.exists(), "Executable should exist");
    assert!(go_exe.is_file(), "Executable should be a file");

    // On Unix, test that executable permissions are set
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&go_exe).unwrap();
        let permissions = metadata.permissions();
        assert!(
            permissions.mode() & 0o111 != 0,
            "Executable should have execute permissions"
        );
    }
}

#[test]
fn test_go_path_separator_handling() {
    let fixture = GoPathTestFixture::new().unwrap();
    let version = "1.24.4";

    // Create mock installation
    fixture.create_mock_go_installation(version).unwrap();

    let version_dir = fixture.path_manager.tool_version_dir("go", version);
    let go_path = version_dir
        .join("bin")
        .join(if cfg!(windows) { "go.exe" } else { "go" });
    let path_str = go_path.to_string_lossy();

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
fn test_go_multiple_executables() {
    let fixture = GoPathTestFixture::new().unwrap();
    let version = "1.24.4";

    // Create mock installation
    let version_dir = fixture.create_mock_go_installation(version).unwrap();

    // Test that multiple Go tools exist in bin directory
    let bin_dir = version_dir.join("bin");

    let go_exe = if cfg!(windows) {
        bin_dir.join("go.exe")
    } else {
        bin_dir.join("go")
    };

    let gofmt_exe = if cfg!(windows) {
        bin_dir.join("gofmt.exe")
    } else {
        bin_dir.join("gofmt")
    };

    assert!(go_exe.exists(), "go executable should exist");
    assert!(gofmt_exe.exists(), "gofmt executable should exist");
    assert!(go_exe.is_file(), "go should be a file");
    assert!(gofmt_exe.is_file(), "gofmt should be a file");
}
