//! E2E tests for new high-priority developer tool providers
//!
//! These tests verify that the new providers (lazygit, delta, hyperfine,
//! zoxide, atuin, chezmoi, eza, tealdeer, dust, xh, bottom, trivy,
//! zellij, dive) are correctly registered and their
//! provider.star files are valid.

use std::env;
use std::path::PathBuf;
use std::process::Command;

/// Get the path to the vx binary for testing
fn vx_binary() -> PathBuf {
    let mut path = env::current_exe().unwrap();
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.push("vx");
    if cfg!(windows) {
        path.set_extension("exe");
    }
    path
}

/// Get the path to the vx-providers directory
fn providers_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("crates")
        .join("vx-providers")
}

// ============================================================================
// Provider Files Existence Tests
// ============================================================================

macro_rules! provider_files_test {
    ($test_name:ident, $provider:literal) => {
        #[test]
        fn $test_name() {
            let dir = providers_dir().join($provider);
            assert!(
                dir.exists(),
                "{} provider directory should exist",
                $provider
            );
            assert!(
                dir.join("provider.star").exists(),
                "provider.star should exist for {}",
                $provider
            );
            assert!(
                dir.join("Cargo.toml").exists(),
                "Cargo.toml should exist for {}",
                $provider
            );
            assert!(
                dir.join("lib.rs").exists(),
                "lib.rs should exist for {}",
                $provider
            );
        }
    };
}

// Batch 1 (existing)
provider_files_test!(test_lazygit_provider_files_exist, "lazygit");
provider_files_test!(test_delta_provider_files_exist, "delta");
provider_files_test!(test_hyperfine_provider_files_exist, "hyperfine");
provider_files_test!(test_zoxide_provider_files_exist, "zoxide");
provider_files_test!(test_atuin_provider_files_exist, "atuin");
provider_files_test!(test_chezmoi_provider_files_exist, "chezmoi");
provider_files_test!(test_eza_provider_files_exist, "eza");

// Batch 2 (new)
provider_files_test!(test_tealdeer_provider_files_exist, "tealdeer");
provider_files_test!(test_dust_provider_files_exist, "dust");
provider_files_test!(test_xh_provider_files_exist, "xh");
provider_files_test!(test_bottom_provider_files_exist, "bottom");
provider_files_test!(test_trivy_provider_files_exist, "trivy");
provider_files_test!(test_zellij_provider_files_exist, "zellij");
provider_files_test!(test_dive_provider_files_exist, "dive");

// ============================================================================
// Local Provider Tests - verify provider.star files are valid via `vx test --local`
// ============================================================================

macro_rules! local_provider_test {
    ($test_name:ident, $provider:literal) => {
        #[test]
        fn $test_name() {
            let dir = providers_dir().join($provider);
            assert!(
                dir.exists(),
                "{} provider directory should exist",
                $provider
            );

            let output = Command::new(vx_binary())
                .args(["test", "--local"])
                .arg(&dir)
                .output()
                .expect("Failed to execute vx test --local");

            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                output.status.success(),
                "{} provider test should pass: {}",
                $provider,
                stdout
            );
            assert!(
                stdout.contains($provider),
                "Output should mention {}: {}",
                $provider,
                stdout
            );
        }
    };
}

// Batch 1 (existing)
local_provider_test!(test_local_provider_lazygit, "lazygit");
local_provider_test!(test_local_provider_delta, "delta");
local_provider_test!(test_local_provider_hyperfine, "hyperfine");
local_provider_test!(test_local_provider_zoxide, "zoxide");
local_provider_test!(test_local_provider_atuin, "atuin");
local_provider_test!(test_local_provider_chezmoi, "chezmoi");
local_provider_test!(test_local_provider_eza, "eza");

// Batch 2 (new)
local_provider_test!(test_local_provider_tealdeer, "tealdeer");
local_provider_test!(test_local_provider_dust, "dust");
local_provider_test!(test_local_provider_xh, "xh");
local_provider_test!(test_local_provider_bottom, "bottom");
local_provider_test!(test_local_provider_trivy, "trivy");
local_provider_test!(test_local_provider_zellij, "zellij");
local_provider_test!(test_local_provider_dive, "dive");
