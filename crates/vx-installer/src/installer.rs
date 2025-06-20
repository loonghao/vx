//! Installation utilities and configuration

use crate::{
    downloader::Downloader,
    formats::ArchiveExtractor,
    progress::{ProgressContext, ProgressStyle},
    Error, Result,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Main installer for tools and packages
pub struct Installer {
    downloader: Downloader,
    extractor: ArchiveExtractor,
}

impl Installer {
    /// Create a new installer
    pub async fn new() -> Result<Self> {
        let downloader = Downloader::new()?;
        let extractor = ArchiveExtractor::new();

        Ok(Self {
            downloader,
            extractor,
        })
    }

    /// Install a tool using the provided configuration
    pub async fn install(&self, config: &InstallConfig) -> Result<PathBuf> {
        // Check if already installed and not forcing reinstall
        if !config.force && self.is_installed(config).await? {
            return Err(Error::AlreadyInstalled {
                tool_name: config.tool_name.clone(),
                version: config.version.clone(),
            });
        }

        // Execute pre-install hooks
        self.execute_lifecycle_actions(
            &config.lifecycle_hooks.pre_install,
            &config.install_dir,
            "pre-install",
        )
        .await?;

        // Create progress context
        let progress = ProgressContext::new(
            crate::progress::create_progress_reporter(ProgressStyle::default(), true),
            true,
        );

        let result = match &config.install_method {
            InstallMethod::Archive { format: _ } => {
                self.install_from_archive(config, &progress).await
            }
            InstallMethod::Binary => self.install_binary(config, &progress).await,
            InstallMethod::Script { url } => self.install_from_script(config, url, &progress).await,
            InstallMethod::PackageManager { manager, package } => {
                self.install_from_package_manager(config, manager, package, &progress)
                    .await
            }
            InstallMethod::Custom { method } => {
                self.install_custom(config, method, &progress).await
            }
        };

        // Execute post-install hooks if installation was successful
        if result.is_ok() {
            self.execute_lifecycle_actions(
                &config.lifecycle_hooks.post_install,
                &config.install_dir,
                "post-install",
            )
            .await?;
        }

        result
    }

    /// Check if a tool version is already installed
    pub async fn is_installed(&self, config: &InstallConfig) -> Result<bool> {
        let install_dir = &config.install_dir;

        // Check if installation directory exists and contains executables
        if !install_dir.exists() {
            return Ok(false);
        }

        // Look for executable files
        let bin_dir = install_dir.join("bin");
        if bin_dir.exists() {
            let exe_name = if cfg!(windows) {
                format!("{}.exe", config.tool_name)
            } else {
                config.tool_name.clone()
            };

            let exe_path = bin_dir.join(&exe_name);
            Ok(exe_path.exists() && exe_path.is_file())
        } else {
            // Check if there are any executable files in the install directory
            self.has_executables(install_dir)
        }
    }

    /// Uninstall a tool with lifecycle hooks
    pub async fn uninstall(&self, tool_name: &str, install_dir: &Path) -> Result<()> {
        self.uninstall_with_hooks(tool_name, install_dir, &LifecycleHooks::default())
            .await
    }

    /// Uninstall a tool with custom lifecycle hooks
    pub async fn uninstall_with_hooks(
        &self,
        tool_name: &str,
        install_dir: &Path,
        hooks: &LifecycleHooks,
    ) -> Result<()> {
        // Execute pre-uninstall hooks
        self.execute_lifecycle_actions(&hooks.pre_uninstall, install_dir, "pre-uninstall")
            .await?;

        // Perform the actual uninstallation
        if install_dir.exists() {
            std::fs::remove_dir_all(install_dir)?;
            tracing::info!("Removed installation directory: {}", install_dir.display());
        }

        // Execute post-uninstall hooks
        self.execute_lifecycle_actions(&hooks.post_uninstall, install_dir, "post-uninstall")
            .await?;

        tracing::info!("Successfully uninstalled {}", tool_name);
        Ok(())
    }

    /// Install from archive (ZIP, TAR, etc.)
    async fn install_from_archive(
        &self,
        config: &InstallConfig,
        progress: &ProgressContext,
    ) -> Result<PathBuf> {
        let download_url = config
            .download_url
            .as_ref()
            .ok_or_else(|| Error::InvalidConfig {
                message: "Download URL is required for archive installation".to_string(),
            })?;

        // Download the archive
        let temp_path = self
            .downloader
            .download_temp(download_url, progress)
            .await?;

        // Extract the archive
        let extracted_files = self
            .extractor
            .extract(&temp_path, &config.install_dir, progress)
            .await?;

        // Execute post-install actions (legacy support)
        #[allow(deprecated)]
        if !config.post_install_actions.is_empty() {
            self.execute_post_install_actions(config, &config.install_dir)
                .await?;
        }

        // Find the best executable
        let executable_path = self
            .extractor
            .find_best_executable(&extracted_files, &config.tool_name)?;

        // Clean up temporary file
        let _ = std::fs::remove_file(temp_path);

        Ok(executable_path)
    }

    /// Install binary file directly
    async fn install_binary(
        &self,
        config: &InstallConfig,
        progress: &ProgressContext,
    ) -> Result<PathBuf> {
        let download_url = config
            .download_url
            .as_ref()
            .ok_or_else(|| Error::InvalidConfig {
                message: "Download URL is required for binary installation".to_string(),
            })?;

        // Create bin directory
        let bin_dir = config.install_dir.join("bin");
        std::fs::create_dir_all(&bin_dir)?;

        // Determine executable name
        let exe_name = if cfg!(windows) {
            format!("{}.exe", config.tool_name)
        } else {
            config.tool_name.clone()
        };

        let exe_path = bin_dir.join(&exe_name);

        // Download directly to the target location
        self.downloader
            .download(download_url, &exe_path, progress)
            .await?;

        // Make executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(&exe_path)?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&exe_path, permissions)?;
        }

        Ok(exe_path)
    }

    /// Install from script
    async fn install_from_script(
        &self,
        _config: &InstallConfig,
        _script_url: &str,
        _progress: &ProgressContext,
    ) -> Result<PathBuf> {
        // TODO: Implement script-based installation
        Err(Error::unsupported_format("script installation"))
    }

    /// Install using package manager
    async fn install_from_package_manager(
        &self,
        _config: &InstallConfig,
        _manager: &str,
        _package: &str,
        _progress: &ProgressContext,
    ) -> Result<PathBuf> {
        // TODO: Implement package manager installation
        Err(Error::unsupported_format("package manager installation"))
    }

    /// Install using custom method
    async fn install_custom(
        &self,
        _config: &InstallConfig,
        _method: &str,
        _progress: &ProgressContext,
    ) -> Result<PathBuf> {
        // TODO: Implement custom installation methods
        Err(Error::unsupported_format("custom installation"))
    }

    /// Check if directory contains executable files
    fn has_executables(&self, dir: &Path) -> Result<bool> {
        if !dir.exists() {
            return Ok(false);
        }

        for entry in walkdir::WalkDir::new(dir).max_depth(3) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(metadata) = std::fs::metadata(path) {
                        let permissions = metadata.permissions();
                        if permissions.mode() & 0o111 != 0 {
                            return Ok(true);
                        }
                    }
                }

                #[cfg(windows)]
                {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if matches!(ext.to_lowercase().as_str(), "exe" | "bat" | "cmd" | "com") {
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    /// Execute lifecycle actions
    async fn execute_lifecycle_actions(
        &self,
        actions: &[LifecycleAction],
        install_dir: &Path,
        stage: &str,
    ) -> Result<()> {
        for action in actions {
            match action {
                LifecycleAction::FlattenDirectory { source_pattern } => {
                    self.flatten_directory(install_dir, source_pattern)?;
                }
                LifecycleAction::MoveFiles { from, to } => {
                    self.move_files(install_dir, from, to)?;
                }
                LifecycleAction::CopyFiles { from, to } => {
                    self.copy_files(install_dir, from, to)?;
                }
                LifecycleAction::CreateSymlink { target, link } => {
                    self.create_symlink(install_dir, target, link)?;
                }
                LifecycleAction::SetExecutable { path } => {
                    self.set_executable(install_dir, path)?;
                }
                LifecycleAction::RunCommand { command, args } => {
                    self.run_command(install_dir, command, args).await?;
                }
                LifecycleAction::CreateDirectory { path } => {
                    self.create_directory(install_dir, path)?;
                }
                LifecycleAction::RemoveFiles { pattern } => {
                    self.remove_files(install_dir, pattern)?;
                }
                LifecycleAction::DownloadFile { url, destination } => {
                    self.download_file(install_dir, url, destination).await?;
                }
                LifecycleAction::ValidateInstallation {
                    command,
                    expected_output,
                } => {
                    self.validate_installation(install_dir, command, expected_output.as_deref())
                        .await?;
                }
                LifecycleAction::SetEnvironment { key, value } => {
                    self.set_environment(key, value)?;
                }
                LifecycleAction::CreateConfig { path, content } => {
                    self.create_config(install_dir, path, content)?;
                }
                LifecycleAction::HealthCheck {
                    command,
                    expected_exit_code,
                } => {
                    self.health_check(install_dir, command, *expected_exit_code)
                        .await?;
                }
                LifecycleAction::CleanupTemp { pattern } => {
                    self.cleanup_temp(install_dir, pattern)?;
                }
                LifecycleAction::CreateShortcut {
                    name,
                    target,
                    description,
                } => {
                    self.create_shortcut(install_dir, name, target, description.as_deref())?;
                }
            }
        }

        if !actions.is_empty() {
            tracing::info!("Executed {} {} actions", actions.len(), stage);
        }

        Ok(())
    }

    /// Execute post-install actions (legacy support)
    #[allow(deprecated)]
    async fn execute_post_install_actions(
        &self,
        config: &InstallConfig,
        install_dir: &Path,
    ) -> Result<()> {
        self.execute_lifecycle_actions(&config.post_install_actions, install_dir, "post-install")
            .await
    }

    /// Flatten directory structure
    fn flatten_directory(&self, install_dir: &Path, source_pattern: &str) -> Result<()> {
        // Find directories matching the pattern
        for entry in walkdir::WalkDir::new(install_dir).max_depth(2) {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir()
                && path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .contains(source_pattern)
            {
                // Move all files from this directory to the parent
                for file_entry in std::fs::read_dir(path)? {
                    let file_entry = file_entry?;
                    let source = file_entry.path();
                    let dest = install_dir.join(file_entry.file_name());

                    if source.is_file() {
                        std::fs::rename(&source, &dest)?;
                    } else if source.is_dir() {
                        // Recursively move directory contents
                        self.move_directory_contents(&source, &dest)?;
                    }
                }

                // Remove the now-empty directory
                let _ = std::fs::remove_dir_all(path);
                break; // Only process the first match
            }
        }
        Ok(())
    }

    /// Move directory contents recursively
    #[allow(clippy::only_used_in_recursion)]
    fn move_directory_contents(&self, source: &Path, dest: &Path) -> Result<()> {
        std::fs::create_dir_all(dest)?;

        for entry in std::fs::read_dir(source)? {
            let entry = entry?;
            let source_path = entry.path();
            let dest_path = dest.join(entry.file_name());

            if source_path.is_file() {
                std::fs::rename(&source_path, &dest_path)?;
            } else if source_path.is_dir() {
                self.move_directory_contents(&source_path, &dest_path)?;
            }
        }

        Ok(())
    }

    /// Move files from source to destination
    fn move_files(&self, install_dir: &Path, from: &str, to: &str) -> Result<()> {
        let source = install_dir.join(from);
        let dest = install_dir.join(to);

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::rename(source, dest)?;
        Ok(())
    }

    /// Copy files from source to destination
    fn copy_files(&self, install_dir: &Path, from: &str, to: &str) -> Result<()> {
        let source = install_dir.join(from);
        let dest = install_dir.join(to);

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::copy(source, dest)?;
        Ok(())
    }

    /// Create symlink
    fn create_symlink(&self, install_dir: &Path, target: &str, link: &str) -> Result<()> {
        let target_path = install_dir.join(target);
        let link_path = install_dir.join(link);

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(target_path, link_path)?;
        }

        #[cfg(windows)]
        {
            if target_path.is_dir() {
                std::os::windows::fs::symlink_dir(target_path, link_path)?;
            } else {
                std::os::windows::fs::symlink_file(target_path, link_path)?;
            }
        }

        Ok(())
    }

    /// Set executable permissions
    fn set_executable(&self, install_dir: &Path, path: &str) -> Result<()> {
        let file_path = install_dir.join(path);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(&file_path)?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&file_path, permissions)?;
        }

        // On Windows, executable permissions are handled by file extension
        #[cfg(windows)]
        {
            let _ = file_path; // Suppress unused variable warning
        }

        Ok(())
    }

    /// Run a command
    async fn run_command(&self, install_dir: &Path, command: &str, args: &[String]) -> Result<()> {
        let mut cmd = tokio::process::Command::new(command);
        cmd.args(args).current_dir(install_dir);

        let output = cmd.output().await?;

        if !output.status.success() {
            return Err(Error::InvalidConfig {
                message: format!(
                    "Post-install command failed: {} {}",
                    command,
                    args.join(" ")
                ),
            });
        }

        Ok(())
    }

    /// Create directory
    pub fn create_directory(&self, install_dir: &Path, path: &str) -> Result<()> {
        let dir_path = install_dir.join(path);
        std::fs::create_dir_all(dir_path)?;
        Ok(())
    }

    /// Remove files matching pattern
    fn remove_files(&self, install_dir: &Path, pattern: &str) -> Result<()> {
        for entry in walkdir::WalkDir::new(install_dir) {
            let entry = entry?;
            let path = entry.path();

            if path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .contains(pattern)
            {
                if path.is_file() {
                    std::fs::remove_file(path)?;
                } else if path.is_dir() {
                    std::fs::remove_dir_all(path)?;
                }
            }
        }
        Ok(())
    }

    /// Download additional file
    async fn download_file(&self, install_dir: &Path, url: &str, destination: &str) -> Result<()> {
        let dest_path = install_dir.join(destination);

        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Create a simple progress context for the download
        let progress = ProgressContext::new(
            crate::progress::create_progress_reporter(ProgressStyle::default(), false),
            false,
        );

        self.downloader.download(url, &dest_path, &progress).await?;
        Ok(())
    }

    /// Validate installation by running a command
    async fn validate_installation(
        &self,
        install_dir: &Path,
        command: &str,
        expected_output: Option<&str>,
    ) -> Result<()> {
        // Parse command and arguments
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(Error::InvalidConfig {
                message: "Empty validation command".to_string(),
            });
        }

        let (cmd_name, args) = parts.split_first().unwrap();

        // Try to find the executable in the install directory first
        let exe_path = install_dir.join(cmd_name);
        let final_cmd = if exe_path.exists() {
            exe_path.to_string_lossy().to_string()
        } else {
            cmd_name.to_string()
        };

        let mut cmd = tokio::process::Command::new(&final_cmd);
        cmd.args(args);
        cmd.current_dir(install_dir);
        cmd.stdin(std::process::Stdio::null());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        // Add timeout to prevent hanging
        let output = tokio::time::timeout(std::time::Duration::from_secs(10), cmd.output()).await;

        let output = match output {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                return Err(Error::InvalidConfig {
                    message: format!("Validation command failed to execute: {}", e),
                })
            }
            Err(_) => {
                return Err(Error::InvalidConfig {
                    message: format!("Validation command timed out: {}", command),
                })
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::InvalidConfig {
                message: format!("Validation command failed: {}. Error: {}", command, stderr),
            });
        }

        if let Some(expected) = expected_output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.contains(expected) {
                return Err(Error::InvalidConfig {
                    message: format!(
                        "Validation failed: expected '{}' in output, got '{}'",
                        expected, stdout
                    ),
                });
            }
        }

        tracing::info!("Validation passed for command: {}", command);
        Ok(())
    }

    /// Set environment variable (note: this only affects the current process)
    fn set_environment(&self, key: &str, value: &str) -> Result<()> {
        std::env::set_var(key, value);
        tracing::info!("Set environment variable: {}={}", key, value);
        Ok(())
    }

    /// Create configuration file
    pub fn create_config(&self, install_dir: &Path, path: &str, content: &str) -> Result<()> {
        let config_path = install_dir.join(path);

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(config_path, content)?;
        Ok(())
    }

    /// Health check - verify tool is working correctly
    async fn health_check(
        &self,
        install_dir: &Path,
        command: &str,
        expected_exit_code: Option<i32>,
    ) -> Result<()> {
        let mut cmd = tokio::process::Command::new(command);
        cmd.current_dir(install_dir);
        cmd.stdin(std::process::Stdio::null()); // Prevent hanging on stdin
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        // Add timeout to prevent hanging
        let output = tokio::time::timeout(std::time::Duration::from_secs(30), cmd.output()).await;

        let output = match output {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                return Err(Error::InvalidConfig {
                    message: format!("Health check command failed to execute: {}", e),
                })
            }
            Err(_) => {
                return Err(Error::InvalidConfig {
                    message: format!("Health check command timed out: {}", command),
                })
            }
        };

        let actual_exit_code = output.status.code().unwrap_or(-1);
        let expected = expected_exit_code.unwrap_or(0);

        if actual_exit_code != expected {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::InvalidConfig {
                message: format!(
                    "Health check failed: command '{}' exited with code {}, expected {}. Error: {}",
                    command, actual_exit_code, expected, stderr
                ),
            });
        }

        tracing::info!("Health check passed for command: {}", command);
        Ok(())
    }

    /// Cleanup temporary files
    pub fn cleanup_temp(&self, install_dir: &Path, pattern: &str) -> Result<()> {
        for entry in walkdir::WalkDir::new(install_dir) {
            let entry = entry?;
            let path = entry.path();

            if path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .contains(pattern)
                && path.is_file()
            {
                let _ = std::fs::remove_file(path); // Ignore errors for cleanup
                tracing::debug!("Cleaned up temporary file: {}", path.display());
            }
        }
        Ok(())
    }

    /// Create desktop shortcuts (platform-specific)
    fn create_shortcut(
        &self,
        install_dir: &Path,
        name: &str,
        target: &str,
        description: Option<&str>,
    ) -> Result<()> {
        let target_path = install_dir.join(target);

        #[cfg(windows)]
        {
            self.create_windows_shortcut(name, &target_path, description)?;
        }

        #[cfg(target_os = "linux")]
        {
            self.create_linux_desktop_entry(name, &target_path, description)?;
        }

        #[cfg(target_os = "macos")]
        {
            // macOS doesn't typically use desktop shortcuts in the same way
            tracing::info!("Shortcut creation not implemented for macOS: {}", name);
        }

        Ok(())
    }

    #[cfg(windows)]
    fn create_windows_shortcut(
        &self,
        name: &str,
        target_path: &Path,
        description: Option<&str>,
    ) -> Result<()> {
        use std::process::Command;

        let desktop_path = dirs::desktop_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Desktop"));

        let shortcut_path = desktop_path.join(format!("{}.lnk", name));

        // Use PowerShell to create shortcut
        let ps_script = format!(
            r#"
            $WshShell = New-Object -comObject WScript.Shell
            $Shortcut = $WshShell.CreateShortcut("{}")
            $Shortcut.TargetPath = "{}"
            $Shortcut.Description = "{}"
            $Shortcut.Save()
            "#,
            shortcut_path.display(),
            target_path.display(),
            description.unwrap_or(name)
        );

        let output = Command::new("powershell")
            .args(["-Command", &ps_script])
            .output()?;

        if !output.status.success() {
            return Err(Error::InvalidConfig {
                message: format!("Failed to create Windows shortcut: {}", name),
            });
        }

        tracing::info!("Created Windows shortcut: {}", shortcut_path.display());
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn create_linux_desktop_entry(
        &self,
        name: &str,
        target_path: &Path,
        description: Option<&str>,
    ) -> Result<()> {
        let desktop_path = dirs::desktop_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Desktop"));

        let entry_path = desktop_path.join(format!("{}.desktop", name));

        let desktop_entry = format!(
            r#"[Desktop Entry]
Version=1.0
Type=Application
Name={}
Comment={}
Exec={}
Terminal=false
"#,
            name,
            description.unwrap_or(name),
            target_path.display()
        );

        std::fs::write(&entry_path, desktop_entry)?;

        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(&entry_path)?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&entry_path, permissions)?;
        }

        tracing::info!("Created Linux desktop entry: {}", entry_path.display());
        Ok(())
    }
}

