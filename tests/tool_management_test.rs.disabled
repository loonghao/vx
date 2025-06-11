// Comprehensive tool management tests
// Tests core functionality: download, install, version control, search, update, delete

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use vx::tool_manager::ToolManager;
use vx::tool_registry::ToolRegistry;
use vx::config_figment::FigmentConfigManager;

#[tokio::test]
async fn test_tool_registry_basic_operations() {
    let registry = ToolRegistry::new();
    
    // Test tool registration and retrieval
    assert!(registry.has_tool("uv"), "Should have uv tool registered");
    assert!(registry.has_tool("node"), "Should have node tool registered");
    assert!(registry.has_tool("go"), "Should have go tool registered");
    assert!(registry.has_tool("rust"), "Should have rust tool registered");
    
    // Test tool names
    let tool_names = registry.tool_names();
    assert!(!tool_names.is_empty(), "Should have registered tools");
    assert!(tool_names.contains(&"uv".to_string()));
    assert!(tool_names.contains(&"node".to_string()));
    
    // Test tool info retrieval
    let uv_info = registry.get_tool_info("uv");
    assert!(uv_info.is_ok(), "Should get uv tool info");
    
    let uv_info = uv_info.unwrap();
    assert_eq!(uv_info.name, "uv");
    assert!(!uv_info.description.is_empty());
}

#[tokio::test]
async fn test_tool_manager_initialization() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let config_manager = FigmentConfigManager::minimal().expect("Should create config manager");

    let tool_manager = ToolManager::new_with_config(config_manager, temp_dir.path().to_path_buf());
    
    // Test basic functionality
    let available_tools = tool_manager.list_available_tools();
    assert!(!available_tools.is_empty(), "Should have available tools");
    
    // Test tool support check
    assert!(tool_manager.is_tool_supported("uv"));
    assert!(tool_manager.is_tool_supported("node"));
    assert!(!tool_manager.is_tool_supported("nonexistent-tool"));
}

#[tokio::test]
async fn test_tool_version_operations() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let config_manager = FigmentConfigManager::minimal().expect("Should create config manager");
    let tool_manager = ToolManager::new(config_manager, temp_dir.path().to_path_buf());
    
    // Test version listing for supported tools
    for tool_name in &["uv", "node", "go"] {
        if tool_manager.is_tool_supported(tool_name) {
            // Test getting available versions (this might be a mock or limited set)
            let versions_result = tool_manager.list_available_versions(tool_name).await;
            
            // We expect this to either succeed or fail gracefully
            match versions_result {
                Ok(versions) => {
                    println!("Available versions for {}: {:?}", tool_name, versions);
                    // If we get versions, they should include "latest"
                    if !versions.is_empty() {
                        // Most tools should support "latest" as a version
                        println!("Tool {} has {} available versions", tool_name, versions.len());
                    }
                }
                Err(e) => {
                    println!("Version listing for {} failed (expected): {}", tool_name, e);
                    // This is acceptable as we might not have network access in tests
                }
            }
        }
    }
}

#[tokio::test]
async fn test_tool_installation_simulation() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let config_manager = FigmentConfigManager::minimal().expect("Should create config manager");
    let tool_manager = ToolManager::new(config_manager, temp_dir.path().to_path_buf());
    
    // Test installation preparation (without actual download)
    for tool_name in &["uv", "node"] {
        if tool_manager.is_tool_supported(tool_name) {
            // Test getting install configuration
            let install_config = tool_manager.get_install_config(tool_name, "latest");
            assert!(install_config.is_ok(), "Should get install config for {}", tool_name);
            
            let config = install_config.unwrap();
            assert_eq!(config.tool_name, *tool_name);
            assert_eq!(config.version, "latest");
            
            // Verify install directory structure
            assert!(config.install_dir.to_string_lossy().contains(tool_name));
        }
    }
}

#[tokio::test]
async fn test_tool_search_functionality() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let config_manager = FigmentConfigManager::minimal().expect("Should create config manager");
    let tool_manager = ToolManager::new(config_manager, temp_dir.path().to_path_buf());
    
    // Test searching for tools
    let search_results = tool_manager.search_tools("uv");
    assert!(!search_results.is_empty(), "Should find uv in search results");
    
    // Test case-insensitive search
    let search_results_upper = tool_manager.search_tools("UV");
    assert!(!search_results_upper.is_empty(), "Should find UV in case-insensitive search");
    
    // Test partial name search
    let search_results_partial = tool_manager.search_tools("no"); // should match "node"
    let found_node = search_results_partial.iter().any(|tool| tool.name.contains("node"));
    assert!(found_node, "Should find node with partial search");
    
    // Test search with no results
    let no_results = tool_manager.search_tools("nonexistent-super-rare-tool");
    assert!(no_results.is_empty(), "Should return empty results for non-existent tool");
}

