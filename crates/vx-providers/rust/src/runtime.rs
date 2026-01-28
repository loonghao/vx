//! Rust runtime implementations
//!
//! Rust is installed via rustup, the official Rust toolchain installer.
//! rustup manages rustc, cargo, and other Rust tools automatically.
//!
//! rustup-init binary is downloaded from static.rust-lang.org and then
//! executed to install the Rust toolchain.

use crate::config::RustupUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use tracing::{info, warn};
use vx_runtime::{
    Arch, Ecosystem, GitHubReleaseOptions, Os, Platform, Runtime, RuntimeContext,
    VerificationResult, VersionInfo,
};

/// Rustup runtime - The Rust toolchain installer
#[derive(Debug, Clone, Default)]
pub struct RustupRuntime;

impl RustupRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for RustupRuntime {
    fn name(&self) -> &str {
        "rustup"
    }

    fn description(&self) -> &str {
        "The Rust toolchain installer"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Rust
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://rustup.rs/".to_string());
        meta.insert("category".to_string(), "toolchain-manager".to_string());
        meta
    }

    fn supported_platforms(&self) -> Vec<Platform> {
        vec![
            Platform::new(Os::Windows, Arch::X86_64),
            Platform::new(Os::Windows, Arch::X86),
            Platform::new(Os::MacOS, Arch::X86_64),
            Platform::new(Os::MacOS, Arch::Aarch64),
            Platform::new(Os::Linux, Arch::X86_64),
            Platform::new(Os::Linux, Arch::Aarch64),
        ]
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        // After installation, rustup is renamed from rustup-init
        RustupUrlBuilder::get_final_executable_name(platform).to_string()
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        ctx.fetch_github_releases(
            "rustup",
            "rust-lang",
            "rustup",
            GitHubReleaseOptions::new()
                .strip_v_prefix(false)
                .skip_prereleases(true),
        )
        .await
    }

    /// Download URL for rustup-init binary
    async fn download_url(&self, _version: &str, platform: &Platform) -> Result<Option<String>> {
        // Note: rustup downloads are not versioned - always downloads latest from the URL
        // The version parameter is used for tracking but the URL gives latest
        Ok(RustupUrlBuilder::download_url(platform))
    }

    /// Post-extract: rename rustup-init to rustup and run it to install toolchain
    fn post_extract(&self, _version: &str, install_path: &std::path::PathBuf) -> Result<()> {
        use std::io::{BufRead, BufReader};
        use std::process::{Command, Stdio};

        let platform = Platform::current();
        let init_name = RustupUrlBuilder::get_executable_name(&platform);
        let final_name = RustupUrlBuilder::get_final_executable_name(&platform);

        let init_path = install_path.join(init_name);
        let final_path = install_path.join(final_name);

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if init_path.exists() {
                let mut perms = std::fs::metadata(&init_path)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&init_path, perms)?;
            }
        }

        info!("⏳ Installing Rust toolchain...");

        // Set RUSTUP_HOME and CARGO_HOME to use vx's managed directories (no leading dots)
        let rustup_home = install_path.join("rustup");
        let cargo_home = install_path.join("cargo");

        // Run rustup-init with -y for non-interactive install
        // --no-modify-path to avoid modifying user's shell profile
        let mut child = Command::new(&init_path)
            .arg("-y")
            .arg("--no-modify-path")
            .arg("--default-toolchain")
            .arg("stable")
            .env("RUSTUP_HOME", &rustup_home)
            .env("CARGO_HOME", &cargo_home)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Read stdout and stderr to show progress
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stderr"))?;

        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        // Spawn threads to read and display output
        let stdout_handle = std::thread::spawn(move || {
            for line in stdout_reader.lines().map_while(Result::ok) {
                if !line.contains("info: downloading component")
                    && !line.contains("info: installing component")
                    && !line.trim().is_empty()
                {
                    tracing::debug!("  {}", line);
                }
            }
        });

        let stderr_handle = std::thread::spawn(move || {
            for line in stderr_reader.lines().map_while(Result::ok) {
                if line.contains("error:") || line.contains("warning:") {
                    tracing::warn!("  {}", line);
                } else if !line.trim().is_empty() {
                    tracing::debug!("  {}", line);
                }
            }
        });

        // Wait for the process to complete
        let status = child.wait()?;

        // Wait for threads to finish
        stdout_handle
            .join()
            .map_err(|e| anyhow::anyhow!("Stdout thread failed: {:?}", e))?;
        stderr_handle
            .join()
            .map_err(|e| anyhow::anyhow!("Stderr thread failed: {:?}", e))?;

        if !status.success() {
            return Err(anyhow::anyhow!(
                "rustup-init failed with exit code: {:?}",
                status.code()
            ));
        }

        // Rename rustup-init to rustup for future use
        if init_path.exists() && init_name != final_name {
            std::fs::rename(&init_path, &final_path)?;
        }

        info!("✓ Rust toolchain installed successfully");
        Ok(())
    }

    fn verify_installation(
        &self,
        _version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_name = RustupUrlBuilder::get_final_executable_name(platform);
        let exe_path = install_path.join(exe_name);

        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            // Check in cargo/bin as well (where rustup installs itself)
            // The cargo directory is a sibling of the install path
            if let Some(store_dir) = install_path.parent() {
                let cargo_bin = store_dir.join("cargo").join("bin").join(exe_name);
                if cargo_bin.exists() {
                    return VerificationResult::success(cargo_bin);
                }
            }

            VerificationResult::failure(
                vec![format!("rustup not found at {}", exe_path.display())],
                vec!["Try reinstalling rustup".to_string()],
            )
        }
    }
}

