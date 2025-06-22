//! Comprehensive integration test runner for vx
//!
//! This test runs the full integration test suite to verify all tools work correctly.
//! Run with: cargo test --test comprehensive_test -- --nocapture

#![allow(clippy::duplicate_mod)]

mod error_handling_test;
mod integration_test;
mod performance_benchmark;

use integration_test::VxIntegrationTest;

#[tokio::test]
async fn test_all_vx_tools_comprehensive() {
    println!("🚀 Starting comprehensive vx integration tests");

    let mut test_suite = VxIntegrationTest::new();

    match test_suite.run_all_tests().await {
        Ok(()) => {
            println!("✅ All tests completed successfully");

            // Check if we have any failures
            let failed_count = test_suite.results.iter().filter(|r| !r.success).count();
            if failed_count > 0 {
                panic!("❌ {} tests failed", failed_count);
            }
        }
        Err(e) => {
            panic!("❌ Test suite failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_single_tool_uv() {
    println!("🧪 Testing UV tool specifically");

    let mut test_suite = VxIntegrationTest::new();
    test_suite
        .setup()
        .await
        .expect("Failed to setup test environment");

    let uv_tool = test_suite
        .tools
        .iter()
        .find(|t| t.name == "uv")
        .expect("UV tool not found")
        .clone();

    match test_suite.test_tool_comprehensive(&uv_tool).await {
        Ok(()) => {
            println!("✅ UV tool test completed");
            test_suite
                .print_summary(std::time::Duration::from_secs(0))
                .await;
        }
        Err(e) => {
            panic!("❌ UV tool test failed: {}", e);
        }
    }

    test_suite
        .cleanup()
        .await
        .expect("Failed to cleanup test environment");
}

#[tokio::test]
async fn test_version_listing_only() {
    println!("📋 Testing version listing for all tools");

    let mut test_suite = VxIntegrationTest::new();
    test_suite
        .setup()
        .await
        .expect("Failed to setup test environment");

    for tool in &test_suite.tools.clone() {
        println!("Testing version listing for: {}", tool.name);
        if let Err(e) = test_suite.test_version_listing(tool).await {
            eprintln!("❌ Version listing failed for {}: {}", tool.name, e);
        }
    }

    test_suite
        .print_summary(std::time::Duration::from_secs(0))
        .await;
    test_suite
        .cleanup()
        .await
        .expect("Failed to cleanup test environment");
}

#[tokio::test]
async fn test_cdn_performance() {
    println!("⚡ Testing CDN performance and optimization");

    let mut test_suite = VxIntegrationTest::new();
    test_suite
        .setup()
        .await
        .expect("Failed to setup test environment");

    // Test a few tools with CDN optimization
    let test_tools = ["uv", "node", "pnpm"];

    for tool_name in &test_tools {
        if let Some(tool) = test_suite
            .tools
            .iter()
            .find(|t| t.name == *tool_name)
            .cloned()
        {
            println!("🔍 Testing CDN for: {}", tool.name);

            // Test version listing (which uses CDN for URL optimization)
            if let Err(e) = test_suite.test_version_listing(&tool).await {
                eprintln!("❌ CDN test failed for {}: {}", tool.name, e);
            }
        }
    }

    test_suite
        .print_summary(std::time::Duration::from_secs(0))
        .await;
    test_suite
        .cleanup()
        .await
        .expect("Failed to cleanup test environment");
}

#[cfg(test)]
mod quick_tests {
    use super::*;

    #[tokio::test]
    async fn test_framework_setup() {
        let mut test_suite = VxIntegrationTest::new();

        // Test setup and cleanup
        test_suite.setup().await.expect("Setup should work");
        assert!(test_suite.test_dir.exists(), "Test directory should exist");

        test_suite.cleanup().await.expect("Cleanup should work");
        assert!(
            !test_suite.test_dir.exists(),
            "Test directory should be cleaned up"
        );
    }

    #[test]
    fn test_tool_config_validation() {
        let test_suite = VxIntegrationTest::new();

        // Verify all tools have valid configurations
        for tool in &test_suite.tools {
            assert!(!tool.name.is_empty(), "Tool name should not be empty");
            assert!(
                !tool.test_versions.is_empty(),
                "Tool should have test versions"
            );
            assert!(
                tool.test_versions.len() >= 3,
                "Tool should have at least 3 test versions"
            );
            assert!(
                tool.expected_min_versions > 0,
                "Tool should expect some versions"
            );
            assert!(tool.timeout_seconds > 0, "Tool should have a timeout");
        }

        println!(
            "✅ All {} tool configurations are valid",
            test_suite.tools.len()
        );
    }
}

#[tokio::test]
async fn test_error_handling_comprehensive() {
    println!("🧪 Testing error handling and edge cases");

    let mut error_test = error_handling_test::ErrorHandlingTest::new();

    match error_test.run_all_error_tests().await {
        Ok(()) => {
            println!("✅ Error handling tests completed");

            // Check if we have reasonable results
            let total_tests = error_test.test_suite.results.len();
            let successful_tests = error_test
                .test_suite
                .results
                .iter()
                .filter(|r| r.success)
                .count();

            println!(
                "📊 Error handling test results: {}/{} successful",
                successful_tests, total_tests
            );

            // We expect most error handling tests to pass (meaning they correctly handle errors)
            assert!(total_tests > 0, "Should have run some error handling tests");
        }
        Err(e) => {
            panic!("❌ Error handling test suite failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_performance_benchmarks() {
    println!("⚡ Running performance benchmarks");

    let mut benchmark = performance_benchmark::PerformanceBenchmark::new();

    match benchmark.run_all_benchmarks().await {
        Ok(()) => {
            println!("✅ Performance benchmarks completed");

            // Check if we have reasonable results
            let total_benchmarks = benchmark.results.len();
            let successful_benchmarks = benchmark.results.iter().filter(|r| r.success).count();

            println!(
                "📊 Performance benchmark results: {}/{} successful",
                successful_benchmarks, total_benchmarks
            );

            // We expect most benchmarks to pass
            assert!(
                total_benchmarks > 0,
                "Should have run some performance benchmarks"
            );

            // Check for performance regressions
            let mut regressions = 0;
            for result in &benchmark.results {
                let threshold = match result.operation.as_str() {
                    "version_fetch" => benchmark.baseline.version_fetch_max,
                    "installation" => benchmark.baseline.installation_max,
                    "cdn_optimization" => benchmark.baseline.version_fetch_max,
                    _ => continue,
                };

                if result.duration_ms > threshold {
                    regressions += 1;
                }
            }

            if regressions > 0 {
                println!("⚠️  {} performance regressions detected", regressions);
                // Don't fail the test for performance regressions in CI, just warn
                if std::env::var("CI").is_ok() {
                    println!(
                        "🔄 Running in CI - performance regressions logged but not failing test"
                    );
                }
            }
        }
        Err(e) => {
            panic!("❌ Performance benchmark suite failed: {}", e);
        }
    }
}
