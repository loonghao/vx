//! Unit tests for smart_detect.star — the Starlark asset scoring engine (RFC 0041).
//!
//! These tests validate the pure Starlark scoring logic via `call_stdlib_function`.
//! No real HTTP calls are made.

use serde_json::json;
use vx_starlark::engine::StarlarkEngine;

/// Build a minimal platform context dict matching what `smart_detect.star` expects.
fn ctx(os: &str, arch: &str) -> serde_json::Value {
    json!({
        "platform": {
            "os": os,
            "arch": arch,
        }
    })
}

/// Call `score_asset` in smart_detect.star with the given arguments.
fn score(
    engine: &StarlarkEngine,
    name: &str,
    ctx: &serde_json::Value,
    version: &str,
    linux_libc: &str,
) -> Option<serde_json::Value> {
    let result = engine
        .call_stdlib_function(
            "@vx//stdlib:smart_detect.star",
            "score_asset",
            &[
                json!(name),
                ctx.clone(),
                json!(version),
                json!(linux_libc),
                json!(null), // extra_excludes
            ],
        )
        .unwrap();
    if result.is_null() { None } else { Some(result) }
}

/// Call `detect_best_asset` in smart_detect.star.
fn detect(
    engine: &StarlarkEngine,
    assets: &[serde_json::Value],
    ctx: &serde_json::Value,
    version: &str,
    threshold: u64,
) -> Option<serde_json::Value> {
    let result = engine
        .call_stdlib_function(
            "@vx//stdlib:smart_detect.star",
            "detect_best_asset",
            &[
                json!(assets),
                ctx.clone(),
                json!(version),
                json!(threshold),
                json!("musl"),
                json!(null),
            ],
        )
        .unwrap();
    if result.is_null() { None } else { Some(result) }
}

// ── Hard exclusion tests ──

#[test]
fn test_exclude_checksum() {
    let engine = StarlarkEngine::new();
    let c = ctx("linux", "x64");
    assert!(
        score(
            &engine,
            "mytool-1.0.0-linux-amd64.tar.gz.sha256",
            &c,
            "1.0.0",
            "musl"
        )
        .is_none()
    );
}

#[test]
fn test_exclude_deb() {
    let engine = StarlarkEngine::new();
    let c = ctx("linux", "x64");
    assert!(score(&engine, "mytool_1.0.0_amd64.deb", &c, "1.0.0", "musl").is_none());
}

#[test]
fn test_exclude_rpm() {
    let engine = StarlarkEngine::new();
    let c = ctx("linux", "x64");
    assert!(score(&engine, "mytool-1.0.0-1.x86_64.rpm", &c, "1.0.0", "musl").is_none());
}

#[test]
fn test_exclude_msi() {
    let engine = StarlarkEngine::new();
    let c = ctx("windows", "x64");
    assert!(score(&engine, "mytool-1.0.0-x64.msi", &c, "1.0.0", "musl").is_none());
}

#[test]
fn test_exclude_sbom() {
    let engine = StarlarkEngine::new();
    let c = ctx("linux", "x64");
    assert!(score(&engine, "mytool-1.0.0-sbom.spdx.json", &c, "1.0.0", "musl").is_none());
}

#[test]
fn test_exclude_source() {
    let engine = StarlarkEngine::new();
    let c = ctx("linux", "x64");
    assert!(score(&engine, "mytool-1.0.0-src.tar.gz", &c, "1.0.0", "musl").is_none());
}

// ── Version presence test ──

#[test]
fn test_version_must_appear_in_filename() {
    let engine = StarlarkEngine::new();
    let c = ctx("linux", "x64");
    // Asset for v1.0.0 but we ask for v2.0.0
    assert!(
        score(
            &engine,
            "mytool-1.0.0-x86_64-linux.tar.gz",
            &c,
            "2.0.0",
            "musl"
        )
        .is_none()
    );
}