/// Configuration for tool installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallConfig {
    /// Name of the tool to install
    pub tool_name: String,

    /// Version to install
    pub version: String,

    /// Installation method
    pub install_method: InstallMethod,

    /// Download URL (if applicable)
    pub download_url: Option<String>,

    /// Installation directory
    pub install_dir: PathBuf,

    /// Whether to force reinstallation
    pub force: bool,

    /// Checksum for verification
    pub checksum: Option<String>,

    /// Additional configuration
    pub metadata: HashMap<String, String>,

    /// Lifecycle hooks for various operations
    pub lifecycle_hooks: LifecycleHooks,

    /// Post-installation actions (deprecated, use lifecycle_hooks.post_install)
    #[deprecated(note = "Use lifecycle_hooks.post_install instead")]
    pub post_install_actions: Vec<PostInstallAction>,
}

/// Different methods for installing tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallMethod {
    /// Download and extract archive
    Archive { format: ArchiveFormat },

    /// Use system package manager
    PackageManager { manager: String, package: String },

    /// Run installation script
    Script { url: String },

    /// Download single binary
    Binary,

    /// Custom installation method
    Custom { method: String },
}

/// Supported archive formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchiveFormat {
    Zip,
    TarGz,
    TarXz,
    TarBz2,
    SevenZip,
}