/// Cargo runtime - Rust package manager (provided by rustup)
#[derive(Debug, Clone, Default)]
pub struct CargoRuntime;

impl CargoRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for CargoRuntime {
    fn name(&self) -> &str {
        "cargo"
    }

    fn description(&self) -> &str {
        "Rust package manager and build tool"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Rust
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn dependencies(&self) -> &[vx_runtime::RuntimeDependency] {
        use once_cell::sync::Lazy;
        use vx_runtime::RuntimeDependency;

        static DEPS: Lazy<Vec<RuntimeDependency>> = Lazy::new(|| {
            vec![RuntimeDependency {
                name: "rustup".to_string(),
                version_req: None,
                min_version: None,
                max_version: None,
                recommended_version: Some("latest".to_string()),
                optional: false,
                reason: Some("cargo is installed via rustup".to_string()),
            }]
        });

        &DEPS
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://doc.rust-lang.org/cargo/".to_string(),
        );
        meta.insert("category".to_string(), "package-manager".to_string());
        meta
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Cargo version is tied to rustc version
        // Use the same channel-based versioning as RustcRuntime
        let channel_versions = vec![
            VersionInfo::new("stable").with_metadata("channel".to_string(), "stable".to_string()),
            VersionInfo::new("beta").with_metadata("channel".to_string(), "beta".to_string()),
            VersionInfo::new("nightly").with_metadata("channel".to_string(), "nightly".to_string()),
        ];

        Ok(channel_versions)
    }

    /// Resolve version - accept any version (passthrough to rustup)
    async fn resolve_version(&self, version: &str, ctx: &RuntimeContext) -> Result<String> {
        let versions = self.fetch_versions(ctx).await?;

        // Check if it's a known channel
        if versions.iter().any(|v| v.version == version) {
            return Ok(version.to_string());
        }

        // For any other version string, pass it through to rustup
        Ok(version.to_string())
    }

    // Cargo is provided by rustup, not direct download
    async fn download_url(&self, _version: &str, _platform: &Platform) -> Result<Option<String>> {
        Ok(None)
    }

    /// Install cargo via rustup
    async fn install(
        &self,
        version: &str,
        ctx: &RuntimeContext,
    ) -> Result<vx_runtime::InstallResult> {
        use vx_runtime::InstallResult;

        // Find rustup executable
        let rustup_exe = Self::find_rustup_executable(ctx)?;
        let rustup_store_dir = ctx.paths.runtime_store_dir("rustup");

        // Set up vx-managed paths (no leading dots)
        let rustup_home = rustup_store_dir.join("rustup");
        let cargo_home = rustup_store_dir.join("cargo");

        // Install path
        let install_path = rustup_store_dir.join("cargo");

        // Check if the requested version is already the default toolchain
        if Self::is_current_toolchain(&rustup_exe, version, &rustup_home, &cargo_home) {
            eprintln!("✓ Rust toolchain {} is already the default", version);
            return Ok(InstallResult::already_installed(
                install_path.clone(),
                install_path.clone(),
                version.to_string(),
            ));
        }

        // Check if the toolchain is installed but not default
        if Self::check_rustup_toolchain_exists(&rustup_exe, version, &rustup_home, &cargo_home) {
            eprintln!("📦 Switching to Rust toolchain {}...", version);
            Self::switch_toolchain(&rustup_exe, version, &rustup_home, &cargo_home)?;
            return Ok(InstallResult::success(
                install_path.clone(),
                install_path,
                version.to_string(),
            ));
        }

        // Install the toolchain and set as default
        self.post_install(version, ctx).await?;

        Ok(InstallResult::success(
            install_path.clone(),
            install_path,
            version.to_string(),
        ))
    }