#[test]
fn test_version_without_v_prefix() {
    let engine = StarlarkEngine::new();
    let c = ctx("linux", "x64");
    let r = score(
        &engine,
        "mytool-1.0.0-x86_64-linux.tar.gz",
        &c,
        "1.0.0",
        "musl",
    );
    assert!(
        r.is_some(),
        "version 1.0.0 should match '1.0.0' in filename"
    );
}

#[test]
fn test_version_with_v_prefix_in_query() {
    let engine = StarlarkEngine::new();
    let c = ctx("linux", "x64");
    let r = score(
        &engine,
        "mytool-1.0.0-x86_64-linux.tar.gz",
        &c,
        "v1.0.0",
        "musl",
    );
    assert!(
        r.is_some(),
        "query 'v1.0.0' should match '1.0.0' in filename"
    );
}

// ── OS scoring tests ──

#[test]
fn test_os_match_linux() {
    let engine = StarlarkEngine::new();
    let c = ctx("linux", "x64");
    let r = score(
        &engine,
        "mytool-1.0.0-x86_64-unknown-linux-musl.tar.gz",
        &c,
        "1.0.0",
        "musl",
    );
    let s = r.unwrap();
    assert_eq!(s["os_score"].as_i64().unwrap(), 35);
}

#[test]
fn test_os_match_windows() {
    let engine = StarlarkEngine::new();
    let c = ctx("windows", "x64");
    let r = score(
        &engine,
        "mytool-1.0.0-x86_64-pc-windows-msvc.zip",
        &c,
        "1.0.0",
        "musl",
    );
    let s = r.unwrap();
    assert_eq!(s["os_score"].as_i64().unwrap(), 35);
}

#[test]
fn test_os_match_darwin() {
    let engine = StarlarkEngine::new();
    let c = ctx("macos", "x64");
    let r = score(
        &engine,
        "mytool-1.0.0-x86_64-apple-darwin.tar.gz",
        &c,
        "1.0.0",
        "musl",
    );
    let s = r.unwrap();
    assert_eq!(s["os_score"].as_i64().unwrap(), 35);
}

#[test]
fn test_os_mismatch_excludes() {
    let engine = StarlarkEngine::new();
    // Windows asset on Linux system
    let c = ctx("linux", "x64");
    assert!(
        score(
            &engine,
            "mytool-1.0.0-x86_64-pc-windows-msvc.zip",
            &c,
            "1.0.0",
            "musl"
        )
        .is_none()
    );
}

// ── Arch scoring tests ──

#[test]
fn test_arch_match_x86_64() {
    let engine = StarlarkEngine::new();
    let c = ctx("linux", "x64");
    let r = score(
        &engine,
        "mytool-1.0.0-x86_64-linux.tar.gz",
        &c,
        "1.0.0",
        "musl",
    );
    let s = r.unwrap();
    assert_eq!(s["arch_score"].as_i64().unwrap(), 30);
}

#[test]
fn test_arch_match_amd64() {
    let engine = StarlarkEngine::new();
    let c = ctx("linux", "x64");
    let r = score(
        &engine,
        "mytool-1.0.0-linux-amd64.tar.gz",
        &c,
        "1.0.0",
        "musl",
    );
    let s = r.unwrap();
    assert_eq!(s["arch_score"].as_i64().unwrap(), 30);
}

#[test]
fn test_arch_match_arm64() {
    let engine = StarlarkEngine::new();
    let c = ctx("linux", "arm64");
    let r = score(
        &engine,
        "mytool-1.0.0-aarch64-linux.tar.gz",
        &c,
        "1.0.0",
        "musl",
    );
    let s = r.unwrap();
    assert_eq!(s["arch_score"].as_i64().unwrap(), 30);
}

