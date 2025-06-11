// NPM package manager implementation
// The default Node.js package manager

use crate::package_managers::{PackageManager, PackageInfo, common};
use crate::tool::{Tool, ToolInfo};
use anyhow::Result;

/// NPM package manager implementation
pub struct NpmTool {
    name: String,
    version: Option<String>,
}

impl NpmTool {
    pub fn new() -> Self {
        Self {
            name: "npm".to_string(),
            version: None,
        }
    }
}

impl Tool for NpmTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_installed(&self) -> Result<bool> {
        match std::process::Command::new("npm").arg("--version").output() {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    fn get_version(&self) -> Result<Option<String>> {
        if let Ok(output) = std::process::Command::new("npm").arg("--version").output() {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return Ok(Some(version));
            }
        }
        Ok(None)
    }

    fn execute(&self, args: &[String]) -> Result<i32> {
        let mut cmd = std::process::Command::new("npm");
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
            description: "Node.js package manager".to_string(),
            homepage: Some("https://www.npmjs.com".to_string()),
            repository: Some("https://github.com/npm/cli".to_string()),
            license: Some("Artistic-2.0".to_string()),
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

impl PackageManager for NpmTool {
    fn install(&self, packages: &[String]) -> Result<()> {
        let mut args = vec!["install".to_string()];
        args.extend_from_slice(packages);
        
        common::execute_pm_command("npm", &args, None)?;
        Ok(())
    }

    fn add(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Err(anyhow::anyhow!("No packages specified to add"));
        }
        
        let mut args = vec!["install".to_string()];
        args.extend_from_slice(packages);
        
        common::execute_pm_command("npm", &args, None)?;
        Ok(())
    }

    fn add_dev(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Err(anyhow::anyhow!("No packages specified to add"));
        }
        
        let mut args = vec!["install".to_string(), "--save-dev".to_string()];
        args.extend_from_slice(packages);
        
        common::execute_pm_command("npm", &args, None)?;
        Ok(())
    }

    fn remove(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Err(anyhow::anyhow!("No packages specified to remove"));
        }
        
        let mut args = vec!["uninstall".to_string()];
        args.extend_from_slice(packages);
        
        common::execute_pm_command("npm", &args, None)?;
        Ok(())
    }

    fn update(&self, packages: &[String]) -> Result<()> {
        let mut args = vec!["update".to_string()];
        args.extend_from_slice(packages);
        
        common::execute_pm_command("npm", &args, None)?;
        Ok(())
    }

    fn run_script(&self, script: &str, args: &[String]) -> Result<()> {
        let mut cmd_args = vec!["run".to_string(), script.to_string()];
        if !args.is_empty() {
            cmd_args.push("--".to_string());
            cmd_args.extend_from_slice(args);
        }
        
        common::execute_pm_command("npm", &cmd_args, None)?;
        Ok(())
    }

    fn list_packages(&self) -> Result<Vec<PackageInfo>> {
        let output = std::process::Command::new("npm")
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

        Ok(packages)
    }

    fn lock_file_name(&self) -> &'static str {
        "package-lock.json"
    }
}

impl Default for NpmTool {
    fn default() -> Self {
        Self::new()
    }
}