/// Lifecycle actions that can be executed at different stages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleAction {
    /// Move files from source to destination
    MoveFiles { from: String, to: String },
    /// Copy files from source to destination
    CopyFiles { from: String, to: String },
    /// Flatten directory structure (move all files from subdirectory to parent)
    FlattenDirectory { source_pattern: String },
    /// Create symlinks
    CreateSymlink { target: String, link: String },
    /// Set executable permissions (Unix only)
    SetExecutable { path: String },
    /// Run a custom command
    RunCommand { command: String, args: Vec<String> },
    /// Create directory
    CreateDirectory { path: String },
    /// Remove files or directories
    RemoveFiles { pattern: String },
    /// Download additional files
    DownloadFile { url: String, destination: String },
    /// Validate installation
    ValidateInstallation {
        command: String,
        expected_output: Option<String>,
    },
    /// Set environment variables
    SetEnvironment { key: String, value: String },
    /// Create configuration files
    CreateConfig { path: String, content: String },
    /// Health check - verify tool is working correctly
    HealthCheck {
        command: String,
        expected_exit_code: Option<i32>,
    },
    /// Cleanup temporary files
    CleanupTemp { pattern: String },
    /// Create desktop shortcuts (Windows/Linux)
    CreateShortcut {
        name: String,
        target: String,
        description: Option<String>,
    },
}