    /// Post-install: use rustup to install the specified toolchain version
    async fn post_install(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
        use std::io::{BufRead, BufReader};
        use std::process::{Command, Stdio};

        let rustup_exe = Self::find_rustup_executable(ctx)?;
        let rustup_store_dir = ctx.paths.runtime_store_dir("rustup");

        info!("⏳ Installing Rust toolchain {}...", version);

        // Set CARGO_HOME to store/cargo (managed by vx, no leading dot)
        // Set RUSTUP_HOME to store/rustup (managed by vx, no leading dot)
        let cargo_home = rustup_store_dir.join("cargo");
        let rustup_home = rustup_store_dir.join("rustup");

        // Run: rustup toolchain install <version>
        let mut child = Command::new(&rustup_exe)
            .arg("toolchain")
            .arg("install")
            .arg(version)
            .env("RUSTUP_HOME", &rustup_home)
            .env("CARGO_HOME", &cargo_home)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Read stdout and stderr to show progress
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stderr"))?;

        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        // Spawn threads to read and display output
        let stdout_handle = std::thread::spawn(move || {
            for line in stdout_reader.lines().map_while(Result::ok) {
                // Filter out common rustup messages
                if !line.contains("info: downloading component")
                    && !line.contains("info: installing component")
                    && !line.trim().is_empty()
                {
                    tracing::debug!("  {}", line);
                }
            }
        });

        let stderr_handle = std::thread::spawn(move || {
            for line in stderr_reader.lines().map_while(Result::ok) {
                if line.contains("error:") || line.contains("warning:") {
                    tracing::warn!("  {}", line);
                } else if !line.trim().is_empty() {
                    tracing::debug!("  {}", line);
                }
            }
        });

        // Wait for the process to complete
        let status = child.wait()?;

        // Wait for threads to finish
        stdout_handle
            .join()
            .map_err(|e| anyhow::anyhow!("Stdout thread failed: {:?}", e))?;
        stderr_handle
            .join()
            .map_err(|e| anyhow::anyhow!("Stderr thread failed: {:?}", e))?;

        if !status.success() {
            return Err(anyhow::anyhow!(
                "Failed to install Rust toolchain {} with exit code: {:?}",
                version,
                status.code()
            ));
        }

        // Set the installed version as default
        let output = Command::new(&rustup_exe)
            .arg("default")
            .arg(version)
            .env("RUSTUP_HOME", &rustup_home)
            .env("CARGO_HOME", &cargo_home)
            .output()?;

        if !output.status.success() {
            warn!("⚠️ Warning: Failed to set {} as default toolchain", version);
        }

        info!("✓ Rust toolchain {} installed successfully", version);
        Ok(())
    }

    fn verify_installation(
        &self,
        _version: &str,
        _install_path: &Path,
        _platform: &Platform,
    ) -> VerificationResult {
        // Cargo is installed via rustup in vx-managed directories
        // Check if it exists in the cargo bin directory
        // Note: We need to check in multiple possible locations since we don't have RuntimeContext here
        // 1. ~/.vx/store/rustup/cargo/bin/cargo (or cargo.exe)
        // 2. ~/.vx/store/rustup/cargo/bin/cargo.exe (Windows)

        // Since we can't access RuntimeContext in verify_installation, we'll check common paths
        // This is a limitation of the current trait design
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Failed to get home directory"));

        if let Ok(home) = home_dir {
            let vx_home = home.join(".vx");
            let cargo_home = vx_home.join("store").join("rustup").join("cargo");
            let cargo_bin = cargo_home.join("bin");
            let exe_name = if cfg!(windows) { "cargo.exe" } else { "cargo" };

            if cargo_bin.join(exe_name).exists() {
                return VerificationResult::success(cargo_bin.join(exe_name));
            }
        }

        VerificationResult::failure(
            vec!["cargo not found in vx-managed directories".to_string()],
            vec![
                "Ensure rustup has installed the toolchain".to_string(),
                "Run 'vx install cargo' to install".to_string(),
            ],
        )
    }