#[test]
fn test_arch_universal() {
    let engine = StarlarkEngine::new();
    let c = ctx("linux", "x64");
    let r = score(&engine, "mytool-1.0.0-portable.tar.gz", &c, "1.0.0", "musl");
    let s = r.unwrap();
    assert_eq!(s["arch_score"].as_i64().unwrap(), 15);
}

// ── Libc scoring tests ──

#[test]
fn test_libc_musl_preference_on_linux() {
    let engine = StarlarkEngine::new();
    let c = ctx("linux", "x64");
    let r = score(
        &engine,
        "mytool-1.0.0-x86_64-unknown-linux-musl.tar.gz",
        &c,
        "1.0.0",
        "musl",
    );
    let s = r.unwrap();
    assert_eq!(s["libc_score"].as_i64().unwrap(), 15);
}

#[test]
fn test_libc_gnu_on_musl_pref() {
    let engine = StarlarkEngine::new();
    let c = ctx("linux", "x64");
    let r = score(
        &engine,
        "mytool-1.0.0-x86_64-unknown-linux-gnu.tar.gz",
        &c,
        "1.0.0",
        "musl",
    );
    let s = r.unwrap();
    assert_eq!(s["libc_score"].as_i64().unwrap(), 5);
}

#[test]
fn test_libc_gnu_preference() {
    let engine = StarlarkEngine::new();
    let c = ctx("linux", "x64");
    let r = score(
        &engine,
        "mytool-1.0.0-x86_64-unknown-linux-gnu.tar.gz",
        &c,
        "1.0.0",
        "gnu",
    );
    let s = r.unwrap();
    assert_eq!(s["libc_score"].as_i64().unwrap(), 15);
}

#[test]
fn test_libc_non_linux_always_full() {
    let engine = StarlarkEngine::new();
    let c = ctx("windows", "x64");
    let r = score(
        &engine,
        "mytool-1.0.0-x86_64-pc-windows-msvc.zip",
        &c,
        "1.0.0",
        "musl",
    );
    let s = r.unwrap();
    assert_eq!(s["libc_score"].as_i64().unwrap(), 15);
}

// ── Format scoring tests ──

#[test]
fn test_format_tar_gz_linux() {
    let engine = StarlarkEngine::new();
    let c = ctx("linux", "x64");
    let r = score(
        &engine,
        "mytool-1.0.0-x86_64-linux.tar.gz",
        &c,
        "1.0.0",
        "musl",
    );
    let s = r.unwrap();
    assert_eq!(s["format_score"].as_i64().unwrap(), 15);
}

#[test]
fn test_format_zip_windows() {
    let engine = StarlarkEngine::new();
    let c = ctx("windows", "x64");
    let r = score(
        &engine,
        "mytool-1.0.0-x86_64-windows.zip",
        &c,
        "1.0.0",
        "musl",
    );
    let s = r.unwrap();
    assert_eq!(s["format_score"].as_i64().unwrap(), 15);
}

// ── Keyword bonus tests ──

#[test]
fn test_keyword_static() {
    let engine = StarlarkEngine::new();
    let c = ctx("linux", "x64");
    let r = score(
        &engine,
        "mytool-1.0.0-x86_64-unknown-linux-musl-static.tar.gz",
        &c,
        "1.0.0",
        "musl",
    );
    let s = r.unwrap();
    assert_eq!(s["keyword_score"].as_i64().unwrap(), 3);
}

#[test]
fn test_keyword_capped_at_5() {
    let engine = StarlarkEngine::new();
    let c = ctx("linux", "x64");
    // Contains "static", "portable", "standalone" → 3+3+3=9, capped at 5
    let r = score(
        &engine,
        "mytool-1.0.0-x86_64-static-portable-standalone.tar.gz",
        &c,
        "1.0.0",
        "musl",
    );
    let s = r.unwrap();
    assert!(s["keyword_score"].as_i64().unwrap() <= 5);
}

// ── Full scoring example (ripgrep-like) ──