/// Lifecycle hooks for different operations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LifecycleHooks {
    /// Actions to run before installation starts
    pub pre_install: Vec<LifecycleAction>,
    /// Actions to run after installation completes
    pub post_install: Vec<LifecycleAction>,
    /// Actions to run before uninstallation starts
    pub pre_uninstall: Vec<LifecycleAction>,
    /// Actions to run after uninstallation completes
    pub post_uninstall: Vec<LifecycleAction>,
    /// Actions to run before update starts
    pub pre_update: Vec<LifecycleAction>,
    /// Actions to run after update completes
    pub post_update: Vec<LifecycleAction>,
    /// Actions to run before version switch
    pub pre_switch: Vec<LifecycleAction>,
    /// Actions to run after version switch
    pub post_switch: Vec<LifecycleAction>,
}

/// Legacy type alias for backward compatibility
pub type PostInstallAction = LifecycleAction;

/// Builder for InstallConfig
pub struct InstallConfigBuilder {
    config: InstallConfig,
}

impl Default for InstallConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl InstallConfigBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: InstallConfig {
                tool_name: String::new(),
                version: String::new(),
                install_method: InstallMethod::Binary,
                download_url: None,
                install_dir: PathBuf::new(),
                force: false,
                checksum: None,
                metadata: HashMap::new(),
                lifecycle_hooks: LifecycleHooks::default(),
                #[allow(deprecated)]
                post_install_actions: Vec::new(),
            },
        }
    }

    /// Set the tool name
    pub fn tool_name(mut self, name: impl Into<String>) -> Self {
        self.config.tool_name = name.into();
        self
    }

    /// Set the version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.config.version = version.into();
        self
    }

    /// Set the installation method
    pub fn install_method(mut self, method: InstallMethod) -> Self {
        self.config.install_method = method;
        self
    }

    /// Set the download URL
    pub fn download_url(mut self, url: impl Into<String>) -> Self {
        self.config.download_url = Some(url.into());
        self
    }

    /// Set the installation directory
    pub fn install_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.config.install_dir = dir.into();
        self
    }

    /// Set force reinstallation
    pub fn force(mut self, force: bool) -> Self {
        self.config.force = force;
        self
    }

    /// Set checksum
    pub fn checksum(mut self, checksum: impl Into<String>) -> Self {
        self.config.checksum = Some(checksum.into());
        self
    }

    /// Add metadata
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.metadata.insert(key.into(), value.into());
        self
    }

    /// Set lifecycle hooks
    pub fn lifecycle_hooks(mut self, hooks: LifecycleHooks) -> Self {
        self.config.lifecycle_hooks = hooks;
        self
    }

    /// Add pre-install action
    pub fn pre_install_action(mut self, action: LifecycleAction) -> Self {
        self.config.lifecycle_hooks.pre_install.push(action);
        self
    }

    /// Add post-install action
    pub fn post_install_action(mut self, action: LifecycleAction) -> Self {
        self.config.lifecycle_hooks.post_install.push(action);
        self
    }

    /// Add pre-uninstall action
    pub fn pre_uninstall_action(mut self, action: LifecycleAction) -> Self {
        self.config.lifecycle_hooks.pre_uninstall.push(action);
        self
    }

    /// Add post-uninstall action
    pub fn post_uninstall_action(mut self, action: LifecycleAction) -> Self {
        self.config.lifecycle_hooks.post_uninstall.push(action);
        self
    }

    /// Add pre-update action
    pub fn pre_update_action(mut self, action: LifecycleAction) -> Self {
        self.config.lifecycle_hooks.pre_update.push(action);
        self
    }

    /// Add post-update action
    pub fn post_update_action(mut self, action: LifecycleAction) -> Self {
        self.config.lifecycle_hooks.post_update.push(action);
        self
    }

    /// Add pre-switch action
    pub fn pre_switch_action(mut self, action: LifecycleAction) -> Self {
        self.config.lifecycle_hooks.pre_switch.push(action);
        self
    }

    /// Add post-switch action
    pub fn post_switch_action(mut self, action: LifecycleAction) -> Self {
        self.config.lifecycle_hooks.post_switch.push(action);
        self
    }

    /// Add multiple post-install actions (deprecated)
    #[deprecated(note = "Use post_install_action or lifecycle_hooks instead")]
    pub fn post_install_actions(mut self, actions: Vec<PostInstallAction>) -> Self {
        #[allow(deprecated)]
        self.config.post_install_actions.extend(actions);
        self
    }

    /// Build the configuration
    pub fn build(self) -> InstallConfig {
        self.config
    }
}

impl InstallConfig {
    /// Create a new builder
    pub fn builder() -> InstallConfigBuilder {
        InstallConfigBuilder::new()
    }
}
