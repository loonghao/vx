//! MSVC Build Tools runtime implementation
//!
//! MSVC Build Tools provides the Visual Studio C/C++ compiler and tools
//! for building native Windows applications.
//!
//! This implementation uses msvc-kit for downloading and installing
//! MSVC Build Tools from Microsoft's official servers.

use crate::installer::{MsvcInstallInfo, MsvcInstaller};
use anyhow::{Context, Result};
use async_trait::async_trait;
use msvc_kit::Architecture;
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};
use vx_runtime::{
    Ecosystem, InstallResult, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo,
};

/// MSVC Build Tools runtime implementation
#[derive(Debug, Clone, Default)]
pub struct MsvcRuntime;

impl MsvcRuntime {
    /// Create a new MSVC runtime
    pub fn new() -> Self {
        Self
    }

    /// Get known stable MSVC versions
    fn get_known_versions(&self) -> Vec<VersionInfo> {
        vec![
            VersionInfo::new("14.42").with_lts(true),  // VS 2022 17.12
            VersionInfo::new("14.41").with_lts(true),  // VS 2022 17.11
            VersionInfo::new("14.40").with_lts(true),  // VS 2022 17.10
            VersionInfo::new("14.39").with_lts(true),  // VS 2022 17.9
            VersionInfo::new("14.38").with_lts(true),  // VS 2022 17.8
            VersionInfo::new("14.37").with_lts(true),  // VS 2022 17.7
            VersionInfo::new("14.36").with_lts(true),  // VS 2022 17.6
            VersionInfo::new("14.35").with_lts(true),  // VS 2022 17.5
            VersionInfo::new("14.34").with_lts(true),  // VS 2022 17.4
            VersionInfo::new("14.29").with_lts(false), // VS 2019 16.11
        ]
    }

    /// Load MSVC installation info for a given version
    fn load_install_info(&self, ctx: &RuntimeContext, version: &str) -> Option<MsvcInstallInfo> {
        let install_path = ctx.paths.version_store_dir(self.name(), version);
        match MsvcInstallInfo::load(&install_path) {
            Ok(Some(info)) => Some(info),
            Ok(None) => {
                debug!("No MSVC install info found for version {}", version);
                None
            }
            Err(e) => {
                warn!("Failed to load MSVC install info for {}: {}", version, e);
                None
            }
        }
    }
}

#[async_trait]
impl Runtime for MsvcRuntime {
    fn name(&self) -> &str {
        "msvc"
    }

    fn description(&self) -> &str {
        "MSVC Build Tools - Microsoft Visual C++ compiler and tools for Windows development"
    }

    fn aliases(&self) -> &[&str] {
        // Only expose non-conflicting aliases
        // "cl" and "nmake" are safe, but "link" and "lib" conflict with system tools
        &["cl", "nmake"]
    }