    /// Execute cargo with proper environment variables
    async fn execute(
        &self,
        args: &[String],
        ctx: &vx_runtime::ExecutionContext,
    ) -> Result<vx_runtime::ExecutionResult> {
        use std::process::Command;

        // Find cargo executable in vx-managed directories
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?;
        let vx_home = home_dir.join(".vx");
        let cargo_home = vx_home.join("store").join("rustup").join("cargo");
        let rustup_home = vx_home.join("store").join("rustup").join("rustup");
        let cargo_bin = cargo_home.join("bin");
        let exe_name = if cfg!(windows) { "cargo.exe" } else { "cargo" };
        let cargo_exe = cargo_bin.join(exe_name);

        if !cargo_exe.exists() {
            return Err(anyhow::anyhow!(
                "cargo not found at {}. Please install it first using 'vx install cargo'",
                cargo_exe.display()
            ));
        }

        let mut cmd = Command::new(&cargo_exe);
        cmd.args(args)
            .env("RUSTUP_HOME", &rustup_home)
            .env("CARGO_HOME", &cargo_home);

        // Add additional environment variables from context
        for (key, value) in &ctx.env {
            cmd.env(key, value);
        }

        // Set working directory if specified
        if let Some(working_dir) = &ctx.working_dir {
            cmd.current_dir(working_dir);
        }

        // Capture output if requested
        let output = if ctx.capture_output {
            cmd.output()?
        } else {
            cmd.spawn()?.wait_with_output()?
        };

        Ok(vx_runtime::ExecutionResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: Some(String::from_utf8_lossy(&output.stdout).to_string()),
            stderr: Some(String::from_utf8_lossy(&output.stderr).to_string()),
        })
    }
}

impl CargoRuntime {
    /// Find rustup executable
    pub fn find_rustup_executable(ctx: &RuntimeContext) -> Result<std::path::PathBuf> {
        let rustup_store_dir = ctx.paths.runtime_store_dir("rustup");
        let exe_name = if cfg!(windows) {
            "rustup.exe"
        } else {
            "rustup"
        };

        // Try to find rustup in the rustup store directory
        // Rustup can be installed in multiple locations:
        // 1. ~/.vx/store/rustup/<version>/rustup.exe (or rustup on Unix)
        // 2. ~/.vx/store/rustup/cargo/bin/rustup.exe (where rustup-init installs itself)
        // 3. System PATH

        // First, try all subdirectories in rustup store
        if let Ok(entries) = std::fs::read_dir(&rustup_store_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                // Try direct path in version directory
                let direct_path = path.join(exe_name);
                if direct_path.exists() {
                    eprintln!("✓ Found rustup at: {}", direct_path.display());
                    return Ok(direct_path);
                }
            }
        }

        // Try cargo/bin directory (where rustup-init installs itself)
        let cargo_bin = rustup_store_dir.join("cargo").join("bin").join(exe_name);
        if cargo_bin.exists() {
            eprintln!("✓ Found rustup at: {}", cargo_bin.display());
            return Ok(cargo_bin);
        }

        // Fallback to system PATH
        if let Ok(system_exe) = which::which(exe_name) {
            eprintln!("✓ Using system rustup at: {}", system_exe.display());
            return Ok(system_exe);
        }