#[test]
fn test_full_score_linux_musl() {
    let engine = StarlarkEngine::new();
    let c = ctx("linux", "x64");
    let r = score(
        &engine,
        "rg-14.1.1-x86_64-unknown-linux-musl.tar.gz",
        &c,
        "14.1.1",
        "musl",
    );
    let s = r.unwrap();
    // OS 35 + Arch 30 + Libc 15 + Format 15 = 95
    assert!(s["total"].as_i64().unwrap() >= 90);
    assert_eq!(s["os_score"].as_i64().unwrap(), 35);
    assert_eq!(s["arch_score"].as_i64().unwrap(), 30);
    assert_eq!(s["libc_score"].as_i64().unwrap(), 15);
    assert_eq!(s["format_score"].as_i64().unwrap(), 15);
}

// ── detect_best_asset integration tests ──

fn make_test_assets() -> Vec<serde_json::Value> {
    vec![
        json!({"name": "rg-14.1.1-x86_64-unknown-linux-musl.tar.gz", "size": 2000000, "browser_download_url": "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/rg-14.1.1-x86_64-unknown-linux-musl.tar.gz"}),
        json!({"name": "rg-14.1.1-x86_64-unknown-linux-gnu.tar.gz", "size": 2100000, "browser_download_url": "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/rg-14.1.1-x86_64-unknown-linux-gnu.tar.gz"}),
        json!({"name": "rg-14.1.1-x86_64-pc-windows-msvc.zip", "size": 3000000, "browser_download_url": "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/rg-14.1.1-x86_64-pc-windows-msvc.zip"}),
        json!({"name": "ripgrep-14.1.1-x86_64-apple-darwin.tar.gz", "size": 1800000, "browser_download_url": "https://github.com/BurntSushi/ripgrep/releases/download/14.1.1/ripgrep-14.1.1-x86_64-apple-darwin.tar.gz"}),
    ]
}

#[test]
fn test_detect_best_linux_musl() {
    let engine = StarlarkEngine::new();
    let c = ctx("linux", "x64");
    let assets = make_test_assets();
    let best = detect(&engine, &assets, &c, "14.1.1", 40);
    let b = best.unwrap();
    assert!(
        b["name"].as_str().unwrap().contains("musl"),
        "Should pick musl asset on Linux"
    );
    assert!(b["score"].as_i64().unwrap() >= 90);
}

#[test]
fn test_detect_best_windows() {
    let engine = StarlarkEngine::new();
    let c = ctx("windows", "x64");
    let assets = make_test_assets();
    let best = detect(&engine, &assets, &c, "14.1.1", 40);
    let b = best.unwrap();
    assert!(
        b["name"].as_str().unwrap().contains("windows"),
        "Should pick windows asset on Windows"
    );
}

#[test]
fn test_detect_below_threshold() {
    let engine = StarlarkEngine::new();
    let c = ctx("linux", "x64");
    let assets = make_test_assets();
    let best = detect(&engine, &assets, &c, "14.1.1", 100);
    assert!(best.is_none(), "No asset should score >= 100");
}

#[test]
fn test_detect_no_assets() {
    let engine = StarlarkEngine::new();
    let c = ctx("linux", "x64");
    let assets: Vec<serde_json::Value> = vec![];
    let best = detect(&engine, &assets, &c, "14.1.1", 40);
    assert!(best.is_none());
}

#[test]
fn test_extra_excludes_work() {
    let engine = StarlarkEngine::new();
    let c = ctx("linux", "x64");
    let result = engine
        .call_stdlib_function(
            "@vx//stdlib:smart_detect.star",
            "score_asset",
            &[
                json!("mytool-1.0.0-x86_64-linux-nightly.tar.gz"),
                c.clone(),
                json!("1.0.0"),
                json!("musl"),
                json!(["nightly"]),
            ],
        )
        .unwrap();
    assert!(result.is_null(), "Asset with 'nightly' should be excluded");
}
