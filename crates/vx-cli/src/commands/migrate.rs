// Migrate command implementation
//
// Unified migration command using vx-migration framework.

use crate::ui::UI;
use anyhow::Result;
use std::env;
use std::path::{Path, PathBuf};
use vx_migration::migrations::create_default_engine;
use vx_migration::prelude::*;

/// Handle migrate command
pub async fn handle(
    path: Option<String>,
    dry_run: bool,
    backup: bool,
    check_only: bool,
    verbose: bool,
) -> Result<()> {
    let target_path = resolve_target_path(path)?;

    if check_only {
        return handle_check(&target_path, verbose).await;
    }

    handle_migrate(&target_path, dry_run, backup, verbose).await
}

/// Check which migrations are needed
async fn handle_check(path: &Path, verbose: bool) -> Result<()> {
    UI::header("🔍 Migration Check");
    println!();

    UI::info(&format!("Checking: {}", path.display()));
    println!();

    let engine = create_default_engine();
    let needed = engine.check(path).await?;

    if needed.is_empty() {
        UI::success("No migrations needed - everything is up to date!");
        return Ok(());
    }

    UI::info(&format!("Found {} migration(s) needed:", needed.len()));
    println!();

    for (i, meta) in needed.iter().enumerate() {
        println!(
            "  {}. {} ({})",
            i + 1,
            meta.name,
            format_priority(&meta.priority)
        );
        if verbose {
            println!("     ID: {}", meta.id);
            println!("     Description: {}", meta.description);
            println!("     Category: {:?}", meta.category);
            if meta.reversible {
                println!("     Reversible: yes");
            }
            println!();
        }
    }

    println!();
    UI::hint("Run 'vx migrate' to apply these migrations");
    UI::hint("Run 'vx migrate --dry-run' to preview changes");

    Ok(())
}

/// Execute migrations
async fn handle_migrate(path: &Path, dry_run: bool, backup: bool, verbose: bool) -> Result<()> {
    UI::header("🔄 Migration");
    println!();

    UI::info(&format!("Target: {}", path.display()));
    if dry_run {
        UI::info("Mode: dry-run (no changes will be made)");
    }
    println!();

    let engine = create_default_engine();

    // First check what migrations are needed
    let needed = engine.check(path).await?;
    if needed.is_empty() {
        UI::success("No migrations needed - everything is up to date!");
        return Ok(());
    }

    UI::info(&format!("Running {} migration(s)...", needed.len()));
    println!();

    let options = MigrationOptions {
        dry_run,
        backup,
        verbose,
        rollback_on_failure: true,
        ..Default::default()
    };

    let report = engine.migrate(path, &options).await?;

    // Display results
    display_report(&report, dry_run, verbose);

    if !report.success {
        return Err(anyhow::anyhow!("Migration failed"));
    }

    Ok(())
}

/// Display migration report
fn display_report(report: &MigrationReport, dry_run: bool, verbose: bool) {
    println!();

    if verbose {
        UI::info("Migration steps:");
        for step in &report.steps {
            let status = if step.skipped {
                "⏭️  skipped"
            } else if step.error.is_some() {
                "❌ failed"
            } else {
                "✅ success"
            };

            println!(
                "  {} {} - {}",
                status, step.migration_name, step.description
            );

            if verbose && !step.result.changes.is_empty() {
                for change in &step.result.changes {
                    println!("      {:?}: {}", change.change_type, change.path.display());
                }
            }

            if let Some(error) = &step.error {
                println!("      Error: {}", error);
            }
        }
        println!();
    }

    // Summary
    UI::info("Summary:");
    println!("  • Successful: {}", report.successful_count);
    println!("  • Skipped: {}", report.skipped_count);
    println!("  • Failed: {}", report.failed_count);
    println!("  • Duration: {:.2}s", report.total_duration.as_secs_f64());
    println!();

    if !report.errors.is_empty() {
        UI::error("Errors:");
        for error in &report.errors {
            println!("  • {}", error);
        }
        println!();
    }

    if dry_run {
        UI::info("Dry run complete - no changes were made");
        UI::hint("Run 'vx migrate' without --dry-run to apply changes");
    } else if report.success {
        UI::success("Migration completed successfully!");
    } else {
        UI::error("Migration failed");
    }
}

/// Resolve target path from option or current directory
fn resolve_target_path(path: Option<String>) -> Result<PathBuf> {
    if let Some(p) = path {
        let path = PathBuf::from(p);
        if !path.exists() {
            return Err(anyhow::anyhow!("Path not found: {}", path.display()));
        }
        Ok(path)
    } else {
        Ok(env::current_dir()?)
    }
}

/// Format priority for display
fn format_priority(priority: &MigrationPriority) -> &'static str {
    match priority {
        MigrationPriority::Critical => "critical",
        MigrationPriority::High => "high",
        MigrationPriority::Normal => "normal",
        MigrationPriority::Low => "low",
        MigrationPriority::Cleanup => "cleanup",
    }
}
