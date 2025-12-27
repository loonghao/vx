//! Remote development configuration generation
//!
//! This module generates configuration files for remote development environments:
//! - GitHub Codespaces (devcontainer.json)
//! - GitPod (.gitpod.yml)
//! - DevContainer (devcontainer.json)

use crate::{CodespacesConfig, DevContainerConfig, GitpodConfig, RemoteConfig, VxConfig};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Remote development configuration generator
pub struct RemoteGenerator;

impl RemoteGenerator {
    /// Generate devcontainer.json for Codespaces/DevContainer
    pub fn generate_devcontainer(config: &VxConfig, devcontainer: &DevContainerConfig) -> Value {
        let mut result = json!({});

        // Name
        if let Some(name) = &devcontainer.name {
            result["name"] = json!(name);
        } else if let Some(project) = &config.project {
            if let Some(name) = &project.name {
                result["name"] = json!(format!("{} Dev Container", name));
            }
        }

        // Image or Dockerfile
        if let Some(image) = &devcontainer.image {
            result["image"] = json!(image);
        } else if let Some(dockerfile) = &devcontainer.dockerfile {
            result["build"] = json!({
                "dockerfile": dockerfile,
                "context": devcontainer.context.as_deref().unwrap_or(".")
            });
        }

        // Features
        if !devcontainer.features.is_empty() {
            result["features"] = json!(devcontainer.features);
        } else {
            // Auto-detect features from tools
            let features = Self::detect_features(config);
            if !features.is_empty() {
                result["features"] = json!(features);
            }
        }

        // Post-create command
        if let Some(cmd) = &devcontainer.post_create_command {
            result["postCreateCommand"] = json!(cmd);
        } else {
            // Default to vx setup
            result["postCreateCommand"] = json!("vx setup");
        }

        // Post-start command
        if let Some(cmd) = &devcontainer.post_start_command {
            result["postStartCommand"] = json!(cmd);
        }

        // Forwarded ports
        if !devcontainer.forward_ports.is_empty() {
            result["forwardPorts"] = json!(devcontainer.forward_ports);
        }

        // Remote user
        if let Some(user) = &devcontainer.remote_user {
            result["remoteUser"] = json!(user);
        }

        // Container environment
        if !devcontainer.container_env.is_empty() {
            result["containerEnv"] = json!(devcontainer.container_env);
        }

        // Mounts
        if !devcontainer.mounts.is_empty() {
            result["mounts"] = json!(devcontainer.mounts);
        }

        // Customizations
        if let Some(customizations) = &devcontainer.customizations {
            let mut custom = json!({});
            if let Some(vscode) = &customizations.vscode {
                let mut vscode_config = json!({});
                if !vscode.extensions.is_empty() {
                    vscode_config["extensions"] = json!(vscode.extensions);
                }
                if !vscode.settings.is_empty() {
                    vscode_config["settings"] = json!(vscode.settings);
                }
                custom["vscode"] = vscode_config;
            }
            result["customizations"] = custom;
        }

        result
    }

    /// Detect features from vx config
    fn detect_features(config: &VxConfig) -> HashMap<String, Value> {
        let mut features = HashMap::new();

        for tool in config.tools.keys() {
            match tool.as_str() {
                "node" | "npm" | "npx" => {
                    if let Some(version) = config.get_tool_version("node") {
                        features.insert(
                            "ghcr.io/devcontainers/features/node:1".to_string(),
                            json!({ "version": version }),
                        );
                    }
                }
                "python" | "uv" | "pip" => {
                    if let Some(version) = config.get_tool_version("python") {
                        features.insert(
                            "ghcr.io/devcontainers/features/python:1".to_string(),
                            json!({ "version": version }),
                        );
                    }
                }
                "go" => {
                    if let Some(version) = config.get_tool_version("go") {
                        features.insert(
                            "ghcr.io/devcontainers/features/go:1".to_string(),
                            json!({ "version": version }),
                        );
                    }
                }
                "rust" | "cargo" => {
                    features.insert(
                        "ghcr.io/devcontainers/features/rust:1".to_string(),
                        json!({}),
                    );
                }
                _ => {}
            }
        }

        features
    }

