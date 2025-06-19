//! Smart execution demo
//!
//! This example demonstrates the intelligent automatic dependency resolution
//! system in action, showing how `vx yarn install` automatically installs
//! Node.js if needed, with beautiful progress bars and optimized performance.

use std::path::PathBuf;
use tokio;
use vx_core::{ExecutionOptions, ExecutionResult, SmartExecutor};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 vx Smart Execution Demo");
    println!("==========================\n");

    // Create the smart executor
    let executor = SmartExecutor::new().await?;

    // Demo 1: Basic tool execution with auto-dependency resolution
    println!("📦 Demo 1: Running 'yarn install' with auto-dependency resolution");
    println!("   This will automatically install Node.js if it's not available\n");

    let result = demo_yarn_install(&executor).await;
    print_execution_result("yarn install", result);

    println!("\n" + "=".repeat(60).as_str() + "\n");

    // Demo 2: PNPM with dependency resolution
    println!("📦 Demo 2: Running 'pnpm install' with auto-dependency resolution");
    println!("   This will automatically install Node.js if it's not available\n");

    let result = demo_pnpm_install(&executor).await;
    print_execution_result("pnpm install", result);

    println!("\n" + "=".repeat(60).as_str() + "\n");

    // Demo 3: Go tool execution (no dependencies)
    println!("🔧 Demo 3: Running 'go version' (independent tool)");
    println!("   Go has no dependencies, so this should be fast\n");

    let result = demo_go_version(&executor).await;
    print_execution_result("go version", result);

    println!("\n" + "=".repeat(60).as_str() + "\n");

    // Demo 4: System PATH execution (bypass vx)
    println!("⚡ Demo 4: Running 'node --version' using system PATH");
    println!("   This bypasses vx resolution for maximum speed\n");

    let result = demo_system_node(&executor).await;
    print_execution_result("node --version (system)", result);

    println!("\n" + "=".repeat(60).as_str() + "\n");

    // Demo 5: Performance metrics
    println!("📊 Performance Metrics:");
    let metrics = executor.get_performance_metrics().await;
    print_performance_metrics(&metrics);

    println!("\n✨ Demo completed! vx provides intelligent, zero-learning-cost tool execution.");
    println!("   Just run 'vx <tool> <command>' and vx handles the rest!");

    Ok(())
}

/// Demo: yarn install with auto-dependency resolution
async fn demo_yarn_install(
    executor: &SmartExecutor,
) -> Result<ExecutionResult, Box<dyn std::error::Error>> {
    let options = ExecutionOptions {
        working_dir: Some(PathBuf::from(".")),
        show_progress: true,
        ..Default::default()
    };

    executor
        .execute("yarn", &["install".to_string()], options)
        .await
}

/// Demo: pnpm install with auto-dependency resolution
async fn demo_pnpm_install(
    executor: &SmartExecutor,
) -> Result<ExecutionResult, Box<dyn std::error::Error>> {
    let options = ExecutionOptions {
        working_dir: Some(PathBuf::from(".")),
        show_progress: true,
        ..Default::default()
    };

    executor
        .execute("pnpm", &["install".to_string()], options)
        .await
}

/// Demo: go version (independent tool)
async fn demo_go_version(
    executor: &SmartExecutor,
) -> Result<ExecutionResult, Box<dyn std::error::Error>> {
    let options = ExecutionOptions {
        show_progress: true,
        ..Default::default()
    };

    executor
        .execute("go", &["version".to_string()], options)
        .await
}

/// Demo: node version using system PATH
async fn demo_system_node(
    executor: &SmartExecutor,
) -> Result<ExecutionResult, Box<dyn std::error::Error>> {
    let options = ExecutionOptions {
        use_system_path: true,
        skip_auto_install: true,
        show_progress: false, // Fast execution, no progress needed
        ..Default::default()
    };

    executor
        .execute("node", &["--version".to_string()], options)
        .await
}

