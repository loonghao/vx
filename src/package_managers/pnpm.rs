// PNPM package manager implementation
// Provides fast, disk space efficient package management

use crate::package_managers::{PackageManager, PackageInfo, common};
use crate::tool::{Tool, ToolInfo};
use anyhow::Result;
use std::path::PathBuf;

/// PNPM package manager implementation
pub struct PnpmTool {
    name: String,
    version: Option<String>,
}

impl PnpmTool {
    pub fn new() -> Self {
        Self {
            name: "pnpm".to_string(),
            version: None,
        }
    }
}

impl Tool for PnpmTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_installed(&self) -> Result<bool> {
        // Check if pnpm is available in PATH
        match std::process::Command::new("pnpm").arg("--version").output() {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    fn get_version(&self) -> Result<Option<String>> {
        if let Ok(output) = std::process::Command::new("pnpm").arg("--version").output() {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return Ok(Some(version));
            }
        }
        Ok(None)
    }

    fn execute(&self, args: &[String]) -> Result<i32> {
        let mut cmd = std::process::Command::new("pnpm");
        cmd.args(args);
        
        let status = cmd.status()?;
        Ok(status.code().unwrap_or(1))
    }

    fn supports_auto_install(&self) -> bool {
        true
    }

    fn get_info(&self) -> ToolInfo {
        ToolInfo {
            name: self.name.clone(),
            version: self.version.clone().unwrap_or_else(|| "unknown".to_string()),
            description: "Fast, disk space efficient package manager".to_string(),
            homepage: Some("https://pnpm.io".to_string()),
            repository: Some("https://github.com/pnpm/pnpm".to_string()),
            license: Some("MIT".to_string()),
            keywords: vec![
                "package-manager".to_string(),
                "npm".to_string(),
                "node".to_string(),
                "javascript".to_string(),
            ],
            categories: vec!["package-manager".to_string()],
            supported_platforms: vec![
                "windows".to_string(),
                "macos".to_string(),
                "linux".to_string(),
            ],
            dependencies: vec!["node".to_string()],
        }
    }
}

impl PackageManager for PnpmTool {
    fn install(&self, packages: &[String]) -> Result<()> {
        let mut args = vec!["install".to_string()];
        args.extend_from_slice(packages);
        
        common::execute_pm_command("pnpm", &args, None)?;
        Ok(())
    }

    fn add(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Err(anyhow::anyhow!("No packages specified to add"));
        }
        
        let mut args = vec!["add".to_string()];
        args.extend_from_slice(packages);
        
        common::execute_pm_command("pnpm", &args, None)?;
        Ok(())
    }

    fn add_dev(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Err(anyhow::anyhow!("No packages specified to add"));
        }
        
        let mut args = vec!["add".to_string(), "--save-dev".to_string()];
        args.extend_from_slice(packages);
        
        common::execute_pm_command("pnpm", &args, None)?;
        Ok(())
    }

    fn remove(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Err(anyhow::anyhow!("No packages specified to remove"));
        }
        
        let mut args = vec!["remove".to_string()];
        args.extend_from_slice(packages);
        
        common::execute_pm_command("pnpm", &args, None)?;
        Ok(())
    }

    fn update(&self, packages: &[String]) -> Result<()> {
        let mut args = vec!["update".to_string()];
        args.extend_from_slice(packages);
        
        common::execute_pm_command("pnpm", &args, None)?;
        Ok(())
    }

    fn run_script(&self, script: &str, args: &[String]) -> Result<()> {
        let mut cmd_args = vec!["run".to_string(), script.to_string()];
        if !args.is_empty() {
            cmd_args.push("--".to_string());
            cmd_args.extend_from_slice(args);
        }
        
        common::execute_pm_command("pnpm", &cmd_args, None)?;
        Ok(())
    }

    fn list_packages(&self) -> Result<Vec<PackageInfo>> {
        let output = std::process::Command::new("pnpm")
            .args(&["list", "--json", "--depth=0"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to list packages"));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let parsed: serde_json::Value = serde_json::from_str(&json_str)?;
        
        let mut packages = Vec::new();
        
        if let Some(dependencies) = parsed.get("dependencies").and_then(|d| d.as_object()) {
            for (name, info) in dependencies {
                if let Some(version) = info.get("version").and_then(|v| v.as_str()) {
                    packages.push(PackageInfo {
                        name: name.clone(),
                        version: version.to_string(),
                        description: None,
                        is_dev_dependency: false,
                        is_peer_dependency: false,
                    });
                }
            }
        }
        
        if let Some(dev_dependencies) = parsed.get("devDependencies").and_then(|d| d.as_object()) {
            for (name, info) in dev_dependencies {
                if let Some(version) = info.get("version").and_then(|v| v.as_str()) {
                    packages.push(PackageInfo {
                        name: name.clone(),
                        version: version.to_string(),
                        description: None,
                        is_dev_dependency: true,
                        is_peer_dependency: false,
                    });
                }
            }
        }

        Ok(packages)
    }

    fn lock_file_name(&self) -> &'static str {
        "pnpm-lock.yaml"
    }
}

impl Default for PnpmTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pnpm_tool_creation() {
        let pnpm = PnpmTool::new();
        assert_eq!(pnpm.name(), "pnpm");
        assert_eq!(pnpm.lock_file_name(), "pnpm-lock.yaml");
    }

    #[test]
    fn test_pnpm_tool_info() {
        let pnpm = PnpmTool::new();
        let info = pnpm.get_info();
        
        assert_eq!(info.name, "pnpm");
        assert!(info.description.contains("package manager"));
        assert!(info.keywords.contains(&"package-manager".to_string()));
    }

    #[test]
    fn test_pnpm_supports_auto_install() {
        let pnpm = PnpmTool::new();
        assert!(pnpm.supports_auto_install());
    }
}
