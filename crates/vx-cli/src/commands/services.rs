//! Services command - Manage development services
//!
//! This command manages services defined in `.vx.toml` using Docker/Podman.
//!
//! ## Configuration Example
//!
//! ```toml
//! [services.postgres]
//! image = "postgres:15"
//! ports = ["5432:5432"]
//! env = { POSTGRES_PASSWORD = "dev" }
//! healthcheck = "pg_isready -U postgres"
//!
//! [services.redis]
//! image = "redis:7"
//! ports = ["6379:6379"]
//! ```
//!
//! ## Commands
//!
//! - `vx services start` - Start all services
//! - `vx services stop` - Stop all services
//! - `vx services status` - Show service status
//! - `vx services logs <service>` - Show service logs

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::{Command, Stdio};
use vx_config::{parse_config, ServiceConfig, VxConfig};
use vx_paths::find_config_file_upward;

use crate::ui::UI;

/// Container runtime (docker or podman)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContainerRuntime {
    Docker,
    Podman,
}

impl ContainerRuntime {
    /// Detect available container runtime
    pub fn detect() -> Option<Self> {
        // Try docker first
        if Command::new("docker")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return Some(ContainerRuntime::Docker);
        }

        // Try podman
        if Command::new("podman")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return Some(ContainerRuntime::Podman);
        }

        None
    }

    /// Get the command name
    pub fn command(&self) -> &str {
        match self {
            ContainerRuntime::Docker => "docker",
            ContainerRuntime::Podman => "podman",
        }
    }
}

/// Service status
#[derive(Debug, Clone)]
pub struct ServiceStatus {
    pub name: String,
    pub running: bool,
    pub container_id: Option<String>,
    pub ports: Vec<String>,
    pub health: Option<String>,
}

/// Handle services start command
pub async fn handle_start(
    services: Option<Vec<String>>,
    detach: bool,
    force: bool,
    verbose: bool,
) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let config_path = find_vx_config(&current_dir)?;
    let config = parse_vx_config(&config_path)?;

    if config.services.is_empty() {
        UI::warn("No services defined in .vx.toml");
        println!();
        println!("Add services to your .vx.toml:");
        println!();
        println!("  [services.postgres]");
        println!("  image = \"postgres:15\"");
        println!("  ports = [\"5432:5432\"]");
        println!("  env = {{ POSTGRES_PASSWORD = \"dev\" }}");
        return Ok(());
    }

    let runtime = ContainerRuntime::detect().ok_or_else(|| {
        anyhow::anyhow!("No container runtime found. Please install Docker or Podman.")
    })?;

    UI::header("🚀 Starting Services");
    println!();

    // Filter services if specified
    let services_to_start: Vec<_> = if let Some(names) = services {
        config
            .services
            .iter()
            .filter(|(name, _)| names.contains(name))
            .collect()
    } else {
        config.services.iter().collect()
    };

    if services_to_start.is_empty() {
        UI::warn("No matching services found");
        return Ok(());
    }

    // Sort by dependencies
    let ordered = order_by_dependencies(&services_to_start);

    let project_name = get_project_name(&config_path);

    for name in ordered {
        if let Some(service_config) = config.services.get(&name) {
            start_service(
                &runtime,
                &project_name,
                &name,
                service_config,
                detach,
                force,
                verbose,
            )?;
        }
    }

    println!();
    UI::success("All services started");

    Ok(())
}

/// Handle services stop command
pub async fn handle_stop(services: Option<Vec<String>>, verbose: bool) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let config_path = find_vx_config(&current_dir)?;
    let config = parse_vx_config(&config_path)?;

    if config.services.is_empty() {
        UI::warn("No services defined in .vx.toml");
        return Ok(());
    }

    let runtime = ContainerRuntime::detect().ok_or_else(|| {
        anyhow::anyhow!("No container runtime found. Please install Docker or Podman.")
    })?;

    UI::header("🛑 Stopping Services");
    println!();

    let project_name = get_project_name(&config_path);

    // Filter services if specified
    let services_to_stop: Vec<_> = if let Some(names) = services {
        config
            .services
            .keys()
            .filter(|name| names.contains(name))
            .cloned()
            .collect()
    } else {
        config.services.keys().cloned().collect()
    };

    // Stop in reverse dependency order
    let ordered: Vec<_> = order_by_dependencies(
        &config
            .services
            .iter()
            .filter(|(name, _)| services_to_stop.contains(name))
            .collect::<Vec<_>>(),
    )
    .into_iter()
    .rev()
    .collect();

    for name in ordered {
        stop_service(&runtime, &project_name, &name, verbose)?;
    }

    println!();
    UI::success("All services stopped");

    Ok(())
}