        Err(anyhow::anyhow!(
            "rustup not found. Please install rustup first using 'vx install rustup'"
        ))
    }

    /// Check if rustup has the specified toolchain installed
    pub fn check_rustup_toolchain_exists(
        rustup_exe: &std::path::PathBuf,
        version: &str,
        rustup_home: &std::path::Path,
        cargo_home: &std::path::Path,
    ) -> bool {
        use std::process::Command;

        let output = Command::new(rustup_exe)
            .arg("toolchain")
            .arg("list")
            .env("RUSTUP_HOME", rustup_home)
            .env("CARGO_HOME", cargo_home)
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.lines().any(|line| line.contains(version))
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Check if the specified version is the current default toolchain
    pub fn is_current_toolchain(
        rustup_exe: &std::path::PathBuf,
        version: &str,
        rustup_home: &std::path::Path,
        cargo_home: &std::path::Path,
    ) -> bool {
        use std::process::Command;

        let output = Command::new(rustup_exe)
            .arg("default")
            .env("RUSTUP_HOME", rustup_home)
            .env("CARGO_HOME", cargo_home)
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                // rustup default outputs e.g., "stable-x86_64-pc-windows-msvc (default)"
                // We need to check if it contains our version (e.g., "1.83-x86_64-pc-windows-msvc")
                stdout.contains(&format!("{}-", version))
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Switch to the specified toolchain
    pub fn switch_toolchain(
        rustup_exe: &std::path::PathBuf,
        version: &str,
        rustup_home: &std::path::Path,
        cargo_home: &std::path::Path,
    ) -> Result<()> {
        use std::process::Command;

        let output = Command::new(rustup_exe)
            .arg("default")
            .arg(version)
            .env("RUSTUP_HOME", rustup_home)
            .env("CARGO_HOME", cargo_home)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(anyhow::anyhow!(
                "Failed to switch to toolchain {}\nstdout: {}\nstderr: {}",
                version,
                stdout,
                stderr
            ));
        }

        eprintln!("✓ Switched to Rust toolchain {}", version);
        Ok(())
    }
}

/// Rustc runtime - The Rust compiler (provided by rustup)
#[derive(Debug, Clone, Default)]
pub struct RustcRuntime;

impl RustcRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for RustcRuntime {
    fn name(&self) -> &str {
        "rustc"
    }

    fn description(&self) -> &str {
        "The Rust compiler"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Rust
    }

    fn aliases(&self) -> &[&str] {
        &["rust"]
    }

    fn dependencies(&self) -> &[vx_runtime::RuntimeDependency] {
        use once_cell::sync::Lazy;
        use vx_runtime::RuntimeDependency;

        static DEPS: Lazy<Vec<RuntimeDependency>> = Lazy::new(|| {
            vec![RuntimeDependency {
                name: "rustup".to_string(),
                version_req: None,
                min_version: None,
                max_version: None,
                recommended_version: Some("latest".to_string()),
                optional: false,
                reason: Some("rustc is installed via rustup".to_string()),
            }]
        });

        &DEPS
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://www.rust-lang.org/".to_string(),
        );
        meta.insert("category".to_string(), "compiler".to_string());
        meta
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Rust is installed via rustup, which uses channels (stable, beta, nightly)
        // or specific version numbers. Since rustup handles version management,
        // we expose channels as "versions" and mark them as passthrough.
        //
        // Note: rust-lang/rust doesn't have GitHub releases, so we can't fetch
        // actual version numbers. Users can specify exact versions in their
        // rust-toolchain.toml or vx.toml (e.g., "1.83.0"), and rustup will
        // handle the installation.
        //
        // The "passthrough" metadata indicates that the version solver should
        // accept any user-specified version without validation.
        let channel_versions = vec![
            VersionInfo::new("stable")
                .with_metadata("channel".to_string(), "stable".to_string())
                .with_metadata("passthrough".to_string(), "true".to_string()),
            VersionInfo::new("beta")
                .with_metadata("channel".to_string(), "beta".to_string())
                .with_metadata("passthrough".to_string(), "true".to_string()),
            VersionInfo::new("nightly")
                .with_metadata("channel".to_string(), "nightly".to_string())
                .with_metadata("passthrough".to_string(), "true".to_string()),
        ];

