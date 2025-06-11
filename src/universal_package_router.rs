// Universal package command router
// Routes package management commands to appropriate package managers

use crate::package_ecosystem::*;
use crate::package_managers_impl::*;
use anyhow::Result;

/// Universal package command router that handles all package management operations
pub struct UniversalPackageRouter {
    registry: PackageEcosystemRegistry,
}

impl UniversalPackageRouter {
    /// Create a new router with all supported package managers
    pub fn new() -> Self {
        let mut registry = PackageEcosystemRegistry::new();

        // Register JavaScript ecosystem managers
        registry.register_manager(Box::new(NpmPackageManager::new()));
        // TODO: Add PnpmPackageManager, YarnPackageManager, BunPackageManager

        // Register system package managers
        #[cfg(target_os = "macos")]
        registry.register_manager(Box::new(HomebrewPackageManager::new()));

        // Register specialized package managers
        registry.register_manager(Box::new(RezPackageManager::new()));

        // TODO: Add more package managers:
        // - Chocolatey (Windows)
        // - APT (Ubuntu/Debian)
        // - YUM/DNF (RHEL/Fedora)
        // - Spack (HPC)
        // - Conda (Python scientific)
        // - Poetry (Python)
        // - Cargo (Rust)
        // - Go modules

        Self { registry }
    }

    /// Route a package command to the appropriate package manager
    pub fn route_command(&self, args: &[String]) -> Result<()> {
        if args.is_empty() {
            return Err(anyhow::anyhow!("No command provided"));
        }

        let command = &args[0];
        let remaining_args = &args[1..];

        match command.as_str() {
            // Universal package commands (auto-detect ecosystem)
            "install" | "add" => self.handle_install_command(remaining_args),
            "remove" | "uninstall" | "rm" => self.handle_remove_command(remaining_args),
            "update" | "upgrade" => self.handle_update_command(remaining_args),
            "list" | "ls" => self.handle_list_command(remaining_args),
            "search" => self.handle_search_command(remaining_args),
            "info" => self.handle_info_command(remaining_args),

            // Ecosystem-specific commands
            "js" | "javascript" => {
                self.handle_ecosystem_command(&Ecosystem::JavaScript, remaining_args)
            }
            "py" | "python" => self.handle_ecosystem_command(&Ecosystem::Python, remaining_args),
            "rs" | "rust" => self.handle_ecosystem_command(&Ecosystem::Rust, remaining_args),
            "go" => self.handle_ecosystem_command(&Ecosystem::Go, remaining_args),
            "system" => self.handle_system_command(remaining_args),
            "vfx" => self.handle_ecosystem_command(&Ecosystem::VFX, remaining_args),
            "scientific" | "hpc" => {
                self.handle_ecosystem_command(&Ecosystem::Scientific, remaining_args)
            }

            // Direct package manager commands
            manager_name => self.handle_direct_manager_command(manager_name, remaining_args),
        }
    }

    /// Handle install/add commands with auto-detection
    fn handle_install_command(&self, args: &[String]) -> Result<()> {
        let current_dir = std::env::current_dir()?;

        if let Some(manager) = self.registry.get_preferred_manager(&current_dir) {
            let packages = self.parse_package_specs(args)?;
            manager.install_packages(&packages)?;
            println!("✅ Packages installed using {}", manager.name());
        } else {
            return Err(anyhow::anyhow!(
                "No suitable package manager found for current project. Try specifying an ecosystem: vx pkg js install <packages>"
            ));
        }

        Ok(())
    }

    /// Handle remove/uninstall commands
    fn handle_remove_command(&self, args: &[String]) -> Result<()> {
        let current_dir = std::env::current_dir()?;

        if let Some(manager) = self.registry.get_preferred_manager(&current_dir) {
            let package_names: Vec<String> = args.to_vec();
            manager.remove_packages(&package_names)?;
            println!("✅ Packages removed using {}", manager.name());
        } else {
            return Err(anyhow::anyhow!(
                "No suitable package manager found for current project"
            ));
        }

        Ok(())
    }

