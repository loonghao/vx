//! Bundle command implementation for offline development environments
//!
//! This module provides the `vx bundle` command for creating portable,
//! offline-capable development environment bundles.
//!
//! # Use Cases
//!
//! - Air-gapped development environments
//! - Consistent tooling in restricted networks
//! - Quick environment setup without internet
//!
//! # Commands
//!
//! - `vx bundle create` - Create a bundle from vx.lock
//! - `vx bundle status` - Show bundle status
//! - `vx bundle clean` - Remove the bundle

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use vx_paths::project::{find_vx_config, LOCK_FILE_NAME, PROJECT_VX_DIR};
use vx_paths::VxPaths;
use vx_resolver::LockFile;
use vx_runtime::ProviderRegistry;

// Re-export from vx_resolver for consistency
pub use vx_resolver::{
    has_bundle, is_online, try_get_bundle_context, BundleContext, BUNDLE_DIR, BUNDLE_MANIFEST,
};

/// Bundle manifest containing metadata about the bundled environment
/// Supports multi-platform bundles with platform-specific tool paths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleManifest {
    /// Manifest version (2 = multi-platform support)
    pub version: u32,
    /// When the bundle was created/updated
    pub created_at: String,
    /// vx version that created the bundle
    pub vx_version: String,
    /// Primary platform (platform that created the bundle)
    pub platform: String,
    /// All platforms included in this bundle
    #[serde(default)]
    pub platforms: Vec<String>,
    /// Bundled tools with their versions and platform info
    pub tools: HashMap<String, BundledTool>,
    /// Total bundle size in bytes
    pub total_size: u64,
}

/// Information about a bundled tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundledTool {
    /// Resolved version
    pub version: String,
    /// Path within bundle (legacy, for single-platform bundles)
    #[serde(default)]
    pub path: String,
    /// Platform-specific paths: platform -> path
    #[serde(default)]
    pub platform_paths: HashMap<String, String>,
    /// Size in bytes (total across all platforms)
    pub size: u64,
    /// Source (where it was bundled from)
    pub source: String,
}

impl BundledTool {
    /// Get the path for a specific platform
    pub fn path_for_platform(&self, platform: &str) -> Option<&str> {
        // First try platform-specific path
        if let Some(path) = self.platform_paths.get(platform) {
            return Some(path.as_str());
        }
        // Fall back to legacy single path (for v1 manifests)
        if !self.path.is_empty() {
            return Some(&self.path);
        }
        None
    }

    /// Get all available platforms for this tool
    pub fn available_platforms(&self) -> Vec<&str> {
        if self.platform_paths.is_empty() && !self.path.is_empty() {
            vec!["current"] // Legacy single-platform
        } else {
            self.platform_paths.keys().map(|s| s.as_str()).collect()
        }
    }
}

impl BundleManifest {
    /// Create a new empty manifest (v2 multi-platform)
    pub fn new() -> Self {
        let platform = current_platform();
        Self {
            version: 2,
            created_at: now_rfc3339(),
            vx_version: env!("CARGO_PKG_VERSION").to_string(),
            platform: platform.clone(),
            platforms: vec![platform],
            tools: HashMap::new(),
            total_size: 0,
        }
    }

    /// Load manifest from file
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read bundle manifest: {}", path.display()))?;
        let mut manifest: Self = serde_json::from_str(&content)
            .with_context(|| "Failed to parse bundle manifest")?;
        
        // Migrate v1 to v2 format if needed
        if manifest.version < 2 {
            manifest.migrate_to_v2();
        }
        