    /// The primary executable is cl.exe (the C/C++ compiler)
    fn executable_name(&self) -> &str {
        "cl"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://visualstudio.microsoft.com/visual-cpp-build-tools/".to_string(),
        );
        meta.insert(
            "documentation".to_string(),
            "https://docs.microsoft.com/en-us/cpp/build/".to_string(),
        );
        meta.insert("category".to_string(), "build-tools".to_string());
        meta.insert("vendor".to_string(), "Microsoft".to_string());
        meta
    }

    /// MSVC Build Tools only supports Windows
    fn supported_platforms(&self) -> Vec<Platform> {
        Platform::windows_only()
    }

    fn executable_relative_path(&self, version: &str, platform: &Platform) -> String {
        // platform.arch.as_str() returns "arm64", "x86", "x64" etc.
        let arch = platform.arch.as_str();
        format!("VC/Tools/MSVC/{}/bin/Host{}/{}/cl.exe", version, arch, arch)
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Return known stable versions
        // In the future, we could query msvc-kit for available versions
        Ok(self.get_known_versions())
    }

    async fn download_url(&self, _version: &str, platform: &Platform) -> Result<Option<String>> {
        // MSVC doesn't have a single download URL - we use msvc-kit for installation
        if !self.is_platform_supported(platform) {
            return Ok(None);
        }
        // Return None to indicate we use custom installation
        Ok(None)
    }

    /// Custom installation for MSVC Build Tools using msvc-kit
    async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        let install_path = ctx.paths.version_store_dir(self.name(), version);
        let platform = Platform::current();

        // Check platform support
        if !self.is_platform_supported(&platform) {
            return Err(anyhow::anyhow!(
                "MSVC Build Tools is only available for Windows"
            ));
        }

        // Check if already installed
        if install_path.exists() {
            let verification = self.verify_installation(version, &install_path, &platform);
            if verification.valid {
                let exe_path = verification
                    .executable_path
                    .unwrap_or_else(|| install_path.join("cl.exe"));
                debug!("MSVC {} already installed: {}", version, exe_path.display());
                return Ok(InstallResult::already_installed(
                    install_path,
                    exe_path,
                    version.to_string(),
                ));
            }
            // Don't clean up - msvc-kit will resume from cached downloads
            debug!("MSVC installation incomplete, will resume download");
        }

        info!("Installing MSVC Build Tools version {}", version);

        // Determine architecture from current platform
        let arch = match platform.arch.as_str() {
            "aarch64" => Architecture::Arm64,
            "x86" => Architecture::X86,
            _ => Architecture::X64,
        };

        // Use msvc-kit installer with correct architecture
        let installer = MsvcInstaller::new(version)
            .with_arch(arch)
            .with_host_arch(arch);
        let install_info = installer
            .install(&install_path)
            .await
            .context("Failed to install MSVC Build Tools")?;

        // Save installation info for later use (environment variables, etc.)
        if let Err(e) = install_info.save() {
            warn!("Failed to save MSVC installation info: {}", e);
            // Don't fail the installation, just warn
        }

        info!(
            "MSVC Build Tools {} installed successfully",
            install_info.msvc_version
        );

        Ok(InstallResult::success(
            install_path,
            install_info.cl_exe_path,
            install_info.msvc_version,
        ))
    }

    /// Prepare environment variables for MSVC compilation
    ///
    /// Instead of setting LIB/INCLUDE/PATH globally (which conflicts with tools
    /// like node-gyp that have their own Visual Studio discovery logic), we only
    /// set VX_MSVC_* marker variables. The actual LIB/INCLUDE/PATH are only
    /// injected when directly invoking MSVC tools (cl, link, nmake, etc.)
    /// via the `execution_environment()` method.
    ///
    /// Additionally, we set VCINSTALLDIR, VCToolsInstallDir, and GYP_MSVS_VERSION
    /// for compatibility with tools like node-gyp that use these variables to
    /// discover Visual Studio installations.
    ///
    /// See: https://github.com/loonghao/vx/issues/573
    async fn prepare_environment(
        &self,
        version: &str,
        ctx: &RuntimeContext,
    ) -> Result<HashMap<String, String>> {
        let mut env = HashMap::new();

        // Try to load saved installation info
        if let Some(info) = self.load_install_info(ctx, version) {
            // For prepare_environment(), we only need cl.exe to exist (is_valid()).
            // We don't require include/lib/SDK paths to be valid because this method
            // only sets discovery/marker variables (VCINSTALLDIR, VCToolsInstallDir, etc.),
            // not the full compilation environment (INCLUDE/LIB/PATH).
            //
            // The full validate_paths() check (which also validates include/lib paths)
            // is only enforced in execution_environment() where those paths are actually needed.
            //
            // This is important because Windows SDK paths may become stale (e.g., SDK
            // uninstalled/upgraded) while the MSVC toolchain itself is still valid.
            if !info.is_valid() {
                warn!(
                    "MSVC {} cl.exe not found at cached path. \
                     Environment variables will not be set. Try reinstalling: vx install msvc@{}",
                    version, version
                );
                return Ok(env);
            }

            debug!(
                "Loaded MSVC {} environment: {} include paths, {} lib paths, {} bin paths",
                version,
                info.include_paths.len(),
                info.lib_paths.len(),
                info.bin_paths.len()
            );

            // Only set VX_MSVC_* marker variables for discovery by other tools
            // This avoids polluting the global environment with LIB/INCLUDE which
            // breaks node-gyp's PowerShell-based Visual Studio discovery
            let install_path = ctx.paths.version_store_dir(self.name(), version);
            env.insert(
                "VX_MSVC_ROOT".to_string(),
                install_path.to_string_lossy().to_string(),
            );
            env.insert("VX_MSVC_VERSION".to_string(), version.to_string());
            env.insert(
                "VX_MSVC_FULL_VERSION".to_string(),
                info.msvc_version.clone(),
            );

            // Set Visual Studio discovery variables that node-gyp and other tools understand
            env.insert("MSVS_VERSION".to_string(), "2022".to_string());
            env.insert("GYP_MSVS_VERSION".to_string(), "2022".to_string());

            // Set VCINSTALLDIR — node-gyp's findVisualStudio2019OrNewerFromSpecifiedLocation()
            // uses this to detect that we're running in a VS Developer Command Prompt-like
            // environment. This allows node-gyp to find the vx-managed MSVC without
            // needing the full LIB/INCLUDE variables that would break its C# compiler.
            let vc_dir = install_path.join("VC");
            if vc_dir.exists() {
                // VCINSTALLDIR must end with trailing backslash (VS convention)
                let vc_install_dir = format!("{}\\", vc_dir.to_string_lossy());
                env.insert("VCINSTALLDIR".to_string(), vc_install_dir);

                // VCToolsInstallDir points to the exact version's tools directory
                let tools_dir = vc_dir.join("Tools").join("MSVC").join(&info.msvc_version);
                if tools_dir.exists() {
                    let vc_tools_dir = format!("{}\\", tools_dir.to_string_lossy());
                    env.insert("VCToolsInstallDir".to_string(), vc_tools_dir);
                }

                // VSCMD_VER indicates VS Command Prompt version
                env.insert("VSCMD_VER".to_string(), "17.0".to_string());
            }

            // Set INCLUDE paths as VX_MSVC_INCLUDE (not INCLUDE)
            let valid_includes = info.validated_include_paths();
            if !valid_includes.is_empty() {
                let include = valid_includes
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join(";");
                env.insert("VX_MSVC_INCLUDE".to_string(), include);
            }

            // Set LIB paths as VX_MSVC_LIB (not LIB)
            let valid_libs = info.validated_lib_paths();
            if !valid_libs.is_empty() {
                let lib = valid_libs
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join(";");
                env.insert("VX_MSVC_LIB".to_string(), lib);
            }

            // Set BIN paths as VX_MSVC_BIN (not PATH)
            let valid_bins = info.validated_bin_paths();
            if !valid_bins.is_empty() {
                let bin = valid_bins
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join(";");
                env.insert("VX_MSVC_BIN".to_string(), bin);
            }

            return Ok(env);
        }

        // If no saved info, return empty environment
        debug!(
            "No MSVC installation info found for {}, environment variables not set",
            version
        );
        Ok(env)
    }

    /// Prepare execution-specific environment for MSVC tools
    ///
    /// When directly invoking MSVC tools (cl, link, nmake, lib, ml64),
    /// we inject the full LIB/INCLUDE/PATH environment variables needed
    /// for compilation. This is only called for the actual MSVC executable.
    ///
    /// Paths are validated to ensure they exist on disk, preventing stale
    /// cached paths from breaking compilation.
    ///
    /// See: https://github.com/loonghao/vx/issues/573
    async fn execution_environment(
        &self,
        version: &str,
        ctx: &RuntimeContext,
    ) -> Result<HashMap<String, String>> {
        // For direct MSVC tool invocation, use the full environment
        if let Some(info) = self.load_install_info(ctx, version) {
            // Validate paths before injection
            if !info.validate_paths() {
                warn!(
                    "MSVC {} cached paths are stale. Some LIB/INCLUDE paths may not exist. \
                     Consider reinstalling: vx install msvc@{}",
                    version, version
                );
                // Still return the environment, but get_environment() will filter
                // out non-existent paths automatically
            }
            debug!("Setting full MSVC execution environment for direct tool invocation");
            return Ok(info.get_environment());
        }
        Ok(HashMap::new())
    }

    fn verify_installation(
        &self,
        version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        // MSVC Build Tools only supports Windows
        if !self.is_platform_supported(platform) {
            return VerificationResult::failure(
                vec!["MSVC Build Tools is only available for Windows".to_string()],
                vec!["Use a Windows system to install MSVC Build Tools".to_string()],
            );
        }

        // platform.arch.as_str() returns "arm64", "x86", "x64" etc.
        let arch = platform.arch.as_str();

        // Check for cl.exe in expected locations
        let expected_paths = [
            // Standard MSVC layout
            install_path.join(format!(
                "VC/Tools/MSVC/{}/bin/Host{}/{}/cl.exe",
                version, arch, arch
            )),
            // msvc-kit layout
            install_path.join(format!("bin/Host{}/{}/cl.exe", arch, arch)),
            // Direct layout
            install_path.join("cl.exe"),
        ];

        for path in &expected_paths {
            if path.exists() {
                return VerificationResult::success(path.clone());
            }
        }

        // Search for cl.exe in the installation directory
        if let Some(entry) = walkdir::WalkDir::new(install_path)
            .max_depth(10)
            .into_iter()
            .filter_map(|e| e.ok())
            .find(|e| e.file_name() == "cl.exe")
        {
            return VerificationResult::success(entry.path().to_path_buf());
        }

        VerificationResult::failure(
            vec![format!(
                "MSVC compiler (cl.exe) not found in {}",
                install_path.display()
            )],
            vec![
                "Try reinstalling MSVC Build Tools: vx install msvc".to_string(),
                "Ensure the installation completed successfully".to_string(),
            ],
        )
    }
}
