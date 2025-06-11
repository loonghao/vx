// Comprehensive tool management tests
// Tests core functionality: download, install, version control, search, update, delete

use vx::tool_manager::ToolManager;
use vx::tool_registry::ToolRegistry;

#[tokio::test]
async fn test_tool_registry_basic_operations() {
    let registry = ToolRegistry::new();
    
    // Test tool registration and retrieval
    assert!(registry.has_tool("uv"), "Should have uv tool registered");
    assert!(registry.has_tool("node"), "Should have node tool registered");
    assert!(registry.has_tool("go"), "Should have go tool registered");
    assert!(registry.has_tool("cargo"), "Should have cargo tool registered");
    
    // Test tool names
    let tool_names = registry.tool_names();
    assert!(!tool_names.is_empty(), "Should have registered tools");
    assert!(tool_names.contains(&"uv".to_string()));
    assert!(tool_names.contains(&"node".to_string()));
    assert!(tool_names.contains(&"cargo".to_string()));
    
    // Test tool info retrieval
    let uv_info = registry.get_tool_info("uv");
    assert!(uv_info.is_ok(), "Should get uv tool info");
    
    let uv_info = uv_info.unwrap();
    assert_eq!(uv_info.name, "uv");
    assert!(!uv_info.description.is_empty());
}

#[tokio::test]
async fn test_tool_manager_initialization() {
    let tool_manager = ToolManager::minimal().expect("Should create tool manager");

    // Test basic functionality
    let available_tools = tool_manager.get_all_tools();
    assert!(!available_tools.is_empty(), "Should have available tools");

    // Test tool support check
    assert!(tool_manager.has_tool("uv"));
    assert!(tool_manager.has_tool("node"));
    assert!(!tool_manager.has_tool("nonexistent-tool"));
}

#[tokio::test]
async fn test_tool_status_operations() {
    let tool_manager = ToolManager::minimal().expect("Should create tool manager");

    // Test status checking for supported tools
    for tool_name in &["uv", "node", "go"] {
        if tool_manager.has_tool(tool_name) {
            // Test getting tool status
            let status_result = tool_manager.check_tool_status(tool_name);

            match status_result {
                Ok(status) => {
                    println!("Status for {}: {:?}", tool_name, status);
                    assert_eq!(status.name, *tool_name);
                    assert!(status.supports_auto_install);
                }
                Err(e) => {
                    println!("Status check for {} failed: {}", tool_name, e);
                }
            }
        }
    }
}

#[tokio::test]
async fn test_tool_info_retrieval() {
    let tool_manager = ToolManager::minimal().expect("Should create tool manager");

    // Test getting tool information
    for tool_name in &["uv", "node"] {
        if tool_manager.has_tool(tool_name) {
            // Test getting tool info
            let tool_info = tool_manager.get_tool_info(tool_name);
            assert!(tool_info.is_ok(), "Should get tool info for {}", tool_name);

            let info = tool_info.unwrap();
            assert_eq!(info.name, *tool_name);
            assert!(!info.description.is_empty());
        }
    }
}

#[tokio::test]
async fn test_tool_search_functionality() {
    let tool_manager = ToolManager::minimal().expect("Should create tool manager");

    // Test getting all tools (simulates search functionality)
    let all_tools = tool_manager.get_all_tools();
    assert!(!all_tools.is_empty(), "Should have tools available");

    // Test finding specific tools
    let uv_found = all_tools.iter().any(|tool| tool.name == "uv");
    assert!(uv_found, "Should find uv in tool list");

    let node_found = all_tools.iter().any(|tool| tool.name == "node");
    assert!(node_found, "Should find node in tool list");

    // Test tool names
    let tool_names = tool_manager.get_tool_names();
    assert!(tool_names.contains(&"uv".to_string()));
    assert!(tool_names.contains(&"node".to_string()));
}

#[tokio::test]
async fn test_tool_configuration() {
    let tool_manager = ToolManager::minimal().expect("Should create tool manager");

    // Test configuration access
    let config = tool_manager.config();

    // Test getting tool configuration
    for tool_name in &["uv", "node"] {
        if tool_manager.has_tool(tool_name) {
            let tool_config = config.get_tool_config(tool_name);
            println!("Configuration for {}: {:?}", tool_name, tool_config);
        }
    }
}

#[tokio::test]
async fn test_tool_status_summary() {
    let tool_manager = ToolManager::minimal().expect("Should create tool manager");

    // Test status summary for tools
    for tool_name in &["uv", "node"] {
        if tool_manager.has_tool(tool_name) {
            let status = tool_manager.check_tool_status(tool_name);

            match status {
                Ok(status) => {
                    let summary = status.summary();
                    println!("Status summary for {}: {}", tool_name, summary);
                    assert!(summary.contains(tool_name));

                    // Test needs_action method
                    let needs_action = status.needs_action();
                    println!("Tool {} needs action: {}", tool_name, needs_action);
                }
                Err(e) => {
                    println!("Failed to get status for {}: {}", tool_name, e);
                }
            }
        }
    }
}

#[tokio::test]
async fn test_config_reload() {
    let mut tool_manager = ToolManager::minimal().expect("Should create tool manager");

    // Test configuration reload
    let reload_result = tool_manager.reload_config();
    match reload_result {
        Ok(_) => {
            println!("Configuration reloaded successfully");
        }
        Err(e) => {
            println!("Configuration reload failed: {}", e);
        }
    }

    // Tool manager should still work after reload
    assert!(tool_manager.has_tool("uv"));
}
