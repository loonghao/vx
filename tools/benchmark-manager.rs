#!/usr/bin/env cargo +nightly -Zscript
//! Benchmark Management Tool
//! 
//! Usage:
//!   cargo run --bin benchmark-manager -- update-baseline results.json
//!   cargo run --bin benchmark-manager -- check-performance results.json
//!   cargo run --bin benchmark-manager -- generate-report results.json

use std::env;
use std::path::PathBuf;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }
    
    match args[1].as_str() {
        "update-baseline" => {
            if args.len() < 3 {
                eprintln!("Usage: {} update-baseline <results.json>", args[0]);
                return Ok(());
            }
            update_baseline(&args[2])?;
        }
        "check-performance" => {
            if args.len() < 3 {
                eprintln!("Usage: {} check-performance <results.json>", args[0]);
                return Ok(());
            }
            check_performance(&args[2])?;
        }
        "generate-report" => {
            if args.len() < 3 {
                eprintln!("Usage: {} generate-report <results.json>", args[0]);
                return Ok(());
            }
            generate_report(&args[2])?;
        }
        "init-baseline" => {
            init_baseline()?;
        }
        "list-baselines" => {
            list_baselines()?;
        }
        _ => {
            print_usage();
        }
    }
    
    Ok(())
}

fn print_usage() {
    println!("🔧 VX Benchmark Manager");
    println!("========================");
    println!("Commands:");
    println!("  update-baseline <results.json>  - Update performance baseline");
    println!("  check-performance <results.json> - Check against baseline");
    println!("  generate-report <results.json>   - Generate performance report");
    println!("  init-baseline                    - Initialize baseline directory");
    println!("  list-baselines                   - List available baselines");
}

fn update_baseline(results_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Updating performance baseline from: {}", results_file);
    
    let results_path = PathBuf::from(results_file);
    if !results_path.exists() {
        return Err(format!("Results file not found: {}", results_file).into());
    }
    
    // Load results
    let content = std::fs::read_to_string(&results_path)?;
    let results: Vec<serde_json::Value> = serde_json::from_str(&content)?;
    
    // Load existing baseline or create new
    let baseline_path = PathBuf::from("benchmarks/baseline.json");
    let mut baseline = if baseline_path.exists() {
        let baseline_content = std::fs::read_to_string(&baseline_path)?;
        serde_json::from_str::<serde_json::Value>(&baseline_content)?
    } else {
        serde_json::json!({
            "version": "1.0.0",
            "created_at": chrono::Utc::now().to_rfc3339(),
            "baselines": {}
        })
    };
    
    // Group results by operation and tool
    let mut operation_data: HashMap<String, Vec<u64>> = HashMap::new();
    
    for result in &results {
        if result["success"].as_bool().unwrap_or(false) {
            let operation = result["operation"].as_str().unwrap_or("unknown");
            let tool = result["tool"].as_str().unwrap_or("unknown");
            let duration_ms = result["duration_ms"].as_u64().unwrap_or(0);
            
            let key = format!("{}_{}", operation, tool);
            operation_data.entry(key).or_insert_with(Vec::new).push(duration_ms);
        }
    }
    
    // Update baselines
    for (key, durations) in operation_data {
        if durations.is_empty() {
            continue;
        }
        
        let mut sorted_durations = durations.clone();
        sorted_durations.sort_unstable();
        
        let average_ms = durations.iter().sum::<u64>() / durations.len() as u64;
        let percentile_95_ms = sorted_durations[(sorted_durations.len() as f64 * 0.95) as usize];
        let max_duration_ms = *sorted_durations.last().unwrap();
        
        // Add 20% buffer to max duration for baseline
        let baseline_max = max_duration_ms * 120 / 100;
        
        let operation_baseline = serde_json::json!({
            "operation": key.split('_').next().unwrap_or("unknown"),
            "max_duration_ms": baseline_max,
            "percentile_95_ms": percentile_95_ms,
            "average_ms": average_ms,
            "sample_count": durations.len(),
            "last_updated": chrono::Utc::now().to_rfc3339()
        });
        
        baseline["baselines"][&key] = operation_baseline;
        
        println!("✅ Updated baseline for {}: max={}ms, p95={}ms, avg={}ms", 
                 key, baseline_max, percentile_95_ms, average_ms);
    }
    
    // Save updated baseline
    std::fs::create_dir_all("benchmarks")?;
    let baseline_content = serde_json::to_string_pretty(&baseline)?;
    std::fs::write(&baseline_path, baseline_content)?;
    
    println!("💾 Baseline saved to: {}", baseline_path.display());
    
    Ok(())
}

