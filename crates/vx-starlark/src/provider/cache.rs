//! Incremental analysis cache for Starlark provider scripts.
//!
//! Inspired by Buck2's incremental analysis: cache the frozen ProviderInfo
//! keyed by the SHA256 hash of the script content. If the script hasn't
//! changed (same hash), reuse the cached analysis result without re-executing.

use super::types::{ProviderMeta, RuntimeMeta};
use crate::engine::FrozenProviderInfo;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

/// Incremental analysis cache entry (Buck2-inspired content-hash cache)
#[derive(Debug, Clone)]
pub(super) struct AnalysisCacheEntry {
    /// SHA256 hash of the provider.star content
    pub script_hash: [u8; 32],
    /// Frozen analysis result (immutable after analysis phase)
    /// NOTE: Used in Phase 2 when full Starlark execution engine is implemented
    #[allow(dead_code)]
    pub frozen_info: FrozenProviderInfo,
    /// Parsed provider metadata (cached to avoid redundant parsing on cache hit)
    pub meta: ProviderMeta,
    /// Parsed runtime metadata (cached to avoid redundant parsing on cache hit)
    pub runtimes: Vec<RuntimeMeta>,
    /// When this entry was cached
    /// NOTE: Used in Phase 2 for TTL-based cache expiration
    #[allow(dead_code)]
    pub cached_at: SystemTime,
}

/// Cache for analysis results, keyed by content hash (not file path)
///
/// Using content hash instead of path means:
/// - Same script content → same cache entry (deduplication)
/// - Modified script → new hash → cache miss → re-analysis
/// - File rename/move → same hash → cache hit (no re-analysis needed)
pub(super) type AnalysisCache = Arc<RwLock<HashMap<[u8; 32], AnalysisCacheEntry>>>;

/// Global incremental analysis cache (content-hash based, Buck2-inspired)
pub(super) static ANALYSIS_CACHE: once_cell::sync::Lazy<AnalysisCache> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

/// Compute SHA256 hash of content bytes
///
/// Uses multiple hash passes to produce a 32-byte representation.
/// In production, this would use the sha2 crate for proper SHA256.
pub(super) fn sha256_bytes(content: &[u8]) -> [u8; 32] {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut result = [0u8; 32];

    // Pass 1: hash the full content
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    let h1 = hasher.finish();

    // Pass 2: hash with length prefix for better distribution
    let mut hasher2 = DefaultHasher::new();
    (content.len() as u64).hash(&mut hasher2);
    content.hash(&mut hasher2);
    let h2 = hasher2.finish();

    // Pass 3 & 4: hash reversed content for additional entropy
    let mut hasher3 = DefaultHasher::new();
    content
        .iter()
        .rev()
        .cloned()
        .collect::<Vec<u8>>()
        .hash(&mut hasher3);
    let h3 = hasher3.finish();

    let mut hasher4 = DefaultHasher::new();
    h1.hash(&mut hasher4);
    h2.hash(&mut hasher4);
    h3.hash(&mut hasher4);
    let h4 = hasher4.finish();

    // Fill 32 bytes from 4 x u64 hashes
    result[0..8].copy_from_slice(&h1.to_le_bytes());
    result[8..16].copy_from_slice(&h2.to_le_bytes());
    result[16..24].copy_from_slice(&h3.to_le_bytes());
    result[24..32].copy_from_slice(&h4.to_le_bytes());

    result
}