    /// Generate .gitpod.yml content
    pub fn generate_gitpod(config: &VxConfig, gitpod: &GitpodConfig) -> Value {
        let mut result = json!({});

        // Image
        if let Some(image) = &gitpod.image {
            result["image"] = json!(image);
        }

        // Tasks
        let mut tasks = Vec::new();

        // Add configured tasks
        for task in &gitpod.tasks {
            let mut task_obj = json!({});
            if let Some(name) = &task.name {
                task_obj["name"] = json!(name);
            }
            if let Some(init) = &task.init {
                task_obj["init"] = json!(init);
            }
            if let Some(command) = &task.command {
                task_obj["command"] = json!(command);
            }
            if let Some(before) = &task.before {
                task_obj["before"] = json!(before);
            }
            tasks.push(task_obj);
        }

        // Add default setup task if no tasks configured
        if tasks.is_empty() {
            tasks.push(json!({
                "name": "Setup",
                "init": "curl -fsSL https://vx.dev/install.sh | sh && vx setup",
                "command": "echo 'Ready!'"
            }));
        }

        result["tasks"] = json!(tasks);

        // VS Code extensions
        let mut extensions = gitpod.extensions.clone();
        if extensions.is_empty() {
            extensions = Self::detect_extensions(config);
        }
        if !extensions.is_empty() {
            result["vscode"] = json!({
                "extensions": extensions
            });
        }

        // Ports
        if !gitpod.ports.is_empty() {
            let ports: Vec<Value> = gitpod
                .ports
                .iter()
                .map(|p| {
                    let mut port = json!({ "port": p.port });
                    if let Some(visibility) = &p.visibility {
                        port["visibility"] = json!(visibility);
                    }
                    if let Some(on_open) = &p.on_open {
                        port["onOpen"] = json!(on_open);
                    }
                    port
                })
                .collect();
            result["ports"] = json!(ports);
        }

        // Prebuilds
        if let Some(prebuilds) = &gitpod.prebuilds {
            let mut pb = json!({});
            if let Some(master) = prebuilds.master {
                pb["master"] = json!(master);
            }
            if let Some(branches) = prebuilds.branches {
                pb["branches"] = json!(branches);
            }
            if let Some(prs) = prebuilds.pull_requests {
                pb["pullRequests"] = json!(prs);
            }
            if let Some(check) = prebuilds.add_check {
                pb["addCheck"] = json!(check);
            }
            result["github"] = json!({ "prebuilds": pb });
        }

        result
    }

    /// Detect VS Code extensions from config
    fn detect_extensions(config: &VxConfig) -> Vec<String> {
        let mut extensions = Vec::new();

        for tool in config.tools.keys() {
            match tool.as_str() {
                "node" | "npm" | "npx" => {
                    extensions.push("dbaeumer.vscode-eslint".to_string());
                }
                "python" | "uv" | "pip" => {
                    extensions.push("ms-python.python".to_string());
                }
                "rust" | "cargo" => {
                    extensions.push("rust-lang.rust-analyzer".to_string());
                }
                "go" => {
                    extensions.push("golang.go".to_string());
                }
                _ => {}
            }
        }

        extensions.sort();
        extensions.dedup();
        extensions
    }

    /// Generate Codespaces configuration (extends devcontainer)
    pub fn generate_codespaces(config: &VxConfig, codespaces: &CodespacesConfig) -> Value {
        // Start with devcontainer base
        let devcontainer = DevContainerConfig {
            enabled: codespaces.enabled,
            features: HashMap::new(),
            forward_ports: codespaces.ports.iter().map(|p| p.port).collect(),
            ..Default::default()
        };

        let mut result = Self::generate_devcontainer(config, &devcontainer);

        // Add Codespaces-specific settings
        if let Some(machine) = &codespaces.machine {
            result["hostRequirements"] = json!({
                "cpus": match machine.as_str() {
                    "basicLinux32gb" => 2,
                    "standardLinux32gb" => 4,
                    "premiumLinux" => 8,
                    _ => 4
                }
            });
        }

        // Add extensions
        if !codespaces.extensions.is_empty() {
            if result.get("customizations").is_none() {
                result["customizations"] = json!({});
            }
            result["customizations"]["vscode"] = json!({
                "extensions": codespaces.extensions
            });
        }

        // Port attributes
        if !codespaces.ports.is_empty() {
            let mut port_attrs = json!({});
            for port in &codespaces.ports {
                let mut attr = json!({});
                if let Some(label) = &port.label {
                    attr["label"] = json!(label);
                }
                if let Some(visibility) = &port.visibility {
                    attr["visibility"] = json!(visibility);
                }
                if let Some(on_forward) = &port.on_auto_forward {
                    attr["onAutoForward"] = json!(on_forward);
                }
                port_attrs[port.port.to_string()] = attr;
            }
            result["portsAttributes"] = port_attrs;
        }

        result
    }

