use std::collections::HashMap;
use vx_plugin::{ToolDependency, VxTool};

// Mock tool for testing dependency display
#[derive(Debug, Clone)]
struct MockTool {
    name: String,
    description: String,
    dependencies: Vec<ToolDependency>,
}

impl MockTool {
    fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            dependencies: Vec::new(),
        }
    }

    fn with_dependency(mut self, dep: ToolDependency) -> Self {
        self.dependencies.push(dep);
        self
    }
}

#[async_trait::async_trait]
impl VxTool for MockTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn aliases(&self) -> Vec<&str> {
        vec![]
    }

    async fn fetch_versions(
        &self,
        _include_prerelease: bool,
    ) -> Result<Vec<vx_plugin::VersionInfo>, anyhow::Error> {
        Ok(vec![])
    }

    async fn install_version(&self, _version: &str, _force: bool) -> Result<(), anyhow::Error> {
        Ok(())
    }

    async fn is_version_installed(&self, _version: &str) -> Result<bool, anyhow::Error> {
        Ok(false)
    }

    async fn get_active_version(&self) -> Result<String, anyhow::Error> {
        Ok("latest".to_string())
    }

    async fn get_installed_versions(&self) -> Result<Vec<String>, anyhow::Error> {
        Ok(vec![])
    }

    async fn execute(
        &self,
        _args: &[String],
        _context: &vx_plugin::ToolContext,
    ) -> Result<vx_plugin::ToolExecutionResult, anyhow::Error> {
        Ok(vx_plugin::ToolExecutionResult {
            exit_code: 0,
            stdout: None,
            stderr: None,
        })
    }

    async fn get_download_url(&self, _version: &str) -> Result<Option<String>, anyhow::Error> {
        Ok(None)
    }

    fn metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    fn get_dependencies(&self) -> Vec<ToolDependency> {
        self.dependencies.clone()
    }
}

#[test]
fn test_tool_dependency_creation() {
    let dep = ToolDependency::required("node", "Node.js runtime");
    assert_eq!(dep.tool_name, "node");
    assert_eq!(dep.description, "Node.js runtime");
    assert!(dep.required);
}

#[test]
fn test_mock_tool_with_dependencies() {
    let yarn_tool = MockTool::new("yarn", "Yarn package manager")
        .with_dependency(ToolDependency::required("node", "Node.js runtime"));

    let deps = yarn_tool.get_dependencies();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].tool_name, "node");
    assert_eq!(deps[0].description, "Node.js runtime");
    assert!(deps[0].required);
}

#[test]
fn test_multiple_dependencies() {
    let complex_tool = MockTool::new("complex-tool", "A tool with multiple dependencies")
        .with_dependency(ToolDependency::required("node", "Node.js runtime"))
        .with_dependency(ToolDependency::optional("python", "Python interpreter"))
        .with_dependency(ToolDependency::required("go", "Go programming language"));

    let deps = complex_tool.get_dependencies();
    assert_eq!(deps.len(), 3);

    let required_deps: Vec<_> = deps.iter().filter(|d| d.required).collect();
    let optional_deps: Vec<_> = deps.iter().filter(|d| !d.required).collect();

    assert_eq!(required_deps.len(), 2);
    assert_eq!(optional_deps.len(), 1);
}

#[test]
fn test_dependency_with_version_requirements() {
    let tool = MockTool::new("modern-tool", "A tool with version requirements")
        .with_dependency(
            ToolDependency::required("node", "Node.js runtime").with_version(">=16.0.0"),
        )
        .with_dependency(
            ToolDependency::optional("python", "Python interpreter").with_version(">=3.8.0"),
        );

    let deps = tool.get_dependencies();
    assert_eq!(deps.len(), 2);

    let node_dep = deps.iter().find(|d| d.tool_name == "node").unwrap();
    assert_eq!(node_dep.version_requirement, Some(">=16.0.0".to_string()));

    let python_dep = deps.iter().find(|d| d.tool_name == "python").unwrap();
    assert_eq!(python_dep.version_requirement, Some(">=3.8.0".to_string()));
}

#[test]
fn test_dependency_serialization() {
    let dep = ToolDependency::required("test-tool", "A test tool").with_version("^1.0.0");

    // Test JSON serialization
    let json = serde_json::to_string(&dep).unwrap();
    let deserialized: ToolDependency = serde_json::from_str(&json).unwrap();

    assert_eq!(dep.tool_name, deserialized.tool_name);
    assert_eq!(dep.description, deserialized.description);
    assert_eq!(dep.required, deserialized.required);
    assert_eq!(dep.version_requirement, deserialized.version_requirement);
}

#[test]
fn test_dependency_edge_cases() {
    // Test with empty strings
    let empty_dep = ToolDependency::required("", "");
    assert_eq!(empty_dep.tool_name, "");
    assert_eq!(empty_dep.description, "");

    // Test with special characters
    let special_dep = ToolDependency::required("tool-with-dashes", "Tool with special chars: @#$%");
    assert_eq!(special_dep.tool_name, "tool-with-dashes");
    assert!(special_dep.description.contains("@#$%"));

    // Test with Unicode
    let unicode_dep = ToolDependency::required("工具", "这是一个工具");
    assert_eq!(unicode_dep.tool_name, "工具");
    assert_eq!(unicode_dep.description, "这是一个工具");
}

#[test]
fn test_dependency_collection_operations() {
    let deps = vec![
        ToolDependency::required("node", "Node.js"),
        ToolDependency::optional("yarn", "Yarn"),
        ToolDependency::required("npm", "NPM"),
        ToolDependency::optional("pnpm", "PNPM"),
    ];

    // Test filtering by required
    let required: Vec<_> = deps.iter().filter(|d| d.required).collect();
    assert_eq!(required.len(), 2);

    // Test filtering by optional
    let optional: Vec<_> = deps.iter().filter(|d| !d.required).collect();
    assert_eq!(optional.len(), 2);

    // Test finding by name
    let node_dep = deps.iter().find(|d| d.tool_name == "node");
    assert!(node_dep.is_some());
    assert!(node_dep.unwrap().required);

    // Test collecting tool names
    let tool_names: Vec<_> = deps.iter().map(|d| &d.tool_name).collect();
    assert!(tool_names.contains(&&"node".to_string()));
    assert!(tool_names.contains(&&"yarn".to_string()));
    assert!(tool_names.contains(&&"npm".to_string()));
    assert!(tool_names.contains(&&"pnpm".to_string()));
}

#[test]
fn test_dependency_version_parsing() {
    let version_patterns = vec![
        ">=1.0.0",
        "^2.0.0",
        "~1.5.0",
        "1.2.3",
        ">=1.0.0 <2.0.0",
        "*",
        "latest",
    ];

    for pattern in version_patterns {
        let dep = ToolDependency::required("test", "Test tool").with_version(pattern);
        assert_eq!(dep.version_requirement, Some(pattern.to_string()));
    }
}

#[test]
fn test_dependency_builder_pattern() {
    let dep = ToolDependency::required("base-tool", "Base description").with_version("1.0.0");

    assert_eq!(dep.tool_name, "base-tool");
    assert_eq!(dep.description, "Base description");
    assert!(dep.required);
    assert_eq!(dep.version_requirement, Some("1.0.0".to_string()));

    // Test optional builder
    let optional_dep =
        ToolDependency::optional("optional-tool", "Optional description").with_version("2.0.0");

    assert_eq!(optional_dep.tool_name, "optional-tool");
    assert_eq!(optional_dep.description, "Optional description");
    assert!(!optional_dep.required);
    assert_eq!(optional_dep.version_requirement, Some("2.0.0".to_string()));
}
