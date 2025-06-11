// Concrete implementations of package managers for different ecosystems
// Demonstrates how to implement the UniversalPackageManager trait

use crate::package_ecosystem::*;
use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// NPM package manager implementation
pub struct NpmPackageManager {
    config: PackageManagerConfig,
}

impl NpmPackageManager {
    pub fn new() -> Self {
        Self {
            config: PackageManagerConfig {
                name: "npm".to_string(),
                version: None,
                executable_path: None,
                config_files: vec![],
                cache_directory: None,
                supports_lockfiles: true,
                supports_workspaces: true,
                isolation_level: IsolationLevel::Project,
            },
        }
    }
}

impl UniversalPackageManager for NpmPackageManager {
    fn name(&self) -> &str {
        "npm"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::JavaScript
    }

    fn install_packages(&self, packages: &[PackageSpec]) -> Result<()> {
        let mut args = vec!["install".to_string()];

        for package in packages {
            let mut package_arg = package.name.clone();
            if let Some(version) = &package.version {
                package_arg.push('@');
                package_arg.push_str(version);
            }
            args.push(package_arg);

            if package.install_options.dev_dependency {
                args.push("--save-dev".to_string());
            }
        }

        let status = Command::new("npm").args(&args).status()?;
        if !status.success() {
            return Err(anyhow::anyhow!("npm install failed"));
        }
        Ok(())
    }

    fn remove_packages(&self, packages: &[String]) -> Result<()> {
        let mut args = vec!["uninstall".to_string()];
        args.extend_from_slice(packages);

        let status = Command::new("npm").args(&args).status()?;
        if !status.success() {
            return Err(anyhow::anyhow!("npm uninstall failed"));
        }
        Ok(())
    }

    fn list_packages(&self) -> Result<Vec<PackageInfo>> {
        let output = Command::new("npm")
            .args(["list", "--json", "--depth=0"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("npm list failed"));
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
                        description: info
                            .get("description")
                            .and_then(|d| d.as_str())
                            .map(|s| s.to_string()),
                        homepage: info
                            .get("homepage")
                            .and_then(|h| h.as_str())
                            .map(|s| s.to_string()),
                        repository: None,
                        license: None,
                        keywords: vec![],
                        dependencies: vec![],
                        is_dev_dependency: false,
                        is_optional: false,
                        install_size: None,
                    });
                }
            }
        }
        Ok(packages)
    }

    fn search_packages(&self, query: &str) -> Result<Vec<PackageInfo>> {
        let output = Command::new("npm")
            .args(["search", query, "--json"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("npm search failed"));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let parsed: serde_json::Value = serde_json::from_str(&json_str)?;

        let mut packages = Vec::new();
        if let Some(results) = parsed.as_array() {
            for result in results {
                if let (Some(name), Some(version)) = (
                    result.get("name").and_then(|n| n.as_str()),
                    result.get("version").and_then(|v| v.as_str()),
                ) {
                    packages.push(PackageInfo {
                        name: name.to_string(),
                        version: version.to_string(),
                        description: result
                            .get("description")
                            .and_then(|d| d.as_str())
                            .map(|s| s.to_string()),
                        homepage: result
                            .get("homepage")
                            .and_then(|h| h.as_str())
                            .map(|s| s.to_string()),
                        repository: None,
                        license: None,
                        keywords: result
                            .get("keywords")
                            .and_then(|k| k.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .collect()
                            })
                            .unwrap_or_default(),
                        dependencies: vec![],
                        is_dev_dependency: false,
                        is_optional: false,
                        install_size: None,
                    });
                }
            }
        }
        Ok(packages)
    }

    fn update_packages(&self, packages: &[String]) -> Result<()> {
        let mut args = vec!["update".to_string()];
        args.extend_from_slice(packages);

        let status = Command::new("npm").args(&args).status()?;
        if !status.success() {
            return Err(anyhow::anyhow!("npm update failed"));
        }
        Ok(())
    }

    fn is_available(&self) -> bool {
        Command::new("npm").arg("--version").output().is_ok()
    }

    fn get_config(&self) -> PackageManagerConfig {
        self.config.clone()
    }

    fn is_preferred_for_project(&self, project_path: &Path) -> bool {
        project_path.join("package-lock.json").exists()
    }
}

/// Homebrew package manager implementation (macOS system packages)
pub struct HomebrewPackageManager {
    config: PackageManagerConfig,
}

impl HomebrewPackageManager {
    pub fn new() -> Self {
        Self {
            config: PackageManagerConfig {
                name: "brew".to_string(),
                version: None,
                executable_path: None,
                config_files: vec![],
                cache_directory: None,
                supports_lockfiles: false,
                supports_workspaces: false,
                isolation_level: IsolationLevel::Global,
            },
        }
    }
}

impl UniversalPackageManager for HomebrewPackageManager {
    fn name(&self) -> &str {
        "brew"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System(SystemType::MacOS)
    }

    fn install_packages(&self, packages: &[PackageSpec]) -> Result<()> {
        for package in packages {
            let mut args = vec!["install".to_string(), package.name.clone()];
            args.extend_from_slice(&package.install_options.custom_flags);

            let status = Command::new("brew").args(&args).status()?;
            if !status.success() {
                return Err(anyhow::anyhow!("brew install failed for {}", package.name));
            }
        }
        Ok(())
    }

    fn remove_packages(&self, packages: &[String]) -> Result<()> {
        for package in packages {
            let status = Command::new("brew").args(["uninstall", package]).status()?;
            if !status.success() {
                return Err(anyhow::anyhow!("brew uninstall failed for {}", package));
            }
        }
        Ok(())
    }

