//! Example: Analyze a project directory
//!
//! Run with: cargo run -p vx-project-analyzer --example analyze_project -- <path>

use std::path::Path;
use vx_project_analyzer::{AnalyzerConfig, ProjectAnalyzer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let root = if args.len() > 1 {
        Path::new(&args[1])
    } else {
        Path::new(".")
    };

    println!("=== Analyzing: {} ===\n", root.display());

    let analyzer = ProjectAnalyzer::new(AnalyzerConfig::default());
    let analysis = analyzer.analyze(root).await?;

    println!("📦 Detected ecosystems: {:?}", analysis.ecosystems);
    println!();

    println!("📋 Dependencies: {} found", analysis.dependencies.len());
    for dep in analysis.dependencies.iter().take(10) {
        println!(
            "    - {} ({}) [{}]",
            dep.name,
            dep.version.as_deref().unwrap_or("*"),
            dep.ecosystem
        );
    }
    if analysis.dependencies.len() > 10 {
        println!("    ... and {} more", analysis.dependencies.len() - 10);
    }
    println!();

    println!("📜 Scripts: {} found", analysis.scripts.len());
    for script in &analysis.scripts {
        println!("    - {}: `{}`", script.name, script.command);
    }
    println!();

    println!("🔧 Required tools:");
    for tool in &analysis.required_tools {
        println!("    - {}: {}", tool.name, tool.reason);
    }

    Ok(())
}
