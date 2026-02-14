// Plugin command implementation

use crate::cli::PluginCommand;
use crate::ui::UI;
use anyhow::Result;
use vx_runtime::ProviderRegistry;

pub async fn handle(registry: &ProviderRegistry, command: PluginCommand) -> Result<()> {
    match command {
        PluginCommand::List {
            enabled: _,
            category: _,
        } => {
            UI::header("Available Providers");

            let providers = registry.providers();
            if providers.is_empty() {
                UI::warn("No providers registered");
                return Ok(());
            }

            for provider in providers {
                UI::item(&format!(
                    "📦 {} - {}",
                    provider.name(),
                    provider.description()
                ));

                // List runtimes in this provider
                for runtime in provider.runtimes() {
                    UI::detail(&format!(
                        "  ├── {} - {}",
                        runtime.name(),
                        runtime.description()
                    ));
                }
            }
        }

        PluginCommand::Info { name } => {
            UI::header(&format!("Runtime: {name}"));

            if let Some(runtime) = registry.get_runtime(&name) {
                println!("Name: {}", runtime.name());
                println!("Description: {}", runtime.description());
                println!("Ecosystem: {:?}", runtime.ecosystem());

                let aliases = runtime.aliases();
                if !aliases.is_empty() {
                    println!("Aliases: {}", aliases.join(", "));
                }

                let deps = runtime.dependencies();
                if !deps.is_empty() {
                    println!("Dependencies:");
                    for dep in deps {
                        println!("  - {}", dep.name);
                    }
                }
            } else {
                UI::error(&format!("Runtime '{}' not found", name));
            }
        }

        PluginCommand::Enable { name: _ } => {
            UI::warning("Enable/disable commands not applicable to the new provider system");
            UI::hint("All providers are automatically available");
        }

        PluginCommand::Disable { name: _ } => {
            UI::warning("Enable/disable commands not applicable to the new provider system");
            UI::hint("All providers are automatically available");
        }

        PluginCommand::Search { query } => {
            UI::header(&format!("Runtimes matching '{query}'"));

            let query_lower = query.to_lowercase();
            let mut found = false;

            for name in registry.runtime_names() {
                if name.to_lowercase().contains(&query_lower)
                    && let Some(runtime) = registry.get_runtime(&name)
                {
                    UI::item(&format!("{} - {}", name, runtime.description()));
                    found = true;
                }
            }

            if !found {
                UI::info(&format!("No runtimes found matching '{query}'"));
            }
        }

        PluginCommand::Stats => {
            UI::header("Provider Statistics");

            let providers = registry.providers();
            let total_providers = providers.len();
            let total_runtimes = registry.runtime_names().len();

            println!("  Total providers: {}", total_providers);
            println!("  Total runtimes: {}", total_runtimes);

            println!("\n  Providers:");
            for provider in providers {
                let runtime_count = provider.runtimes().len();
                println!("    {} ({} runtimes)", provider.name(), runtime_count);
            }
        }
    }

    Ok(())
}
