//! Bridge ecosystem-native global install commands to vx global package installation.
//!
//! This enables commands such as:
//! - `vx npm install -g <pkg>`
//! - `vx pnpm add -g <pkg>`
//! - `vx yarn global add <pkg>`
//! - `vx pip install --user <pkg>`
//! - `vx cargo install <pkg>`
//! - `vx go install <pkg>@<version>`
//! - `vx gem install <pkg>`
//!
//! to participate in vx's package registry and shim generation.

use crate::commands::global::{GlobalCommand, InstallGlobalArgs};
use crate::commands::{CommandContext, global};
use crate::ui::UI;
use anyhow::Result;

/// Parsed global install request extracted from forwarded package-manager args.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpmGlobalInstallRequest {
    pub ecosystem: String,
    pub packages: Vec<String>,
    pub extra_args: Vec<String>,
    pub force: bool,
    pub verbose: bool,
}

/// Parse runtime args and detect ecosystem global-install patterns.
///
/// Returns `None` when args do not represent a global install command.
pub fn parse_global_install_bridge_args(
    runtime_name: &str,
    args: &[String],
) -> Option<NpmGlobalInstallRequest> {
    let runtime = runtime_name.to_ascii_lowercase();
    match runtime.as_str() {
        "npm" => parse_npm_like_args("npm", args, &["install", "i", "add"], true),
        "pnpm" => parse_npm_like_args("pnpm", args, &["install", "i", "add"], true),
        "yarn" => parse_yarn_global_args(args),
        "pip" => parse_pip_user_install_args(args),
        "cargo" => parse_cargo_install_args(args),
        "go" => parse_go_install_args(args),
        "gem" => parse_gem_install_args(args),
        _ => None,
    }
}

/// Backward-compatible parser kept for existing tests/callers.
pub fn parse_npm_global_install_args(args: &[String]) -> Option<NpmGlobalInstallRequest> {
    parse_global_install_bridge_args("npm", args)
}

fn parse_npm_like_args(
    ecosystem: &str,
    args: &[String],
    install_verbs: &[&str],
    require_global_flag: bool,
) -> Option<NpmGlobalInstallRequest> {
    if args.is_empty() {
        return None;
    }

    let install_idx = args
        .iter()
        .position(|arg| install_verbs.iter().any(|verb| *verb == arg.as_str()))?;

    if require_global_flag && !args.iter().any(|arg| is_global_flag(arg)) {
        return None;
    }

    let mut packages = Vec::new();
    let mut extra_args = Vec::new();
    let mut force = false;
    let mut verbose = false;

    for (idx, arg) in args.iter().enumerate() {
        if idx == install_idx {
            continue;
        }

        if is_global_flag(arg) {
            continue;
        }

        if arg == "--force" || arg == "-f" {
            force = true;
            continue;
        }

        if arg == "--verbose" || arg == "-d" {
            verbose = true;
            continue;
        }

        if idx > 0 && option_requires_value(&args[idx - 1]) {
            extra_args.push(arg.clone());
            continue;
        }

        if arg.starts_with('-') || idx < install_idx {
            extra_args.push(arg.clone());
            continue;
        }

        packages.push(arg.clone());
    }

    if packages.is_empty() {
        return None;
    }

    Some(NpmGlobalInstallRequest {
        ecosystem: ecosystem.to_string(),
        packages,
        extra_args,
        force,
        verbose,
    })
}

/// Route parsed npm global install packages into vx global package workflow.
pub async fn bridge_npm_global_install(
    ctx: &CommandContext,
    request: &NpmGlobalInstallRequest,
) -> Result<()> {
    UI::info(
        "Detected global install command. Routing to vx global package workflow for shim creation.",
    );

    for package in &request.packages {
        let install_args = InstallGlobalArgs {
            package: format!("{}:{}", request.ecosystem, package),
            force: request.force,
            verbose: request.verbose,
            extra_args: request.extra_args.clone(),
        };

        global::handle(ctx, &GlobalCommand::Install(install_args)).await?;
    }

    Ok(())
}

fn is_global_flag(arg: &str) -> bool {
    arg == "-g" || arg == "--global"
}

fn option_requires_value(option: &str) -> bool {
    matches!(
        option,
        "--registry"
            | "--cache"
            | "--prefix"
            | "--location"
            | "--userconfig"
            | "--loglevel"
            | "--workspace"
            | "-w"
            | "--tag"
            | "--before"
            | "--otp"
            | "--proxy"
            | "--https-proxy"
            | "--noproxy"
            | "--cafile"
            | "--script-shell"
            | "--include"
            | "--omit"
            | "--save"
            | "--save-exact"
            | "--save-prefix"
    )
}