fn check_performance(results_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Checking performance against baseline: {}", results_file);
    
    let results_path = PathBuf::from(results_file);
    if !results_path.exists() {
        return Err(format!("Results file not found: {}", results_file).into());
    }
    
    // Load results
    let content = std::fs::read_to_string(&results_path)?;
    let results: Vec<serde_json::Value> = serde_json::from_str(&content)?;
    
    // Load baseline
    let baseline_path = PathBuf::from("benchmarks/baseline.json");
    if !baseline_path.exists() {
        println!("⚠️  No baseline found. Run 'init-baseline' or 'update-baseline' first.");
        return Ok(());
    }
    
    let baseline_content = std::fs::read_to_string(&baseline_path)?;
    let baseline: serde_json::Value = serde_json::from_str(&baseline_content)?;
    let baselines = baseline["baselines"].as_object().unwrap();
    
    let mut failed_tests = 0;
    let mut warning_tests = 0;
    let mut passed_tests = 0;
    
    println!("\n📊 Performance Check Results:");
    println!("=" * 60);
    
    for result in &results {
        let operation = result["operation"].as_str().unwrap_or("unknown");
        let tool = result["tool"].as_str().unwrap_or("unknown");
        let duration_ms = result["duration_ms"].as_u64().unwrap_or(0);
        let success = result["success"].as_bool().unwrap_or(false);
        
        let key = format!("{}_{}", operation, tool);
        
        if !success {
            println!("❌ {} {}: FAILED", tool, operation);
            failed_tests += 1;
            continue;
        }
        
        if let Some(baseline_op) = baselines.get(&key) {
            let max_duration = baseline_op["max_duration_ms"].as_u64().unwrap_or(u64::MAX);
            let percentile_95 = baseline_op["percentile_95_ms"].as_u64().unwrap_or(u64::MAX);
            
            if duration_ms > max_duration {
                let deviation = ((duration_ms as f64 - max_duration as f64) / max_duration as f64) * 100.0;
                println!("❌ {} {}: {}ms > {}ms (+{:.1}%)", tool, operation, duration_ms, max_duration, deviation);
                failed_tests += 1;
            } else if duration_ms > percentile_95 {
                let deviation = ((duration_ms as f64 - percentile_95 as f64) / percentile_95 as f64) * 100.0;
                println!("⚠️  {} {}: {}ms > {}ms (+{:.1}%)", tool, operation, duration_ms, percentile_95, deviation);
                warning_tests += 1;
            } else {
                println!("✅ {} {}: {}ms (within baseline)", tool, operation, duration_ms);
                passed_tests += 1;
            }
        } else {
            println!("❓ {} {}: {}ms (no baseline)", tool, operation, duration_ms);
            warning_tests += 1;
        }
    }
    
    println!("=" * 60);
    println!("Summary: {} passed, {} warnings, {} failed", passed_tests, warning_tests, failed_tests);
    
    if failed_tests > 0 {
        println!("❌ PERFORMANCE CHECK FAILED");
        std::process::exit(1);
    } else if warning_tests > 0 {
        println!("⚠️  PERFORMANCE CHECK WARNING");
    } else {
        println!("✅ PERFORMANCE CHECK PASSED");
    }
    
    Ok(())
}