        Ok(manifest)
    }

    /// Migrate v1 manifest to v2 format
    fn migrate_to_v2(&mut self) {
        self.version = 2;
        if self.platforms.is_empty() {
            self.platforms = vec![self.platform.clone()];
        }
        
        // Migrate tool paths to platform_paths
        for tool in self.tools.values_mut() {
            if tool.platform_paths.is_empty() && !tool.path.is_empty() {
                tool.platform_paths.insert(self.platform.clone(), tool.path.clone());
            }
        }
    }

    /// Save manifest to file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
            .with_context(|| format!("Failed to write bundle manifest: {}", path.display()))?;
        Ok(())
    }

    /// Add a bundled tool for the current platform
    pub fn add_tool(&mut self, name: String, tool: BundledTool) {
        self.total_size += tool.size;
        self.tools.insert(name, tool);
    }

    /// Add or update a tool for a specific platform
    pub fn add_tool_platform(&mut self, name: String, platform: &str, version: String, path: String, size: u64, source: String) {
        // Add platform to list if not present
        if !self.platforms.contains(&platform.to_string()) {
            self.platforms.push(platform.to_string());
        }

        let tool = self.tools.entry(name).or_insert_with(|| BundledTool {
            version: version.clone(),
            path: String::new(),
            platform_paths: HashMap::new(),
            size: 0,
            source: source.clone(),
        });

        // Update version if different (should be same across platforms)
        tool.version = version;
        tool.platform_paths.insert(platform.to_string(), path);
        tool.size += size;
        tool.source = source;
        
        self.total_size += size;
    }

    /// Check if this bundle supports a specific platform
    pub fn supports_platform(&self, platform: &str) -> bool {
        self.platforms.contains(&platform.to_string())
    }

    /// Get tools available for a specific platform
    pub fn tools_for_platform(&self, platform: &str) -> Vec<(&String, &BundledTool)> {
        self.tools
            .iter()
            .filter(|(_, tool)| tool.path_for_platform(platform).is_some())
            .collect()
    }
}

impl Default for BundleManifest {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current platform string
fn current_platform() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

/// Get current time as RFC3339 string
fn now_rfc3339() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    // Simple ISO 8601 format without chrono dependency
    let secs = duration.as_secs();
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;
    
    // Calculate year, month, day from days since epoch (1970-01-01)
    let mut year = 1970i32;
    let mut remaining_days = days as i32;
    
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }
    
    let is_leap = is_leap_year(year);
    let days_in_months: [i32; 12] = [
        31,
        if is_leap { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];
    
    let mut month = 1;
    for &days_in_month in &days_in_months {
        if remaining_days < days_in_month {
            break;
        }
        remaining_days -= days_in_month;
        month += 1;
    }
    let day = remaining_days + 1;
    
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Handle bundle create command
pub async fn handle_create(
    _registry: &ProviderRegistry,
    _ctx: &vx_runtime::RuntimeContext,
    tools: Option<Vec<String>>,
    verbose: bool,
) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let config_path = find_vx_config(&current_dir)
        .map_err(|e| anyhow::anyhow!("No vx.toml found: {}", e))?;

    let project_root = config_path.parent().unwrap_or(&current_dir);
    let lock_path = project_root.join(LOCK_FILE_NAME);
    let bundle_dir = project_root.join(PROJECT_VX_DIR).join(BUNDLE_DIR);
    let bundle_store = bundle_dir.join("store");
    let manifest_path = bundle_dir.join(BUNDLE_MANIFEST);

    // Load lock file
    if !lock_path.exists() {
        return Err(anyhow::anyhow!(
            "No {} found. Run 'vx lock' first to generate one.",
            LOCK_FILE_NAME
        ));
    }

    let lockfile = LockFile::load(&lock_path)
        .with_context(|| format!("Failed to load {}", lock_path.display()))?;

    // Determine which tools to bundle
    let tools_to_bundle: Vec<String> = match tools {
        Some(t) => t,
        None => lockfile.tool_names().iter().map(|s| s.to_string()).collect(),
    };

    if tools_to_bundle.is_empty() {
        println!("No tools to bundle");
        return Ok(());
    }

    println!("Creating bundle for {} tools...", tools_to_bundle.len());

    // Create bundle directory
    fs::create_dir_all(&bundle_store)?;

    let mut manifest = BundleManifest::new();
    let vx_paths = VxPaths::default();

    for tool_name in &tools_to_bundle {
        let locked = match lockfile.get_tool(tool_name) {
            Some(t) => t,
            None => {
                eprintln!("  ⚠ {} not found in lock file, skipping", tool_name);
                continue;
            }
        };

        if verbose {
            println!("  Bundling {} {}...", tool_name, locked.version);
        }

        // Find source path in global store
        let source_path = vx_paths.store_dir.join(tool_name).join(&locked.version);

        if !source_path.exists() {
            eprintln!(
                "  ⚠ {} {} not installed, run 'vx install {}@{}' first",
                tool_name, locked.version, tool_name, locked.version
            );
            continue;
        }

        let platform = current_platform();
        
        // Copy to bundle with platform subdirectory: store/{tool}/{version}/{platform}/
        let dest_path = bundle_store.join(tool_name).join(&locked.version).join(&platform);
        let size = copy_dir_recursive(&source_path, &dest_path)?;

        // Add tool with platform-specific path
        manifest.add_tool_platform(
            tool_name.clone(),
            &platform,
            locked.version.clone(),
            format!("store/{}/{}/{}", tool_name, locked.version, platform),
            size,
            source_path.display().to_string(),
        );

        println!("  ✓ {} {} [{}] ({} MB)", tool_name, locked.version, platform, size / 1024 / 1024);
    }

    // Save manifest
    manifest.save(&manifest_path)?;

    let platform = current_platform();
    println!(
        "\n✓ Bundle created: {} tools for {}, {} MB total",
        manifest.tools.len(),
        platform,
        manifest.total_size / 1024 / 1024
    );
    println!("  Location: {}", bundle_dir.display());
    println!("\nThe bundle will be used automatically when offline.");
    println!("Use 'vx bundle update' on other platforms to add their binaries.");

    Ok(())
}

