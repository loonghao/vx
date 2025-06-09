use crate::plugin::{
    InstallResult, Platform, Plugin, PluginCategory, PluginCommand, PluginMetadata,
};
use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Command;

pub struct UvPlugin {
    metadata: PluginMetadata,
}

impl UvPlugin {
    pub fn new() -> Self {
        let metadata = PluginMetadata {
            name: "uv".to_string(),
            version: "1.0.0".to_string(),
            description: "An extremely fast Python package installer and resolver".to_string(),
            author: "vx team".to_string(),
            homepage: Some("https://docs.astral.sh/uv/".to_string()),
            repository: Some("https://github.com/astral-sh/uv".to_string()),
            license: "Apache-2.0".to_string(),
            keywords: vec![
                "uv".to_string(),
                "python".to_string(),
                "package".to_string(),
                "installer".to_string(),
                "pip".to_string(),
            ],
            categories: vec![PluginCategory::PackageManager, PluginCategory::Language],
            supported_platforms: vec![Platform::Windows, Platform::MacOS, Platform::Linux],
            dependencies: vec![],
            conflicts: vec![],
        };

        Self { metadata }
    }
}

#[async_trait]
impl Plugin for UvPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn is_installed(&self) -> Result<bool> {
        match which::which("uv") {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn get_installed_version(&self) -> Result<Option<String>> {
        let output = Command::new("uv").arg("--version").output();

        match output {
            Ok(output) if output.status.success() => {
                let version_str = String::from_utf8_lossy(&output.stdout);
                // Parse "uv 0.5.26 (5ef3d5139 2025-01-30)" -> "0.5.26"
                if let Some(version_part) = version_str.split_whitespace().nth(1) {
                    return Ok(Some(version_part.to_string()));
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    async fn get_latest_version(&self) -> Result<String> {
        // For simplicity, return a known stable version
        // In a real implementation, this would fetch from GitHub API
        Ok("0.5.26".to_string())
    }

    async fn install(&self, version: &str, _install_dir: &PathBuf) -> Result<InstallResult> {
        // Use the existing installer infrastructure
        let config = crate::install_configs::get_install_config("uv", version)
            .ok_or_else(|| anyhow::anyhow!("No install config for UV"))?;

        let installer = crate::installer::Installer::new();
        let executable_path = installer.install(&config).await?;

        // Calculate installed files and size
        let installed_files = vec![executable_path.clone()];
        let size = Self::calculate_size(&executable_path)?;

        Ok(InstallResult {
            executable_path,
            installed_files,
            size,
            checksum: None,
        })
    }

    async fn uninstall(&self, _version: &str, install_dir: &PathBuf) -> Result<()> {
        if install_dir.exists() {
            std::fs::remove_dir_all(install_dir)?;
            println!("🗑️  Removed UV installation from {}", install_dir.display());
        }
        Ok(())
    }

    fn get_executable_path(&self, _version: &str, install_dir: &PathBuf) -> PathBuf {
        if cfg!(windows) {
            install_dir.join("uv.exe")
        } else {
            install_dir.join("uv")
        }
    }

    async fn validate_installation(&self, install_dir: &PathBuf) -> Result<bool> {
        let exe_path = self.get_executable_path("", install_dir);
        Ok(exe_path.exists())
    }

    fn get_commands(&self) -> Vec<PluginCommand> {
        vec![
            PluginCommand {
                name: "pip".to_string(),
                description: "Manage Python packages with a pip-compatible interface".to_string(),
                usage: "vx uv pip <command> [options]".to_string(),
                examples: vec![
                    "vx uv pip install requests".to_string(),
                    "vx uv pip install -r requirements.txt".to_string(),
                    "vx uv pip list".to_string(),
                    "vx uv pip show requests".to_string(),
                    "vx uv pip uninstall requests".to_string(),
                    "vx uv pip freeze > requirements.txt".to_string(),
                ],
            },
            PluginCommand {
                name: "venv".to_string(),
                description: "Create and manage virtual environments".to_string(),
                usage: "vx uv venv [options] [path]".to_string(),
                examples: vec![
                    "vx uv venv".to_string(),
                    "vx uv venv .venv".to_string(),
                    "vx uv venv --python 3.11".to_string(),
                    "vx uv venv --python python3.12 myenv".to_string(),
                ],
            },
            PluginCommand {
                name: "run".to_string(),
                description: "Run a command in a virtual environment".to_string(),
                usage: "vx uv run [options] <command> [args]".to_string(),
                examples: vec![
                    "vx uv run python script.py".to_string(),
                    "vx uv run --with requests python -c \"import requests\"".to_string(),
                    "vx uv run --with \"fastapi[all]\" uvicorn main:app".to_string(),
                    "vx uv run pytest".to_string(),
                ],
            },
            PluginCommand {
                name: "add".to_string(),
                description: "Add dependencies to the project".to_string(),
                usage: "vx uv add [options] <packages>".to_string(),
                examples: vec![
                    "vx uv add requests".to_string(),
                    "vx uv add --dev pytest".to_string(),
                    "vx uv add \"django>=4.0\"".to_string(),
                    "vx uv add --group test pytest coverage".to_string(),
                ],
            },
            PluginCommand {
                name: "sync".to_string(),
                description: "Sync the project environment".to_string(),
                usage: "vx uv sync [options]".to_string(),
                examples: vec![
                    "vx uv sync".to_string(),
                    "vx uv sync --dev".to_string(),
                    "vx uv sync --all-extras".to_string(),
                ],
            },
            PluginCommand {
                name: "init".to_string(),
                description: "Initialize a new Python project".to_string(),
                usage: "vx uv init [options] [path]".to_string(),
                examples: vec![
                    "vx uv init".to_string(),
                    "vx uv init my-project".to_string(),
                    "vx uv init --lib my-library".to_string(),
                ],
            },
            PluginCommand {
                name: "lock".to_string(),
                description: "Update the project lockfile".to_string(),
                usage: "vx uv lock [options]".to_string(),
                examples: vec![
                    "vx uv lock".to_string(),
                    "vx uv lock --upgrade".to_string(),
                ],
            },
            PluginCommand {
                name: "tree".to_string(),
                description: "Display the dependency tree".to_string(),
                usage: "vx uv tree [options]".to_string(),
                examples: vec![
                    "vx uv tree".to_string(),
                    "vx uv tree --depth 2".to_string(),
                ],
            },
        ]
    }

    async fn execute_command(&self, command: &str, args: &[String]) -> Result<i32> {
        let mut cmd = Command::new("uv");

        // If command is empty, just pass args directly to uv
        if command.is_empty() {
            cmd.args(args);
        } else {
            cmd.arg(command);
            cmd.args(args);
        }

        let status = cmd.status()?;
        Ok(status.code().unwrap_or(1))
    }
}

impl UvPlugin {
    fn calculate_size(path: &PathBuf) -> Result<u64> {
        if path.is_file() {
            Ok(std::fs::metadata(path)?.len())
        } else if path.is_dir() {
            let mut size = 0;
            for entry in walkdir::WalkDir::new(path) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    size += entry.metadata()?.len();
                }
            }
            Ok(size)
        } else {
            Ok(0)
        }
    }
}