    fn list_packages(&self) -> Result<Vec<PackageInfo>> {
        let output = Command::new("brew").args(["list", "--json"]).output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("brew list failed"));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let parsed: serde_json::Value = serde_json::from_str(&json_str)?;

        let mut packages = Vec::new();
        if let Some(formulae) = parsed.as_array() {
            for formula in formulae {
                if let (Some(name), Some(version)) = (
                    formula.get("name").and_then(|n| n.as_str()),
                    formula
                        .get("installed")
                        .and_then(|i| i.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|v| v.get("version"))
                        .and_then(|v| v.as_str()),
                ) {
                    packages.push(PackageInfo {
                        name: name.to_string(),
                        version: version.to_string(),
                        description: formula
                            .get("desc")
                            .and_then(|d| d.as_str())
                            .map(|s| s.to_string()),
                        homepage: formula
                            .get("homepage")
                            .and_then(|h| h.as_str())
                            .map(|s| s.to_string()),
                        repository: None,
                        license: None,
                        keywords: vec![],
                        dependencies: vec![],
                        is_dev_dependency: false,
                        is_optional: false,
                        install_size: None,
                    });
                }
            }
        }
        Ok(packages)
    }

    fn search_packages(&self, query: &str) -> Result<Vec<PackageInfo>> {
        let output = Command::new("brew")
            .args(["search", query, "--json"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("brew search failed"));
        }

        // Parse brew search results (simplified)
        let json_str = String::from_utf8_lossy(&output.stdout);
        let parsed: serde_json::Value = serde_json::from_str(&json_str)?;

        let mut packages = Vec::new();
        if let Some(formulae) = parsed.get("formulae").and_then(|f| f.as_array()) {
            for formula in formulae {
                if let Some(name) = formula.get("name").and_then(|n| n.as_str()) {
                    packages.push(PackageInfo {
                        name: name.to_string(),
                        version: "unknown".to_string(),
                        description: formula
                            .get("desc")
                            .and_then(|d| d.as_str())
                            .map(|s| s.to_string()),
                        homepage: formula
                            .get("homepage")
                            .and_then(|h| h.as_str())
                            .map(|s| s.to_string()),
                        repository: None,
                        license: None,
                        keywords: vec![],
                        dependencies: vec![],
                        is_dev_dependency: false,
                        is_optional: false,
                        install_size: None,
                    });
                }
            }
        }
        Ok(packages)
    }

    fn update_packages(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            // Update all packages
            let status = Command::new("brew").args(["upgrade"]).status()?;
            if !status.success() {
                return Err(anyhow::anyhow!("brew upgrade failed"));
            }
        } else {
            // Update specific packages
            for package in packages {
                let status = Command::new("brew").args(["upgrade", package]).status()?;
                if !status.success() {
                    return Err(anyhow::anyhow!("brew upgrade failed for {}", package));
                }
            }
        }
        Ok(())
    }

    fn is_available(&self) -> bool {
        Command::new("brew").arg("--version").output().is_ok()
    }

    fn get_config(&self) -> PackageManagerConfig {
        self.config.clone()
    }

    fn is_preferred_for_project(&self, _project_path: &Path) -> bool {
        // Homebrew is not project-specific
        false
    }
}

/// Rez package manager implementation (VFX industry)
pub struct RezPackageManager {
    config: PackageManagerConfig,
}

impl RezPackageManager {
    pub fn new() -> Self {
        Self {
            config: PackageManagerConfig {
                name: "rez".to_string(),
                version: None,
                executable_path: None,
                config_files: vec![],
                cache_directory: None,
                supports_lockfiles: false,
                supports_workspaces: true,
                isolation_level: IsolationLevel::Sandbox,
            },
        }
    }
}

impl UniversalPackageManager for RezPackageManager {
    fn name(&self) -> &str {
        "rez"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::VFX
    }

    fn install_packages(&self, packages: &[PackageSpec]) -> Result<()> {
        // Rez doesn't "install" packages in the traditional sense
        // It resolves and creates environments
        for package in packages {
            let mut args = vec!["env".to_string(), package.name.clone()];
            if let Some(version) = &package.version {
                args[1] = format!("{}-{}", package.name, version);
            }

            let status = Command::new("rez").args(&args).status()?;
            if !status.success() {
                return Err(anyhow::anyhow!("rez env failed for {}", package.name));
            }
        }
        Ok(())
    }

    fn remove_packages(&self, _packages: &[String]) -> Result<()> {
        // Rez doesn't have a traditional remove operation
        // Packages are managed through environment resolution
        Ok(())
    }

    fn list_packages(&self) -> Result<Vec<PackageInfo>> {
        let output = Command::new("rez")
            .args(["search", "--format", "json"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("rez search failed"));
        }

        // Parse rez package list (simplified)
        let packages = Vec::new(); // Would parse actual rez output
        Ok(packages)
    }

    fn search_packages(&self, query: &str) -> Result<Vec<PackageInfo>> {
        let output = Command::new("rez")
            .args(["search", query, "--format", "json"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("rez search failed"));
        }

        // Parse rez search results (simplified)
        let packages = Vec::new(); // Would parse actual rez output
        Ok(packages)
    }

    fn update_packages(&self, _packages: &[String]) -> Result<()> {
        // Rez packages are typically updated through repository management
        Ok(())
    }

    fn is_available(&self) -> bool {
        Command::new("rez").arg("--version").output().is_ok()
    }

    fn get_config(&self) -> PackageManagerConfig {
        self.config.clone()
    }

    fn is_preferred_for_project(&self, project_path: &Path) -> bool {
        project_path.join("package.py").exists() || project_path.join("rezbuild.py").exists()
    }
}

impl Default for NpmPackageManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for HomebrewPackageManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RezPackageManager {
    fn default() -> Self {
        Self::new()
    }
}