fn generate_report(results_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Generating performance report from: {}", results_file);
    
    let results_path = PathBuf::from(results_file);
    if !results_path.exists() {
        return Err(format!("Results file not found: {}", results_file).into());
    }
    
    // Load results
    let content = std::fs::read_to_string(&results_path)?;
    let results: Vec<serde_json::Value> = serde_json::from_str(&content)?;
    
    // Generate markdown report
    let mut report = String::new();
    report.push_str("# 📊 VX Performance Benchmark Report\n\n");
    report.push_str(&format!("**Generated**: {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    report.push_str(&format!("**Results File**: {}\n\n", results_file));
    
    // Summary
    let total_tests = results.len();
    let successful_tests = results.iter().filter(|r| r["success"].as_bool().unwrap_or(false)).count();
    let failed_tests = total_tests - successful_tests;
    
    report.push_str("## Summary\n\n");
    report.push_str(&format!("- **Total Tests**: {}\n", total_tests));
    report.push_str(&format!("- **Successful**: {}\n", successful_tests));
    report.push_str(&format!("- **Failed**: {}\n", failed_tests));
    report.push_str(&format!("- **Success Rate**: {:.1}%\n\n", (successful_tests as f64 / total_tests as f64) * 100.0));
    
    // Detailed results
    report.push_str("## Detailed Results\n\n");
    report.push_str("| Tool | Operation | Duration | Status | Notes |\n");
    report.push_str("|------|-----------|----------|--------|-------|\n");
    
    for result in &results {
        let tool = result["tool"].as_str().unwrap_or("unknown");
        let operation = result["operation"].as_str().unwrap_or("unknown");
        let duration_ms = result["duration_ms"].as_u64().unwrap_or(0);
        let success = result["success"].as_bool().unwrap_or(false);
        
        let status = if success { "✅ Pass" } else { "❌ Fail" };
        let version = result["version"].as_str().unwrap_or("N/A");
        
        report.push_str(&format!(
            "| {} | {} | {}ms | {} | {} |\n",
            tool, operation, duration_ms, status, version
        ));
    }
    
    // Save report
    let report_path = PathBuf::from("benchmarks/performance_report.md");
    std::fs::create_dir_all("benchmarks")?;
    std::fs::write(&report_path, report)?;
    
    println!("📄 Report saved to: {}", report_path.display());
    
    Ok(())
}

fn init_baseline() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Initializing benchmark baseline directory");
    
    std::fs::create_dir_all("benchmarks/results")?;
    std::fs::create_dir_all("benchmarks/baselines")?;
    
    // Create empty baseline if it doesn't exist
    let baseline_path = PathBuf::from("benchmarks/baseline.json");
    if !baseline_path.exists() {
        let empty_baseline = serde_json::json!({
            "version": "1.0.0",
            "created_at": chrono::Utc::now().to_rfc3339(),
            "baselines": {}
        });
        
        let content = serde_json::to_string_pretty(&empty_baseline)?;
        std::fs::write(&baseline_path, content)?;
        
        println!("📄 Created empty baseline: {}", baseline_path.display());
    }
    
    // Create .gitignore
    let gitignore_path = PathBuf::from("benchmarks/.gitignore");
    if !gitignore_path.exists() {
        let gitignore_content = "# Benchmark results (keep baseline)\nresults/\n*.tmp\n*.log\n";
        std::fs::write(&gitignore_path, gitignore_content)?;
        println!("📄 Created .gitignore: {}", gitignore_path.display());
    }
    
    println!("✅ Benchmark directory initialized");
    
    Ok(())
}

fn list_baselines() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Available Baselines");
    
    let baseline_path = PathBuf::from("benchmarks/baseline.json");
    if !baseline_path.exists() {
        println!("❌ No baseline found. Run 'init-baseline' first.");
        return Ok(());
    }
    
    let content = std::fs::read_to_string(&baseline_path)?;
    let baseline: serde_json::Value = serde_json::from_str(&content)?;
    
    println!("Version: {}", baseline["version"].as_str().unwrap_or("unknown"));
    println!("Created: {}", baseline["created_at"].as_str().unwrap_or("unknown"));
    
    if let Some(baselines) = baseline["baselines"].as_object() {
        println!("\nBaselines ({} operations):", baselines.len());
        for (key, data) in baselines {
            let max_duration = data["max_duration_ms"].as_u64().unwrap_or(0);
            let avg_duration = data["average_ms"].as_u64().unwrap_or(0);
            println!("  {} - max: {}ms, avg: {}ms", key, max_duration, avg_duration);
        }
    }
    
    Ok(())
}

// Minimal dependencies for the script
mod serde_json {
    pub use ::serde_json::*;
}

mod chrono {
    pub use ::chrono::*;
}
