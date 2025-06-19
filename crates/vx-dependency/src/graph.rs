//! Dependency graph representation and algorithms

use crate::{types::*, Result};
use std::collections::{HashMap, HashSet};

/// Dependency graph for tools
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Nodes in the graph (tools)
    nodes: HashMap<String, DependencyNode>,
    /// Adjacency list (tool -> dependencies)
    edges: HashMap<String, Vec<String>>,
    /// Reverse adjacency list (tool -> dependents)
    reverse_edges: HashMap<String, Vec<String>>,
}

/// Node in the dependency graph
#[derive(Debug, Clone)]
pub struct DependencyNode {
    /// Tool specification
    pub tool_spec: ToolSpec,
    /// Whether this tool is currently available
    pub available: bool,
    /// Installed version (if available)
    pub installed_version: Option<String>,
    /// Installation state
    pub state: NodeState,
}

/// State of a dependency node
#[derive(Debug, Clone, PartialEq)]
pub enum NodeState {
    /// Not processed yet
    Unvisited,
    /// Currently being processed (for cycle detection)
    Visiting,
    /// Processing completed
    Visited,
    /// Installation pending
    Pending,
    /// Currently installing
    Installing,
    /// Installation completed
    Installed,
    /// Installation failed
    Failed(String),
}

/// Result of dependency resolution
#[derive(Debug, Clone)]
pub struct ResolutionResult {
    /// Installation order (dependencies first)
    pub install_order: Vec<String>,
    /// Tools that need to be installed
    pub missing_tools: Vec<String>,
    /// Tools that are already available
    pub available_tools: Vec<String>,
    /// Circular dependencies detected
    pub circular_dependencies: Vec<Vec<String>>,
    /// Version conflicts
    pub version_conflicts: Vec<VersionConflict>,
}

/// Version conflict information
#[derive(Debug, Clone)]
pub struct VersionConflict {
    /// Tool with version conflict
    pub tool: String,
    /// Required version constraint
    pub required: String,
    /// Currently installed version
    pub installed: String,
    /// Tools that require this version
    pub required_by: Vec<String>,
}

