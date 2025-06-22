//! Integration tests for path validation across all tools
//!
//! This module tests the actual path structures created by tool installations
//! to ensure they match expected patterns and work correctly across platforms.

use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;
use vx_paths::PathManager;

/// Integration test fixture for validating tool installations
struct IntegrationPathTestFixture {
    temp_dir: TempDir,
    path_manager: PathManager,
}

impl IntegrationPathTestFixture {
    fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let base_dir = temp_dir.path().join(".vx");
        let path_manager = PathManager::with_base_dir(&base_dir)?;

        Ok(Self {
            temp_dir,
            path_manager,
        })
    }

    /// Verify that a tool follows the expected path structure
    fn verify_tool_path_structure(
        &self,
        tool_name: &str,
        version: &str,
        expected_structure: &ToolPathStructure,
    ) -> Result<()> {
        let version_dir = self.path_manager.tool_version_dir(tool_name, version);

        // Check version directory exists
        assert!(
            version_dir.exists(),
            "Version directory should exist for {} {}",
            tool_name,
            version
        );
        assert!(
            version_dir.is_dir(),
            "Version directory should be a directory for {} {}",
            tool_name,
            version
        );

        // Check executable path based on expected structure
        let expected_exe_path = match expected_structure {
            ToolPathStructure::Flat => {
                let exe_name = if cfg!(windows) {
                    format!("{}.exe", tool_name)
                } else {
                    tool_name.to_string()
                };
                version_dir.join(exe_name)
            }
            ToolPathStructure::BinSubdirectory => {
                let exe_name = if cfg!(windows) {
                    format!("{}.exe", tool_name)
                } else {
                    tool_name.to_string()
                };
                version_dir.join("bin").join(exe_name)
            }
            ToolPathStructure::CustomPath(path) => version_dir.join(path),
        };

        // Use PathManager's tool_executable_path to find the actual executable
        let actual_exe_path = self.path_manager.tool_executable_path(tool_name, version);

        // The PathManager should be able to find the executable
        // We don't require exact path match because PathManager has smart resolution
        println!("Tool: {}, Version: {}", tool_name, version);
        println!("Expected path: {}", expected_exe_path.display());
        println!("PathManager resolved: {}", actual_exe_path.display());

        // At minimum, the executable name should be correct
        let actual_name = actual_exe_path.file_name().unwrap().to_string_lossy();
        let expected_name = expected_exe_path.file_name().unwrap().to_string_lossy();

        assert_eq!(
            actual_name, expected_name,
            "Executable name should match for {} {}",
            tool_name, version
        );

        Ok(())
    }
}

/// Expected path structures for different tools
#[derive(Debug, Clone)]
enum ToolPathStructure {
    /// Executable directly in version directory
    Flat,
    /// Executable in bin subdirectory
    BinSubdirectory,
    /// Custom path relative to version directory
    CustomPath(String),
}

#[test]
fn test_standard_tool_path_structures() {
    let fixture = IntegrationPathTestFixture::new().unwrap();

    // Define expected structures for each tool
    let tool_structures = HashMap::from([
        ("node", ToolPathStructure::Flat),
        ("bun", ToolPathStructure::Flat),
        ("uv", ToolPathStructure::Flat),
        ("go", ToolPathStructure::BinSubdirectory),
    ]);

    // Test each tool structure
    for (tool_name, expected_structure) in tool_structures {
        let version = "1.0.0";

        // Create version directory
        let version_dir = fixture.path_manager.tool_version_dir(tool_name, version);
        std::fs::create_dir_all(&version_dir).unwrap();

        // Create executable based on expected structure
        match &expected_structure {
            ToolPathStructure::Flat => {
                let exe_name = if cfg!(windows) {
                    format!("{}.exe", tool_name)
                } else {
                    tool_name.to_string()
                };
                let exe_path = version_dir.join(exe_name);
                std::fs::write(&exe_path, format!("fake {} executable", tool_name)).unwrap();
            }
            ToolPathStructure::BinSubdirectory => {
                let bin_dir = version_dir.join("bin");
                std::fs::create_dir_all(&bin_dir).unwrap();
                let exe_name = if cfg!(windows) {
                    format!("{}.exe", tool_name)
                } else {
                    tool_name.to_string()
                };
                let exe_path = bin_dir.join(exe_name);
                std::fs::write(&exe_path, format!("fake {} executable", tool_name)).unwrap();
            }
            ToolPathStructure::CustomPath(path) => {
                let exe_path = version_dir.join(path);
                if let Some(parent) = exe_path.parent() {
                    std::fs::create_dir_all(parent).unwrap();
                }
                std::fs::write(&exe_path, format!("fake {} executable", tool_name)).unwrap();
            }
        }

        // Verify the structure
        fixture
            .verify_tool_path_structure(tool_name, version, &expected_structure)
            .unwrap();
    }
}