/// Handle bundle update command (incremental)
pub async fn handle_update(
    _registry: &ProviderRegistry,
    _ctx: &vx_runtime::RuntimeContext,
    tools: Option<Vec<String>>,
    verbose: bool,
) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let config_path = find_vx_config(&current_dir)
        .map_err(|e| anyhow::anyhow!("No vx.toml found: {}", e))?;

    let project_root = config_path.parent().unwrap_or(&current_dir);
    let lock_path = project_root.join(LOCK_FILE_NAME);
    let bundle_dir = project_root.join(PROJECT_VX_DIR).join(BUNDLE_DIR);
    let bundle_store = bundle_dir.join("store");
    let manifest_path = bundle_dir.join(BUNDLE_MANIFEST);

    // Load existing manifest
    if !manifest_path.exists() {
        return Err(anyhow::anyhow!(
            "No bundle found. Run 'vx bundle create' first."
        ));
    }

    let mut manifest = BundleManifest::load(&manifest_path)?;
    let platform = current_platform();

    // Load lock file
    if !lock_path.exists() {
        return Err(anyhow::anyhow!(
            "No {} found. Run 'vx lock' first to generate one.",
            LOCK_FILE_NAME
        ));
    }

    let lockfile = LockFile::load(&lock_path)
        .with_context(|| format!("Failed to load {}", lock_path.display()))?;

    // Determine which tools to check
    let tools_to_check: Vec<String> = match tools {
        Some(t) => t,
        None => lockfile.tool_names().iter().map(|s| s.to_string()).collect(),
    };

    let vx_paths = VxPaths::default();
    let mut updated = 0;
    let mut added = 0;
    let mut platform_added = 0;

    let is_new_platform = !manifest.supports_platform(&platform);
    if is_new_platform {
        println!("Adding platform {} to bundle...", platform);
    } else {
        println!("Updating bundle for platform {}...", platform);
    }

    // Check for updates and additions
    for tool_name in &tools_to_check {
        let locked = match lockfile.get_tool(tool_name) {
            Some(t) => t,
            None => continue,
        };

        // Check if this tool/version/platform combination exists
        let existing_tool = manifest.tools.get(tool_name);
        let needs_update = match existing_tool {
            Some(bundled) => {
                // Version changed or platform not present
                bundled.version != locked.version || 
                bundled.path_for_platform(&platform).is_none()
            },
            None => true, // New tool
        };

        if !needs_update {
            if verbose {
                println!("  ○ {} {} [{}] (unchanged)", tool_name, locked.version, platform);
            }
            continue;
        }

        // Find source path
        let source_path = vx_paths.store_dir.join(tool_name).join(&locked.version);

        if !source_path.exists() {
            eprintln!(
                "  ⚠ {} {} not installed, run 'vx install {}@{}' first",
                tool_name, locked.version, tool_name, locked.version
            );
            continue;
        }

        // Remove old version for this platform if version changed
        let is_version_update = existing_tool.map(|t| t.version != locked.version).unwrap_or(false);
        let is_platform_add = existing_tool.map(|t| t.path_for_platform(&platform).is_none()).unwrap_or(false);
        
        if is_version_update {
            if let Some(old_tool) = existing_tool {
                // Remove old version's platform directory
                let old_path = bundle_store.join(tool_name).join(&old_tool.version).join(&platform);
                if old_path.exists() {
                    fs::remove_dir_all(&old_path)?;
                }
                // If this was the only platform, remove the version directory too
                let version_dir = bundle_store.join(tool_name).join(&old_tool.version);
                if version_dir.exists() && fs::read_dir(&version_dir)?.next().is_none() {
                    fs::remove_dir_all(&version_dir)?;
                }
            }
            updated += 1;
        } else if is_platform_add {
            platform_added += 1;
        } else {
            added += 1;
        }

        // Copy new version with platform subdirectory
        let dest_path = bundle_store.join(tool_name).join(&locked.version).join(&platform);
        let size = copy_dir_recursive(&source_path, &dest_path)?;

        // Add/update tool with platform-specific path
        manifest.add_tool_platform(
            tool_name.clone(),
            &platform,
            locked.version.clone(),
            format!("store/{}/{}/{}", tool_name, locked.version, platform),
            size,
            source_path.display().to_string(),
        );

        let action = if is_version_update { "↑" } else if is_platform_add { "⊕" } else { "+" };
        println!(
            "  {} {} {} [{}] ({} MB)",
            action,
            tool_name,
            locked.version,
            platform,
            size / 1024 / 1024
        );
    }

    // Update timestamp and save
    manifest.created_at = now_rfc3339();
    manifest.save(&manifest_path)?;

    if updated == 0 && added == 0 && platform_added == 0 {
        println!("✓ Bundle is up to date for {}", platform);
    } else {
        println!(
            "\n✓ Bundle updated: {} version updates, {} new tools, {} platform additions",
            updated, added, platform_added
        );
        println!("  Platforms: {}", manifest.platforms.join(", "));
        println!("  Total: {} tools, {} MB", manifest.tools.len(), manifest.total_size / 1024 / 1024);
    }

    Ok(())
}

