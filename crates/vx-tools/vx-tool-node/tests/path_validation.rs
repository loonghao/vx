//! Path validation tests for Node.js tools
//!
//! This module tests the path calculation and validation logic for Node.js tools
//! across different platforms to ensure consistent behavior.

use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use vx_paths::PathManager;
use vx_plugin::VxTool;
use vx_tool_node::{NodeTool, NpmTool, NpxTool};

/// Test fixture for path validation tests
struct PathTestFixture {
    temp_dir: TempDir,
    path_manager: PathManager,
    node_tool: NodeTool,
    npm_tool: NpmTool,
    npx_tool: NpxTool,
}

impl PathTestFixture {
    fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let base_dir = temp_dir.path().join(".vx");
        let path_manager = PathManager::with_base_dir(&base_dir)?;

        Ok(Self {
            temp_dir,
            path_manager,
            node_tool: NodeTool::new(),
            npm_tool: NpmTool::new(),
            npx_tool: NpxTool::new(),
        })
    }

    /// Create a mock Node.js installation with all executables
    fn create_mock_node_installation(&self, version: &str) -> Result<PathBuf> {
        let version_dir = self.path_manager.tool_version_dir("node", version);
        fs::create_dir_all(&version_dir)?;

        // Create Node.js executable
        let node_exe = if cfg!(windows) {
            version_dir.join("node.exe")
        } else {
            version_dir.join("node")
        };
        fs::write(&node_exe, "fake node executable")?;

        // Create npm executable
        let npm_exe = if cfg!(windows) {
            version_dir.join("npm.cmd")
        } else {
            version_dir.join("npm")
        };
        fs::write(&npm_exe, "fake npm executable")?;

        // Create npx executable
        let npx_exe = if cfg!(windows) {
            version_dir.join("npx.cmd")
        } else {
            version_dir.join("npx")
        };
        fs::write(&npx_exe, "fake npx executable")?;

        // Set executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&node_exe)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&node_exe, perms)?;

            let mut perms = fs::metadata(&npm_exe)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&npm_exe, perms)?;

            let mut perms = fs::metadata(&npx_exe)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&npx_exe, perms)?;
        }

        Ok(version_dir)
    }
}

#[test]
fn test_node_path_structure() {
    let fixture = PathTestFixture::new().unwrap();
    let version = "24.2.0";

    // Create mock installation
    let version_dir = fixture.create_mock_node_installation(version).unwrap();

    // Test Node.js path structure
    let expected_node_exe = if cfg!(windows) {
        version_dir.join("node.exe")
    } else {
        version_dir.join("node")
    };

    assert!(
        expected_node_exe.exists(),
        "Node.js executable should exist at {}",
        expected_node_exe.display()
    );
    assert!(
        expected_node_exe.is_file(),
        "Node.js executable should be a file"
    );
}

#[test]
fn test_npm_path_structure() {
    let fixture = PathTestFixture::new().unwrap();
    let version = "24.2.0";

    // Create mock installation
    let version_dir = fixture.create_mock_node_installation(version).unwrap();

    // Test npm path structure
    let expected_npm_exe = if cfg!(windows) {
        version_dir.join("npm.cmd")
    } else {
        version_dir.join("npm")
    };

    assert!(
        expected_npm_exe.exists(),
        "npm executable should exist at {}",
        expected_npm_exe.display()
    );
    assert!(
        expected_npm_exe.is_file(),
        "npm executable should be a file"
    );
}

#[test]
fn test_npx_path_structure() {
    let fixture = PathTestFixture::new().unwrap();
    let version = "24.2.0";

    // Create mock installation
    let version_dir = fixture.create_mock_node_installation(version).unwrap();

    // Test npx path structure
    let expected_npx_exe = if cfg!(windows) {
        version_dir.join("npx.cmd")
    } else {
        version_dir.join("npx")
    };

    assert!(
        expected_npx_exe.exists(),
        "npx executable should exist at {}",
        expected_npx_exe.display()
    );
    assert!(
        expected_npx_exe.is_file(),
        "npx executable should be a file"
    );
}

#[test]
fn test_cross_platform_path_consistency() {
    let fixture = PathTestFixture::new().unwrap();
    let version = "24.2.0";

    // Create mock installation
    fixture.create_mock_node_installation(version).unwrap();

    // Test that path manager returns consistent paths
    let node_path = fixture.path_manager.tool_executable_path("node", version);
    let version_dir = fixture.path_manager.tool_version_dir("node", version);

    // Verify path structure
    assert_eq!(node_path.parent().unwrap(), version_dir);

    // Verify executable name is platform-appropriate
    let exe_name = node_path.file_name().unwrap().to_string_lossy();
    if cfg!(windows) {
        assert_eq!(exe_name, "node.exe");
    } else {
        assert_eq!(exe_name, "node");
    }
}

#[test]
fn test_version_directory_structure() {
    let fixture = PathTestFixture::new().unwrap();
    let versions = ["20.11.0", "22.0.0", "24.2.0"];

    // Create multiple versions
    for version in &versions {
        fixture.create_mock_node_installation(version).unwrap();
    }

    // Test that all versions are properly structured
    for version in &versions {
        let version_dir = fixture.path_manager.tool_version_dir("node", version);
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

        // Check that the version directory is directly under tools/node/
        let expected_path = fixture.path_manager.tool_dir("node").join(version);
        assert_eq!(
            version_dir, expected_path,
            "Version directory path should be correct for {}",
            version
        );
    }
}

#[test]
fn test_tool_version_listing() {
    let fixture = PathTestFixture::new().unwrap();
    let versions = ["20.11.0", "22.0.0", "24.2.0"];

    // Initially no versions
    let listed_versions = fixture.path_manager.list_tool_versions("node").unwrap();
    assert!(
        listed_versions.is_empty(),
        "Should have no versions initially"
    );

    // Create versions
    for version in &versions {
        fixture.create_mock_node_installation(version).unwrap();
    }

    // Test version listing
    let listed_versions = fixture.path_manager.list_tool_versions("node").unwrap();
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
fn test_tool_installation_detection() {
    let fixture = PathTestFixture::new().unwrap();
    let version = "24.2.0";

    // Initially not installed
    assert!(!fixture
        .path_manager
        .is_tool_version_installed("node", version));

    // Create installation
    fixture.create_mock_node_installation(version).unwrap();

    // Should now be detected as installed
    assert!(fixture
        .path_manager
        .is_tool_version_installed("node", version));
}

#[test]
fn test_executable_permissions() {
    let fixture = PathTestFixture::new().unwrap();
    let version = "24.2.0";

    // Create mock installation
    fixture.create_mock_node_installation(version).unwrap();

    let node_exe = fixture.path_manager.tool_executable_path("node", version);

    // Test that executable exists and is a file
    assert!(node_exe.exists(), "Executable should exist");
    assert!(node_exe.is_file(), "Executable should be a file");

    // On Unix, test that executable permissions are set
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&node_exe).unwrap();
        let permissions = metadata.permissions();
        assert!(
            permissions.mode() & 0o111 != 0,
            "Executable should have execute permissions"
        );
    }
}

#[test]
fn test_path_separator_handling() {
    let fixture = PathTestFixture::new().unwrap();
    let version = "24.2.0";

    // Create mock installation
    fixture.create_mock_node_installation(version).unwrap();

    let node_path = fixture.path_manager.tool_executable_path("node", version);
    let path_str = node_path.to_string_lossy();

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