impl DependencyGraph {
    /// Create a new empty dependency graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            reverse_edges: HashMap::new(),
        }
    }

    /// Add a tool to the graph
    pub fn add_tool(&mut self, tool_spec: ToolSpec) -> Result<()> {
        let tool_name = tool_spec.name.clone();

        // Add node
        self.nodes.insert(
            tool_name.clone(),
            DependencyNode {
                tool_spec: tool_spec.clone(),
                available: false,
                installed_version: None,
                state: NodeState::Unvisited,
            },
        );

        // Add edges for dependencies
        let mut dependencies = Vec::new();
        for dep in &tool_spec.dependencies {
            if !dep.optional || dep.dependency_type == DependencyType::Runtime {
                dependencies.push(dep.tool_name.clone());
            }
        }

        self.edges.insert(tool_name.clone(), dependencies.clone());

        // Update reverse edges
        for dep in dependencies {
            self.reverse_edges
                .entry(dep)
                .or_default()
                .push(tool_name.clone());
        }

        Ok(())
    }

    /// Get a tool node
    pub fn get_tool(&self, tool_name: &str) -> Option<&DependencyNode> {
        self.nodes.get(tool_name)
    }

    /// Get a mutable tool node
    pub fn get_tool_mut(&mut self, tool_name: &str) -> Option<&mut DependencyNode> {
        self.nodes.get_mut(tool_name)
    }

    /// Update tool availability
    pub fn set_tool_available(
        &mut self,
        tool_name: &str,
        available: bool,
        version: Option<String>,
    ) {
        if let Some(node) = self.nodes.get_mut(tool_name) {
            node.available = available;
            node.installed_version = version;
            if available {
                node.state = NodeState::Installed;
            }
        }
    }

    /// Resolve dependencies for a tool
    pub fn resolve_dependencies(&mut self, tool_name: &str) -> Result<ResolutionResult> {
        // Reset all node states
        for node in self.nodes.values_mut() {
            node.state = NodeState::Unvisited;
        }

        let mut install_order = Vec::new();
        let mut circular_dependencies = Vec::new();
        let mut version_conflicts = Vec::new();

        // Perform topological sort with cycle detection
        self.topological_sort_dfs(tool_name, &mut install_order, &mut circular_dependencies)?;

        // Check version conflicts
        self.check_version_conflicts(&mut version_conflicts)?;

        // Categorize tools
        let mut missing_tools = Vec::new();
        let mut available_tools = Vec::new();

        for tool in &install_order {
            if let Some(node) = self.nodes.get(tool) {
                if node.available {
                    available_tools.push(tool.clone());
                } else {
                    missing_tools.push(tool.clone());
                }
            }
        }

        Ok(ResolutionResult {
            install_order,
            missing_tools,
            available_tools,
            circular_dependencies,
            version_conflicts,
        })
    }

    /// Perform topological sort using DFS with cycle detection
    fn topological_sort_dfs(
        &mut self,
        tool_name: &str,
        install_order: &mut Vec<String>,
        circular_dependencies: &mut Vec<Vec<String>>,
    ) -> Result<()> {
        let node_state = self
            .nodes
            .get(tool_name)
            .map(|n| n.state.clone())
            .unwrap_or(NodeState::Unvisited);

        match node_state {
            NodeState::Visiting => {
                // Cycle detected
                circular_dependencies.push(vec![tool_name.to_string()]);
                return Ok(());
            }
            NodeState::Visited => {
                // Already processed
                return Ok(());
            }
            _ => {}
        }

        // Mark as visiting
        if let Some(node) = self.nodes.get_mut(tool_name) {
            node.state = NodeState::Visiting;
        }

        // Visit dependencies first
        if let Some(dependencies) = self.edges.get(tool_name).cloned() {
            for dep in dependencies {
                self.topological_sort_dfs(&dep, install_order, circular_dependencies)?;
            }
        }

        // Mark as visited and add to install order
        if let Some(node) = self.nodes.get_mut(tool_name) {
            node.state = NodeState::Visited;
        }

        if !install_order.contains(&tool_name.to_string()) {
            install_order.push(tool_name.to_string());
        }

        Ok(())
    }

    /// Check for version conflicts
    fn check_version_conflicts(&self, _conflicts: &mut Vec<VersionConflict>) -> Result<()> {
        // TODO: Implement version conflict detection
        // This would check if multiple tools require different versions of the same dependency
        Ok(())
    }

    /// Get all tools that depend on a given tool
    pub fn get_dependents(&self, tool_name: &str) -> Vec<String> {
        self.reverse_edges
            .get(tool_name)
            .cloned()
            .unwrap_or_default()
    }

    /// Get direct dependencies of a tool
    pub fn get_dependencies(&self, tool_name: &str) -> Vec<String> {
        self.edges.get(tool_name).cloned().unwrap_or_default()
    }

    /// Check if the graph has cycles
    pub fn has_cycles(&mut self) -> bool {
        // Reset states
        for node in self.nodes.values_mut() {
            node.state = NodeState::Unvisited;
        }

        for tool_name in self.nodes.keys().cloned().collect::<Vec<_>>() {
            if self.nodes.get(&tool_name).unwrap().state == NodeState::Unvisited
                && self.has_cycle_dfs(&tool_name)
            {
                return true;
            }
        }

        false
    }

    /// DFS cycle detection helper
    fn has_cycle_dfs(&mut self, tool_name: &str) -> bool {
        if let Some(node) = self.nodes.get_mut(tool_name) {
            if node.state == NodeState::Visiting {
                return true; // Back edge found - cycle detected
            }
            if node.state == NodeState::Visited {
                return false; // Already processed
            }

            node.state = NodeState::Visiting;
        }

        // Check dependencies
        if let Some(dependencies) = self.edges.get(tool_name).cloned() {
            for dep in dependencies {
                if self.has_cycle_dfs(&dep) {
                    return true;
                }
            }
        }

        // Mark as visited
        if let Some(node) = self.nodes.get_mut(tool_name) {
            node.state = NodeState::Visited;
        }

        false
    }

    /// Get tools in installation order (dependencies first)
    pub fn get_install_order(&mut self, tools: &[String]) -> Result<Vec<String>> {
        let mut all_tools = HashSet::new();

        // Collect all tools and their dependencies
        for tool in tools {
            let resolution = self.resolve_dependencies(tool)?;
            all_tools.extend(resolution.install_order);
        }

        // Convert to sorted vector
        let ordered_tools: Vec<String> = all_tools.into_iter().collect();

        // Sort by dependency order using topological sort
        let mut final_order = Vec::new();
        for tool in &ordered_tools {
            let resolution = self.resolve_dependencies(tool)?;
            for dep_tool in resolution.install_order {
                if !final_order.contains(&dep_tool) {
                    final_order.push(dep_tool);
                }
            }
        }

        Ok(final_order)
    }

    /// Get statistics about the dependency graph
    pub fn get_stats(&self) -> GraphStats {
        let total_tools = self.nodes.len();
        let total_dependencies = self.edges.values().map(|deps| deps.len()).sum();
        let available_tools = self.nodes.values().filter(|n| n.available).count();
        let missing_tools = total_tools - available_tools;

        GraphStats {
            total_tools,
            total_dependencies,
            available_tools,
            missing_tools,
        }
    }
}

