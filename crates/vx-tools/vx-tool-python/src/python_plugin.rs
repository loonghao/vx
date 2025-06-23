//! Python plugin implementation

use crate::python_tool::{PipTool, PipxTool, PythonTool};
use std::collections::HashMap;
use vx_plugin::VxPlugin;

/// Python plugin that provides Python, pip, and pipx tools
pub struct PythonPlugin;

impl VxPlugin for PythonPlugin {
    fn name(&self) -> &str {
        "python"
    }

    fn description(&self) -> &str {
        "Python programming language tools using Python Build Standalone"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn tools(&self) -> Vec<Box<dyn vx_plugin::VxTool>> {
        vec![
            Box::new(PythonTool::new()),
            Box::new(PipTool::new()),
            Box::new(PipxTool::new()),
        ]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://www.python.org/".to_string(),
        );
        meta.insert(
            "repository".to_string(),
            "https://github.com/astral-sh/python-build-standalone".to_string(),
        );
        meta.insert(
            "license".to_string(),
            "Python Software Foundation License".to_string(),
        );
        meta.insert("ecosystem".to_string(), "python".to_string());
        meta.insert("category".to_string(), "programming-language".to_string());
        meta.insert(
            "tags".to_string(),
            "python,programming,interpreter,pip,pipx".to_string(),
        );
        meta
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_plugin_creation() {
        let plugin = PythonPlugin;
        assert_eq!(plugin.name(), "python");
        assert_eq!(
            plugin.description(),
            "Python programming language tools using Python Build Standalone"
        );
        assert!(!plugin.version().is_empty());
    }

    #[test]
    fn test_python_plugin_tools() {
        let plugin = PythonPlugin;
        let tools = plugin.tools();

        assert_eq!(tools.len(), 3);
        // Check that we have the expected number of tools
        // Individual tool names are checked by their respective implementations
    }

    #[test]
    fn test_python_plugin_metadata() {
        let plugin = PythonPlugin;
        let metadata = plugin.metadata();

        assert_eq!(
            metadata.get("homepage"),
            Some(&"https://www.python.org/".to_string())
        );
        assert_eq!(metadata.get("ecosystem"), Some(&"python".to_string()));
        assert_eq!(
            metadata.get("category"),
            Some(&"programming-language".to_string())
        );
    }
}
