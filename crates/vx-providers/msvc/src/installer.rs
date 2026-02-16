//! MSVC Build Tools installer using msvc-kit
//!
//! This module provides a thin wrapper around msvc-kit for downloading
//! and installing MSVC Build Tools.

use anyhow::{Context, Result};
use msvc_kit::{
    Architecture, DownloadOptions, MsvcComponent, download_msvc, download_sdk,
    extract_and_finalize_msvc, extract_and_finalize_sdk, setup_environment,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// MSVC installer using msvc-kit
pub struct MsvcInstaller {
    /// Target MSVC version (e.g., "14.40" or "14.40.33807")
    pub msvc_version: Option<String>,
    /// Windows SDK version (optional)
    pub sdk_version: Option<String>,
    /// Target architecture
    pub arch: Architecture,
    /// Host architecture
    pub host_arch: Architecture,
    /// Optional MSVC components to include (e.g., Spectre, MFC, ATL)
    pub include_components: HashSet<MsvcComponent>,
    /// Package ID patterns to exclude from installation
    pub exclude_patterns: Vec<String>,
}

impl MsvcInstaller {
    /// Create a new installer with the specified MSVC version
    pub fn new(msvc_version: &str) -> Self {
        // Parse version - msvc-kit expects major.minor format (e.g., "14.40")
        let version = Self::normalize_version(msvc_version);

        Self {
            msvc_version: Some(version),
            sdk_version: None,
            arch: Architecture::X64,
            host_arch: Architecture::X64,
            include_components: HashSet::new(),
            exclude_patterns: Vec::new(),
        }
    }

    /// Create installer for latest version
    pub fn latest() -> Self {
        Self {
            msvc_version: None,
            sdk_version: None,
            arch: Architecture::X64,
            host_arch: Architecture::X64,
            include_components: HashSet::new(),
            exclude_patterns: Vec::new(),
        }
    }

    /// Set the Windows SDK version
    pub fn with_sdk_version(mut self, version: &str) -> Self {
        self.sdk_version = Some(version.to_string());
        self
    }

    /// Set the target architecture
    pub fn with_arch(mut self, arch: Architecture) -> Self {
        self.arch = arch;
        self
    }

    /// Set the host architecture
    pub fn with_host_arch(mut self, host_arch: Architecture) -> Self {
        self.host_arch = host_arch;
        self
    }

    /// Set the optional MSVC components to include
    pub fn with_components(mut self, components: HashSet<MsvcComponent>) -> Self {
        self.include_components = components;
        self
    }

    /// Set the package ID patterns to exclude
    pub fn with_exclude_patterns(mut self, patterns: Vec<String>) -> Self {
        self.exclude_patterns = patterns;
        self
    }

    /// Normalize version string to major.minor format
    fn normalize_version(version: &str) -> String {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() >= 2 {
            format!("{}.{}", parts[0], parts[1])
        } else {
            version.to_string()
        }
    }

    /// Clean up incomplete installation markers
    ///
    /// msvc-kit uses `.msvc-kit-extracted/*.done` files to track extraction status.
    /// If a previous installation was incomplete, we need to remove these markers
    /// to force re-extraction.
    ///
    /// Note: msvc-kit 0.2.0 fixed the bug where extraction was skipped when target_dir
    /// had existing content. This cleanup is now only needed for truly incomplete installs.
    fn cleanup_incomplete_installation(install_path: &Path) -> Result<()> {
        let marker_dir = install_path.join(".msvc-kit-extracted");

        // Check for common indicators that installation completed successfully
        let has_vc_tools = install_path.join("VC").join("Tools").exists();
        let has_bin = install_path.join("bin").exists();

        // If we have markers but no actual installation, clean up
        if marker_dir.exists() && !has_vc_tools && !has_bin {
            warn!(
                "Detected incomplete MSVC installation, cleaning up markers to force re-extraction..."
            );
            if let Err(e) = std::fs::remove_dir_all(&marker_dir) {
                debug!("Failed to remove marker directory: {}", e);
            }
        }

        Ok(())
    }

    /// Install MSVC Build Tools to the specified directory
    pub async fn install(&self, install_path: &Path) -> Result<MsvcInstallInfo> {
        info!("Installing MSVC Build Tools to {}", install_path.display());

        // Clean up any incomplete installation markers
        Self::cleanup_incomplete_installation(install_path)?;

        // Build download options
        let mut options_builder = DownloadOptions::builder()
            .target_dir(install_path)
            .arch(self.arch)
            .host_arch(self.host_arch)
            .verify_hashes(true)
            .parallel_downloads(4);

        if let Some(ref version) = self.msvc_version {
            options_builder = options_builder.msvc_version(version);
        }

        if let Some(ref sdk_version) = self.sdk_version {
            options_builder = options_builder.sdk_version(sdk_version);
        }

        // Add optional components
        for component in &self.include_components {
            options_builder = options_builder.include_component(component.clone());
        }

        // Add exclude patterns
        for pattern in &self.exclude_patterns {
            options_builder = options_builder.exclude_pattern(pattern);
        }

        let options = options_builder.build();

        // Download MSVC
        debug!("Downloading MSVC components...");
        let mut msvc_info = download_msvc(&options)
            .await
            .context("Failed to download MSVC Build Tools")?;

        info!(
            "MSVC {} downloaded to {}",
            msvc_info.version,
            msvc_info.install_path.display()
        );

        // Extract MSVC packages (modifies msvc_info in place)
        debug!("Extracting MSVC packages...");
        extract_and_finalize_msvc(&mut msvc_info)
            .await
            .context("Failed to extract MSVC Build Tools")?;

        info!(
            "MSVC {} extracted to {}",
            msvc_info.version,
            msvc_info.install_path.display()
        );

        // Optionally download and extract Windows SDK
        let sdk_info = if self.sdk_version.is_some() {
            debug!("Downloading Windows SDK...");
            match download_sdk(&options).await {
                Ok(sdk) => {
                    debug!("Extracting Windows SDK...");
                    match extract_and_finalize_sdk(&sdk).await {
                        Ok(()) => {
                            info!(
                                "Windows SDK {} installed to {}",
                                sdk.version,
                                sdk.install_path.display()
                            );
                            Some(sdk)
                        }
                        Err(e) => {
                            debug!("Failed to extract Windows SDK: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    debug!("Failed to download Windows SDK: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Setup environment to get tool paths
        let env = setup_environment(&msvc_info, sdk_info.as_ref())
            .context("Failed to setup MSVC environment")?;

        // Get cl.exe path - try multiple methods
        let cl_path = env
            .cl_exe_path()
            .or_else(|| {
                // Fallback: search in msvc_info.install_path
                Self::find_cl_exe(&msvc_info.install_path)
            })
            .or_else(|| {
                // Fallback: search in the original install_path
                Self::find_cl_exe(install_path)
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "cl.exe not found after installation. Install path: {}, MSVC info path: {}",
                    install_path.display(),
                    msvc_info.install_path.display()
                )
            })?;

        Ok(MsvcInstallInfo {
            install_path: install_path.to_path_buf(),
            msvc_version: msvc_info.version,
            sdk_version: sdk_info.map(|s| s.version),
            cl_exe_path: cl_path,
            link_exe_path: env.link_exe_path(),
            lib_exe_path: env.lib_exe_path(),
            nmake_exe_path: env.nmake_exe_path(),
            include_paths: env.include_paths.clone(),
            lib_paths: env.lib_paths.clone(),
            bin_paths: env.bin_paths.clone(),
        })
    }

    /// Search for cl.exe in the given directory
    fn find_cl_exe(dir: &Path) -> Option<PathBuf> {
        walkdir::WalkDir::new(dir)
            .max_depth(10)
            .into_iter()
            .filter_map(|e| e.ok())
            .find(|e| e.file_name() == "cl.exe")
            .map(|e| e.path().to_path_buf())
    }
}

/// Information about an MSVC installation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MsvcInstallInfo {
    /// Root installation path
    pub install_path: PathBuf,
    /// Installed MSVC version
    pub msvc_version: String,
    /// Installed SDK version (if any)
    pub sdk_version: Option<String>,
    /// Path to cl.exe
    pub cl_exe_path: PathBuf,
    /// Path to link.exe
    pub link_exe_path: Option<PathBuf>,
    /// Path to lib.exe
    pub lib_exe_path: Option<PathBuf>,
    /// Path to nmake.exe
    pub nmake_exe_path: Option<PathBuf>,
    /// Include paths for compilation
    pub include_paths: Vec<PathBuf>,
    /// Library paths for linking
    pub lib_paths: Vec<PathBuf>,
    /// Binary paths (for PATH)
    pub bin_paths: Vec<PathBuf>,
}

/// The filename used to store MSVC installation info
const MSVC_INFO_FILENAME: &str = "msvc-info.json";

impl MsvcInstallInfo {
    /// Check if the installation is valid
    pub fn is_valid(&self) -> bool {
        self.cl_exe_path.exists()
    }

    /// Validate that all cached paths still exist on disk
    ///
    /// Returns true if all critical paths are valid, false if the cached info
    /// is stale (e.g., version mismatch between cached msvc-info.json and
    /// actual installation).
    ///
    /// See: https://github.com/loonghao/vx/issues/573
    pub fn validate_paths(&self) -> bool {
        // Critical: cl.exe must exist
        if !self.cl_exe_path.exists() {
            return false;
        }

        // At least some include paths should exist
        if !self.include_paths.is_empty() && !self.include_paths.iter().any(|p| p.exists()) {
            return false;
        }

        // At least some lib paths should exist
        if !self.lib_paths.is_empty() && !self.lib_paths.iter().any(|p| p.exists()) {
            return false;
        }

        true
    }

    /// Get only the paths that actually exist on disk
    ///
    /// Filters out stale/invalid paths from cached msvc-info.json to prevent
    /// injecting non-existent paths into environment variables.
    pub fn validated_include_paths(&self) -> Vec<&Path> {
        self.include_paths
            .iter()
            .filter(|p| p.exists())
            .map(|p| p.as_path())
            .collect()
    }

    /// Get only the lib paths that actually exist on disk
    pub fn validated_lib_paths(&self) -> Vec<&Path> {
        self.lib_paths
            .iter()
            .filter(|p| p.exists())
            .map(|p| p.as_path())
            .collect()
    }

    /// Get only the bin paths that actually exist on disk
    pub fn validated_bin_paths(&self) -> Vec<&Path> {
        self.bin_paths
            .iter()
            .filter(|p| p.exists())
            .map(|p| p.as_path())
            .collect()
    }

    /// Get all tool executables
    pub fn get_tool_path(&self, tool: &str) -> Option<PathBuf> {
        match tool.to_lowercase().as_str() {
            "cl" | "cl.exe" => Some(self.cl_exe_path.clone()),
            "link" | "link.exe" => self.link_exe_path.clone(),
            "lib" | "lib.exe" => self.lib_exe_path.clone(),
            "nmake" | "nmake.exe" => self.nmake_exe_path.clone(),
            _ => {
                // Search in bin paths
                for bin_path in &self.bin_paths {
                    let tool_path = bin_path.join(format!("{}.exe", tool));
                    if tool_path.exists() {
                        return Some(tool_path);
                    }
                }
                None
            }
        }
    }

    /// Save the installation info to disk
    ///
    /// The info is saved as a JSON file in the installation directory.
    pub fn save(&self) -> Result<()> {
        let info_path = self.install_path.join(MSVC_INFO_FILENAME);
        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize MSVC installation info")?;
        std::fs::write(&info_path, json)
            .with_context(|| format!("Failed to write MSVC info to {}", info_path.display()))?;
        debug!("Saved MSVC installation info to {}", info_path.display());
        Ok(())
    }

    /// Load installation info from disk
    ///
    /// # Arguments
    ///
    /// * `install_path` - The installation directory to load info from
    ///
    /// # Returns
    ///
    /// The loaded installation info, or None if the info file doesn't exist.
    pub fn load(install_path: &Path) -> Result<Option<Self>> {
        let info_path = install_path.join(MSVC_INFO_FILENAME);
        if !info_path.exists() {
            debug!("MSVC info file not found at {}", info_path.display());
            return Ok(None);
        }

        let json = std::fs::read_to_string(&info_path)
            .with_context(|| format!("Failed to read MSVC info from {}", info_path.display()))?;
        let info: Self = serde_json::from_str(&json)
            .with_context(|| format!("Failed to parse MSVC info from {}", info_path.display()))?;

        debug!("Loaded MSVC installation info from {}", info_path.display());
        Ok(Some(info))
    }

    /// Get environment variables for MSVC compilation
    ///
    /// Returns a HashMap with INCLUDE, LIB, and PATH environment variables
    /// configured for MSVC compilation.
    ///
    /// **Important**: Only paths that actually exist on disk are included.
    /// This prevents stale cached paths (e.g., from a version mismatch between
    /// msvc-info.json and the actual installation) from being injected into
    /// the environment, which could break tools like node-gyp's C# compiler.
    ///
    /// See: https://github.com/loonghao/vx/issues/573
    pub fn get_environment(&self) -> std::collections::HashMap<String, String> {
        use std::collections::HashMap;

        let mut env = HashMap::new();

        // Set INCLUDE path (only existing paths)
        let valid_includes = self.validated_include_paths();
        if !valid_includes.is_empty() {
            let include = valid_includes
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(";");
            env.insert("INCLUDE".to_string(), include);
        }

        // Set LIB path (only existing paths)
        let valid_libs = self.validated_lib_paths();
        if !valid_libs.is_empty() {
            let lib = valid_libs
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(";");
            env.insert("LIB".to_string(), lib);
        }

        // Prepend to PATH (only existing paths)
        let valid_bins = self.validated_bin_paths();
        if !valid_bins.is_empty() {
            let current_path = std::env::var("PATH").unwrap_or_default();
            let new_path = valid_bins
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .chain(std::iter::once(current_path))
                .collect::<Vec<_>>()
                .join(";");
            env.insert("PATH".to_string(), new_path);
        }

        // Set VCINSTALLDIR and VCToolsInstallDir for compatibility with tools
        // that use these variables to discover Visual Studio (e.g., node-gyp's
        // Find-VisualStudio.cs uses VCINSTALLDIR to detect VS Command Prompt)
        let vc_dir = self.install_path.join("VC");
        if vc_dir.exists() {
            // VCINSTALLDIR should end with a trailing backslash (VS convention)
            let vc_install_dir = format!("{}\\", vc_dir.to_string_lossy());
            env.insert("VCINSTALLDIR".to_string(), vc_install_dir);

            // VCToolsInstallDir points to the specific MSVC tools version directory
            let tools_dir = vc_dir.join("Tools").join("MSVC").join(&self.msvc_version);
            if tools_dir.exists() {
                let vc_tools_dir = format!("{}\\", tools_dir.to_string_lossy());
                env.insert("VCToolsInstallDir".to_string(), vc_tools_dir);
            }
        }

        env
    }
}