fn parse_yarn_global_args(args: &[String]) -> Option<NpmGlobalInstallRequest> {
    if args.len() < 3 {
        return None;
    }

    if args.first()?.as_str() != "global" {
        return None;
    }

    let sub = args.get(1)?.as_str();
    if !matches!(sub, "add" | "install") {
        return None;
    }

    parse_npm_like_args("yarn", &args[1..], &["add", "install"], false)
}

fn parse_pip_user_install_args(args: &[String]) -> Option<NpmGlobalInstallRequest> {
    if args.is_empty() || args.first()?.as_str() != "install" {
        return None;
    }

    if !args.iter().any(|a| a == "--user") {
        return None;
    }

    let mut packages = Vec::new();
    let mut extra_args = Vec::new();
    let mut force = false;
    let mut verbose = false;

    for (idx, arg) in args.iter().enumerate() {
        if idx == 0 || arg == "--user" {
            continue;
        }

        if arg == "--force-reinstall" {
            force = true;
            continue;
        }

        if arg == "--verbose" || arg == "-v" || arg == "-vv" || arg == "-vvv" {
            verbose = true;
            continue;
        }

        if idx > 0 && option_requires_value(&args[idx - 1]) {
            extra_args.push(arg.clone());
            continue;
        }

        if arg.starts_with('-') {
            extra_args.push(arg.clone());
            continue;
        }

        packages.push(arg.clone());
    }

    if packages.is_empty() {
        return None;
    }

    Some(NpmGlobalInstallRequest {
        ecosystem: "pip".to_string(),
        packages,
        extra_args,
        force,
        verbose,
    })
}

fn parse_cargo_install_args(args: &[String]) -> Option<NpmGlobalInstallRequest> {
    if args.is_empty() || args.first()?.as_str() != "install" {
        return None;
    }

    let mut package: Option<String> = None;
    let mut version: Option<String> = None;
    let mut extra_args = Vec::new();
    let mut verbose = false;

    let mut skip_next = false;
    for (idx, arg) in args.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        if idx == 0 {
            continue;
        }

        if arg == "--verbose" || arg == "-v" {
            verbose = true;
            continue;
        }

        if arg == "--version"
            && let Some(v) = args.get(idx + 1)
        {
            version = Some(v.clone());
            skip_next = true;
            continue;
        }

        if idx > 0 && option_requires_value(&args[idx - 1]) {
            extra_args.push(arg.clone());
            continue;
        }

        if arg.starts_with('-') {
            extra_args.push(arg.clone());
            continue;
        }

        if package.is_none() {
            package = Some(arg.clone());
        } else {
            extra_args.push(arg.clone());
        }
    }

    let mut package = package?;
    if let Some(version) = version {
        package = format!("{}@{}", package, version);
    }

    Some(NpmGlobalInstallRequest {
        ecosystem: "cargo".to_string(),
        packages: vec![package],
        extra_args,
        force: false,
        verbose,
    })
}

fn parse_go_install_args(args: &[String]) -> Option<NpmGlobalInstallRequest> {
    if args.is_empty() || args.first()?.as_str() != "install" || args.len() < 2 {
        return None;
    }

    let mut packages = Vec::new();
    let mut extra_args = Vec::new();

    for (idx, arg) in args.iter().enumerate() {
        if idx == 0 {
            continue;
        }
        if arg.starts_with('-') {
            extra_args.push(arg.clone());
            continue;
        }
        packages.push(arg.clone());
    }

    if packages.is_empty() {
        return None;
    }

    Some(NpmGlobalInstallRequest {
        ecosystem: "go".to_string(),
        packages,
        extra_args,
        force: false,
        verbose: false,
    })
}

fn parse_gem_install_args(args: &[String]) -> Option<NpmGlobalInstallRequest> {
    if args.is_empty() || args.first()?.as_str() != "install" {
        return None;
    }

    let mut package: Option<String> = None;
    let mut version: Option<String> = None;
    let mut extra_args = Vec::new();
    let mut verbose = false;

    let mut skip_next = false;
    for (idx, arg) in args.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        if idx == 0 {
            continue;
        }

        if arg == "--version" || arg == "-v" {
            if let Some(v) = args.get(idx + 1) {
                version = Some(v.clone());
                skip_next = true;
            }
            continue;
        }

        if arg == "--verbose" || arg == "-V" {
            verbose = true;
            continue;
        }

        if idx > 0 && option_requires_value(&args[idx - 1]) {
            extra_args.push(arg.clone());
            continue;
        }

        if arg.starts_with('-') {
            extra_args.push(arg.clone());
            continue;
        }

        if package.is_none() {
            package = Some(arg.clone());
        } else {
            extra_args.push(arg.clone());
        }
    }

    let mut package = package?;
    if let Some(version) = version {
        package = format!("{}@{}", package, version);
    }

    Some(NpmGlobalInstallRequest {
        ecosystem: "gem".to_string(),
        packages: vec![package],
        extra_args,
        force: false,
        verbose,
    })
}
