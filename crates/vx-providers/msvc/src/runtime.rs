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
use msvc_kit::{Architecture, MsvcComponent};
use std::collections::{HashMap, HashSet};
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

    /// Detect Windows SDK version from MSVC installation info.
    ///
    /// Looks at include/lib paths to find the SDK version pattern like "10.0.22621.0".
    /// This is needed for node-gyp's `findVSFromSpecifiedLocation()` which reads
    /// `WindowsSDKVersion` to populate SDK package info.
    fn detect_windows_sdk_version(info: &MsvcInstallInfo) -> Option<String> {
        // First check if sdk_version is directly available
        if let Some(ref sdk_ver) = info.sdk_version {
            return Some(sdk_ver.clone());
        }

        // Try to extract from include/lib paths (even if paths don't exist on disk,
        // the version number embedded in the path is still valid)
        // Paths look like: C:\...\Windows Kits\10\Include\10.0.22621.0\ucrt
        for path in info.include_paths.iter().chain(info.lib_paths.iter()) {
            let path_str = path.to_string_lossy();
            if let Some(ver) = Self::extract_sdk_version_from_path(&path_str) {
                return Some(ver);
            }
        }

        // Fallback: scan standard Windows SDK locations
        for sdk_root in &[
            r"C:\Program Files (x86)\Windows Kits\10\Include",
            r"C:\Program Files\Windows Kits\10\Include",
        ] {
            let sdk_path = std::path::Path::new(sdk_root);
            if sdk_path.exists()
                && let Ok(entries) = std::fs::read_dir(sdk_path)
            {
                let mut versions: Vec<String> = entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| {
                        let name = e.file_name().to_string_lossy().to_string();
                        if name.starts_with("10.0.") {
                            Some(name)
                        } else {
                            None
                        }
                    })
                    .collect();
                // Use the latest SDK version
                versions.sort();
                if let Some(ver) = versions.last() {
                    return Some(ver.clone());
                }
            }
        }

        None
    }

    /// Extract SDK version (e.g., "10.0.22621.0") from a path string
    fn extract_sdk_version_from_path(path_str: &str) -> Option<String> {
        if let Some(pos) = path_str.find("Windows Kits") {
            let after = &path_str[pos..];
            for segment in after.split(['\\', '/']) {
                if segment.starts_with("10.0.")
                    && segment.chars().filter(|c| *c == '.').count() == 3
                {
                    return Some(segment.to_string());
                }
            }
        }
        None
    }

    /// Build the correct Windows SDK paths based on the detected version.
    ///
    /// msvc-kit may record incorrect SDK paths (e.g., pointing to user directory
    /// instead of Program Files). This method builds correct paths from the
    /// standard Windows SDK installation location.
    fn build_sdk_paths(sdk_version: &str, arch: &str) -> (Vec<String>, Vec<String>) {
        let mut include_paths = Vec::new();
        let mut lib_paths = Vec::new();

        for sdk_root in &[
            r"C:\Program Files (x86)\Windows Kits\10",
            r"C:\Program Files\Windows Kits\10",
        ] {
            let root = std::path::Path::new(sdk_root);
            let inc_base = root.join("Include").join(sdk_version);
            if inc_base.exists() {
                for subdir in &["ucrt", "shared", "um", "winrt", "cppwinrt"] {
                    let p = inc_base.join(subdir);
                    if p.exists() {
                        include_paths.push(p.to_string_lossy().to_string());
                    }
                }

                let lib_base = root.join("Lib").join(sdk_version);
                for subdir in &["ucrt", "um"] {
                    let p = lib_base.join(subdir).join(arch);
                    if p.exists() {
                        lib_paths.push(p.to_string_lossy().to_string());
                    }
                }
                break; // Found the SDK, no need to check other roots
            }
        }

        (include_paths, lib_paths)
    }

    /// Deploy MSBuild.exe bridge to the MSVC installation directory.
    ///
    /// node-gyp and other build tools expect `MSBuild.exe` at:
    /// `{install_path}/MSBuild/Current/Bin/MSBuild.exe`
    ///
    /// This bridge delegates to `dotnet msbuild`, enabling native Node.js
    /// addon compilation without a full Visual Studio installation.
    fn deploy_msbuild_bridge(&self, install_path: &Path) {
        let target = install_path
            .join("MSBuild")
            .join("Current")
            .join("Bin")
            .join("MSBuild.exe");

        match vx_bridge::deploy_bridge("MSBuild", &target) {
            Ok(path) => {
                info!("Deployed MSBuild bridge to {}", path.display());
            }
            Err(e) => {
                // Non-fatal: MSVC tools still work, just node-gyp won't find MSBuild
                warn!(
                    "Failed to deploy MSBuild bridge (node-gyp may not work): {}",
                    e
                );
            }
        }
    }

    /// Integrate vcpkg environment for native module builds
    ///
    /// This method detects if vcpkg is installed and adds its paths to the
    /// MSVC environment. This enables native Node.js modules (like node-pty)
    /// to find C++ libraries installed via vcpkg.
    ///
    /// # Environment Variables Added
    ///
    /// - `VCPKG_ROOT`: Path to vcpkg installation
    /// - `CMAKE_TOOLCHAIN_FILE`: Path to vcpkg.cmake (for CMake builds)
    /// - `VCPKG_DEFAULT_TRIPLET`: Default triplet (e.g., x64-windows)
    /// - Updated `INCLUDE`: Adds vcpkg include paths
    /// - Updated `LIB`: Adds vcpkg lib paths
    /// - Updated `PATH`: Adds vcpkg bin paths
    ///
    /// # Usage
    ///
    /// 1. Install vcpkg: `vx install vcpkg`
    /// 2. Install winpty: `vx vcpkg install winpty`
    /// 3. Build node-pty: `vx npm install node-pty`
    fn integrate_vcpkg_environment(env: &mut HashMap<String, String>, vx_home: &Path, arch: &str) {
        // Check for vcpkg in vx store — find the latest installed version dynamically
        let vcpkg_store_dir = vx_home.join("store").join("vcpkg");

        if !vcpkg_store_dir.exists() {
            debug!("vcpkg store directory not found, skipping vcpkg integration");
            return;
        }

        // Find the most recent vcpkg version directory
        let vcpkg_path = match std::fs::read_dir(&vcpkg_store_dir)
            .ok()
            .and_then(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .max_by_key(|e| e.file_name().to_string_lossy().to_string())
                    .map(|e| e.path())
            }) {
            Some(path) => path,
            None => {
                debug!("No vcpkg version installed, skipping vcpkg integration");
                return;
            }
        };

        if !vcpkg_path.exists() {
            debug!("vcpkg not installed, skipping vcpkg integration");
            return;
        }

        let vcpkg_exe = if cfg!(windows) {
            vcpkg_path.join("vcpkg.exe")
        } else {
            vcpkg_path.join("vcpkg")
        };

        if !vcpkg_exe.exists() {
            debug!("vcpkg executable not found, skipping integration");
            return;
        }

        info!("Integrating vcpkg environment for native module builds");

        // Determine triplet based on architecture
        let triplet = format!("{}-windows", arch);

        // Set VCPKG environment variables
        env.insert(
            "VCPKG_ROOT".to_string(),
            vcpkg_path.to_string_lossy().to_string(),
        );
        env.insert("VCPKG_DEFAULT_TRIPLET".to_string(), triplet.clone());

        // Set CMAKE_TOOLCHAIN_FILE
        let toolchain_file = vcpkg_path
            .join("scripts")
            .join("buildsystems")
            .join("vcpkg.cmake");
        if toolchain_file.exists() {
            env.insert(
                "CMAKE_TOOLCHAIN_FILE".to_string(),
                toolchain_file.to_string_lossy().to_string(),
            );
        }

        // Add vcpkg installed paths to INCLUDE/LIB/PATH
        let installed_dir = vcpkg_path.join("installed").join(&triplet);
        if installed_dir.exists() {
            // Add include path
            let include_dir = installed_dir.join("include");
            if include_dir.exists() {
                if let Some(existing) = env.get("INCLUDE") {
                    env.insert(
                        "INCLUDE".to_string(),
                        format!("{};{}", include_dir.to_string_lossy(), existing),
                    );
                } else {
                    env.insert(
                        "INCLUDE".to_string(),
                        include_dir.to_string_lossy().to_string(),
                    );
                }
            }

            // Add lib path
            let lib_dir = installed_dir.join("lib");
            if lib_dir.exists() {
                if let Some(existing) = env.get("LIB") {
                    env.insert(
                        "LIB".to_string(),
                        format!("{};{}", lib_dir.to_string_lossy(), existing),
                    );
                } else {
                    env.insert("LIB".to_string(), lib_dir.to_string_lossy().to_string());
                }
            }

            // Add bin path to PATH
            let bin_dir = installed_dir.join("bin");
            if bin_dir.exists() {
                if let Some(existing) = env.get("PATH") {
                    env.insert(
                        "PATH".to_string(),
                        format!("{};{}", bin_dir.to_string_lossy(), existing),
                    );
                } else {
                    env.insert("PATH".to_string(), bin_dir.to_string_lossy().to_string());
                }
            }
        }

        debug!("vcpkg environment integrated successfully");
    }

    /// Check which requested components are missing from the existing installation.
    ///
    /// For example, Spectre-mitigated libraries are installed under:
    /// `VC/Tools/MSVC/{version}/lib/{arch}/spectre/`
    ///
    /// Returns a list of component names that are requested but not found.
    fn check_missing_components(
        install_path: &Path,
        requested: &HashSet<MsvcComponent>,
        platform: &Platform,
    ) -> Vec<String> {
        if requested.is_empty() {
            return Vec::new();
        }

        let arch = platform.arch.as_str();
        let mut missing = Vec::new();

        // Find the actual MSVC version directory
        let tools_dir = install_path.join("VC").join("Tools").join("MSVC");
        let version_dir = if tools_dir.exists() {
            // Find the first (usually only) version subdirectory
            std::fs::read_dir(&tools_dir).ok().and_then(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .find(|e| e.path().is_dir())
                    .map(|e| e.path())
            })
        } else {
            None
        };

        for component in requested {
            let is_present = match component {
                MsvcComponent::Spectre => {
                    // Spectre libs: VC/Tools/MSVC/{ver}/lib/{arch}/spectre/
                    if let Some(ref ver_dir) = version_dir {
                        let spectre_dir = ver_dir.join("lib").join(arch).join("spectre");
                        spectre_dir.exists()
                            && std::fs::read_dir(&spectre_dir)
                                .map(|mut entries| entries.next().is_some())
                                .unwrap_or(false)
                    } else {
                        false
                    }
                }
                MsvcComponent::Mfc => {
                    // MFC: VC/Tools/MSVC/{ver}/atlmfc/
                    version_dir
                        .as_ref()
                        .is_some_and(|d| d.join("atlmfc").join("include").exists())
                }
                MsvcComponent::Atl => {
                    // ATL: same location as MFC (atlmfc/)
                    version_dir
                        .as_ref()
                        .is_some_and(|d| d.join("atlmfc").join("include").exists())
                }
                MsvcComponent::Asan => {
                    // ASAN: VC/Tools/MSVC/{ver}/lib/{arch}/clang_rt.asan*.lib
                    if let Some(ref ver_dir) = version_dir {
                        let lib_dir = ver_dir.join("lib").join(arch);
                        lib_dir.exists()
                            && std::fs::read_dir(&lib_dir)
                                .map(|entries| {
                                    entries.filter_map(|e| e.ok()).any(|e| {
                                        e.file_name().to_string_lossy().contains("clang_rt.asan")
                                    })
                                })
                                .unwrap_or(false)
                    } else {
                        false
                    }
                }
                // For other components, assume present (we can't easily check)
                _ => true,
            };

            if !is_present {
                missing.push(format!("{:?}", component));
            }
        }

        missing
    }

    /// Parse MSVC component names from `RuntimeContext.install_options` with env var fallback.
    ///
    /// Priority order:
    /// 1. `ctx.install_options["VX_MSVC_COMPONENTS"]` — set by Executor from vx.toml
    /// 2. `std::env::var("VX_MSVC_COMPONENTS")` — set by `vx sync` subprocess or user
    /// 3. `std::env::var("MSVC_KIT_INCLUDE_COMPONENTS")` — direct msvc-kit compat
    ///
    /// Valid values: spectre, mfc, atl, asan, uwp, cli, modules, redist, custom:<pattern>
    ///
    /// This can be set via vx.toml:
    /// ```toml
    /// [tools.msvc]
    /// version = "14.42"
    /// components = ["spectre"]
    /// ```
    ///
    /// Or via environment variable:
    /// ```bash
    /// VX_MSVC_COMPONENTS=spectre,asan vx install msvc
    /// ```
    fn parse_components(ctx: &RuntimeContext) -> HashSet<MsvcComponent> {
        let mut components = HashSet::new();

        // Priority 1: Read from RuntimeContext.install_options (set by Executor/sync)
        let components_str = ctx
            .get_install_option("VX_MSVC_COMPONENTS")
            .map(|s| s.to_string())
            // Priority 2: Fallback to environment variable
            .or_else(|| std::env::var("VX_MSVC_COMPONENTS").ok());

        if let Some(val) = components_str {
            for name in val.split(',') {
                let name = name.trim();
                if !name.is_empty() {
                    match name.parse::<MsvcComponent>() {
                        Ok(component) => {
                            components.insert(component);
                        }
                        Err(e) => {
                            warn!("Unknown MSVC component '{}': {}", name, e);
                        }
                    }
                }
            }
        }

        // Priority 3: Also check MSVC_KIT_INCLUDE_COMPONENTS for direct msvc-kit compatibility
        if let Ok(val) = std::env::var("MSVC_KIT_INCLUDE_COMPONENTS") {
            for name in val.split(',') {
                let name = name.trim();
                if !name.is_empty()
                    && let Ok(component) = name.parse::<MsvcComponent>()
                {
                    components.insert(component);
                }
            }
        }

        components
    }

    /// Parse exclude patterns from `RuntimeContext.install_options` with env var fallback.
    ///
    /// Priority order:
    /// 1. `ctx.install_options["VX_MSVC_EXCLUDE_PATTERNS"]`
    /// 2. `std::env::var("VX_MSVC_EXCLUDE_PATTERNS")`
    /// 3. `std::env::var("MSVC_KIT_EXCLUDE_PATTERNS")`
    fn parse_exclude_patterns(ctx: &RuntimeContext) -> Vec<String> {
        let mut patterns = Vec::new();

        // Priority 1: Read from RuntimeContext.install_options
        let patterns_str = ctx
            .get_install_option("VX_MSVC_EXCLUDE_PATTERNS")
            .map(|s| s.to_string())
            // Priority 2: Fallback to environment variable
            .or_else(|| std::env::var("VX_MSVC_EXCLUDE_PATTERNS").ok());

        if let Some(val) = patterns_str {
            for pattern in val.split(',') {
                let pattern = pattern.trim().to_string();
                if !pattern.is_empty() {
                    patterns.push(pattern);
                }
            }
        }

        // Priority 3: Also check MSVC_KIT_EXCLUDE_PATTERNS for direct msvc-kit compatibility
        if let Ok(val) = std::env::var("MSVC_KIT_EXCLUDE_PATTERNS") {
            for pattern in val.split(',') {
                let pattern = pattern.trim().to_string();
                if !pattern.is_empty() {
                    patterns.push(pattern);
                }
            }
        }

        patterns
    }
}