/// Handle bundle status command
pub async fn handle_status(verbose: bool) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let config_path = find_vx_config(&current_dir)
        .map_err(|e| anyhow::anyhow!("No vx.toml found: {}", e))?;

    let project_root = config_path.parent().unwrap_or(&current_dir);
    let bundle_dir = project_root.join(PROJECT_VX_DIR).join(BUNDLE_DIR);
    let manifest_path = bundle_dir.join(BUNDLE_MANIFEST);

    if !manifest_path.exists() {
        println!("No bundle found");
        println!("Run 'vx bundle create' to create one");
        return Ok(());
    }

    let manifest = BundleManifest::load(&manifest_path)?;
    let current_platform = current_platform();

    println!("Bundle Status");
    println!("─────────────────────────────────────");
    println!("Version:   v{}", manifest.version);
    println!("Created:   {}", manifest.created_at);
    println!("vx:        {}", manifest.vx_version);
    println!("Size:      {} MB", manifest.total_size / 1024 / 1024);
    println!("Tools:     {}", manifest.tools.len());
    println!();
    println!("Platforms: {}", manifest.platforms.join(", "));
    
    let supports_current = manifest.supports_platform(&current_platform);
    if supports_current {
        println!("           ✓ Current platform ({}) is supported", current_platform);
    } else {
        println!("           ⚠ Current platform ({}) NOT in bundle", current_platform);
        println!("           Run 'vx bundle update' to add it");
    }

    if verbose || manifest.tools.len() <= 10 {
        println!("\nBundled tools:");
        for (name, tool) in &manifest.tools {
            let platforms: Vec<&str> = tool.available_platforms();
            let has_current = tool.path_for_platform(&current_platform).is_some();
            let status = if has_current { "✓" } else { "○" };
            println!(
                "  {} {} {} ({} MB) [{}]",
                status,
                name,
                tool.version,
                tool.size / 1024 / 1024,
                platforms.join(", ")
            );
        }
    }

    // Check if online detection is working
    println!("\nNetwork status: {}", if is_online() { "Online" } else { "Offline" });

    Ok(())
}