    /// Handle update/upgrade commands
    fn handle_update_command(&self, args: &[String]) -> Result<()> {
        let current_dir = std::env::current_dir()?;

        if let Some(manager) = self.registry.get_preferred_manager(&current_dir) {
            let package_names: Vec<String> = args.to_vec();
            manager.update_packages(&package_names)?;
            println!("✅ Packages updated using {}", manager.name());
        } else {
            return Err(anyhow::anyhow!(
                "No suitable package manager found for current project"
            ));
        }

        Ok(())
    }

    /// Handle list commands
    fn handle_list_command(&self, _args: &[String]) -> Result<()> {
        let current_dir = std::env::current_dir()?;

        if let Some(manager) = self.registry.get_preferred_manager(&current_dir) {
            let packages = manager.list_packages()?;

            if packages.is_empty() {
                println!("No packages installed");
            } else {
                println!("📦 Installed packages (using {}):", manager.name());
                for package in packages {
                    println!("  {} {}", package.name, package.version);
                    if let Some(description) = package.description {
                        println!("    {description}");
                    }
                }
            }
        } else {
            // List available ecosystems
            println!("Available ecosystems:");
            for ecosystem in self.registry.list_ecosystems() {
                println!("  {ecosystem:?}");
            }
        }

        Ok(())
    }

    /// Handle search commands
    fn handle_search_command(&self, args: &[String]) -> Result<()> {
        if args.is_empty() {
            return Err(anyhow::anyhow!("Search query required"));
        }

        let query = &args[0];
        let current_dir = std::env::current_dir()?;

        if let Some(manager) = self.registry.get_preferred_manager(&current_dir) {
            let packages = manager.search_packages(query)?;

            if packages.is_empty() {
                println!("No packages found for query: {query}");
            } else {
                println!(
                    "🔍 Search results for '{}' (using {}):",
                    query,
                    manager.name()
                );
                for package in packages.iter().take(10) {
                    // Limit to 10 results
                    println!("  {} {}", package.name, package.version);
                    if let Some(description) = &package.description {
                        println!("    {description}");
                    }
                }
                if packages.len() > 10 {
                    println!("  ... and {} more results", packages.len() - 10);
                }
            }
        } else {
            return Err(anyhow::anyhow!(
                "No suitable package manager found for current project"
            ));
        }

        Ok(())
    }

    /// Handle info commands
    fn handle_info_command(&self, args: &[String]) -> Result<()> {
        if args.is_empty() {
            return Err(anyhow::anyhow!("Package name required"));
        }

        let package_name = &args[0];
        let current_dir = std::env::current_dir()?;

        if let Some(manager) = self.registry.get_preferred_manager(&current_dir) {
            let packages = manager.search_packages(package_name)?;

            if let Some(package) = packages.iter().find(|p| p.name == *package_name) {
                println!("📋 Package information:");
                println!("  Name: {}", package.name);
                println!("  Version: {}", package.version);
                if let Some(description) = &package.description {
                    println!("  Description: {description}");
                }
                if let Some(homepage) = &package.homepage {
                    println!("  Homepage: {homepage}");
                }
                if !package.keywords.is_empty() {
                    println!("  Keywords: {}", package.keywords.join(", "));
                }
            } else {
                println!("Package '{package_name}' not found");
            }
        } else {
            return Err(anyhow::anyhow!(
                "No suitable package manager found for current project"
            ));
        }

        Ok(())
    }