/// Statistics about the dependency graph
#[derive(Debug, Clone)]
pub struct GraphStats {
    /// Total number of tools in the graph
    pub total_tools: usize,
    /// Total number of dependency relationships
    pub total_dependencies: usize,
    /// Number of available tools
    pub available_tools: usize,
    /// Number of missing tools
    pub missing_tools: usize,
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tool(name: &str, deps: Vec<&str>) -> ToolSpec {
        ToolSpec {
            name: name.to_string(),
            dependencies: deps
                .into_iter()
                .map(|dep| DependencySpec::required(dep, format!("{} requires {}", name, dep)))
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn test_simple_dependency_resolution() {
        let mut graph = DependencyGraph::new();

        // node (no dependencies)
        graph.add_tool(create_test_tool("node", vec![])).unwrap();

        // yarn depends on node
        graph
            .add_tool(create_test_tool("yarn", vec!["node"]))
            .unwrap();

        let resolution = graph.resolve_dependencies("yarn").unwrap();
        assert_eq!(resolution.install_order, vec!["node", "yarn"]);
    }

    #[test]
    fn test_multi_layer_dependencies() {
        let mut graph = DependencyGraph::new();

        // Layer 1: base tools
        graph.add_tool(create_test_tool("node", vec![])).unwrap();
        graph.add_tool(create_test_tool("python", vec![])).unwrap();

        // Layer 2: tools that depend on base tools
        graph
            .add_tool(create_test_tool("npm", vec!["node"]))
            .unwrap();
        graph
            .add_tool(create_test_tool("pip", vec!["python"]))
            .unwrap();

        // Layer 3: tools that depend on layer 2
        graph
            .add_tool(create_test_tool("yarn", vec!["node", "npm"]))
            .unwrap();

        let resolution = graph.resolve_dependencies("yarn").unwrap();

        // Should install in dependency order
        let order = resolution.install_order;
        let node_pos = order.iter().position(|x| x == "node").unwrap();
        let npm_pos = order.iter().position(|x| x == "npm").unwrap();
        let yarn_pos = order.iter().position(|x| x == "yarn").unwrap();

        assert!(node_pos < npm_pos); // node before npm
        assert!(npm_pos < yarn_pos); // npm before yarn
        assert!(node_pos < yarn_pos); // node before yarn
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut graph = DependencyGraph::new();

        // Create circular dependency: a -> b -> c -> a
        graph.add_tool(create_test_tool("a", vec!["b"])).unwrap();
        graph.add_tool(create_test_tool("b", vec!["c"])).unwrap();
        graph.add_tool(create_test_tool("c", vec!["a"])).unwrap();

        assert!(graph.has_cycles());

        let resolution = graph.resolve_dependencies("a").unwrap();
        assert!(!resolution.circular_dependencies.is_empty());
    }

    #[test]
    fn test_diamond_dependency() {
        let mut graph = DependencyGraph::new();

        // Diamond dependency: d -> b,c -> a
        graph.add_tool(create_test_tool("a", vec![])).unwrap();
        graph.add_tool(create_test_tool("b", vec!["a"])).unwrap();
        graph.add_tool(create_test_tool("c", vec!["a"])).unwrap();
        graph
            .add_tool(create_test_tool("d", vec!["b", "c"]))
            .unwrap();

        let resolution = graph.resolve_dependencies("d").unwrap();

        // 'a' should appear only once and before 'b' and 'c'
        let order = resolution.install_order;
        let a_count = order.iter().filter(|&x| x == "a").count();
        assert_eq!(a_count, 1);

        let a_pos = order.iter().position(|x| x == "a").unwrap();
        let b_pos = order.iter().position(|x| x == "b").unwrap();
        let c_pos = order.iter().position(|x| x == "c").unwrap();
        let d_pos = order.iter().position(|x| x == "d").unwrap();

        assert!(a_pos < b_pos);
        assert!(a_pos < c_pos);
        assert!(b_pos < d_pos);
        assert!(c_pos < d_pos);
    }

    #[test]
    fn test_tool_availability() {
        let mut graph = DependencyGraph::new();

        graph.add_tool(create_test_tool("node", vec![])).unwrap();
        graph
            .add_tool(create_test_tool("yarn", vec!["node"]))
            .unwrap();

        // Mark node as available
        graph.set_tool_available("node", true, Some("18.0.0".to_string()));

        let resolution = graph.resolve_dependencies("yarn").unwrap();
        assert_eq!(resolution.available_tools, vec!["node"]);
        assert_eq!(resolution.missing_tools, vec!["yarn"]);
    }

    #[test]
    fn test_graph_stats() {
        let mut graph = DependencyGraph::new();

        graph.add_tool(create_test_tool("node", vec![])).unwrap();
        graph
            .add_tool(create_test_tool("yarn", vec!["node"]))
            .unwrap();
        graph.set_tool_available("node", true, Some("18.0.0".to_string()));

        let stats = graph.get_stats();
        assert_eq!(stats.total_tools, 2);
        assert_eq!(stats.available_tools, 1);
        assert_eq!(stats.missing_tools, 1);
        assert_eq!(stats.total_dependencies, 1); // yarn -> node
    }
}
