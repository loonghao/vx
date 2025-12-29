//! Package manager detection and commands
//!
//! Detects which package manager is used in a Node.js project.

use std::path::Path;

/// Supported Node.js package managers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun,
}

impl PackageManager {
    /// Detect the package manager from lock files
    pub fn detect(root: &Path) -> Self {
        if root.join("pnpm-lock.yaml").exists() {
            PackageManager::Pnpm
        } else if root.join("yarn.lock").exists() {
            PackageManager::Yarn
        } else if root.join("bun.lockb").exists() {
            PackageManager::Bun
        } else {
            PackageManager::Npm
        }
    }

    /// Get the package manager name
    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            PackageManager::Npm => "npm",
            PackageManager::Yarn => "yarn",
            PackageManager::Pnpm => "pnpm",
            PackageManager::Bun => "bun",
        }
    }

    /// Get the run command prefix
    pub fn run_prefix(&self) -> &'static str {
        match self {
            PackageManager::Npm => "npm run",
            PackageManager::Yarn => "yarn",
            PackageManager::Pnpm => "pnpm",
            PackageManager::Bun => "bun run",
        }
    }

    /// Generate install command for a package
    #[allow(dead_code)]
    pub fn install_cmd(&self, package: &str) -> String {
        match self {
            PackageManager::Npm => format!("npm install {}", package),
            PackageManager::Yarn => format!("yarn add {}", package),
            PackageManager::Pnpm => format!("pnpm add {}", package),
            PackageManager::Bun => format!("bun add {}", package),
        }
    }

    /// Generate dev install command for a package
    #[allow(dead_code)]
    pub fn install_dev_cmd(&self, package: &str) -> String {
        match self {
            PackageManager::Npm => format!("npm install --save-dev {}", package),
            PackageManager::Yarn => format!("yarn add --dev {}", package),
            PackageManager::Pnpm => format!("pnpm add --save-dev {}", package),
            PackageManager::Bun => format!("bun add --dev {}", package),
        }
    }
}
