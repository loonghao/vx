use crate::plugin::{
    InstallResult, Platform, Plugin, PluginCategory, PluginCommand, PluginMetadata,
};
use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Command;

pub struct RustPlugin {
    metadata: PluginMetadata,
}

impl RustPlugin {
    pub fn new() -> Self {
        let metadata = PluginMetadata {
            name: "rust".to_string(),
            version: "1.0.0".to_string(),
            description: "Rust programming language toolchain with Cargo package manager"
                .to_string(),
            author: "vx team".to_string(),
            homepage: Some("https://www.rust-lang.org".to_string()),
            repository: Some("https://github.com/rust-lang/rust".to_string()),
            license: "MIT OR Apache-2.0".to_string(),
            keywords: vec![
                "rust".to_string(),
                "cargo".to_string(),
                "rustc".to_string(),
                "programming".to_string(),
                "language".to_string(),
            ],
            categories: vec![
                PluginCategory::Language,
                PluginCategory::BuildTool,
                PluginCategory::PackageManager,
            ],
            supported_platforms: vec![Platform::Windows, Platform::MacOS, Platform::Linux],
            dependencies: vec![],
            conflicts: vec![],
        };

        Self { metadata }
    }
}

#[async_trait]
impl Plugin for RustPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn is_installed(&self) -> Result<bool> {
        match which::which("cargo") {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn get_installed_version(&self) -> Result<Option<String>> {
        let output = Command::new("rustc").arg("--version").output();

        match output {
            Ok(output) if output.status.success() => {
                let version_str = String::from_utf8_lossy(&output.stdout);
                // Parse "rustc 1.75.0 (82e1608df 2023-12-21)" -> "1.75.0"
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
        // In a real implementation, this would fetch from forge.rust-lang.org API
        Ok("1.75.0".to_string())
    }

    async fn install(&self, version: &str, _install_dir: &PathBuf) -> Result<InstallResult> {
        // Use the existing installer infrastructure
        let config = crate::install_configs::get_install_config("rust", version)
            .ok_or_else(|| anyhow::anyhow!("No install config for Rust"))?;

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
            println!(
                "🗑️  Removed Rust installation from {}",
                install_dir.display()
            );
        }
        Ok(())
    }

    fn get_executable_path(&self, _version: &str, install_dir: &PathBuf) -> PathBuf {
        if cfg!(windows) {
            install_dir.join("bin").join("cargo.exe")
        } else {
            install_dir.join("bin").join("cargo")
        }
    }

    async fn validate_installation(&self, install_dir: &PathBuf) -> Result<bool> {
        let exe_path = self.get_executable_path("", install_dir);
        Ok(exe_path.exists())
    }

    fn get_commands(&self) -> Vec<PluginCommand> {
        vec![
            PluginCommand {
                name: "build".to_string(),
                description: "Compile the current package".to_string(),
                usage: "vx cargo build [options]".to_string(),
                examples: vec![
                    "vx cargo build".to_string(),
                    "vx cargo build --release".to_string(),
                    "vx cargo build --target x86_64-pc-windows-gnu".to_string(),
                ],
            },
            PluginCommand {
                name: "run".to_string(),
                description: "Run a binary or example of the local package".to_string(),
                usage: "vx cargo run [options] [-- args]".to_string(),
                examples: vec![
                    "vx cargo run".to_string(),
                    "vx cargo run --bin myapp".to_string(),
                    "vx cargo run -- --help".to_string(),
                ],
            },
            PluginCommand {
                name: "test".to_string(),
                description: "Run the tests".to_string(),
                usage: "vx cargo test [options] [testname] [-- test-options]".to_string(),
                examples: vec![
                    "vx cargo test".to_string(),
                    "vx cargo test my_test".to_string(),
                    "vx cargo test -- --nocapture".to_string(),
                ],
            },
            PluginCommand {
                name: "new".to_string(),
                description: "Create a new cargo package".to_string(),
                usage: "vx cargo new [options] <path>".to_string(),
                examples: vec![
                    "vx cargo new hello_world".to_string(),
                    "vx cargo new --lib my_lib".to_string(),
                    "vx cargo new --bin my_app".to_string(),
                ],
            },
            PluginCommand {
                name: "add".to_string(),
                description: "Add dependencies to a Cargo.toml manifest file".to_string(),
                usage: "vx cargo add [options] <dep>[@<version>]".to_string(),
                examples: vec![
                    "vx cargo add serde".to_string(),
                    "vx cargo add serde@1.0".to_string(),
                    "vx cargo add --dev tokio".to_string(),
                ],
            },
            PluginCommand {
                name: "install".to_string(),
                description: "Install a Rust binary".to_string(),
                usage: "vx cargo install [options] <crate>".to_string(),
                examples: vec![
                    "vx cargo install ripgrep".to_string(),
                    "vx cargo install --git https://github.com/user/repo".to_string(),
                ],
            },
        ]
    }

    async fn execute_command(&self, command: &str, args: &[String]) -> Result<i32> {
        let mut cmd = Command::new("cargo");

        // If command is empty, just pass args directly to cargo
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

impl RustPlugin {
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
