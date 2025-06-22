//! Python tool implementations using Python Build Standalone

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use vx_installer::Installer;
use vx_plugin::{StandardTool, ToolContext, ToolExecutionResult, VxTool};
use vx_version::{TurboCdnVersionFetcher, VersionFetcher, VersionInfo};

/// Python tool using Python Build Standalone
#[derive(Debug, Clone)]
pub struct PythonTool {
    version_fetcher: Option<TurboCdnVersionFetcher>,
}

impl PythonTool {
    /// Create a new Python tool
    pub fn new() -> Self {
        Self {
            version_fetcher: None,
        }
    }

    /// Initialize the tool with turbo-cdn support
    pub async fn init() -> Result<Self> {
        let version_fetcher =
            TurboCdnVersionFetcher::new("astral-sh", "python-build-standalone").await?;
        Ok(Self {
            version_fetcher: Some(version_fetcher),
        })
    }

    /// Get or initialize the version fetcher
    async fn get_version_fetcher(&self) -> Result<TurboCdnVersionFetcher> {
        match &self.version_fetcher {
            Some(fetcher) => Ok(fetcher.clone()),
            None => {
                // Create a new fetcher if not initialized
                TurboCdnVersionFetcher::new("astral-sh", "python-build-standalone")
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to create TurboCdnVersionFetcher: {}", e))
            }
        }
    }
}

impl Default for PythonTool {
    fn default() -> Self {
        Self::new()
    }
}

// Override specific methods for Python-specific behavior
#[async_trait]
impl VxTool for PythonTool {
    fn name(&self) -> &str {
        "python"
    }

    fn description(&self) -> &str {
        "Python programming language using Python Build Standalone"
    }

    async fn fetch_versions(&self, use_cache: bool) -> Result<Vec<VersionInfo>, anyhow::Error> {
        let fetcher = self.get_version_fetcher().await?;
        fetcher
            .fetch_versions(use_cache)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch versions: {}", e))
    }

    async fn install_version(&self, version: &str, force: bool) -> Result<(), anyhow::Error> {
        // Resolve "latest" to actual build date
        let build_date = if version == "latest" {
            let versions = self.fetch_versions(true).await?;
            if let Some(latest) = versions.first() {
                latest.version.clone()
            } else {
                return Err(anyhow::anyhow!("No versions available for Python"));
            }
        } else {
            version.to_string()
        };

        // Get the actual Python version for this build date
        let python_version =
            crate::config::PythonUrlBuilder::get_python_version_for_build(&build_date)?;

        let path_manager = vx_paths::PathManager::new().unwrap_or_default();
        // Use the actual Python version for the directory structure
        let install_dir = path_manager.tool_version_dir(self.name(), &python_version);

        // Create config with build date for download URL, but install to Python version directory
        let config = crate::config::create_install_config(&build_date, install_dir, force);
        let installer = Installer::new()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create installer: {}", e))?;
        installer
            .install(&config)
            .await
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("Installation failed: {}", e))
    }

    async fn is_version_installed(&self, version: &str) -> Result<bool, anyhow::Error> {
        let path_manager = vx_paths::PathManager::new().unwrap_or_default();

        // If this is a build date, convert to Python version
        let check_version = if version.len() == 8 && version.chars().all(|c| c.is_ascii_digit()) {
            // This looks like a build date (YYYYMMDD)
            crate::config::PythonUrlBuilder::get_python_version_for_build(version)?
        } else {
            version.to_string()
        };

        let install_dir = path_manager.tool_version_dir(self.name(), &check_version);
        Ok(install_dir.exists())
    }

    async fn get_active_version(&self) -> Result<String, anyhow::Error> {
        // Simple implementation - return latest installed version
        let installed_versions = self.get_installed_versions().await?;
        if let Some(latest) = installed_versions.first() {
            Ok(latest.clone())
        } else {
            Ok("none".to_string())
        }
    }

    async fn get_installed_versions(&self) -> Result<Vec<String>, anyhow::Error> {
        let path_manager = vx_paths::PathManager::new().unwrap_or_default();
        let mut versions = path_manager.list_tool_versions(self.name())?;

        // Filter out any build date directories and keep only Python version directories
        versions.retain(|v| {
            // Keep versions that look like Python versions (e.g., "3.13.5")
            // and filter out build dates (e.g., "20250612")
            !v.chars().all(|c| c.is_ascii_digit()) || v.contains('.')
        });

        versions.sort_by(|a, b| b.cmp(a)); // Sort newest first
        Ok(versions)
    }

    async fn execute(
        &self,
        args: &[String],
        context: &ToolContext,
    ) -> Result<ToolExecutionResult, anyhow::Error> {
        let path_manager = vx_paths::PathManager::new().unwrap_or_default();
        let executable_path = if let Ok(versions) = self.get_installed_versions().await {
            if !versions.is_empty() {
                let latest_version = &versions[0];
                let install_dir = path_manager.tool_version_dir(self.name(), latest_version);
                let bin_dir = crate::config::PythonUrlBuilder::get_bin_dir()?;
                let exe_name = crate::config::PythonUrlBuilder::get_executable_name()?;
                install_dir.join(bin_dir).join(exe_name)
            } else if context.use_system_path {
                which::which("python3")
                    .or_else(|_| which::which("python"))
                    .map_err(|_| anyhow::anyhow!("Python not found"))?
            } else {
                return Err(anyhow::anyhow!("Python not installed"));
            }
        } else {
            return Err(anyhow::anyhow!("Failed to get installed versions"));
        };

        let mut cmd = std::process::Command::new(&executable_path);
        cmd.args(args);

        if let Some(cwd) = &context.working_directory {
            cmd.current_dir(cwd);
        }

        for (key, value) in &context.environment_variables {
            cmd.env(key, value);
        }

        let status = cmd
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to execute Python: {}", e))?;

        Ok(ToolExecutionResult {
            exit_code: status.code().unwrap_or(1),
            stdout: None,
            stderr: None,
        })
    }

    async fn get_download_url(&self, version: &str) -> Result<Option<String>, anyhow::Error> {
        let url = crate::config::PythonUrlBuilder::build_url(version)?;
        Ok(Some(url))
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://www.python.org/".to_string(),
        );
        meta.insert("ecosystem".to_string(), "python".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/astral-sh/python-build-standalone".to_string(),
        );
        meta.insert(
            "license".to_string(),
            "Python Software Foundation License".to_string(),
        );
        meta.insert(
            "description".to_string(),
            "Python programming language using Python Build Standalone".to_string(),
        );
        meta
    }

    fn get_dependencies(&self) -> Vec<vx_plugin::ToolDependency> {
        Vec::new() // Python has no dependencies
    }
}