        Ok(channel_versions)
    }

    /// Resolve version - accept any version (passthrough to rustup)
    async fn resolve_version(&self, version: &str, ctx: &RuntimeContext) -> Result<String> {
        let versions = self.fetch_versions(ctx).await?;

        // Check if it's a known channel
        if versions.iter().any(|v| v.version == version) {
            return Ok(version.to_string());
        }

        // For any other version string, pass it through to rustup
        // rustup will validate and install the version
        Ok(version.to_string())
    }

    // Rustc is provided by rustup, not direct download
    async fn download_url(&self, _version: &str, _platform: &Platform) -> Result<Option<String>> {
        Ok(None)
    }

    /// Install rustc via rustup
    async fn install(
        &self,
        version: &str,
        ctx: &RuntimeContext,
    ) -> Result<vx_runtime::InstallResult> {
        use vx_runtime::InstallResult;

        // Rustc versions are managed by rustup toolchains
        // We use a single install path for all rustc versions
        let rustup_store_dir = ctx.paths.runtime_store_dir("rustup");
        let install_path = rustup_store_dir.join("rustc");

        // Find rustup executable
        let rustup_exe = CargoRuntime::find_rustup_executable(ctx)?;

        // Set up vx-managed paths (no leading dots)
        let rustup_home = rustup_store_dir.join("rustup");
        let cargo_home = rustup_store_dir.join("cargo");

        // Check if the requested version is already the default toolchain
        if CargoRuntime::is_current_toolchain(&rustup_exe, version, &rustup_home, &cargo_home) {
            eprintln!("✓ Rust toolchain {} is already the default", version);
            return Ok(InstallResult::already_installed(
                install_path.clone(),
                install_path.clone(),
                version.to_string(),
            ));
        }

        // Check if the toolchain is installed but not default
        if CargoRuntime::check_rustup_toolchain_exists(
            &rustup_exe,
            version,
            &rustup_home,
            &cargo_home,
        ) {
            eprintln!("📦 Switching to Rust toolchain {}...", version);
            CargoRuntime::switch_toolchain(&rustup_exe, version, &rustup_home, &cargo_home)?;
            return Ok(InstallResult::success(
                install_path.clone(),
                install_path,
                version.to_string(),
            ));
        }

        // Install the toolchain and set as default
        self.post_install(version, ctx).await?;

        Ok(InstallResult::success(
            install_path.clone(),
            install_path,
            version.to_string(),
        ))
    }

    /// Post-install: use rustup to install the specified toolchain version
    async fn post_install(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
        use std::io::{BufRead, BufReader};
        use std::process::{Command, Stdio};

        // Find rustup executable
        // rustup could be in several locations:
        // 1. ~/.vx/store/rustup/<version>/rustup (or rustup.exe on Windows)
        // 2. ~/.vx/store/rustup/cargo/bin/rustup (where rustup-init installs itself)
        // 3. System PATH (if installed system-wide)

        let rustup_store_dir = ctx.paths.runtime_store_dir("rustup");

        // Try to find the latest installed rustup version
        let rustup_exe = if let Ok(versions) = std::fs::read_dir(&rustup_store_dir) {
            let mut found_exe = None;

            for version_entry in versions.flatten() {
                if !version_entry.path().is_dir() {
                    continue;
                }

                // Check direct path
                let exe_name = if cfg!(windows) {
                    "rustup.exe"
                } else {
                    "rustup"
                };
                let direct_path = version_entry.path().join(exe_name);
                if direct_path.exists() {
                    found_exe = Some(direct_path);
                    break;
                }
            }

            if found_exe.is_none() {
                // Check cargo/bin directory (where rustup-init installs itself)
                let exe_name = if cfg!(windows) {
                    "rustup.exe"
                } else {
                    "rustup"
                };
                let cargo_bin_path = rustup_store_dir.join("cargo").join("bin").join(exe_name);
                if cargo_bin_path.exists() {
                    found_exe = Some(cargo_bin_path);
                }
            }

            found_exe
        } else {
            None
        };

        let rustup_exe = if let Some(exe) = rustup_exe {
            exe
        } else {
            // Try system PATH as fallback
            let exe_name = if cfg!(windows) {
                "rustup.exe"
            } else {
                "rustup"
            };
            which::which(exe_name).map_err(|_| {
                anyhow::anyhow!(
                    "rustup not found. Please install rustup first using 'vx install rustup'"
                )
            })?
        };

        info!("⏳ Installing Rust toolchain {}...", version);

        // Always set RUSTUP_HOME and CARGO_HOME to use vx's managed directories (no leading dots)
        let rustup_home = rustup_store_dir.join("rustup");
        let cargo_home = rustup_store_dir.join("cargo");

        // Run: rustup toolchain install <version>
        let mut child = Command::new(&rustup_exe)
            .arg("toolchain")
            .arg("install")
            .arg(version)
            .env("RUSTUP_HOME", &rustup_home)
            .env("CARGO_HOME", &cargo_home)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Read stdout and stderr to show progress
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stderr"))?;

        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        // Spawn threads to read and display output
        let stdout_handle = std::thread::spawn(move || {
            for line in stdout_reader.lines().map_while(Result::ok) {
                if !line.contains("info: downloading component")
                    && !line.contains("info: installing component")
                    && !line.trim().is_empty()
                {
                    tracing::debug!("  {}", line);
                }
            }
        });

        let stderr_handle = std::thread::spawn(move || {
            for line in stderr_reader.lines().map_while(Result::ok) {
                if line.contains("error:") || line.contains("warning:") {
                    tracing::warn!("  {}", line);
                } else if !line.trim().is_empty() {
                    tracing::debug!("  {}", line);
                }
            }
        });

        // Wait for the process to complete
        let status = child.wait()?;

        // Wait for threads to finish
        stdout_handle
            .join()
            .map_err(|e| anyhow::anyhow!("Stdout thread failed: {:?}", e))?;
        stderr_handle
            .join()
            .map_err(|e| anyhow::anyhow!("Stderr thread failed: {:?}", e))?;

        if !status.success() {
            return Err(anyhow::anyhow!(
                "Failed to install Rust toolchain {} with exit code: {:?}",
                version,
                status.code()
            ));
        }

        // Set the installed version as default
        let output = Command::new(&rustup_exe)
            .arg("default")
            .arg(version)
            .env("RUSTUP_HOME", &rustup_home)
            .env("CARGO_HOME", &cargo_home)
            .output()?;

        if !output.status.success() {
            warn!("⚠️ Warning: Failed to set {} as default toolchain", version);
        }

        info!("✓ Rust toolchain {} installed successfully", version);
        Ok(())
    }
    fn verify_installation(
        &self,
        _version: &str,
        _install_path: &Path,
        _platform: &Platform,
    ) -> VerificationResult {
        // Rustc is installed via rustup in vx-managed directories
        // Check if it exists in the cargo bin directory
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Failed to get home directory"));

        if let Ok(home) = home_dir {
            let vx_home = home.join(".vx");
            let cargo_home = vx_home.join("store").join("rustup").join("cargo");
            let cargo_bin = cargo_home.join("bin");
            let exe_name = if cfg!(windows) { "rustc.exe" } else { "rustc" };

            if cargo_bin.join(exe_name).exists() {
                return VerificationResult::success(cargo_bin.join(exe_name));
            }
        }

        VerificationResult::failure(
            vec!["rustc not found in vx-managed directories".to_string()],
            vec![
                "Ensure rustup has installed the toolchain".to_string(),
                "Run 'vx install rustc' to install".to_string(),
            ],
        )
    }

    /// Execute rustc with proper environment variables
    async fn execute(
        &self,
        args: &[String],
        ctx: &vx_runtime::ExecutionContext,
    ) -> Result<vx_runtime::ExecutionResult> {
        use std::process::Command;

        // Find rustc executable in vx-managed directories
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?;
        let vx_home = home_dir.join(".vx");
        let cargo_home = vx_home.join("store").join("rustup").join("cargo");
        let rustup_home = vx_home.join("store").join("rustup").join("rustup");
        let cargo_bin = cargo_home.join("bin");
        let exe_name = if cfg!(windows) { "rustc.exe" } else { "rustc" };
        let rustc_exe = cargo_bin.join(exe_name);

        if !rustc_exe.exists() {
            return Err(anyhow::anyhow!(
                "rustc not found at {}. Please install it first using 'vx install rustc'",
                rustc_exe.display()
            ));
        }

        let mut cmd = Command::new(&rustc_exe);
        cmd.args(args)
            .env("RUSTUP_HOME", &rustup_home)
            .env("CARGO_HOME", &cargo_home);

        // Add additional environment variables from context
        for (key, value) in &ctx.env {
            cmd.env(key, value);
        }

        // Set working directory if specified
        if let Some(working_dir) = &ctx.working_dir {
            cmd.current_dir(working_dir);
        }

        // Capture output if requested
        let output = if ctx.capture_output {
            cmd.output()?
        } else {
            cmd.spawn()?.wait_with_output()?
        };

        Ok(vx_runtime::ExecutionResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: Some(String::from_utf8_lossy(&output.stdout).to_string()),
            stderr: Some(String::from_utf8_lossy(&output.stderr).to_string()),
        })
    }
}
