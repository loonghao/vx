//! Context factory functions

use super::{RealFileSystem, RealHttpClient, RealInstaller, RealPathProvider};
use crate::context::RuntimeContext;
use crate::traits::PathProvider;
use crate::version_cache::VersionCache;
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

/// Create a real runtime context for production use
///
/// Includes:
/// - Version cache (bincode format for fast serialization)
/// - Download cache (content-addressable storage for archives)
pub fn create_runtime_context() -> Result<RuntimeContext> {
    let paths = Arc::new(RealPathProvider::new()?);
    let cache_dir = paths.cache_dir().to_path_buf();

    // Create HTTP client with download caching
    let http = Arc::new(RealHttpClient::new().with_download_cache(cache_dir.clone()));
    let fs = Arc::new(RealFileSystem::new());
    // Create installer with download caching
    let installer = Arc::new(RealInstaller::with_download_cache(cache_dir.clone()));

    // Create version cache (high-performance bincode format)
    let version_cache = VersionCache::new(cache_dir);

    Ok(RuntimeContext::new(paths, http, fs, installer).with_version_cache(version_cache))
}

/// Create a real runtime context with custom base directory
///
/// Includes:
/// - Version cache (bincode format for fast serialization)
/// - Download cache (content-addressable storage for archives)
pub fn create_runtime_context_with_base(base_dir: impl AsRef<Path>) -> RuntimeContext {
    let base_dir = base_dir.as_ref();
    let paths = Arc::new(RealPathProvider::with_base_dir(base_dir));
    let cache_dir = paths.cache_dir().to_path_buf();

    // Create HTTP client with download caching
    let http = Arc::new(RealHttpClient::new().with_download_cache(cache_dir.clone()));
    let fs = Arc::new(RealFileSystem::new());
    // Create installer with download caching
    let installer = Arc::new(RealInstaller::with_download_cache(cache_dir.clone()));

    // Create version cache (high-performance bincode format)
    let version_cache = VersionCache::new(cache_dir);

    RuntimeContext::new(paths, http, fs, installer).with_version_cache(version_cache)
}