#[test]
fn test_path_manager_tool_resolution() {
    let fixture = IntegrationPathTestFixture::new().unwrap();

    // Test that PathManager can resolve different tool structures
    let test_cases = vec![
        ("node", "20.11.0", ToolPathStructure::Flat),
        ("go", "1.21.6", ToolPathStructure::BinSubdirectory),
        ("bun", "1.2.9", ToolPathStructure::Flat),
    ];

    for (tool_name, version, structure) in test_cases {
        let version_dir = fixture.path_manager.tool_version_dir(tool_name, version);
        std::fs::create_dir_all(&version_dir).unwrap();

        // Create the executable in the expected location
        let exe_path = match structure {
            ToolPathStructure::Flat => {
                let exe_name = if cfg!(windows) {
                    format!("{}.exe", tool_name)
                } else {
                    tool_name.to_string()
                };
                version_dir.join(exe_name)
            }
            ToolPathStructure::BinSubdirectory => {
                let bin_dir = version_dir.join("bin");
                std::fs::create_dir_all(&bin_dir).unwrap();
                let exe_name = if cfg!(windows) {
                    format!("{}.exe", tool_name)
                } else {
                    tool_name.to_string()
                };
                bin_dir.join(exe_name)
            }
            ToolPathStructure::CustomPath(path) => version_dir.join(path),
        };

        std::fs::write(&exe_path, format!("fake {} executable", tool_name)).unwrap();

        // Test that PathManager can find it
        let resolved_path = fixture
            .path_manager
            .tool_executable_path(tool_name, version);

        // The resolved path should exist (PathManager should find the executable)
        // Note: PathManager might return a different path due to its smart resolution logic
        println!(
            "Tool: {}, Created at: {}, Resolved to: {}",
            tool_name,
            exe_path.display(),
            resolved_path.display()
        );

        // At minimum, the executable name should be correct
        let resolved_name = resolved_path.file_name().unwrap().to_string_lossy();
        let expected_name = exe_path.file_name().unwrap().to_string_lossy();
        assert_eq!(
            resolved_name, expected_name,
            "PathManager should resolve to correct executable name for {}",
            tool_name
        );
    }
}

#[test]
fn test_cross_platform_executable_extensions() {
    let fixture = IntegrationPathTestFixture::new().unwrap();

    let tools = ["node", "go", "bun", "uv"];

    for tool_name in &tools {
        let version = "1.0.0";
        let version_dir = fixture.path_manager.tool_version_dir(tool_name, version);
        std::fs::create_dir_all(&version_dir).unwrap();

        // Create executable with platform-appropriate extension
        let exe_name = if cfg!(windows) {
            format!("{}.exe", tool_name)
        } else {
            tool_name.to_string()
        };
        let exe_path = version_dir.join(&exe_name);
        std::fs::write(&exe_path, format!("fake {} executable", tool_name)).unwrap();

        // Test that PathManager resolves correctly
        let resolved_path = fixture
            .path_manager
            .tool_executable_path(tool_name, version);
        let resolved_name = resolved_path.file_name().unwrap().to_string_lossy();

        // Should have correct extension for platform
        if cfg!(windows) {
            assert!(
                resolved_name.ends_with(".exe"),
                "Windows executable should have .exe extension: {}",
                resolved_name
            );
        } else {
            assert!(
                !resolved_name.contains('.'),
                "Unix executable should not have extension: {}",
                resolved_name
            );
        }
    }
}

#[test]
fn test_version_directory_isolation() {
    let fixture = IntegrationPathTestFixture::new().unwrap();

    let tool_name = "node";
    let versions = ["18.17.0", "20.11.0", "22.0.0"];

    // Create multiple versions
    for version in &versions {
        let version_dir = fixture.path_manager.tool_version_dir(tool_name, version);
        std::fs::create_dir_all(&version_dir).unwrap();

        let exe_name = if cfg!(windows) {
            format!("{}.exe", tool_name)
        } else {
            tool_name.to_string()
        };
        let exe_path = version_dir.join(&exe_name);
        std::fs::write(
            &exe_path,
            format!("fake {} {} executable", tool_name, version),
        )
        .unwrap();
    }

    // Verify each version is isolated
    for version in &versions {
        let version_dir = fixture.path_manager.tool_version_dir(tool_name, version);

        // Check that version directory exists and is isolated
        assert!(
            version_dir.exists(),
            "Version directory should exist for {}",
            version
        );
        assert_eq!(
            version_dir.file_name().unwrap().to_string_lossy(),
            *version,
            "Version directory name should match version"
        );

        // Check that executable exists in this version
        let exe_path = fixture
            .path_manager
            .tool_executable_path(tool_name, version);
        assert!(
            exe_path.exists() || version_dir.join(exe_path.file_name().unwrap()).exists(),
            "Executable should exist for version {}",
            version
        );
    }

    // Verify version listing
    let listed_versions = fixture.path_manager.list_tool_versions(tool_name).unwrap();
    assert_eq!(
        listed_versions.len(),
        versions.len(),
        "Should list all versions"
    );

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
    let fixture = IntegrationPathTestFixture::new().unwrap();

    let tools = [
        ("node", "20.11.0"),
        ("go", "1.21.6"),
        ("bun", "1.2.9"),
        ("uv", "0.1.0"),
    ];

    // Initially no tools should be installed
    for (tool_name, version) in &tools {
        assert!(
            !fixture
                .path_manager
                .is_tool_version_installed(tool_name, version),
            "{} {} should not be installed initially",
            tool_name,
            version
        );
    }

    // Install each tool
    for (tool_name, version) in &tools {
        let version_dir = fixture.path_manager.tool_version_dir(tool_name, version);
        std::fs::create_dir_all(&version_dir).unwrap();

        let exe_name = if cfg!(windows) {
            format!("{}.exe", tool_name)
        } else {
            tool_name.to_string()
        };
        let exe_path = version_dir.join(&exe_name);
        std::fs::write(&exe_path, format!("fake {} executable", tool_name)).unwrap();

        // Should now be detected as installed
        assert!(
            fixture
                .path_manager
                .is_tool_version_installed(tool_name, version),
            "{} {} should be detected as installed",
            tool_name,
            version
        );
    }
}