    /// Generate all remote development configs
    pub fn generate_all(config: &VxConfig) -> HashMap<String, String> {
        let mut files = HashMap::new();

        if let Some(remote) = &config.remote {
            if remote.enabled != Some(false) {
                // DevContainer
                if let Some(devcontainer) = &remote.devcontainer {
                    if devcontainer.enabled != Some(false) {
                        let content = Self::generate_devcontainer(config, devcontainer);
                        files.insert(
                            ".devcontainer/devcontainer.json".to_string(),
                            serde_json::to_string_pretty(&content).unwrap(),
                        );
                    }
                }

                // Codespaces
                if let Some(codespaces) = &remote.codespaces {
                    if codespaces.enabled != Some(false) {
                        let content = Self::generate_codespaces(config, codespaces);
                        files.insert(
                            ".devcontainer/devcontainer.json".to_string(),
                            serde_json::to_string_pretty(&content).unwrap(),
                        );
                    }
                }

                // GitPod
                if let Some(gitpod) = &remote.gitpod {
                    if gitpod.enabled != Some(false) {
                        let content = Self::generate_gitpod(config, gitpod);
                        files.insert(
                            ".gitpod.yml".to_string(),
                            serde_yaml::to_string(&content).unwrap_or_default(),
                        );
                    }
                }
            }
        }

        files
    }
}

/// Generate devcontainer.json content as a string
pub fn generate_devcontainer_json(config: &VxConfig, remote_config: &RemoteConfig) -> String {
    let devcontainer = remote_config.devcontainer.clone().unwrap_or_default();

    let content = RemoteGenerator::generate_devcontainer(config, &devcontainer);
    serde_json::to_string_pretty(&content).unwrap_or_default()
}

/// Generate .gitpod.yml content as a string
pub fn generate_gitpod_yml(config: &VxConfig, remote_config: &RemoteConfig) -> String {
    let gitpod = remote_config.gitpod.clone().unwrap_or_default();

    let content = RemoteGenerator::generate_gitpod(config, &gitpod);
    serde_yaml::to_string(&content).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToolVersion;

    #[test]
    fn test_generate_devcontainer() {
        let mut config = VxConfig::default();
        config
            .tools
            .insert("node".to_string(), ToolVersion::Simple("20".to_string()));

        let devcontainer = DevContainerConfig {
            enabled: Some(true),
            name: Some("Test Project".to_string()),
            image: Some("mcr.microsoft.com/devcontainers/base:ubuntu".to_string()),
            ..Default::default()
        };

        let result = RemoteGenerator::generate_devcontainer(&config, &devcontainer);
        assert_eq!(result["name"], "Test Project");
        assert!(result["image"].as_str().unwrap().contains("ubuntu"));
    }

    #[test]
    fn test_detect_features() {
        let mut config = VxConfig::default();
        config
            .tools
            .insert("node".to_string(), ToolVersion::Simple("20".to_string()));
        config.tools.insert(
            "rust".to_string(),
            ToolVersion::Simple("stable".to_string()),
        );

        let features = RemoteGenerator::detect_features(&config);
        assert!(features.contains_key("ghcr.io/devcontainers/features/node:1"));
        assert!(features.contains_key("ghcr.io/devcontainers/features/rust:1"));
    }
}
