use vx_plugin::ToolDependency;

#[test]
fn test_tool_dependency_required() {
    let dep = ToolDependency::required("node", "Node.js runtime");

    assert_eq!(dep.tool_name, "node");
    assert_eq!(dep.description, "Node.js runtime");
    assert!(dep.required);
    assert!(dep.version_requirement.is_none());
}

#[test]
fn test_tool_dependency_optional() {
    let dep = ToolDependency::optional("python", "Python interpreter");

    assert_eq!(dep.tool_name, "python");
    assert_eq!(dep.description, "Python interpreter");
    assert!(!dep.required);
    assert!(dep.version_requirement.is_none());
}

#[test]
fn test_tool_dependency_with_version() {
    let dep = ToolDependency::required("node", "Node.js runtime").with_version(">=16.0.0");

    assert_eq!(dep.tool_name, "node");
    assert_eq!(dep.description, "Node.js runtime");
    assert!(dep.required);
    assert_eq!(dep.version_requirement, Some(">=16.0.0".to_string()));
}

#[test]
fn test_tool_dependency_chaining() {
    let dep = ToolDependency::optional("go", "Go programming language")
        .with_version(">=1.19.0");

    assert_eq!(dep.tool_name, "go");
    assert_eq!(dep.description, "Go programming language");
    assert!(!dep.required);
    assert_eq!(dep.version_requirement, Some(">=1.19.0".to_string()));
}

#[test]
fn test_tool_dependency_serialization() {
    let dep = ToolDependency::required("test", "Test tool").with_version("1.0.0");

    let serialized = serde_json::to_string(&dep).unwrap();
    let deserialized: ToolDependency = serde_json::from_str(&serialized).unwrap();

    assert_eq!(dep, deserialized);
}

#[test]
fn test_tool_dependency_complex_version_requirements() {
    let deps = vec![
        ToolDependency::required("node", "Node.js").with_version(">=16.0.0"),
        ToolDependency::required("python", "Python").with_version(">=3.8.0"),
        ToolDependency::optional("go", "Go").with_version("^1.19"),
        ToolDependency::optional("rust", "Rust").with_version("~1.70.0"),
    ];

    assert_eq!(deps.len(), 4);
    assert!(deps[0].required);
    assert!(deps[1].required);
    assert!(!deps[2].required);
    assert!(!deps[3].required);

    for dep in &deps {
        assert!(dep.version_requirement.is_some());
    }
}

#[test]
fn test_tool_dependency_edge_cases() {
    // Empty strings
    let dep1 = ToolDependency::required("", "");
    assert_eq!(dep1.tool_name, "");
    assert_eq!(dep1.description, "");

    // Unicode strings
    let dep2 = ToolDependency::required("工具", "这是一个工具");
    assert_eq!(dep2.tool_name, "工具");
    assert_eq!(dep2.description, "这是一个工具");

    // Long strings
    let long_name = "a".repeat(1000);
    let long_desc = "b".repeat(1000);
    let dep3 = ToolDependency::required(&long_name, &long_desc);
    assert_eq!(dep3.tool_name.len(), 1000);
    assert_eq!(dep3.description.len(), 1000);
}

#[test]
fn test_tool_dependency_json_format() {
    let dep = ToolDependency::required("node", "Node.js runtime").with_version(">=16.0.0");

    let json = serde_json::to_string_pretty(&dep).unwrap();
    
    // Check that JSON contains expected fields
    assert!(json.contains("\"tool_name\""));
    assert!(json.contains("\"description\""));
    assert!(json.contains("\"required\""));
    assert!(json.contains("\"version_requirement\""));
    assert!(json.contains("\"node\""));
    assert!(json.contains("\"Node.js runtime\""));
    assert!(json.contains("true"));
    assert!(json.contains("\">=16.0.0\""));
}

#[test]
fn test_tool_dependency_collection() {
    let mut deps = Vec::new();
    
    deps.push(ToolDependency::required("node", "Node.js"));
    deps.push(ToolDependency::optional("yarn", "Yarn package manager"));
    deps.push(ToolDependency::required("npm", "NPM package manager"));

    // Test filtering
    let required_deps: Vec<_> = deps.iter().filter(|d| d.required).collect();
    let optional_deps: Vec<_> = deps.iter().filter(|d| !d.required).collect();

    assert_eq!(required_deps.len(), 2);
    assert_eq!(optional_deps.len(), 1);

    // Test finding by name
    let node_dep = deps.iter().find(|d| d.tool_name == "node");
    assert!(node_dep.is_some());
    assert!(node_dep.unwrap().required);

    let nonexistent_dep = deps.iter().find(|d| d.tool_name == "nonexistent");
    assert!(nonexistent_dep.is_none());
}