    /// Handle ecosystem-specific commands
    fn handle_ecosystem_command(&self, ecosystem: &Ecosystem, args: &[String]) -> Result<()> {
        if let Some(managers) = self.registry.get_managers(ecosystem) {
            if let Some(manager) = managers.first() {
                if args.is_empty() {
                    println!("Available commands for {ecosystem:?} ecosystem:");
                    println!("  install <packages>  - Install packages");
                    println!("  remove <packages>   - Remove packages");
                    println!("  update [packages]   - Update packages");
                    println!("  list               - List installed packages");
                    println!("  search <query>     - Search for packages");
                    return Ok(());
                }

                let command = &args[0];
                let remaining_args = &args[1..];

                match command.as_str() {
                    "install" | "add" => {
                        let packages = self.parse_package_specs(remaining_args)?;
                        manager.install_packages(&packages)?;
                        println!("✅ Packages installed using {}", manager.name());
                    }
                    "remove" | "uninstall" => {
                        let package_names: Vec<String> = remaining_args.to_vec();
                        manager.remove_packages(&package_names)?;
                        println!("✅ Packages removed using {}", manager.name());
                    }
                    "update" | "upgrade" => {
                        let package_names: Vec<String> = remaining_args.to_vec();
                        manager.update_packages(&package_names)?;
                        println!("✅ Packages updated using {}", manager.name());
                    }
                    "list" => {
                        let packages = manager.list_packages()?;
                        for package in packages {
                            println!("  {} {}", package.name, package.version);
                        }
                    }
                    "search" => {
                        if let Some(query) = remaining_args.first() {
                            let packages = manager.search_packages(query)?;
                            for package in packages.iter().take(10) {
                                println!("  {} {}", package.name, package.version);
                            }
                        }
                    }
                    _ => {
                        // For non-package-manager commands, let the tool manager handle it
                        return Err(anyhow::anyhow!(
                            "Command '{}' is not a package management command for {:?} ecosystem",
                            command,
                            ecosystem
                        ));
                    }
                }
            } else {
                return Err(anyhow::anyhow!(
                    "No package managers available for {:?} ecosystem",
                    ecosystem
                ));
            }
        } else {
            return Err(anyhow::anyhow!("Ecosystem {:?} not supported", ecosystem));
        }

        Ok(())
    }

    /// Handle system package manager commands
    fn handle_system_command(&self, args: &[String]) -> Result<()> {
        let system_ecosystem = if cfg!(target_os = "macos") {
            Ecosystem::System(SystemType::MacOS)
        } else if cfg!(target_os = "windows") {
            Ecosystem::System(SystemType::Windows)
        } else {
            Ecosystem::System(SystemType::Linux(LinuxDistro::Other("unknown".to_string())))
        };

        self.handle_ecosystem_command(&system_ecosystem, args)
    }

    /// Handle direct package manager commands
    fn handle_direct_manager_command(&self, manager_name: &str, args: &[String]) -> Result<()> {
        if let Some(_manager) = self.registry.get_manager_by_name(manager_name) {
            // For direct manager commands, we pass through the arguments
            // This would typically involve executing the manager directly
            println!("Executing {manager_name} with args: {args:?}");
            // TODO: Implement direct execution
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Package manager '{}' not found",
                manager_name
            ))
        }
    }

    /// Check if a package manager with the given name is available
    pub fn has_manager(&self, name: &str) -> bool {
        self.registry.get_manager_by_name(name).is_some()
    }

    /// Parse package specifications from command line arguments
    fn parse_package_specs(&self, args: &[String]) -> Result<Vec<PackageSpec>> {
        let mut specs = Vec::new();

        for arg in args {
            let (name, version) = if arg.contains('@') {
                let parts: Vec<&str> = arg.splitn(2, '@').collect();
                (parts[0].to_string(), Some(parts[1].to_string()))
            } else {
                (arg.clone(), None)
            };

            specs.push(PackageSpec {
                name,
                version,
                source: PackageSource::Registry,
                install_options: InstallOptions::default(),
                dependencies: vec![],
            });
        }

        Ok(specs)
    }
}

impl Default for UniversalPackageRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let router = UniversalPackageRouter::new();
        // Basic test to ensure router can be created
        assert!(!router.registry.list_ecosystems().is_empty());
    }

    #[test]
    fn test_package_spec_parsing() {
        let router = UniversalPackageRouter::new();

        let specs = router
            .parse_package_specs(&["react".to_string(), "lodash@4.17.21".to_string()])
            .unwrap();

        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].name, "react");
        assert_eq!(specs[0].version, None);
        assert_eq!(specs[1].name, "lodash");
        assert_eq!(specs[1].version, Some("4.17.21".to_string()));
    }
}