/// Pip tool (Python package installer)
#[derive(Debug, Clone)]
pub struct PipTool {
    python_tool: PythonTool,
}

impl PipTool {
    pub fn new() -> Self {
        Self {
            python_tool: PythonTool::new(),
        }
    }
}

impl Default for PipTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VxTool for PipTool {
    fn name(&self) -> &str {
        "pip"
    }

    fn description(&self) -> &str {
        "Python package installer"
    }

    async fn fetch_versions(&self, use_cache: bool) -> Result<Vec<VersionInfo>, anyhow::Error> {
        // Pip versions follow Python versions
        self.python_tool.fetch_versions(use_cache).await
    }

    async fn install_version(&self, version: &str, force: bool) -> Result<(), anyhow::Error> {
        // Install Python first, pip comes with it
        // Resolve "latest" to actual version number
        let actual_version = if version == "latest" {
            let versions = self.python_tool.fetch_versions(true).await?;
            if let Some(latest) = versions.first() {
                latest.version.clone()
            } else {
                return Err(anyhow::anyhow!("No versions available for Python"));
            }
        } else {
            version.to_string()
        };

        self.python_tool
            .install_version(&actual_version, force)
            .await
    }

    async fn is_version_installed(&self, version: &str) -> Result<bool, anyhow::Error> {
        self.python_tool.is_version_installed(version).await
    }

    async fn get_active_version(&self) -> Result<String, anyhow::Error> {
        self.python_tool.get_active_version().await
    }

    async fn get_installed_versions(&self) -> Result<Vec<String>, anyhow::Error> {
        self.python_tool.get_installed_versions().await
    }

    async fn execute(
        &self,
        args: &[String],
        context: &ToolContext,
    ) -> Result<ToolExecutionResult, anyhow::Error> {
        // Get Python executable and run pip module
        let installed_versions = self.python_tool.get_installed_versions().await?;
        let python_exe = if !installed_versions.is_empty() {
            let latest_version = &installed_versions[0];
            let path_manager = vx_paths::PathManager::new().unwrap_or_default();
            let install_dir = path_manager.tool_version_dir("python", latest_version);
            let bin_dir = crate::config::PythonUrlBuilder::get_bin_dir()?;
            let exe_name = crate::config::PythonUrlBuilder::get_executable_name()?;
            install_dir.join(bin_dir).join(exe_name)
        } else {
            return Err(anyhow::anyhow!(
                "Python not installed. Install Python first."
            ));
        };

        let mut cmd = std::process::Command::new(&python_exe);
        cmd.arg("-m").arg("pip").args(args);

        if let Some(cwd) = &context.working_directory {
            cmd.current_dir(cwd);
        }

        for (key, value) in &context.environment_variables {
            cmd.env(key, value);
        }

        let status = cmd
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to execute pip: {}", e))?;

        Ok(ToolExecutionResult {
            exit_code: status.code().unwrap_or(1),
            stdout: None,
            stderr: None,
        })
    }