/// Handle services status command
pub async fn handle_status(verbose: bool) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let config_path = find_vx_config(&current_dir)?;
    let config = parse_vx_config(&config_path)?;

    if config.services.is_empty() {
        UI::warn("No services defined in .vx.toml");
        return Ok(());
    }

    let runtime = ContainerRuntime::detect().ok_or_else(|| {
        anyhow::anyhow!("No container runtime found. Please install Docker or Podman.")
    })?;

    UI::header("📊 Service Status");
    println!();

    let project_name = get_project_name(&config_path);

    let mut any_running = false;

    for (name, service_config) in &config.services {
        let status = get_service_status(&runtime, &project_name, name)?;

        let status_icon = if status.running {
            any_running = true;
            "🟢"
        } else {
            "⚪"
        };

        let image = service_config.image.as_deref().unwrap_or("(command)");

        if status.running {
            println!("  {} {} ({})", status_icon, name, image);
            if !status.ports.is_empty() {
                println!("     Ports: {}", status.ports.join(", "));
            }
            if let Some(health) = &status.health {
                println!("     Health: {}", health);
            }
            if verbose {
                if let Some(id) = &status.container_id {
                    println!("     Container: {}", id);
                }
            }
        } else {
            println!("  {} {} ({})", status_icon, name, image);
        }
    }

    println!();
    if any_running {
        UI::info("Use 'vx services logs <service>' to view logs");
    } else {
        UI::info("Use 'vx services start' to start services");
    }

    Ok(())
}

/// Handle services logs command
pub async fn handle_logs(service: &str, follow: bool, tail: Option<usize>) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let config_path = find_vx_config(&current_dir)?;
    let config = parse_vx_config(&config_path)?;

    if !config.services.contains_key(service) {
        let available: Vec<_> = config.services.keys().collect();
        return Err(anyhow::anyhow!(
            "Service '{}' not found. Available: {}",
            service,
            available
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    let runtime = ContainerRuntime::detect().ok_or_else(|| {
        anyhow::anyhow!("No container runtime found. Please install Docker or Podman.")
    })?;

    let project_name = get_project_name(&config_path);
    let container_name = format!("vx-{}-{}", project_name, service);

    let mut args = vec!["logs".to_string()];

    if follow {
        args.push("-f".to_string());
    }

    if let Some(n) = tail {
        args.push("--tail".to_string());
        args.push(n.to_string());
    }

    args.push(container_name);

    let status = Command::new(runtime.command())
        .args(&args)
        .status()
        .context("Failed to get logs")?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "Failed to get logs for service '{}'",
            service
        ));
    }

    Ok(())
}

/// Handle services restart command
pub async fn handle_restart(services: Option<Vec<String>>, verbose: bool) -> Result<()> {
    handle_stop(services.clone(), verbose).await?;
    handle_start(services, true, false, verbose).await?;
    Ok(())
}

// ============================================
// Helper functions
// ============================================

fn find_vx_config(start_dir: &Path) -> Result<std::path::PathBuf> {
    find_config_file_upward(start_dir)
        .ok_or_else(|| anyhow::anyhow!("No vx.toml found. Run 'vx init' to create one."))
}

fn parse_vx_config(path: &Path) -> Result<VxConfig> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    parse_config(&content).context("Failed to parse .vx.toml")
}

fn get_project_name(config_path: &Path) -> String {
    config_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("vx")
        .to_string()
        .replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "-")
        .to_lowercase()
}

fn order_by_dependencies(services: &[(&String, &ServiceConfig)]) -> Vec<String> {
    // Simple topological sort
    let mut result = Vec::new();
    let mut visited = std::collections::HashSet::new();
    let service_map: HashMap<_, _> = services.iter().map(|(k, v)| (k.as_str(), *v)).collect();

    fn visit(
        name: &str,
        service_map: &HashMap<&str, &ServiceConfig>,
        visited: &mut std::collections::HashSet<String>,
        result: &mut Vec<String>,
    ) {
        if visited.contains(name) {
            return;
        }
        visited.insert(name.to_string());

        if let Some(config) = service_map.get(name) {
            for dep in &config.depends_on {
                visit(dep, service_map, visited, result);
            }
        }

        result.push(name.to_string());
    }

    for (name, _) in services {
        visit(name, &service_map, &mut visited, &mut result);
    }

    result
}

