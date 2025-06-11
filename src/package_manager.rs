use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub install_path: PathBuf,
    pub executable_path: PathBuf,
    pub installed_at: chrono::DateTime<chrono::Utc>,
    pub dependencies: Vec<String>,
    pub metadata: PackageMetadata,
}

impl std::fmt::Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.version)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageMetadata {
    pub description: String,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub size: u64,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageRegistry {
    pub packages: HashMap<String, Vec<Package>>, // tool_name -> versions
    pub active_versions: HashMap<String, String>, // tool_name -> active_version
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Default for PackageRegistry {
    fn default() -> Self {
        Self {
            packages: HashMap::new(),
            active_versions: HashMap::new(),
            last_updated: chrono::Utc::now(),
        }
    }
}

pub struct PackageManager {
    registry_path: PathBuf,
    packages_dir: PathBuf,
    registry: PackageRegistry,
}

impl PackageManager {
    pub fn new() -> Result<Self> {
        let vx_dir = Self::get_vx_dir()?;
        let registry_path = vx_dir.join("registry.json");
        let packages_dir = vx_dir.join("packages");

        // Create directories if they don't exist
        fs::create_dir_all(&packages_dir)?;

        // Load or create registry
        let registry = if registry_path.exists() {
            let content = fs::read_to_string(&registry_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            PackageRegistry::default()
        };

        Ok(Self {
            registry_path,
            packages_dir,
            registry,
        })
    }

    /// Get vx directory path
    fn get_vx_dir() -> Result<PathBuf> {
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home_dir.join(".vx"))
    }

    /// Save registry to disk
    pub fn save_registry(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.registry)?;
        fs::write(&self.registry_path, content)?;
        Ok(())
    }

    /// Install a package
    pub fn install_package(&mut self, package: Package) -> Result<()> {
        let tool_name = &package.name;
        let version = &package.version;

        // Add to registry
        self.registry
            .packages
            .entry(tool_name.clone())
            .or_default()
            .push(package.clone());

        // Set as active version if it's the first or latest
        if !self.registry.active_versions.contains_key(tool_name) {
            self.registry
                .active_versions
                .insert(tool_name.clone(), version.clone());
        }

        self.registry.last_updated = chrono::Utc::now();
        self.save_registry()?;

        println!("📦 Installed {} {} to registry", tool_name, version);
        Ok(())
    }

    /// List all installed packages
    pub fn list_packages(&self) -> Vec<&Package> {
        self.registry.packages.values().flatten().collect()
    }

    /// List installed versions of a specific tool
    pub fn list_versions(&self, tool_name: &str) -> Vec<&Package> {
        self.registry
            .packages
            .get(tool_name)
            .map(|versions| versions.iter().collect())
            .unwrap_or_default()
    }

    /// Get active version of a tool
    pub fn get_active_version(&self, tool_name: &str) -> Option<&Package> {
        let active_version = self.registry.active_versions.get(tool_name)?;
        self.registry
            .packages
            .get(tool_name)?
            .iter()
            .find(|pkg| &pkg.version == active_version)
    }

    /// Switch active version of a tool
    pub fn switch_version(&mut self, tool_name: &str, version: &str) -> Result<()> {
        // Check if version exists
        let versions = self
            .registry
            .packages
            .get(tool_name)
            .ok_or_else(|| anyhow::anyhow!("Tool {} not found", tool_name))?;

        let package = versions
            .iter()
            .find(|pkg| pkg.version == version)
            .ok_or_else(|| anyhow::anyhow!("Version {} not found for {}", version, tool_name))?;

        // Update active version
        self.registry
            .active_versions
            .insert(tool_name.to_string(), version.to_string());
        self.registry.last_updated = chrono::Utc::now();
        self.save_registry()?;

        println!("🔄 Switched {} to version {}", tool_name, version);
        println!("📍 Active path: {}", package.executable_path.display());

        Ok(())
    }

    /// Remove a specific version
    pub fn remove_version(&mut self, tool_name: &str, version: &str) -> Result<()> {
        let versions = self
            .registry
            .packages
            .get_mut(tool_name)
            .ok_or_else(|| anyhow::anyhow!("Tool {} not found", tool_name))?;

        let package_index = versions
            .iter()
            .position(|pkg| pkg.version == version)
            .ok_or_else(|| anyhow::anyhow!("Version {} not found for {}", version, tool_name))?;

        let package = &versions[package_index];

        // Remove files
        if package.install_path.exists() {
            fs::remove_dir_all(&package.install_path)?;
            println!("🗑️  Removed files from {}", package.install_path.display());
        }

        // Remove from registry
        versions.remove(package_index);

        // If this was the active version, switch to latest available
        if let Some(active_version) = self.registry.active_versions.get(tool_name) {
            if active_version == version {
                if let Some(latest) = versions.last() {
                    self.registry
                        .active_versions
                        .insert(tool_name.to_string(), latest.version.clone());
                    println!("🔄 Switched to version {}", latest.version);
                } else {
                    self.registry.active_versions.remove(tool_name);
                    println!("⚠️  No versions left for {}", tool_name);
                }
            }
        }

        // Remove tool entirely if no versions left
        if versions.is_empty() {
            self.registry.packages.remove(tool_name);
        }

        self.registry.last_updated = chrono::Utc::now();
        self.save_registry()?;

        println!("✅ Removed {} version {}", tool_name, version);
        Ok(())
    }

    /// Clean up orphaned packages
    pub fn cleanup(&mut self) -> Result<()> {
        let mut removed_count = 0;

        // Check each package directory
        for (tool_name, versions) in &mut self.registry.packages {
            versions.retain(|package| {
                if !package.install_path.exists() {
                    println!(
                        "🧹 Cleaning up orphaned package: {} {}",
                        tool_name, package.version
                    );
                    removed_count += 1;
                    false
                } else {
                    true
                }
            });
        }

        // Remove empty tool entries
        self.registry
            .packages
            .retain(|_, versions| !versions.is_empty());

        // Clean up active versions that no longer exist
        let mut to_remove = Vec::new();
        for (tool_name, active_version) in &self.registry.active_versions {
            if let Some(versions) = self.registry.packages.get(tool_name) {
                if !versions.iter().any(|pkg| &pkg.version == active_version) {
                    to_remove.push(tool_name.clone());
                }
            } else {
                to_remove.push(tool_name.clone());
            }
        }

        for tool_name in to_remove {
            self.registry.active_versions.remove(&tool_name);
        }

        if removed_count > 0 {
            self.registry.last_updated = chrono::Utc::now();
            self.save_registry()?;
            println!("✅ Cleaned up {} orphaned packages", removed_count);
        } else {
            println!("✨ No cleanup needed");
        }

        Ok(())
    }

    /// Get package installation directory for a tool and version
    pub fn get_package_dir(&self, tool_name: &str, version: &str) -> PathBuf {
        self.packages_dir.join(tool_name).join(version)
    }

    /// Check for updates
    pub async fn check_updates(&self) -> Result<Vec<(String, String, String)>> {
        let updates = Vec::new();

        for (tool_name, versions) in &self.registry.packages {
            if let Some(active_version) = self.registry.active_versions.get(tool_name) {
                // This would integrate with version checking logic
                // For now, we'll return empty - this would be implemented
                // with the existing version manager
                let _ = (tool_name, active_version, versions);
            }
        }

        Ok(updates)
    }

    /// Get registry statistics
    pub fn get_stats(&self) -> PackageStats {
        let total_packages = self.registry.packages.len();
        let total_versions: usize = self.registry.packages.values().map(|v| v.len()).sum();
        let total_size: u64 = self
            .registry
            .packages
            .values()
            .flatten()
            .map(|pkg| pkg.metadata.size)
            .sum();

        PackageStats {
            total_packages,
            total_versions,
            total_size,
            last_updated: self.registry.last_updated,
        }
    }

    /// Get the installation path for a specific version of a tool
    pub fn get_version_path(&self, tool_name: &str, version: &Package) -> Result<PathBuf> {
        if let Some(versions) = self.registry.packages.get(tool_name) {
            if let Some(package) = versions.iter().find(|p| p.version == version.version) {
                return Ok(package.install_path.clone());
            }
        }
        Err(anyhow::anyhow!(
            "Version {} of {} not found",
            version.version,
            tool_name
        ))
    }

    /// List all installed tools (unique tool names)
    pub fn list_installed_tools(&self) -> Result<Vec<String>> {
        Ok(self.registry.packages.keys().cloned().collect())
    }
}

#[derive(Debug)]
pub struct PackageStats {
    pub total_packages: usize,
    pub total_versions: usize,
    pub total_size: u64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl PackageStats {
    pub fn format_size(&self) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = self.total_size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.1} {}", size, UNITS[unit_index])
    }
}