/// Handle bundle clean command
pub async fn handle_clean(force: bool) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let config_path = find_vx_config(&current_dir)
        .map_err(|e| anyhow::anyhow!("No vx.toml found: {}", e))?;

    let project_root = config_path.parent().unwrap_or(&current_dir);
    let bundle_dir = project_root.join(PROJECT_VX_DIR).join(BUNDLE_DIR);

    if !bundle_dir.exists() {
        println!("No bundle to clean");
        return Ok(());
    }

    if !force {
        println!("This will remove the bundle at: {}", bundle_dir.display());
        println!("Use --force to confirm");
        return Ok(());
    }

    fs::remove_dir_all(&bundle_dir)?;
    println!("✓ Bundle removed");

    Ok(())
}

/// Handle bundle export command - create a portable tar.gz archive
/// Supports exporting specific tools and/or platforms
pub async fn handle_export(
    output: Option<PathBuf>,
    tools: Option<Vec<String>>,
    platforms: Option<Vec<String>>,
    verbose: bool,
) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let config_path = find_vx_config(&current_dir)
        .map_err(|e| anyhow::anyhow!("No vx.toml found: {}", e))?;

    let project_root = config_path.parent().unwrap_or(&current_dir);
    let bundle_dir = project_root.join(PROJECT_VX_DIR).join(BUNDLE_DIR);
    let manifest_path = bundle_dir.join(BUNDLE_MANIFEST);

    if !manifest_path.exists() {
        return Err(anyhow::anyhow!(
            "No bundle found. Run 'vx bundle create' first."
        ));
    }

    let manifest = BundleManifest::load(&manifest_path)?;

    // Determine which platforms to export
    let platforms_to_export: Vec<String> = match &platforms {
        Some(p) => p.iter()
            .filter(|plat| manifest.platforms.contains(plat))
            .cloned()
            .collect(),
        None => manifest.platforms.clone(),
    };

    if platforms_to_export.is_empty() {
        return Err(anyhow::anyhow!(
            "No matching platforms. Available: {}",
            manifest.platforms.join(", ")
        ));
    }

    // Determine output path (include platforms in name if filtered)
    let output_path = output.unwrap_or_else(|| {
        let platform_suffix = if platforms_to_export.len() == 1 {
            platforms_to_export[0].clone()
        } else if platforms_to_export.len() == manifest.platforms.len() {
            "multi".to_string()
        } else {
            format!("{}-platforms", platforms_to_export.len())
        };
        current_dir.join(format!("vx-bundle-{}.tar.gz", platform_suffix))
    });

    // Filter tools if specified
    let tools_to_export: Vec<&String> = match &tools {
        Some(t) => manifest
            .tools
            .keys()
            .filter(|k| t.contains(k))
            .collect(),
        None => manifest.tools.keys().collect(),
    };

    if tools_to_export.is_empty() {
        return Err(anyhow::anyhow!("No tools to export"));
    }

    println!("Exporting bundle to {}...", output_path.display());
    if platforms_to_export.len() < manifest.platforms.len() {
        println!("  Platforms: {}", platforms_to_export.join(", "));
    }

    // Create tar.gz archive
    let file = fs::File::create(&output_path)
        .with_context(|| format!("Failed to create {}", output_path.display()))?;

    let enc = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut tar = tar::Builder::new(enc);

    // Create filtered manifest for export
    let mut export_manifest = manifest.clone();
    
    // Filter tools
    if tools.is_some() {
        export_manifest.tools.retain(|k, _| tools_to_export.contains(&k));
    }
    
    // Filter platforms in manifest and tool paths
    if platforms.is_some() {
        export_manifest.platforms = platforms_to_export.clone();
        for tool in export_manifest.tools.values_mut() {
            tool.platform_paths.retain(|k, _| platforms_to_export.contains(k));
        }
    }
    
    // Recalculate total size (will be approximate since we don't track per-platform sizes)
    export_manifest.total_size = export_manifest.tools.values().map(|t| t.size).sum();
    
    // Write manifest
    let manifest_json = serde_json::to_string_pretty(&export_manifest)?;
    let manifest_bytes = manifest_json.as_bytes();
    let mut header = tar::Header::new_gnu();
    header.set_path(BUNDLE_MANIFEST)?;
    header.set_size(manifest_bytes.len() as u64);
    header.set_mode(0o644);
    header.set_mtime(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs());
    header.set_cksum();
    tar.append(&header, manifest_bytes)?;

    // Add each tool
    let bundle_store = bundle_dir.join("store");
    for tool_name in &tools_to_export {
        let tool_info = &manifest.tools[*tool_name];
        let tool_version_path = bundle_store.join(tool_name).join(&tool_info.version);

        if !tool_version_path.exists() {
            eprintln!("  ⚠ {} not found in bundle, skipping", tool_name);
            continue;
        }

        // Export only selected platforms
        for platform in &platforms_to_export {
            let platform_path = tool_version_path.join(platform);
            if platform_path.exists() {
                if verbose {
                    println!(
                        "  Adding {} {} [{}]...",
                        tool_name,
                        tool_info.version,
                        platform
                    );
                }
                add_dir_to_tar(
                    &mut tar,
                    &platform_path,
                    &format!("store/{}/{}/{}", tool_name, tool_info.version, platform),
                )?;
            } else {
                // Check for legacy v1 structure (no platform subdirectory)
                let has_platform_subdirs = tool_version_path
                    .read_dir()
                    .map(|d| d.filter_map(|e| e.ok()).any(|e| {
                        e.file_name().to_string_lossy().contains('-')
                    }))
                    .unwrap_or(false);
                
                if !has_platform_subdirs {
                    // v1 structure - export entire version directory
                    if verbose {
                        println!(
                            "  Adding {} {} (legacy)...",
                            tool_name,
                            tool_info.version,
                        );
                    }
                    add_dir_to_tar(
                        &mut tar,
                        &tool_version_path,
                        &format!("store/{}/{}", tool_name, tool_info.version),
                    )?;
                    break; // Only add once for v1 structure
                }
            }
        }
    }

    tar.finish()?;

    let file_size = fs::metadata(&output_path)?.len();
    println!(
        "\n✓ Bundle exported: {} tools × {} platforms, {} MB compressed",
        tools_to_export.len(),
        platforms_to_export.len(),
        file_size / 1024 / 1024
    );
    println!("  Archive: {}", output_path.display());

    Ok(())
}

