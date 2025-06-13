//! CLI command tests
//!
//! This module contains unit tests for all CLI commands.

pub mod venv_tests;

use std::sync::atomic::{AtomicU32, Ordering};
use tempfile::TempDir;
use vx_core::{Result, VenvManager};

static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Create a temporary VenvManager for testing with unique directory
pub fn create_test_venv_manager() -> Result<(VenvManager, TempDir)> {
    let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Set VX_HOME to the temp directory for testing with unique path
    let test_home = temp_dir.path().join(format!("vx_test_{}", test_id));
    std::fs::create_dir_all(&test_home).expect("Failed to create test home");
    std::env::set_var("VX_HOME", &test_home);

    let manager = VenvManager::new()?;
    Ok((manager, temp_dir))
}

/// Clean up test environment
pub fn cleanup_test_env() {
    std::env::remove_var("VX_HOME");
    std::env::remove_var("VX_VENV");
}
