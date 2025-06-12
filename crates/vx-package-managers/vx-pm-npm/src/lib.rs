//! Simple example showing how easy it is to implement a package manager
//!
//! This demonstrates the minimal code needed to add package manager support.

use std::path::Path;
use vx_core::{Ecosystem, PackageSpec, Result, VxPackageManager, VxPlugin, VxTool};

/// Example: Simple NPM package manager implementation
///
/// This shows how little code is needed to add package manager support to vx.
#[derive(Default)]
pub struct SimpleNpmPackageManager;

#[async_trait::async_trait]
impl VxPackageManager for SimpleNpmPackageManager {
    fn name(&self) -> &str {
        "npm"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::JavaScript
    }

    fn description(&self) -> &str {
        "Node Package Manager"
    }

    /// Detect if this is an npm project by looking for package.json
    fn is_preferred_for_project(&self, project_path: &Path) -> bool {
        project_path.join("package.json").exists()
    }

    fn get_config_files(&self) -> Vec<&str> {
        vec!["package.json", "package-lock.json"]
    }

    /// Main method to implement - install packages
    async fn install_packages(&self, packages: &[PackageSpec], project_path: &Path) -> Result<()> {
        if packages.is_empty() {
            // Just run `npm install` to install from package.json
            self.run_command(&["install"], &[], project_path).await
        } else {
            // Install specific packages
            let package_names: Vec<String> = packages
                .iter()
                .map(|pkg| {
                    if let Some(version) = &pkg.version {
                        format!("{}@{}", pkg.name, version)
                    } else {
                        pkg.name.clone()
                    }
                })
                .collect();

            let command = if packages.iter().any(|pkg| pkg.dev_dependency) {
                vec!["install", "--save-dev"]
            } else {
                vec!["install", "--save"]
            };

            self.run_command(&command, &package_names, project_path)
                .await
        }
    }

    /// Override the default remove command
    async fn remove_packages(&self, packages: &[String], project_path: &Path) -> Result<()> {
        self.run_command(&["uninstall"], packages, project_path)
            .await
    }

    /// Override the default update command
    async fn update_packages(&self, packages: &[String], project_path: &Path) -> Result<()> {
        if packages.is_empty() {
            self.run_command(&["update"], &[], project_path).await
        } else {
            self.run_command(&["update"], packages, project_path).await
        }
    }
}

/// Example: Simple plugin that provides NPM package manager
#[derive(Default)]
pub struct SimpleNpmPlugin;

#[async_trait::async_trait]
impl VxPlugin for SimpleNpmPlugin {
    fn name(&self) -> &str {
        "simple-npm"
    }

    fn description(&self) -> &str {
        "Simple NPM package manager support for vx"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn tools(&self) -> Vec<Box<dyn VxTool>> {
        vec![]
    }

    fn package_managers(&self) -> Vec<Box<dyn VxPackageManager>> {
        vec![Box::new(SimpleNpmPackageManager)]
    }

    fn supports_tool(&self, _tool_name: &str) -> bool {
        false
    }
}

/// Factory function to create the plugin
pub fn create_simple_npm_plugin() -> Box<dyn VxPlugin> {
    Box::new(SimpleNpmPlugin)
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_simple_npm_package_manager() {
        let pm = SimpleNpmPackageManager;

        assert_eq!(pm.name(), "npm");
        assert_eq!(pm.ecosystem(), Ecosystem::JavaScript);
        assert_eq!(pm.description(), "Node Package Manager");

        // Test project detection
        let temp_dir = std::env::temp_dir().join("test-npm-project");
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Should not be preferred without package.json
        assert!(!pm.is_preferred_for_project(&temp_dir));

        // Should be preferred with package.json
        std::fs::write(temp_dir.join("package.json"), "{}").unwrap();
        assert!(pm.is_preferred_for_project(&temp_dir));

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_simple_npm_plugin() {
        let plugin = SimpleNpmPlugin;

        assert_eq!(plugin.name(), "simple-npm");
        assert_eq!(plugin.version(), "1.0.0");
        assert!(plugin.supports_package_manager("npm"));
        assert!(!plugin.supports_package_manager("yarn"));

        let package_managers = plugin.package_managers();
        assert_eq!(package_managers.len(), 1);
        assert_eq!(package_managers[0].name(), "npm");
    }

    #[tokio::test]
    async fn test_package_spec_formatting() {
        let pm = SimpleNpmPackageManager;

        let packages = vec![
            PackageSpec::new("express".to_string()),
            PackageSpec::new("lodash".to_string()).with_version("4.17.21".to_string()),
            PackageSpec::new("jest".to_string()).as_dev_dependency(),
        ];

        // This would test the package formatting logic
        // In a real test, we'd mock the command execution
        assert_eq!(packages.len(), 3);
        assert_eq!(packages[0].name, "express");
        assert_eq!(packages[1].version, Some("4.17.21".to_string()));
        assert!(packages[2].dev_dependency);
    }
}

/*
USAGE EXAMPLE:

To use this package manager plugin in your vx installation:

```rust
use vx_core::PluginRegistry;
use vx_pm_npm::create_simple_npm_plugin;

let mut registry = PluginRegistry::new();
registry.register(create_simple_npm_plugin())?;

// Now you can use npm through vx:
// vx npm install express
// vx npm install --save-dev jest
// vx npm update
// vx npm uninstall lodash
```

That's it! With just ~100 lines of code, you have full NPM support in vx.

The framework handles:
- ✅ Project detection (package.json)
- ✅ Command execution
- ✅ Error handling
- ✅ Plugin registration
- ✅ Ecosystem classification

You only need to implement:
- ✅ Package installation logic
- ✅ Project detection rules
- ✅ Package manager metadata

For even simpler cases, you can rely entirely on the default implementations
and just specify the package manager name and config files!
*/