/// Handle bundle import command - restore from a tar.gz archive
pub async fn handle_import(archive: &Path, force: bool, verbose: bool) -> Result<()> {
    if !archive.exists() {
        return Err(anyhow::anyhow!("Archive not found: {}", archive.display()));
    }

    let current_dir = std::env::current_dir()?;
    let config_path = find_vx_config(&current_dir)
        .map_err(|e| anyhow::anyhow!("No vx.toml found: {}", e))?;

    let project_root = config_path.parent().unwrap_or(&current_dir);
    let bundle_dir = project_root.join(PROJECT_VX_DIR).join(BUNDLE_DIR);
    let manifest_path = bundle_dir.join(BUNDLE_MANIFEST);

    // Check if bundle already exists
    if manifest_path.exists() && !force {
        return Err(anyhow::anyhow!(
            "Bundle already exists. Use --force to overwrite."
        ));
    }

    println!("Importing bundle from {}...", archive.display());

    // Create bundle directory
    fs::create_dir_all(&bundle_dir)?;

    // Extract tar.gz archive
    let file = fs::File::open(archive)
        .with_context(|| format!("Failed to open {}", archive.display()))?;

    let dec = flate2::read::GzDecoder::new(file);
    let mut tar = tar::Archive::new(dec);

    let mut manifest: Option<BundleManifest> = None;

    for entry in tar.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_path_buf();

        if path.to_string_lossy() == BUNDLE_MANIFEST {
            // Read manifest
            let mut content = String::new();
            entry.read_to_string(&mut content)?;
            manifest = Some(serde_json::from_str(&content)?);
            if verbose {
                println!("  Found manifest");
            }
        } else if path.starts_with("store/") {
            // Extract tool file
            let dest_path = bundle_dir.join(&path);

            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut file = fs::File::create(&dest_path)?;
            std::io::copy(&mut entry, &mut file)?;

            // Preserve permissions
            if let Ok(mode) = entry.header().mode() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(&dest_path, fs::Permissions::from_mode(mode))?;
                }
                let _ = mode; // Suppress unused warning on Windows
            }

            if verbose && path.components().count() == 4 {
                // Only log top-level tool directories
                let parts: Vec<_> = path.components().collect();
                if parts.len() >= 3 {
                    println!("  Extracting {}/{}", 
                        parts[1].as_os_str().to_string_lossy(),
                        parts[2].as_os_str().to_string_lossy()
                    );
                }
            }
        }
    }

    // Write manifest
    let imported_tools = if let Some(mut manifest) = manifest {
        manifest.created_at = now_rfc3339();
        let count = manifest.tools.len();
        manifest.save(&manifest_path)?;
        count
    } else {
        return Err(anyhow::anyhow!("Archive does not contain a valid manifest"));
    };

    println!("\n✓ Bundle imported: {} tools", imported_tools);
    println!("  Location: {}", bundle_dir.display());

    Ok(())
}