#[async_trait]
impl Runtime for MsvcRuntime {
    fn name(&self) -> &str {
        // Sourced from provider.star: `def name(): return "msvc"`
        crate::star_metadata().name_or("msvc")
    }

    fn description(&self) -> &str {
        // Sourced from provider.star: `def description(): return "..."`
        use std::sync::OnceLock;
        static DESC: OnceLock<&'static str> = OnceLock::new();
        DESC.get_or_init(|| {
            let s = crate::star_metadata()
                .description
                .as_deref()
                .unwrap_or("MSVC Build Tools - Microsoft Visual C++ compiler and tools for Windows development");
            Box::leak(s.to_string().into_boxed_str())
        })
    }

    fn aliases(&self) -> &[&str] {
        // Sourced from provider.star runtimes[0].aliases
        // Only expose non-conflicting aliases ("cl" and "nmake" are safe;
        // "link" and "lib" conflict with system tools on some platforms).
        use std::sync::OnceLock;
        static ALIASES: OnceLock<Vec<&'static str>> = OnceLock::new();
        ALIASES.get_or_init(|| {
            let meta = crate::star_metadata();
            // Use the primary "msvc" runtime's aliases from .star
            if let Some(rt) = meta
                .runtimes
                .iter()
                .find(|r| r.name.as_deref() == Some("msvc"))
            {
                rt.aliases
                    .iter()
                    .filter(|a| *a != "link" && *a != "lib") // avoid system tool conflicts
                    .map(|a| Box::leak(a.clone().into_boxed_str()) as &'static str)
                    .collect()
            } else {
                vec!["cl", "nmake"]
            }
        })
    }

    /// The primary executable is cl.exe (the C/C++ compiler)
    fn executable_name(&self) -> &str {
        // Sourced from provider.star runtimes[0].executable
        use std::sync::OnceLock;
        static EXE: OnceLock<&'static str> = OnceLock::new();
        EXE.get_or_init(|| {
            let meta = crate::star_metadata();
            if let Some(rt) = meta
                .runtimes
                .iter()
                .find(|r| r.name.as_deref() == Some("msvc"))
                && let Some(exe) = rt.executable.as_deref()
            {
                return Box::leak(exe.to_string().into_boxed_str());
            }
            "cl"
        })
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::System
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        // Sourced from provider.star: `def homepage(): return "..."`
        let star = crate::star_metadata();
        if let Some(hp) = star.homepage.as_deref() {
            meta.insert("homepage".to_string(), hp.to_string());
        } else {
            meta.insert(
                "homepage".to_string(),
                "https://visualstudio.microsoft.com/visual-cpp-build-tools/".to_string(),
            );
        }
        if let Some(repo) = star.repository.as_deref() {
            meta.insert("repository".to_string(), repo.to_string());
        }
        if let Some(license) = star.license.as_deref() {
            meta.insert("license".to_string(), license.to_string());
        }
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

        // Read component configuration from RuntimeContext or environment variables
        // Priority: ctx.install_options > env vars (set by vx sync or user)
        let include_components = Self::parse_components(ctx);

        // Check if already installed
        if install_path.exists() {
            let verification = self.verify_installation(version, &install_path, &platform);
            if verification.valid {
                // Check if requested components are actually present
                // e.g., Spectre libraries live in lib/{arch}/spectre/
                let missing_components =
                    Self::check_missing_components(&install_path, &include_components, &platform);

                if missing_components.is_empty() {
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

                // Components are missing — but have we already tried to install them?
                // Use a marker file to prevent infinite re-installation loops.
                // This happens when components like Spectre are requested in vx.toml
                // but msvc-kit cannot provide them (e.g., the VSIX packages don't
                // include Spectre libs for this version). Without this check, every
                // `vx npm ...` invocation would re-download and re-extract all 107+
                // MSVC packages only to find the components still missing.
                let component_attempt_marker = install_path.join(".component-install-attempted");
                if component_attempt_marker.exists() {
                    // We already tried installing these components — don't retry.
                    // The MSBuild bridge handles missing Spectre libs at build time
                    // by injecting /p:SpectreMitigation=false automatically.
                    let exe_path = verification
                        .executable_path
                        .unwrap_or_else(|| install_path.join("cl.exe"));
                    warn!(
                        "MSVC {} missing components {:?} but installation was already attempted. \
                         Skipping re-installation. The MSBuild bridge will handle missing Spectre \
                         libs at build time.",
                        version, missing_components
                    );
                    return Ok(InstallResult::already_installed(
                        install_path,
                        exe_path,
                        version.to_string(),
                    ));
                }

                // First attempt to install missing components.
                // msvc-kit uses `.done` marker files in `.msvc-kit-extracted/` to track
                // which VSIX packages have been extracted. If a previous installation was
                // interrupted after downloading but before extraction completed, these
                // markers may exist despite the actual files being absent.
                // We must remove ALL `.done` markers so msvc-kit re-extracts everything,
                // because the component files (e.g., spectre libs) are packed inside the
                // same VSIX archives as the base libraries.
                let marker_dir = install_path.join(".msvc-kit-extracted");
                if marker_dir.exists() {
                    info!(
                        "Cleaning extraction markers to force re-extraction of missing components: {:?}",
                        missing_components
                    );
                    if let Err(e) = std::fs::remove_dir_all(&marker_dir) {
                        warn!("Failed to remove extraction markers: {}", e);
                    }
                }

                info!(
                    "MSVC {} installed but missing components: {:?}. Re-installing to add them.",
                    version, missing_components
                );
            } else {
                // Don't clean up - msvc-kit will resume from cached downloads
                debug!("MSVC installation incomplete, will resume download");
            }
        }

        info!("Installing MSVC Build Tools version {}", version);

        // Determine architecture from current platform
        let arch = match platform.arch.as_str() {
            "aarch64" => Architecture::Arm64,
            "x86" => Architecture::X86,
            _ => Architecture::X64,
        };

        // Use msvc-kit installer with correct architecture
        let mut installer = MsvcInstaller::new(version)
            .with_arch(arch)
            .with_host_arch(arch);

        // Apply component configuration (already parsed above)
        if !include_components.is_empty() {
            info!(
                "Including optional MSVC components: {:?}",
                include_components
            );
            installer = installer.with_components(include_components);
        }

        let exclude_patterns = Self::parse_exclude_patterns(ctx);
        if !exclude_patterns.is_empty() {
            info!("Excluding MSVC packages matching: {:?}", exclude_patterns);
            installer = installer.with_exclude_patterns(exclude_patterns);
        }

        let install_info = installer
            .install(&install_path)
            .await
            .context("Failed to install MSVC Build Tools")?;

        // Save installation info for later use (environment variables, etc.)
        if let Err(e) = install_info.save() {
            warn!("Failed to save MSVC installation info: {}", e);
            // Don't fail the installation, just warn
        }

        // Write a marker indicating that we've attempted component installation.
        // This prevents infinite re-installation loops when requested components
        // (e.g., Spectre) are genuinely unavailable in the VSIX packages.
        let component_attempt_marker = install_path.join(".component-install-attempted");
        if let Err(e) = std::fs::write(&component_attempt_marker, version) {
            debug!("Failed to write component attempt marker: {}", e);
        }

        info!(
            "MSVC Build Tools {} installed successfully",
            install_info.msvc_version
        );

        // Deploy MSBuild.exe bridge for node-gyp compatibility
        // node-gyp expects MSBuild.exe at {VCINSTALLDIR}/MSBuild/Current/Bin/MSBuild.exe
        self.deploy_msbuild_bridge(&install_path);

        // Return the *requested* version (e.g., "14.42"), not the internal MSVC
        // version (e.g., "14.42.34433"). The requested version is what the unified
        // version resolver produced and what version_store_dir() uses for the
        // directory name. Returning the internal version would cause callers
        // (e.g., prepare_environment) to look up a non-existent directory.
        Ok(InstallResult::success(
            install_path,
            install_info.cl_exe_path,
            version.to_string(),
        ))
    }

    /// Prepare environment variables for MSVC compilation
    ///
    /// Sets both discovery variables (VCINSTALLDIR, GYP_MSVS_VERSION, etc.)
    /// and compilation variables (INCLUDE, LIB) so that:
    ///
    /// 1. **Detection tools** (node-gyp's find-visualstudio.js) can discover
    ///    our vx-managed MSVC via VCINSTALLDIR + VSCMD_VER + WindowsSDKVersion
    /// 2. **Compilers** (cl.exe invoked by node-gyp, cmake, etc.) can find
    ///    headers and libraries via INCLUDE/LIB environment variables
    ///
    /// node-gyp detection flow (find-visualstudio.js):
    /// - VCINSTALLDIR → envVcInstallDir = resolve(VCINSTALLDIR, '..')
    /// - VSCMD_VER → getVersionInfo() → versionYear (17.x = 2022)
    /// - WindowsSDKVersion → getSDK() → SDK package info
    /// - Checks MSBuild.exe at {envVcInstallDir}/MSBuild/Current/Bin/MSBuild.exe
    ///
    /// With all these variables properly set, findVSFromSpecifiedLocation()
    /// succeeds first, bypassing the PowerShell/C# detection methods entirely.
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

            // Ensure MSBuild.exe bridge is deployed (handles upgrades where
            // MSVC was installed before the bridge mechanism existed)
            let msbuild_path = install_path
                .join("MSBuild")
                .join("Current")
                .join("Bin")
                .join("MSBuild.exe");
            if !msbuild_path.exists() {
                self.deploy_msbuild_bridge(&install_path);
            }
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
            //
            // node-gyp detection flow (find-visualstudio.js):
            // 1. envVcInstallDir = path.resolve(VCINSTALLDIR, '..') → VS install root
            // 2. Uses VSCMD_VER to determine VS version year (17.x → 2022)
            // 3. Checks MSBuild.exe exists at {root}/MSBuild/Current/Bin/MSBuild.exe
            // 4. Uses WindowsSDKVersion to determine SDK packages
            // 5. checkConfigVersion verifies envVcInstallDir matches info.path
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
                // Must be parseable as major.minor where major 17 → VS 2022
                env.insert("VSCMD_VER".to_string(), "17.0".to_string());
            }

            // WindowsSDKVersion — node-gyp's findVSFromSpecifiedLocation() uses this
            // to populate SDK package info. Without it, getSDK() returns null and
            // the entire VS detection fails with "missing any Windows SDK".
            //
            // Format: "10.0.XXXXX.0\" (with trailing backslash, as set by vcvarsall.bat)
            let sdk_version = Self::detect_windows_sdk_version(&info);
            if let Some(ref sdk_ver) = sdk_version {
                env.insert("WindowsSDKVersion".to_string(), format!("{}\\", sdk_ver));
            }

            // Build INCLUDE/LIB/BIN paths for compilation.
            //
            // We dynamically construct paths from the actual installation rather than
            // relying solely on msvc-info.json, because:
            // 1. msvc-kit may record incorrect SDK paths (e.g., user dir instead of Program Files)
            // 2. Cached paths may become stale after SDK updates
            //
            // Path construction:
            // - MSVC headers: {install_path}/VC/Tools/MSVC/{version}/include
            // - MSVC libs: {install_path}/VC/Tools/MSVC/{version}/lib/{arch}
            // - MSVC bins: {install_path}/VC/Tools/MSVC/{version}/bin/Host{arch}/{arch}
            // - SDK headers: C:\Program Files (x86)\Windows Kits\10\Include\{sdk_ver}\{ucrt,shared,um,...}
            // - SDK libs: C:\Program Files (x86)\Windows Kits\10\Lib\{sdk_ver}\{ucrt,um}\{arch}
            // - SDK bins: C:\Program Files (x86)\Windows Kits\10\bin\{sdk_ver}\{arch}
            let arch = if cfg!(target_arch = "aarch64") {
                "arm64"
            } else {
                "x64"
            };

            let mut include_paths = Vec::new();
            let mut lib_paths = Vec::new();
            let mut bin_paths = Vec::new();

            // MSVC toolchain paths
            let tools_dir = install_path
                .join("VC")
                .join("Tools")
                .join("MSVC")
                .join(&info.msvc_version);
            if tools_dir.exists() {
                let inc = tools_dir.join("include");
                if inc.exists() {
                    include_paths.push(inc.to_string_lossy().to_string());
                }
                let lib = tools_dir.join("lib").join(arch);
                if lib.exists() {
                    lib_paths.push(lib.to_string_lossy().to_string());
                }
                let bin = tools_dir
                    .join("bin")
                    .join(format!("Host{}", arch))
                    .join(arch);
                if bin.exists() {
                    bin_paths.push(bin.to_string_lossy().to_string());
                }
            }

            // Windows SDK paths (dynamically detected from standard locations)
            if let Some(ref sdk_ver) = sdk_version {
                let (sdk_incs, sdk_libs) = Self::build_sdk_paths(sdk_ver, arch);
                include_paths.extend(sdk_incs);
                lib_paths.extend(sdk_libs);

                // SDK bin path
                for sdk_root in &[
                    r"C:\Program Files (x86)\Windows Kits\10",
                    r"C:\Program Files\Windows Kits\10",
                ] {
                    let bin = std::path::Path::new(sdk_root)
                        .join("bin")
                        .join(sdk_ver)
                        .join(arch);
                    if bin.exists() {
                        bin_paths.push(bin.to_string_lossy().to_string());
                        break;
                    }
                }
            }

            // Fallback: if we didn't find any paths dynamically, try validated cached paths
            if include_paths.is_empty() {
                for p in info.validated_include_paths() {
                    include_paths.push(p.to_string_lossy().to_string());
                }
            }
            if lib_paths.is_empty() {
                for p in info.validated_lib_paths() {
                    lib_paths.push(p.to_string_lossy().to_string());
                }
            }
            if bin_paths.is_empty() {
                for p in info.validated_bin_paths() {
                    bin_paths.push(p.to_string_lossy().to_string());
                }
            }

            if !include_paths.is_empty() {
                let include = include_paths.join(";");
                env.insert("VX_MSVC_INCLUDE".to_string(), include.clone());
                env.insert("INCLUDE".to_string(), include);
            }

            if !lib_paths.is_empty() {
                let lib = lib_paths.join(";");
                env.insert("VX_MSVC_LIB".to_string(), lib.clone());
                env.insert("LIB".to_string(), lib);
            }

            if !bin_paths.is_empty() {
                let bin = bin_paths.join(";");
                env.insert("VX_MSVC_BIN".to_string(), bin);
            }

            // Integrate vcpkg environment if available
            // This allows native Node.js modules (like node-pty) to find C++ libraries
            Self::integrate_vcpkg_environment(&mut env, &ctx.paths.vx_home(), arch);

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
