use crate::plugin::{
    InstallResult, Platform, Plugin, PluginCategory, PluginCommand, PluginMetadata,
};
use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Command;

pub struct GoPlugin {
    metadata: PluginMetadata,
}

impl GoPlugin {
    pub fn new() -> Self {
        let metadata = PluginMetadata {
            name: "go".to_string(),
            version: "1.0.0".to_string(),
            description: "Go programming language toolchain".to_string(),
            author: "vx team".to_string(),
            homepage: Some("https://golang.org".to_string()),
            repository: Some("https://github.com/golang/go".to_string()),
            license: "BSD-3-Clause".to_string(),
            keywords: vec![
                "go".to_string(),
                "golang".to_string(),
                "programming".to_string(),
                "language".to_string(),
            ],
            categories: vec![PluginCategory::Language, PluginCategory::BuildTool],
            supported_platforms: vec![Platform::Windows, Platform::MacOS, Platform::Linux],
            dependencies: vec![],
            conflicts: vec![],
        };

        Self { metadata }
    }

    fn get_download_url(&self, version: &str) -> String {
        let platform = if cfg!(windows) {
            "windows-amd64.zip"
        } else if cfg!(target_os = "macos") {
            "darwin-amd64.tar.gz"
        } else {
            "linux-amd64.tar.gz"
        };

        format!("https://golang.org/dl/go{}.{}", version, platform)
    }
}

#[async_trait]
impl Plugin for GoPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn is_installed(&self) -> Result<bool> {
        match which::which("go") {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn get_installed_version(&self) -> Result<Option<String>> {
        let output = Command::new("go").arg("version").output();

        match output {
            Ok(output) if output.status.success() => {
                let version_str = String::from_utf8_lossy(&output.stdout);
                // Parse "go version go1.21.6 windows/amd64" -> "1.21.6"
                if let Some(version_part) = version_str.split_whitespace().nth(2) {
                    if let Some(version) = version_part.strip_prefix("go") {
                        return Ok(Some(version.to_string()));
                    }
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    async fn get_latest_version(&self) -> Result<String> {
        // For simplicity, return a known stable version
        // In a real implementation, this would fetch from golang.org API
        Ok("1.21.6".to_string())
    }

    async fn install(&self, version: &str, _install_dir: &PathBuf) -> Result<InstallResult> {
        let _download_url = self.get_download_url(version);

        // Use the existing installer infrastructure
        let config = crate::install_configs::get_install_config("go", version)
            .ok_or_else(|| anyhow::anyhow!("No install config for Go"))?;

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
            println!("🗑️  Removed Go installation from {}", install_dir.display());
        }
        Ok(())
    }

    fn get_executable_path(&self, _version: &str, install_dir: &PathBuf) -> PathBuf {
        if cfg!(windows) {
            install_dir.join("go").join("bin").join("go.exe")
        } else {
            install_dir.join("go").join("bin").join("go")
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
                description: "Compile packages and dependencies".to_string(),
                usage: "vx go build [packages]".to_string(),
                examples: vec![
                    "vx go build".to_string(),
                    "vx go build ./...".to_string(),
                    "vx go build -o myapp main.go".to_string(),
                ],
            },
            PluginCommand {
                name: "run".to_string(),
                description: "Compile and run Go program".to_string(),
                usage: "vx go run [build flags] [-exec xprog] package [arguments...]".to_string(),
                examples: vec![
                    "vx go run main.go".to_string(),
                    "vx go run -race main.go".to_string(),
                ],
            },
            PluginCommand {
                name: "test".to_string(),
                description: "Test packages".to_string(),
                usage: "vx go test [build/test flags] [packages] [build/test flags & test binary flags]".to_string(),
                examples: vec![
                    "vx go test".to_string(),
                    "vx go test ./...".to_string(),
                    "vx go test -v -race".to_string(),
                ],
            },
            PluginCommand {
                name: "mod".to_string(),
                description: "Module maintenance".to_string(),
                usage: "vx go mod <command> [arguments]".to_string(),
                examples: vec![
                    "vx go mod init mymodule".to_string(),
                    "vx go mod tidy".to_string(),
                    "vx go mod download".to_string(),
                ],
            },
        ]
    }

    async fn execute_command(&self, command: &str, args: &[String]) -> Result<i32> {
        let mut cmd = Command::new("go");

        // If command is empty, just pass args directly to go
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

impl GoPlugin {
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
