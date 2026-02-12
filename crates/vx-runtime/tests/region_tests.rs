//! Tests for the region detection module
//!
//! These tests verify region detection logic including:
//! - Environment variable overrides (VX_MIRROR_REGION, VX_CDN)
//! - CI environment detection
//! - China environment heuristics (locale, timezone)
//! - Region string representation

use vx_runtime::region::{self, Region};

// ============================================================================
// Region enum tests
// ============================================================================

#[test]
fn test_region_as_str_china() {
    assert_eq!(Region::China.as_str(), "cn");
}

#[test]
fn test_region_as_str_global() {
    assert_eq!(Region::Global.as_str(), "global");
}

#[test]
fn test_region_equality() {
    assert_eq!(Region::China, Region::China);
    assert_eq!(Region::Global, Region::Global);
    assert_ne!(Region::China, Region::Global);
}

#[test]
fn test_region_clone() {
    let r = Region::China;
    let r2 = r;
    assert_eq!(r, r2);
}

// ============================================================================
// CI environment detection tests
// ============================================================================

#[test]
fn test_is_ci_environment_github_actions() {
    // In CI (where these tests run), CI env var is usually set
    // Just test the function doesn't panic
    let _ = region::is_ci_environment();
}

// ============================================================================
// China environment detection tests
// ============================================================================

#[test]
fn test_is_china_environment_returns_bool() {
    // Just ensure no panic; actual result depends on the system
    let _ = region::is_china_environment();
}

// ============================================================================
// detect_region() integration tests
// ============================================================================

#[test]
fn test_detect_region_returns_valid_region() {
    let region = region::detect_region();
    // Should be either China or Global
    assert!(region == Region::China || region == Region::Global);
}

/// Test that VX_MIRROR_REGION=cn returns China
///
/// NOTE: This test manipulates global environment variables and may interfere
/// with parallel tests. Run with `--test-threads=1` or separately.
#[test]
#[ignore = "Modifies global env vars - run with --test-threads=1"]
fn test_detect_region_with_vx_mirror_region_cn() {
    // Save and set
    let prev = std::env::var("VX_MIRROR_REGION").ok();
    let prev_cdn = std::env::var("VX_CDN").ok();
    std::env::remove_var("VX_CDN");
    std::env::set_var("VX_MIRROR_REGION", "cn");

    let region = region::detect_region();
    assert_eq!(region, Region::China);

    // Restore
    std::env::remove_var("VX_MIRROR_REGION");
    if let Some(v) = prev {
        std::env::set_var("VX_MIRROR_REGION", v);
    }
    if let Some(v) = prev_cdn {
        std::env::set_var("VX_CDN", v);
    }
}

/// Test that VX_MIRROR_REGION=china returns China
///
/// NOTE: This test manipulates global environment variables and may interfere
/// with parallel tests. Run with `--test-threads=1` or separately.
#[test]
#[ignore = "Modifies global env vars - run with --test-threads-1"]
fn test_detect_region_with_vx_mirror_region_china() {
    let prev = std::env::var("VX_MIRROR_REGION").ok();
    let prev_cdn = std::env::var("VX_CDN").ok();
    std::env::remove_var("VX_CDN");
    std::env::set_var("VX_MIRROR_REGION", "china");

    let region = region::detect_region();
    assert_eq!(region, Region::China);

    std::env::remove_var("VX_MIRROR_REGION");
    if let Some(v) = prev {
        std::env::set_var("VX_MIRROR_REGION", v);
    }
    if let Some(v) = prev_cdn {
        std::env::set_var("VX_CDN", v);
    }
}

/// Test that VX_MIRROR_REGION with non-CN value returns Global
///
/// NOTE: This test manipulates global environment variables and may interfere
/// with parallel tests. Run with `--test-threads=1` or separately.
#[test]
#[ignore = "Modifies global env vars - run with --test-threads=1"]
fn test_detect_region_with_vx_mirror_region_global() {
    let prev = std::env::var("VX_MIRROR_REGION").ok();
    let prev_cdn = std::env::var("VX_CDN").ok();
    std::env::remove_var("VX_CDN");
    std::env::set_var("VX_MIRROR_REGION", "us");

    let region = region::detect_region();
    assert_eq!(region, Region::Global);

    std::env::remove_var("VX_MIRROR_REGION");
    if let Some(v) = prev {
        std::env::set_var("VX_MIRROR_REGION", v);
    }
    if let Some(v) = prev_cdn {
        std::env::set_var("VX_CDN", v);
    }
}

/// Test that VX_CDN=1 implies China region
///
/// NOTE: This test manipulates global environment variables and may interfere
/// with parallel tests. Run with `--test-threads=1` or separately.
#[test]
#[ignore = "Modifies global env vars - run with --test-threads=1"]
fn test_detect_region_vx_cdn_1_implies_china() {
    let prev_region = std::env::var("VX_MIRROR_REGION").ok();
    let prev_cdn = std::env::var("VX_CDN").ok();
    std::env::remove_var("VX_MIRROR_REGION");
    std::env::set_var("VX_CDN", "1");

    let region = region::detect_region();
    assert_eq!(region, Region::China);

    std::env::remove_var("VX_CDN");
    if let Some(v) = prev_region {
        std::env::set_var("VX_MIRROR_REGION", v);
    }
    if let Some(v) = prev_cdn {
        std::env::set_var("VX_CDN", v);
    }
}

/// Test that VX_CDN=0 implies Global region
///
/// NOTE: This test manipulates global environment variables and may interfere
/// with parallel tests. Run with `--test-threads=1` or separately.
#[test]
#[ignore = "Modifies global env vars - run with --test-threads=1"]
fn test_detect_region_vx_cdn_0_implies_global() {
    let prev_region = std::env::var("VX_MIRROR_REGION").ok();
    let prev_cdn = std::env::var("VX_CDN").ok();
    std::env::remove_var("VX_MIRROR_REGION");
    std::env::set_var("VX_CDN", "0");

    let region = region::detect_region();
    assert_eq!(region, Region::Global);

    std::env::remove_var("VX_CDN");
    if let Some(v) = prev_region {
        std::env::set_var("VX_MIRROR_REGION", v);
    }
    if let Some(v) = prev_cdn {
        std::env::set_var("VX_CDN", v);
    }
}

/// Test that VX_MIRROR_REGION takes priority over VX_CDN
///
/// NOTE: This test manipulates global environment variables and may interfere
/// with parallel tests. Run with `--test-threads=1` or separately.
#[test]
#[ignore = "Modifies global env vars - run with --test-threads=1"]
fn test_vx_mirror_region_takes_priority_over_vx_cdn() {
    let prev_region = std::env::var("VX_MIRROR_REGION").ok();
    let prev_cdn = std::env::var("VX_CDN").ok();

    // VX_MIRROR_REGION=cn should win even if VX_CDN=0
    std::env::set_var("VX_MIRROR_REGION", "cn");
    std::env::set_var("VX_CDN", "0");

    let region = region::detect_region();
    assert_eq!(region, Region::China);

    std::env::remove_var("VX_MIRROR_REGION");
    std::env::remove_var("VX_CDN");
    if let Some(v) = prev_region {
        std::env::set_var("VX_MIRROR_REGION", v);
    }
    if let Some(v) = prev_cdn {
        std::env::set_var("VX_CDN", v);
    }
}