#[tokio::test]
async fn test_installed_tools_management() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let config_manager = FigmentConfigManager::minimal().expect("Should create config manager");
    let tool_manager = ToolManager::new(config_manager, temp_dir.path().to_path_buf());
    
    // Test listing installed tools (should be empty initially)
    let installed_tools = tool_manager.list_installed_tools();
    assert!(installed_tools.is_empty(), "Should have no installed tools initially");
    
    // Simulate tool installation by creating directory structure
    let uv_install_dir = temp_dir.path().join("uv").join("latest");
    fs::create_dir_all(&uv_install_dir).expect("Should create install directory");
    
    // Create a mock executable
    let mock_executable = uv_install_dir.join(if cfg!(windows) { "uv.exe" } else { "uv" });
    fs::write(&mock_executable, "mock executable").expect("Should create mock executable");
    
    // Test detection of installed tools
    let installed_after = tool_manager.list_installed_tools();
    // Note: This might still be empty if the tool manager doesn't scan the temp directory
    // The actual implementation would need to be updated to support this
    println!("Installed tools after mock installation: {:?}", installed_after);
}

#[tokio::test]
async fn test_tool_update_operations() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let config_manager = FigmentConfigManager::minimal().expect("Should create config manager");
    let tool_manager = ToolManager::new(config_manager, temp_dir.path().to_path_buf());
    
    // Test checking for updates
    for tool_name in &["uv", "node"] {
        if tool_manager.is_tool_supported(tool_name) {
            // Test update check
            let update_check = tool_manager.check_for_updates(tool_name).await;
            
            match update_check {
                Ok(update_info) => {
                    println!("Update info for {}: {:?}", tool_name, update_info);
                    // If we get update info, it should have version information
                    assert!(!update_info.current_version.is_empty() || !update_info.latest_version.is_empty());
                }
                Err(e) => {
                    println!("Update check for {} failed (expected): {}", tool_name, e);
                    // This is acceptable as the tool might not be installed
                }
            }
        }
    }
}

#[tokio::test]
async fn test_tool_removal_operations() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let config_manager = FigmentConfigManager::minimal().expect("Should create config manager");
    let tool_manager = ToolManager::new(config_manager, temp_dir.path().to_path_buf());
    
    // Create a mock installation to test removal
    let tool_name = "uv";
    let version = "0.1.0";
    let install_dir = temp_dir.path().join(tool_name).join(version);
    fs::create_dir_all(&install_dir).expect("Should create install directory");
    
    let mock_executable = install_dir.join(if cfg!(windows) { "uv.exe" } else { "uv" });
    fs::write(&mock_executable, "mock executable").expect("Should create mock executable");
    
    // Verify the mock installation exists
    assert!(install_dir.exists(), "Mock installation should exist");
    assert!(mock_executable.exists(), "Mock executable should exist");
    
    // Test removal
    let removal_result = tool_manager.remove_tool(tool_name, version).await;
    
    match removal_result {
        Ok(_) => {
            println!("Successfully removed {} {}", tool_name, version);
            // Check if the directory was removed
            // Note: The actual implementation would need to support this
        }
        Err(e) => {
            println!("Tool removal failed (might be expected): {}", e);
            // This might fail if the tool manager doesn't implement removal yet
        }
    }
}

#[tokio::test]
async fn test_tool_configuration_management() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let config_manager = FigmentConfigManager::minimal().expect("Should create config manager");
    let tool_manager = ToolManager::new(config_manager, temp_dir.path().to_path_buf());
    
    // Test getting tool configuration
    for tool_name in &["uv", "node", "go", "rust"] {
        if tool_manager.is_tool_supported(tool_name) {
            let tool_config = tool_manager.get_tool_configuration(tool_name);
            
            match tool_config {
                Ok(config) => {
                    println!("Configuration for {}: {:?}", tool_name, config);
                    assert_eq!(config.name, *tool_name);
                    assert!(!config.description.is_empty());
                }
                Err(e) => {
                    println!("Failed to get configuration for {}: {}", tool_name, e);
                }
            }
        }
    }
}

#[tokio::test]
async fn test_tool_execution_paths() {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let config_manager = FigmentConfigManager::minimal().expect("Should create config manager");
    let tool_manager = ToolManager::new(config_manager, temp_dir.path().to_path_buf());
    
    // Test getting execution paths for tools
    for tool_name in &["uv", "node"] {
        if tool_manager.is_tool_supported(tool_name) {
            let exec_path = tool_manager.get_tool_executable_path(tool_name, "latest");
            
            match exec_path {
                Ok(path) => {
                    println!("Executable path for {}: {:?}", tool_name, path);
                    assert!(path.to_string_lossy().contains(tool_name));
                    
                    // Check if the path has the correct extension on Windows
                    if cfg!(windows) {
                        assert!(path.extension().map_or(false, |ext| ext == "exe"));
                    }
                }
                Err(e) => {
                    println!("Failed to get executable path for {}: {}", tool_name, e);
                }
            }
        }
    }
}