    async fn get_download_url(&self, version: &str) -> Result<Option<String>, anyhow::Error> {
        VxTool::get_download_url(&self.python_tool, version).await
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://pip.pypa.io/".to_string());
        meta.insert("ecosystem".to_string(), "python".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/pypa/pip".to_string(),
        );
        meta.insert("license".to_string(), "MIT".to_string());
        meta.insert(
            "description".to_string(),
            "Python package installer".to_string(),
        );
        meta
    }

    fn get_dependencies(&self) -> Vec<vx_plugin::ToolDependency> {
        vec![vx_plugin::ToolDependency::required(
            "python",
            "pip requires Python",
        )]
    }
}

/// Pipx tool (Install and run Python applications in isolated environments)
#[derive(Debug, Clone)]
pub struct PipxTool {
    python_tool: PythonTool,
}

impl PipxTool {
    pub fn new() -> Self {
        Self {
            python_tool: PythonTool::new(),
        }
    }
}

impl Default for PipxTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VxTool for PipxTool {
    fn name(&self) -> &str {
        "pipx"
    }

    fn description(&self) -> &str {
        "Install and run Python applications in isolated environments"
    }

    async fn fetch_versions(&self, use_cache: bool) -> Result<Vec<VersionInfo>, anyhow::Error> {
        // Pipx versions follow Python versions
        self.python_tool.fetch_versions(use_cache).await
    }

    async fn install_version(&self, version: &str, force: bool) -> Result<(), anyhow::Error> {
        // Install Python first, then install pipx
        // Resolve "latest" to actual version number
        let actual_version = if version == "latest" {
            let versions = self.python_tool.fetch_versions(true).await?;
            if let Some(latest) = versions.first() {
                latest.version.clone()
            } else {
                return Err(anyhow::anyhow!("No versions available for Python"));
            }
        } else {
            version.to_string()
        };

        self.python_tool
            .install_version(&actual_version, force)
            .await?;

        // Install pipx using pip
        let pip_tool = PipTool::new();
        let context = ToolContext::default();
        pip_tool
            .execute(&["install".to_string(), "pipx".to_string()], &context)
            .await?;

        Ok(())
    }

    async fn is_version_installed(&self, version: &str) -> Result<bool, anyhow::Error> {
        // Check if Python is installed and pipx is available
        if !self.python_tool.is_version_installed(version).await? {
            return Ok(false);
        }

        // Check if pipx is installed
        let context = ToolContext::default();
        let result = self.execute(&["--version".to_string()], &context).await;
        Ok(result.is_ok())
    }

    async fn get_active_version(&self) -> Result<String, anyhow::Error> {
        self.python_tool.get_active_version().await
    }

    async fn get_installed_versions(&self) -> Result<Vec<String>, anyhow::Error> {
        self.python_tool.get_installed_versions().await
    }

    async fn execute(
        &self,
        args: &[String],
        context: &ToolContext,
    ) -> Result<ToolExecutionResult, anyhow::Error> {
        // Get Python executable and run pipx module
        let installed_versions = self.python_tool.get_installed_versions().await?;
        let python_exe = if !installed_versions.is_empty() {
            let latest_version = &installed_versions[0];
            let path_manager = vx_paths::PathManager::new().unwrap_or_default();
            let install_dir = path_manager.tool_version_dir("python", latest_version);
            let bin_dir = crate::config::PythonUrlBuilder::get_bin_dir()?;
            let exe_name = crate::config::PythonUrlBuilder::get_executable_name()?;
            install_dir.join(bin_dir).join(exe_name)
        } else {
            return Err(anyhow::anyhow!(
                "Python not installed. Install Python first."
            ));
        };

        let mut cmd = std::process::Command::new(&python_exe);
        cmd.arg("-m").arg("pipx").args(args);

        if let Some(cwd) = &context.working_directory {
            cmd.current_dir(cwd);
        }

        for (key, value) in &context.environment_variables {
            cmd.env(key, value);
        }

        let status = cmd
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to execute pipx: {}", e))?;

        Ok(ToolExecutionResult {
            exit_code: status.code().unwrap_or(1),
            stdout: None,
            stderr: None,
        })
    }

    async fn get_download_url(&self, version: &str) -> Result<Option<String>, anyhow::Error> {
        VxTool::get_download_url(&self.python_tool, version).await
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://pypa.github.io/pipx/".to_string(),
        );
        meta.insert("ecosystem".to_string(), "python".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/pypa/pipx".to_string(),
        );
        meta.insert("license".to_string(), "MIT".to_string());
        meta.insert(
            "description".to_string(),
            "Install and run Python applications in isolated environments".to_string(),
        );
        meta
    }

    fn get_dependencies(&self) -> Vec<vx_plugin::ToolDependency> {
        vec![
            vx_plugin::ToolDependency::required("python", "pipx requires Python"),
            vx_plugin::ToolDependency::required("pip", "pipx requires pip"),
        ]
    }
}