/// Add a directory recursively to a tar archive
/// Uses append_path_with_name for PAX extension support (long paths)
fn add_dir_to_tar<W: Write>(
    tar: &mut tar::Builder<W>,
    src: &Path,
    prefix: &str,
) -> Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let tar_path = format!("{}/{}", prefix, name.to_string_lossy());

        if path.is_dir() {
            add_dir_to_tar(tar, &path, &tar_path)?;
        } else {
            // Use append_path_with_name which handles long paths automatically
            // by using PAX extensions when necessary
            tar.append_path_with_name(&path, &tar_path)
                .with_context(|| format!("Failed to add {} to archive", tar_path))?;
        }
    }

    Ok(())
}

/// Copy directory recursively and return total size
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<u64> {
    let mut total_size = 0u64;

    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            total_size += copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            let metadata = fs::metadata(&src_path)?;
            total_size += metadata.len();
            fs::copy(&src_path, &dst_path)?;

            // Preserve executable permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = metadata.permissions();
                fs::set_permissions(&dst_path, perms)?;
            }
        }
    }

    Ok(total_size)
}

/// Quick network connectivity check
///
/// Re-exported from vx_resolver for CLI commands that need it directly.

/// Check if a bundle exists for the current project
pub fn has_bundle_at(project_root: &Path) -> bool {
    let manifest_path = project_root
        .join(PROJECT_VX_DIR)
        .join(BUNDLE_DIR)
        .join(BUNDLE_MANIFEST);
    manifest_path.exists()
}

/// Load bundle manifest for the project (local version for CLI status display)
pub fn load_bundle_manifest_local(project_root: &Path) -> Option<BundleManifest> {
    let manifest_path = project_root
        .join(PROJECT_VX_DIR)
        .join(BUNDLE_DIR)
        .join(BUNDLE_MANIFEST);

    BundleManifest::load(&manifest_path).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_bundle_manifest_serialization() {
        let mut manifest = BundleManifest::new();
        manifest.add_tool(
            "python".to_string(),
            BundledTool {
                version: "3.12.12".to_string(),
                path: "store/python/3.12.12".to_string(),
                size: 100_000_000,
                source: "/home/user/.vx/store/python/3.12.12".to_string(),
            },
        );

        let json = serde_json::to_string(&manifest).unwrap();
        let parsed: BundleManifest = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.tools.len(), 1);
        assert!(parsed.tools.contains_key("python"));
    }

    #[test]
    fn test_has_bundle() {
        let temp = TempDir::new().unwrap();
        let project = temp.path();

        assert!(!has_bundle(project));

        // Create bundle structure
        let bundle_dir = project.join(PROJECT_VX_DIR).join(BUNDLE_DIR);
        fs::create_dir_all(&bundle_dir).unwrap();

        let manifest = BundleManifest::new();
        manifest.save(&bundle_dir.join(BUNDLE_MANIFEST)).unwrap();

        assert!(has_bundle(project));
    }
}