fn start_service(
    runtime: &ContainerRuntime,
    project_name: &str,
    name: &str,
    config: &ServiceConfig,
    detach: bool,
    force: bool,
    verbose: bool,
) -> Result<()> {
    let container_name = format!("vx-{}-{}", project_name, name);

    // Check if already running
    let status = get_service_status(runtime, project_name, name)?;
    if status.running && !force {
        UI::info(&format!("{} is already running", name));
        return Ok(());
    }

    // Stop existing container if force
    if status.running && force {
        stop_service(runtime, project_name, name, verbose)?;
    }

    // Remove existing container
    let _ = Command::new(runtime.command())
        .args(["rm", "-f", &container_name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    // Build run command
    let mut args = vec!["run".to_string()];

    if detach {
        args.push("-d".to_string());
    }

    args.push("--name".to_string());
    args.push(container_name);

    // Add ports
    for port in &config.ports {
        args.push("-p".to_string());
        args.push(port.clone());
    }

    // Add environment variables
    for (key, value) in &config.env {
        args.push("-e".to_string());
        args.push(format!("{}={}", key, value));
    }

    // Add env file
    if let Some(env_file) = &config.env_file {
        args.push("--env-file".to_string());
        args.push(env_file.clone());
    }

    // Add volumes
    for volume in &config.volumes {
        args.push("-v".to_string());
        args.push(volume.clone());
    }

    // Add working directory
    if let Some(working_dir) = &config.working_dir {
        args.push("-w".to_string());
        args.push(working_dir.clone());
    }

    // Add healthcheck
    if let Some(healthcheck) = &config.healthcheck {
        args.push("--health-cmd".to_string());
        args.push(healthcheck.clone());
        args.push("--health-interval".to_string());
        args.push("10s".to_string());
        args.push("--health-timeout".to_string());
        args.push("5s".to_string());
        args.push("--health-retries".to_string());
        args.push("3".to_string());
    }

    // Add image or command
    if let Some(image) = &config.image {
        args.push(image.clone());
    } else if let Some(command) = &config.command {
        // For command-based services, we need a base image
        args.push("alpine:latest".to_string());
        args.push("sh".to_string());
        args.push("-c".to_string());
        args.push(command.clone());
    } else {
        return Err(anyhow::anyhow!(
            "Service '{}' must have either 'image' or 'command'",
            name
        ));
    }

    if verbose {
        UI::info(&format!(
            "Running: {} {}",
            runtime.command(),
            args.join(" ")
        ));
    }

    UI::info(&format!("Starting {}...", name));

    let output = Command::new(runtime.command())
        .args(&args)
        .output()
        .context("Failed to start container")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to start {}: {}", name, stderr));
    }

    UI::success(&format!("{} started", name));

    Ok(())
}

fn stop_service(
    runtime: &ContainerRuntime,
    project_name: &str,
    name: &str,
    verbose: bool,
) -> Result<()> {
    let container_name = format!("vx-{}-{}", project_name, name);

    if verbose {
        UI::info(&format!("Stopping container: {}", container_name));
    }

    UI::info(&format!("Stopping {}...", name));

    let status = Command::new(runtime.command())
        .args(["stop", &container_name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match status {
        Ok(s) if s.success() => {
            UI::success(&format!("{} stopped", name));
        }
        _ => {
            if verbose {
                UI::warn(&format!("{} was not running", name));
            }
        }
    }

    // Remove container
    let _ = Command::new(runtime.command())
        .args(["rm", "-f", &container_name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    Ok(())
}

fn get_service_status(
    runtime: &ContainerRuntime,
    project_name: &str,
    name: &str,
) -> Result<ServiceStatus> {
    let container_name = format!("vx-{}-{}", project_name, name);

    let output = Command::new(runtime.command())
        .args([
            "inspect",
            "--format",
            "{{.State.Running}}|{{.Id}}|{{range .NetworkSettings.Ports}}{{.}}{{end}}|{{.State.Health.Status}}",
            &container_name,
        ])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let parts: Vec<_> = stdout.trim().split('|').collect();

            let running = parts.first().map(|s| *s == "true").unwrap_or(false);
            let container_id = parts.get(1).map(|s| s[..12.min(s.len())].to_string());
            let health = parts.get(3).and_then(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(s.to_string())
                }
            });

            Ok(ServiceStatus {
                name: name.to_string(),
                running,
                container_id,
                ports: vec![], // TODO: parse ports
                health,
            })
        }
        _ => Ok(ServiceStatus {
            name: name.to_string(),
            running: false,
            container_id: None,
            ports: vec![],
            health: None,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_runtime_command() {
        assert_eq!(ContainerRuntime::Docker.command(), "docker");
        assert_eq!(ContainerRuntime::Podman.command(), "podman");
    }

    #[test]
    fn test_get_project_name() {
        let path = PathBuf::from("/home/user/my-project/.vx.toml");
        assert_eq!(get_project_name(&path), "my-project");

        let path = PathBuf::from("/home/user/My Project/.vx.toml");
        assert_eq!(get_project_name(&path), "my-project");
    }

    #[test]
    fn test_order_by_dependencies() {
        let postgres = ServiceConfig {
            image: Some("postgres:15".to_string()),
            ..Default::default()
        };
        let app = ServiceConfig {
            image: Some("app:latest".to_string()),
            depends_on: vec!["postgres".to_string()],
            ..Default::default()
        };

        let postgres_name = "postgres".to_string();
        let app_name = "app".to_string();
        let services: Vec<(&String, &ServiceConfig)> =
            vec![(&app_name, &app), (&postgres_name, &postgres)];

        let ordered = order_by_dependencies(&services);
        assert_eq!(ordered, vec!["postgres", "app"]);
    }
}
