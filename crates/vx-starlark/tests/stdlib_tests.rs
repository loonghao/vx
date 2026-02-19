//! Standard library tests for vx-starlark

use vx_starlark::stdlib::{template, url, version};

// ============================================================
// template module tests
// ============================================================

#[test]
fn test_template_render_basic_substitution() {
    let mut vars = std::collections::HashMap::new();
    vars.insert("version", "20.0.0");
    vars.insert("platform", "windows");

    let result = template::render("https://example.com/{version}/tool-{platform}.zip", &vars);
    assert_eq!(result, "https://example.com/20.0.0/tool-windows.zip");
}

#[test]
fn test_template_render_multiple_occurrences() {
    let mut vars = std::collections::HashMap::new();
    vars.insert("name", "node");

    let result = template::render("{name}-{name}-{name}", &vars);
    assert_eq!(result, "node-node-node");
}

#[test]
fn test_template_render_no_substitution() {
    let vars = std::collections::HashMap::new();
    let result = template::render("https://example.com/static/file.zip", &vars);
    assert_eq!(result, "https://example.com/static/file.zip");
}

#[test]
fn test_template_render_with_platform() {
    let result = template::render_with_platform(
        "https://example.com/{version}/{platform}/tool.tar.gz",
        "linux",
        "x64",
        "20.0.0",
    );
    assert_eq!(result, "https://example.com/20.0.0/linux-x64/tool.tar.gz");
}

#[test]
fn test_template_render_with_vversion() {
    let result = template::render_with_platform(
        "https://example.com/releases/{vversion}/tool.tar.gz",
        "linux",
        "x64",
        "20.0.0",
    );
    assert_eq!(result, "https://example.com/releases/v20.0.0/tool.tar.gz");
}

// ============================================================
// version module tests
// ============================================================

#[test]
fn test_version_compare_equal() {
    assert_eq!(version::compare("1.0.0", "1.0.0"), Some(0));
    assert_eq!(version::compare("20.0.0", "20.0.0"), Some(0));
}

#[test]
fn test_version_compare_greater() {
    assert_eq!(version::compare("1.0.1", "1.0.0"), Some(1));
    assert_eq!(version::compare("2.0.0", "1.9.9"), Some(1));
    assert_eq!(version::compare("1.1.0", "1.0.9"), Some(1));
}

#[test]
fn test_version_compare_less() {
    assert_eq!(version::compare("1.0.0", "1.0.1"), Some(-1));
    assert_eq!(version::compare("1.9.9", "2.0.0"), Some(-1));
}

#[test]
fn test_version_compare_with_v_prefix() {
    // v prefix should be stripped
    assert_eq!(version::compare("v1.0.0", "1.0.0"), Some(0));
    assert_eq!(version::compare("v2.0.0", "v1.0.0"), Some(1));
}

#[test]
fn test_version_gt_lt_eq() {
    assert!(version::gt("1.0.1", "1.0.0"));
    assert!(!version::gt("1.0.0", "1.0.1"));
    assert!(!version::gt("1.0.0", "1.0.0"));

    assert!(version::lt("1.0.0", "1.0.1"));
    assert!(!version::lt("1.0.1", "1.0.0"));
    assert!(!version::lt("1.0.0", "1.0.0"));

    assert!(version::eq("1.0.0", "1.0.0"));
    assert!(!version::eq("1.0.0", "1.0.1"));
}

#[test]
fn test_version_gte_lte() {
    assert!(version::gte("1.0.1", "1.0.0"));
    assert!(version::gte("1.0.0", "1.0.0"));
    assert!(!version::gte("1.0.0", "1.0.1"));

    assert!(version::lte("1.0.0", "1.0.1"));
    assert!(version::lte("1.0.0", "1.0.0"));
    assert!(!version::lte("1.0.1", "1.0.0"));
}

#[test]
fn test_version_strip_v_prefix() {
    assert_eq!(version::strip_v_prefix("v1.0.0"), "1.0.0");
    assert_eq!(version::strip_v_prefix("1.0.0"), "1.0.0");
    assert_eq!(version::strip_v_prefix("v20.11.1"), "20.11.1");
}

#[test]
fn test_version_compare_with_prerelease() {
    // Pre-release versions: numeric part only
    assert_eq!(version::compare("1.0.0-rc1", "1.0.0"), Some(0));
}

// ============================================================
// url module tests
// ============================================================

#[test]
fn test_url_filename_basic() {
    assert_eq!(
        url::filename("https://example.com/path/to/file.zip"),
        Some("file.zip")
    );
}

#[test]
fn test_url_filename_with_query() {
    // rsplit('/') will include query string in filename
    assert_eq!(
        url::filename("https://example.com/file.zip?token=abc"),
        Some("file.zip?token=abc")
    );
}

#[test]
fn test_url_filename_trailing_slash() {
    // Trailing slash returns empty string
    let result = url::filename("https://example.com/path/");
    assert_eq!(result, Some(""));
}

#[test]
fn test_url_host_https() {
    assert_eq!(
        url::host("https://example.com/path/to/file"),
        Some("example.com")
    );
}

#[test]
fn test_url_host_http() {
    assert_eq!(
        url::host("http://api.github.com/repos"),
        Some("api.github.com")
    );
}

#[test]
fn test_url_host_no_scheme() {
    assert_eq!(url::host("example.com/path"), None);
}

#[test]
fn test_url_host_github() {
    assert_eq!(
        url::host("https://github.com/facebook/buck2/releases/download/v1.0.0/buck2.tar.gz"),
        Some("github.com")
    );
}
