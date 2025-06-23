#!/usr/bin/env cargo +nightly -Zscript
//! CI Benchmark Check Script
//! 
//! This script runs performance benchmarks and checks against baselines.
//! If any benchmark exceeds the baseline, the CI will fail.

use std::env;
use std::path::PathBuf;
use std::process::{Command, exit};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting CI Benchmark Check");
    
    // Get environment variables
    let workspace_root = env::var("GITHUB_WORKSPACE")
        .or_else(|_| env::current_dir().map(|p| p.to_string_lossy().to_string()))?;
    let commit_hash = env::var("GITHUB_SHA").ok();
    let pr_number = env::var("GITHUB_PR_NUMBER").ok();
    
    println!("📁 Workspace: {}", workspace_root);
    if let Some(ref hash) = commit_hash {
        println!("🔗 Commit: {}", hash);
    }
    if let Some(ref pr) = pr_number {
        println!("🔀 PR: #{}", pr);
    }
    
    let workspace_path = PathBuf::from(&workspace_root);
    let baseline_path = workspace_path.join("benchmarks").join("baseline.json");
    let results_dir = workspace_path.join("benchmarks").join("results");
    
    // Ensure benchmark directories exist
    std::fs::create_dir_all(&results_dir)?;
    
    // Run performance benchmarks
    println!("⚡ Running performance benchmarks...");
    let benchmark_output = Command::new("cargo")
        .args(&["test", "--test", "comprehensive_test", "test_performance_benchmarks", "--", "--nocapture"])
        .current_dir(&workspace_path)
        .output()?;
    
    if !benchmark_output.status.success() {
        eprintln!("❌ Benchmark tests failed to run");
        eprintln!("stdout: {}", String::from_utf8_lossy(&benchmark_output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&benchmark_output.stderr));
        exit(1);
    }
    
    println!("✅ Benchmark tests completed");
    
    // Find the latest benchmark results file
    let latest_results = find_latest_benchmark_file(&results_dir)?;
    
    if let Some(results_file) = latest_results {
        println!("📊 Found benchmark results: {}", results_file.display());
        
        // Load and analyze results
        let analysis_result = analyze_benchmark_results(&results_file, &baseline_path)?;
        
        // Generate report
        generate_ci_report(&analysis_result)?;
        
        // Check if CI should fail
        if !analysis_result.passed {
            println!("❌ CI FAILED: Benchmark performance regression detected");
            exit(1);
        } else if analysis_result.should_warn {
            println!("⚠️  CI WARNING: Some benchmarks have warnings");
        } else {
            println!("✅ CI PASSED: All benchmarks within baseline");
        }
    } else {
        eprintln!("❌ No benchmark results found");
        exit(1);
    }
    
    Ok(())
}

fn find_latest_benchmark_file(results_dir: &PathBuf) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    if !results_dir.exists() {
        return Ok(None);
    }
    
    let mut latest_file: Option<PathBuf> = None;
    let mut latest_time = std::time::SystemTime::UNIX_EPOCH;
    
    for entry in std::fs::read_dir(results_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && 
           path.file_name()
               .and_then(|n| n.to_str())
               .map(|s| s.starts_with("benchmark_results_") && s.ends_with(".json"))
               .unwrap_or(false) {
            
            let metadata = entry.metadata()?;
            let modified = metadata.modified()?;
            
            if modified > latest_time {
                latest_time = modified;
                latest_file = Some(path);
            }
        }
    }
    
    Ok(latest_file)
}

fn analyze_benchmark_results(
    results_file: &PathBuf, 
    baseline_path: &PathBuf
) -> Result<CIAnalysisResult, Box<dyn std::error::Error>> {
    // This would use the vx-benchmark crate in a real implementation
    // For now, we'll simulate the analysis
    
    let results_content = std::fs::read_to_string(results_file)?;
    let results: Vec<serde_json::Value> = serde_json::from_str(&results_content)?;
    
    let mut passed = true;
    let mut should_warn = false;
    let mut failed_count = 0;
    let mut warning_count = 0;
    
    // Load baseline if exists
    let baseline = if baseline_path.exists() {
        let baseline_content = std::fs::read_to_string(baseline_path)?;
        Some(serde_json::from_str::<serde_json::Value>(&baseline_content)?)
    } else {
        None
    };
    
    for result in &results {
        let operation = result["operation"].as_str().unwrap_or("unknown");
        let tool = result["tool"].as_str().unwrap_or("unknown");
        let duration_ms = result["duration_ms"].as_u64().unwrap_or(0);
        let success = result["success"].as_bool().unwrap_or(false);
        
        if !success {
            failed_count += 1;
            passed = false;
            continue;
        }
        
        // Check against baseline
        if let Some(ref baseline_data) = baseline {
            let key = format!("{}_{}", operation, tool);
            if let Some(baseline_op) = baseline_data["baselines"][&key].as_object() {
                let max_duration = baseline_op["max_duration_ms"].as_u64().unwrap_or(u64::MAX);
                let percentile_95 = baseline_op["percentile_95_ms"].as_u64().unwrap_or(u64::MAX);
                
                if duration_ms > max_duration {
                    failed_count += 1;
                    passed = false;
                } else if duration_ms > percentile_95 {
                    warning_count += 1;
                    should_warn = true;
                }
            } else {
                // No baseline for this operation
                should_warn = true;
            }
        } else {
            // No baseline file
            should_warn = true;
        }
    }
    
    Ok(CIAnalysisResult {
        passed,
        should_warn,
        total_tests: results.len(),
        failed_count,
        warning_count,
        results_file: results_file.clone(),
    })
}

fn generate_ci_report(analysis: &CIAnalysisResult) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 CI Benchmark Report");
    println!("======================");
    println!("Total Tests: {}", analysis.total_tests);
    println!("Failed: {}", analysis.failed_count);
    println!("Warnings: {}", analysis.warning_count);
    println!("Passed: {}", analysis.total_tests - analysis.failed_count);
    
    let status = if analysis.passed {
        "✅ PASS"
    } else {
        "❌ FAIL"
    };
    println!("Status: {}", status);
    
    // Set GitHub Actions output if running in CI
    if env::var("GITHUB_ACTIONS").is_ok() {
        if let Ok(github_output) = env::var("GITHUB_OUTPUT") {
            let output_content = format!(
                "benchmark_status={}\nbenchmark_passed={}\nbenchmark_failed={}\nbenchmark_warnings={}\n",
                if analysis.passed { "pass" } else { "fail" },
                analysis.total_tests - analysis.failed_count,
                analysis.failed_count,
                analysis.warning_count
            );
            std::fs::write(github_output, output_content)?;
        }
    }
    
    Ok(())
}

#[derive(Debug)]
struct CIAnalysisResult {
    passed: bool,
    should_warn: bool,
    total_tests: usize,
    failed_count: usize,
    warning_count: usize,
    results_file: PathBuf,
}

// Minimal serde support for the script
mod serde_json {
    pub use ::serde_json::*;
}