/// Print execution result in a nice format
fn print_execution_result(
    command: &str,
    result: Result<ExecutionResult, Box<dyn std::error::Error>>,
) {
    match result {
        Ok(result) => {
            println!("✅ {} completed successfully!", command);
            println!("   Exit code: {}", result.exit_code);
            println!("   Total time: {:.2}s", result.total_time.as_secs_f64());

            if result.dependencies_installed {
                println!("   🔧 Dependencies installed: {:?}", result.installed_tools);
                println!(
                    "   ⏱️  Resolution time: {:.2}s",
                    result.resolution_time.as_secs_f64()
                );
            } else {
                println!("   ⚡ No dependencies needed (fast execution)");
            }

            println!(
                "   🚀 Execution time: {:.2}s",
                result.execution_time.as_secs_f64()
            );
        }
        Err(e) => {
            println!("❌ {} failed: {}", command, e);
        }
    }
}

/// Print performance metrics
fn print_performance_metrics(metrics: &vx_core::PerformanceMetrics) {
    println!(
        "   Cache hit rate: {:.1}%",
        if metrics.availability_cache_hits + metrics.availability_cache_misses > 0 {
            (metrics.availability_cache_hits as f64
                / (metrics.availability_cache_hits + metrics.availability_cache_misses) as f64)
                * 100.0
        } else {
            0.0
        }
    );

    println!("   Download cache hits: {}", metrics.download_cache_hits);
    println!(
        "   Average resolution time: {:.2}s",
        metrics.avg_resolution_time.as_secs_f64()
    );
    println!(
        "   Average installation time: {:.2}s",
        metrics.avg_installation_time.as_secs_f64()
    );
    println!(
        "   Time saved by caching: {:.2}s",
        metrics.time_saved_by_cache.as_secs_f64()
    );
}

/// Example of the user experience:
///
/// ```bash
/// # User runs this command
/// vx yarn install
///
/// # vx automatically does:
/// # 1. 🔍 Checking if yarn is available
/// # 2. 🔗 Resolving dependencies (finds that yarn needs node)
/// # 3. 🔍 Checking if node is available
/// # 4. ⬇️  Downloading Node.js (if not available)
/// # 5. 📦 Extracting Node.js
/// # 6. 🔧 Installing Node.js
/// # 7. ✅ Verifying Node.js installation
/// # 8. ⬇️  Downloading yarn (if not available)
/// # 9. 📦 Installing yarn
/// # 10. 🚀 Running 'yarn install'
/// #
/// # All with beautiful progress bars and optimal performance!
/// ```
///
/// The key benefits:
///
/// 1. **Zero Learning Cost**: Users just run `vx <tool> <command>`
/// 2. **Intelligent Dependencies**: Automatically installs what's needed
/// 3. **Beautiful Progress**: Detailed progress bars for all operations
/// 4. **Optimized Performance**:
///    - Caching for repeated operations
///    - Parallel downloads and installations
///    - Incremental dependency checks
///    - Smart tool detection
/// 5. **Transparent Execution**: Tool runs exactly as if installed normally
///
/// Example execution flow:
///
/// ```
/// $ vx yarn install
/// 🔧 vx - Universal Tool Manager
///
/// 🔍 yarn Checking availability
/// [████████████████████████████████████████] 100%
/// ⏱️  ETA: 0s
/// ⏰ Elapsed: 0.1s
///
/// 🔗 yarn Resolving dependencies  
/// [████████████████████████████████████████] 100%
/// Found dependency: node >=16.0.0
/// ⏰ Elapsed: 0.2s
///
/// ⬇️  node Downloading
/// [██████████████████████░░░░░░░░░░░░░░░░░░░░] 65%
/// Downloading Node.js v20.11.0 for linux-x64
/// ⏱️  ETA: 3s
/// ⏰ Elapsed: 2.1s
///
/// 📦 node Extracting
/// [████████████████████████████████████████] 100%
/// Extracting to ~/.vx/tools/node/20.11.0
/// ⏰ Elapsed: 2.8s
///
/// ✅ node Completed in 3.2s
///
/// 🚀 yarn Ready to execute
/// [████████████████████████████████████████] 100%
/// ⏰ Elapsed: 3.5s
///
/// yarn install v1.22.19
/// info No lockfile found.
/// info Reading package.json...
/// [1/4] 🔍  Resolving packages...
/// [2/4] 🚚  Fetching packages...
/// [3/4] 🔗  Linking dependencies...
/// [4/4] 🔨  Building fresh packages...
/// ✨  Done in 2.34s.
/// ```
fn main() {
    println!("This is a demo file showing vx smart execution capabilities");
}
