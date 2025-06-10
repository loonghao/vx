use crate::plugin::{
    InstallResult, Platform, Plugin, PluginCategory, PluginCommand, PluginMetadata,
};
use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

pub struct NodePlugin {
    metadata: PluginMetadata,
}

impl NodePlugin {
    pub fn new() -> Self {
        let metadata = PluginMetadata {
            name: "node".to_string(),
            version: "1.0.0".to_string(),
            description: "Node.js JavaScript runtime and npm package manager".to_string(),
            author: "vx team".to_string(),
            homepage: Some("https://nodejs.org".to_string()),
            repository: Some("https://github.com/nodejs/node".to_string()),
            license: "MIT".to_string(),
            keywords: vec![
                "node".to_string(),
                "nodejs".to_string(),
                "javascript".to_string(),
                "npm".to_string(),
                "runtime".to_string(),
            ],
            categories: vec![
                PluginCategory::Runtime,
                PluginCategory::PackageManager,
                PluginCategory::Language,
            ],
            supported_platforms: vec![Platform::Windows, Platform::MacOS, Platform::Linux],
            dependencies: vec![],
            conflicts: vec![],
        };

        Self { metadata }
    }
}

impl Plugin for NodePlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn is_installed(&self) -> Result<bool> {
        match which::which("node") {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    fn get_installed_version(&self) -> Result<Option<String>> {
        let output = Command::new("node").arg("--version").output();

        match output {
            Ok(output) if output.status.success() => {
                let version_str = String::from_utf8_lossy(&output.stdout);
                // Parse "v18.19.0" -> "18.19.0"
                let version = version_str
                    .trim()
                    .strip_prefix('v')
                    .unwrap_or(version_str.trim());
                Ok(Some(version.to_string()))
            }
            _ => Ok(None),
        }
    }

    fn get_latest_version(&self) -> Result<String> {
        // For simplicity, return a known LTS version
        // In a real implementation, this would fetch from nodejs.org API
        Ok("20.11.0".to_string())
    }

    fn install(&self, version: &str, _install_dir: &PathBuf) -> Result<InstallResult> {
        // Use the existing installer infrastructure
        let _config = crate::install_configs::get_install_config("node", version)
            .ok_or_else(|| anyhow::anyhow!("No install config for Node.js"))?;

        // Note: This should be called from an async context
        // For now, we'll return an error indicating async installation is needed
        Err(anyhow::anyhow!(
            "Installation requires async context. Use plugin manager install method."
        ))
    }

    fn uninstall(&self, _version: &str, install_dir: &PathBuf) -> Result<()> {
        if install_dir.exists() {
            std::fs::remove_dir_all(install_dir)?;
            println!(
                "🗑️ Removed Node.js installation from {}",
                install_dir.display()
            );
        }
        Ok(())
    }

    fn get_executable_path(&self, _version: &str, install_dir: &PathBuf) -> PathBuf {
        if cfg!(windows) {
            install_dir.join("node.exe")
        } else {
            install_dir.join("bin").join("node")
        }
    }

    fn validate_installation(&self, install_dir: &PathBuf) -> Result<bool> {
        let exe_path = self.get_executable_path("", install_dir);
        Ok(exe_path.exists())
    }

    fn get_commands(&self) -> Vec<PluginCommand> {
        vec![
            PluginCommand {
                name: "run".to_string(),
                description: "Run a Node.js script".to_string(),
                usage: "vx node [options] [script.js] [arguments]".to_string(),
                examples: vec![
                    "vx node app.js".to_string(),
                    "vx node --version".to_string(),
                    "vx node -e \"console.log('Hello')\"".to_string(),
                ],
            },
            PluginCommand {
                name: "npm".to_string(),
                description: "Node Package Manager".to_string(),
                usage: "vx npm <command> [args]".to_string(),
                examples: vec![
                    "vx npm install".to_string(),
                    "vx npm install express".to_string(),
                    "vx npm run start".to_string(),
                    "vx npm publish".to_string(),
                ],
            },
            PluginCommand {
                name: "npx".to_string(),
                description: "Execute npm package binaries".to_string(),
                usage: "vx npx [options] <command>[@version] [command-arg]...".to_string(),
                examples: vec![
                    "vx npx create-react-app my-app".to_string(),
                    "vx npx -p typescript tsc --version".to_string(),
                ],
            },
        ]
    }

    fn execute_command(&self, command: &str, args: &[String]) -> Result<i32> {
        let cmd_name = if command.is_empty() {
            "node"
        } else {
            match command {
                "run" => "node",
                "npm" => "npm",
                "npx" => "npx",
                _ => command,
            }
        };

        let mut cmd = Command::new(cmd_name);
        cmd.args(args);

        let status = cmd.status()?;
        Ok(status.code().unwrap_or(1))
    }
}

impl NodePlugin {
    #[allow(dead_code)]
    fn calculate_size(path: &PathBuf) -> Result<u64> {
        if path.is_file() {
            Ok(path.metadata()?.len())
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
