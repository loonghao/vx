//! Runtime Index - High-performance runtime lookup cache
//!
//! This module provides a global runtime index that enables fast command dispatch:
//! - Maps runtime names to their store locations
//! - Tracks installed versions
//! - Handles bundled runtime relationships (uvx -> uv, bunx -> bun, etc.)
//!
//! ## Design Principles
//!
//! 1. **Binary format (bincode)**: 10-100x faster than JSON, smaller files
//! 2. **Separate metadata file**: Quick validity check without loading full data
//! 3. **Unified path interface**: Uses vx-paths for all path operations
//! 4. **Flexible mapping**: Supports aliases, bundled runtimes, command prefixes
//!
//! ## Cache Directory Structure
//!
//! ```text
//! ~/.vx/cache/
//! └── runtime_index/
//!     ├── index.meta      # Small metadata file (< 100 bytes)
//!     └── index.data      # Compact runtime data (bincode)
//! ```
//!
//! ## Fast Path
//!
//! When executing `vx uvx ruff check .`:
//! 1. Read index.meta (< 0.1ms) - check validity
//! 2. Read index.data (< 1ms) - load runtime mappings
//! 3. Look up "uvx" -> store_name = "uv", version = "1.0.0"
//! 4. Execute ~/.vx/store/uv/1.0.0/uvx directly
//!
//! No provider.toml parsing needed for installed runtimes!

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tracing::debug;

/// Current schema version for the runtime index
pub const RUNTIME_INDEX_SCHEMA_VERSION: u32 = 2;

/// Cache directory name under ~/.vx/cache/
pub const RUNTIME_INDEX_DIR: &str = "runtime_index";

/// Default TTL for the index (1 hour - shorter than version cache since it's local data)
pub const DEFAULT_INDEX_TTL: Duration = Duration::from_secs(60 * 60);

/// Index metadata - stored separately for quick validity checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    /// Schema version for migrations
    pub schema_version: u32,
    /// Creation timestamp (Unix epoch seconds)
    pub created_at: u64,
    /// TTL in seconds
    pub ttl_secs: u64,
    /// Number of runtimes indexed
    pub runtime_count: u32,
    /// Number of aliases
    pub alias_count: u32,
    /// VX version that created this index
    pub vx_version: String,
    /// Platform OS
    pub os: String,
    /// Platform architecture
    pub arch: String,
    /// Hash of provider manifests (for invalidation)
    pub manifest_hash: Option<u64>,
}

impl IndexMetadata {
    pub fn new(runtime_count: usize, alias_count: usize, ttl: Duration) -> Self {
        Self {
            schema_version: RUNTIME_INDEX_SCHEMA_VERSION,
            created_at: now_epoch_secs(),
            ttl_secs: ttl.as_secs(),
            runtime_count: runtime_count as u32,
            alias_count: alias_count as u32,
            vx_version: env!("CARGO_PKG_VERSION").to_string(),
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            manifest_hash: None,
        }
    }

    pub fn with_manifest_hash(mut self, hash: u64) -> Self {
        self.manifest_hash = Some(hash);
        self
    }

    /// Check if index is still valid
    pub fn is_valid(&self) -> bool {
        // Schema version check
        if self.schema_version != RUNTIME_INDEX_SCHEMA_VERSION {
            return false;
        }
        // Platform check
        if self.os != std::env::consts::OS || self.arch != std::env::consts::ARCH {
            return false;
        }
        // TTL check
        let now = now_epoch_secs();
        now.saturating_sub(self.created_at) < self.ttl_secs
    }

    /// Get age in seconds
    pub fn age(&self) -> u64 {
        now_epoch_secs().saturating_sub(self.created_at)
    }
}

/// Compact runtime entry - optimized for fast lookup
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuntimeIndexEntry {
    /// Provider name (e.g., "uv", "bun", "node")
    pub provider: String,

    /// Executable name (e.g., "uvx", "bun", "npm")
    pub executable: String,

    /// Store directory name - where the files are actually stored
    /// For bundled runtimes, this points to the parent (e.g., uvx -> "uv")
    pub store_name: String,

    /// If this runtime is bundled with another (e.g., uvx bundled with uv)
    pub bundled_with: Option<String>,

    /// Command prefix for aliased commands (e.g., bunx -> ["x"])
    pub command_prefix: Vec<String>,

    /// Executable relative path within the version directory
    /// e.g., "bin/node" or just "uv" for flat layouts
    pub executable_path: Option<String>,

    /// Installed versions (sorted, newest first)
    pub installed_versions: Vec<String>,
}

impl RuntimeIndexEntry {
    /// Check if this runtime has any installed version
    #[inline]
    pub fn is_installed(&self) -> bool {
        !self.installed_versions.is_empty()
    }

    /// Get the latest installed version
    #[inline]
    pub fn latest_version(&self) -> Option<&str> {
        self.installed_versions.first().map(|s| s.as_str())
    }

    /// Get the executable path for a specific version
    ///
    /// This method uses `vx_paths::RuntimeRoot` to correctly resolve the path,
    /// handling platform directories and nested archive structures.
    pub fn get_executable_path(&self, store_base: &Path, version: &str) -> Option<PathBuf> {
        // Use RuntimeRoot to correctly resolve the path with platform directory
        // and nested structures (e.g., node-v25.6.0-win-x64/)
        let base_dir = store_base.parent()?;
        let paths = vx_paths::VxPaths::with_base_dir(base_dir);

        // For bundled runtimes (like npm bundled with node), use store_name
        // which points to the parent runtime
        if let Ok(Some(root)) = vx_paths::RuntimeRoot::find(&self.store_name, version, &paths) {
            // If we have a specific executable name different from store_name,
            // look for that bundled tool
            if self.executable != self.store_name {
                // This is a bundled runtime (e.g., npm, npx bundled with node)
                if let Some(bundled_path) = root.bundled_tool_path(&self.executable) {
                    return Some(bundled_path);
                }
            }

            // Return the main executable path
            if root.executable_exists() {
                return Some(root.executable_path.clone());
            }
        }

        // Fallback to direct path construction (for simple layouts)
        if let Some(exe_path) = &self.executable_path {
            let direct_path = store_base
                .join(&self.store_name)
                .join(version)
                .join(exe_path);
            if direct_path.exists() {
                return Some(direct_path);
            }
        }

        None
    }

    /// Get the executable path for the latest installed version
    pub fn get_latest_executable_path(&self, store_base: &Path) -> Option<PathBuf> {
        let version = self.latest_version()?;
        self.get_executable_path(store_base, version)
    }
}

/// Index data - the actual runtime mappings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IndexData {
    /// Runtime entries keyed by runtime name
    pub runtimes: HashMap<String, RuntimeIndexEntry>,
    /// Alias mappings (e.g., "nodejs" -> "node", "python3" -> "python")
    pub aliases: HashMap<String, String>,
}

/// High-performance runtime index
#[derive(Debug, Clone)]
pub struct RuntimeIndex {
    /// Cache directory (e.g., ~/.vx/cache/runtime_index)
    cache_dir: PathBuf,
    /// Store directory (e.g., ~/.vx/store)
    store_dir: PathBuf,
    /// Default TTL
    ttl: Duration,
    /// In-memory data (lazy loaded)
    data: Option<IndexData>,
}

impl RuntimeIndex {
    /// Create a new runtime index using vx-paths
    pub fn new() -> anyhow::Result<Self> {
        let paths = vx_paths::VxPaths::new()?;
        Ok(Self::with_paths(&paths))
    }

    /// Create a new runtime index with custom paths
    pub fn with_paths(paths: &vx_paths::VxPaths) -> Self {
        Self {
            cache_dir: paths.cache_dir.join(RUNTIME_INDEX_DIR),
            store_dir: paths.store_dir.clone(),
            ttl: DEFAULT_INDEX_TTL,
            data: None,
        }
    }

    /// Create a new runtime index with custom base directory
    pub fn with_base_dir(base_dir: &Path) -> Self {
        let paths = vx_paths::VxPaths::with_base_dir(base_dir);
        Self::with_paths(&paths)
    }

    /// Set custom TTL
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    /// Get metadata file path
    fn meta_path(&self) -> PathBuf {
        self.cache_dir.join("index.meta")
    }

    /// Get data file path
    fn data_path(&self) -> PathBuf {
        self.cache_dir.join("index.data")
    }

    /// Read metadata only (fast validity check)
    pub fn get_metadata(&self) -> Option<IndexMetadata> {
        let meta_path = self.meta_path();
        if !meta_path.exists() {
            return None;
        }

        let file = std::fs::File::open(&meta_path).ok()?;
        let reader = BufReader::new(file);
        bincode::deserialize_from(reader).ok()
    }

    /// Check if index is valid (without loading data)
    pub fn is_valid(&self) -> bool {
        self.get_metadata().map(|m| m.is_valid()).unwrap_or(false)
    }

    /// Check if index is valid with specific manifest hash
    pub fn is_valid_with_hash(&self, manifest_hash: u64) -> bool {
        self.get_metadata()
            .map(|m| m.is_valid() && m.manifest_hash == Some(manifest_hash))
            .unwrap_or(false)
    }

    /// Load index data (returns None if cache miss or expired)
    pub fn load(&mut self) -> Option<&IndexData> {
        if self.data.is_some() {
            return self.data.as_ref();
        }

        // Check metadata first
        let metadata = self.get_metadata()?;
        if !metadata.is_valid() {
            debug!(
                "Runtime index expired (age: {}s, ttl: {}s)",
                metadata.age(),
                metadata.ttl_secs
            );
            self.clear().ok();
            return None;
        }

        // Load data
        let data_path = self.data_path();
        if !data_path.exists() {
            return None;
        }

        let file = std::fs::File::open(&data_path).ok()?;
        let reader = BufReader::new(file);
        let data: IndexData = bincode::deserialize_from(reader).ok()?;

        debug!(
            "Loaded runtime index: {} runtimes, {} aliases",
            data.runtimes.len(),
            data.aliases.len()
        );

        self.data = Some(data);
        self.data.as_ref()
    }

    /// Force load index data (ignores TTL, for fallback)
    pub fn load_stale(&mut self) -> Option<&IndexData> {
        if self.data.is_some() {
            return self.data.as_ref();
        }

        let data_path = self.data_path();
        if !data_path.exists() {
            return None;
        }

        // Check schema version only
        let metadata = self.get_metadata()?;
        if metadata.schema_version != RUNTIME_INDEX_SCHEMA_VERSION {
            return None;
        }

        let file = std::fs::File::open(&data_path).ok()?;
        let reader = BufReader::new(file);
        let data: IndexData = bincode::deserialize_from(reader).ok()?;

        debug!(
            "Loaded stale runtime index: {} runtimes (age: {}s)",
            data.runtimes.len(),
            metadata.age()
        );

        self.data = Some(data);
        self.data.as_ref()
    }

    /// Save index data
    pub fn save(&self, data: &IndexData, manifest_hash: Option<u64>) -> anyhow::Result<()> {
        // Ensure cache directory exists
        std::fs::create_dir_all(&self.cache_dir)?;

        // Create metadata
        let mut metadata = IndexMetadata::new(data.runtimes.len(), data.aliases.len(), self.ttl);
        if let Some(hash) = manifest_hash {
            metadata = metadata.with_manifest_hash(hash);
        }

        // Write metadata (atomic)
        let meta_path = self.meta_path();
        let meta_tmp = meta_path.with_extension("meta.tmp");
        {
            let file = std::fs::File::create(&meta_tmp)?;
            let writer = BufWriter::new(file);
            bincode::serialize_into(writer, &metadata)?;
        }
        std::fs::rename(&meta_tmp, &meta_path)?;

        // Write data (atomic)
        let data_path = self.data_path();
        let data_tmp = data_path.with_extension("data.tmp");
        {
            let file = std::fs::File::create(&data_tmp)?;
            let writer = BufWriter::new(file);
            bincode::serialize_into(writer, data)?;
        }
        std::fs::rename(&data_tmp, &data_path)?;

        debug!(
            "Saved runtime index: {} runtimes, {} aliases ({} bytes)",
            data.runtimes.len(),
            data.aliases.len(),
            std::fs::metadata(&data_path).map(|m| m.len()).unwrap_or(0)
        );

        Ok(())
    }

    /// Clear the index cache
    pub fn clear(&self) -> anyhow::Result<()> {
        let meta_path = self.meta_path();
        let data_path = self.data_path();

        if meta_path.exists() {
            std::fs::remove_file(&meta_path)?;
        }
        if data_path.exists() {
            std::fs::remove_file(&data_path)?;
        }

        Ok(())
    }

    // ========== Query Methods ==========

    /// Look up a runtime by name (including aliases)
    pub fn get(&mut self, name: &str) -> Option<&RuntimeIndexEntry> {
        let data = self.load()?;

        // Direct lookup
        if let Some(entry) = data.runtimes.get(name) {
            return Some(entry);
        }

        // Alias lookup
        if let Some(primary) = data.aliases.get(name) {
            return data.runtimes.get(primary);
        }

        None
    }

    /// Check if a runtime is installed (fast path)
    pub fn is_installed(&mut self, name: &str) -> bool {
        self.get(name).map(|e| e.is_installed()).unwrap_or(false)
    }

    /// Get the store name for a runtime (handles bundled runtimes)
    pub fn get_store_name(&mut self, name: &str) -> Option<String> {
        self.get(name).map(|e| e.store_name.clone())
    }

    /// Get the executable path for a runtime (fast path for execution)
    pub fn get_executable_path(&mut self, name: &str) -> Option<PathBuf> {
        let store_dir = self.store_dir.clone();
        let entry = self.get(name)?;
        entry.get_latest_executable_path(&store_dir)
    }

    /// Get command prefix for a runtime (e.g., bunx -> ["x"])
    pub fn get_command_prefix(&mut self, name: &str) -> Option<Vec<String>> {
        self.get(name).map(|e| e.command_prefix.clone())
    }

    /// Get the parent runtime for a bundled runtime (e.g., msbuild -> dotnet)
    /// Returns None if the runtime is not bundled with another runtime.
    pub fn get_bundled_parent(&mut self, name: &str) -> Option<String> {
        self.get(name).and_then(|e| e.bundled_with.clone())
    }

    // ========== Build Methods ==========

    /// Build index from provider manifests
    pub fn build_from_manifests(manifests: &[vx_manifest::ProviderManifest]) -> IndexData {
        let mut data = IndexData::default();

        for manifest in manifests {
            let provider_name = &manifest.provider.name;

            for runtime_def in &manifest.runtimes {
                let store_name = runtime_def
                    .bundled_with
                    .as_ref()
                    .unwrap_or(&runtime_def.name)
                    .clone();

                // Extract executable path from layout configuration (RFC 0019)
                let executable_path = runtime_def
                    .layout
                    .as_ref()
                    .and_then(|l| l.get_executable_path_owned());

                let entry = RuntimeIndexEntry {
                    provider: provider_name.clone(),
                    executable: runtime_def.name.clone(),
                    store_name,
                    bundled_with: runtime_def.bundled_with.clone(),
                    command_prefix: runtime_def.command_prefix.clone(),
                    executable_path,
                    installed_versions: Vec::new(),
                };

                data.runtimes.insert(runtime_def.name.clone(), entry);

                // Register aliases
                for alias in &runtime_def.aliases {
                    data.aliases.insert(alias.clone(), runtime_def.name.clone());
                }
            }
        }

        data
    }

    /// Scan the store directory and update installed versions
    pub fn scan_store(&self, data: &mut IndexData) -> anyhow::Result<()> {
        if !self.store_dir.exists() {
            return Ok(());
        }

        // Get unique store names
        let store_names: Vec<String> = data
            .runtimes
            .values()
            .map(|e| e.store_name.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        for store_name in store_names {
            let runtime_store = self.store_dir.join(&store_name);
            if !runtime_store.exists() {
                continue;
            }

            // List version directories
            let mut versions: Vec<String> = std::fs::read_dir(&runtime_store)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect();

            // Sort versions (newest first) - using semver-like sorting
            versions.sort_by(|a, b| version_cmp(b, a));

            // Update all runtimes that use this store
            for entry in data.runtimes.values_mut() {
                if entry.store_name == store_name {
                    entry.installed_versions = versions.clone();
                }
            }
        }

        Ok(())
    }

    /// Build and save a complete index from manifests and store scan
    pub fn build_and_save(
        &self,
        manifests: &[vx_manifest::ProviderManifest],
    ) -> anyhow::Result<()> {
        // Build from manifests
        let mut data = Self::build_from_manifests(manifests);

        // Scan store for installed versions
        self.scan_store(&mut data)?;

        // Compute manifest hash
        let manifest_hash = Self::compute_manifest_hash(manifests);

        // Save index
        self.save(&data, Some(manifest_hash))?;

        Ok(())
    }

    /// Compute hash of manifests for cache invalidation
    pub fn compute_manifest_hash(manifests: &[vx_manifest::ProviderManifest]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        for manifest in manifests {
            manifest.provider.name.hash(&mut hasher);
            for runtime in &manifest.runtimes {
                runtime.name.hash(&mut hasher);
                runtime.bundled_with.hash(&mut hasher);
                runtime.aliases.hash(&mut hasher);
            }
        }

        hasher.finish()
    }

    // ========== Update Methods ==========

    /// Mark a version as installed for a runtime
    pub fn mark_installed(&mut self, runtime_name: &str, version: &str) -> anyhow::Result<()> {
        let mut data = self.load().cloned().unwrap_or_default();

        // Update the runtime entry
        if let Some(entry) = data.runtimes.get_mut(runtime_name) {
            if !entry.installed_versions.contains(&version.to_string()) {
                entry.installed_versions.push(version.to_string());
                entry.installed_versions.sort_by(|a, b| version_cmp(b, a));
            }
        }

        // Also update bundled runtimes that share this store
        let bundled_runtimes: Vec<String> = data
            .runtimes
            .iter()
            .filter(|(_, e)| e.store_name == runtime_name && e.bundled_with.is_some())
            .map(|(name, _)| name.clone())
            .collect();

        for bundled_name in bundled_runtimes {
            if let Some(entry) = data.runtimes.get_mut(&bundled_name) {
                if !entry.installed_versions.contains(&version.to_string()) {
                    entry.installed_versions.push(version.to_string());
                    entry.installed_versions.sort_by(|a, b| version_cmp(b, a));
                }
            }
        }

        // Save updated data
        self.save(&data, None)?;
        self.data = Some(data);

        Ok(())
    }

    /// Mark a version as uninstalled
    pub fn mark_uninstalled(&mut self, runtime_name: &str, version: &str) -> anyhow::Result<()> {
        let mut data = self.load().cloned().unwrap_or_default();

        // Update the runtime entry
        if let Some(entry) = data.runtimes.get_mut(runtime_name) {
            entry.installed_versions.retain(|v| v != version);
        }

        // Also update bundled runtimes
        let bundled_runtimes: Vec<String> = data
            .runtimes
            .iter()
            .filter(|(_, e)| e.store_name == runtime_name && e.bundled_with.is_some())
            .map(|(name, _)| name.clone())
            .collect();

        for bundled_name in bundled_runtimes {
            if let Some(entry) = data.runtimes.get_mut(&bundled_name) {
                entry.installed_versions.retain(|v| v != version);
            }
        }

        // Save updated data
        self.save(&data, None)?;
        self.data = Some(data);

        Ok(())
    }
}

impl Default for RuntimeIndex {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback to current directory if home directory is not available
            Self::with_base_dir(Path::new(".vx"))
        })
    }
}

/// Compare version strings (semver-like)
fn version_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let parse_version = |s: &str| -> Vec<u64> {
        s.trim_start_matches('v')
            .split(|c: char| !c.is_ascii_digit())
            .filter_map(|p| p.parse().ok())
            .collect()
    };

    let va = parse_version(a);
    let vb = parse_version(b);

    for (a, b) in va.iter().zip(vb.iter()) {
        match a.cmp(b) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }

    va.len().cmp(&vb.len())
}

/// Helper to get current Unix epoch seconds
fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_runtime_index_basic() {
        let temp_dir = TempDir::new().unwrap();
        let mut index = RuntimeIndex::with_base_dir(temp_dir.path());

        let mut data = IndexData::default();

        data.runtimes.insert(
            "uv".to_string(),
            RuntimeIndexEntry {
                provider: "uv".to_string(),
                executable: "uv".to_string(),
                store_name: "uv".to_string(),
                bundled_with: None,
                command_prefix: vec![],
                executable_path: Some("uv".to_string()),
                installed_versions: vec!["1.0.0".to_string()],
            },
        );

        data.runtimes.insert(
            "uvx".to_string(),
            RuntimeIndexEntry {
                provider: "uv".to_string(),
                executable: "uvx".to_string(),
                store_name: "uv".to_string(),
                bundled_with: Some("uv".to_string()),
                command_prefix: vec![],
                executable_path: Some("uvx".to_string()),
                installed_versions: vec!["1.0.0".to_string()],
            },
        );

        // Save and reload
        index.save(&data, None).unwrap();

        let loaded = index.load().unwrap();
        assert_eq!(loaded.runtimes.len(), 2);
        assert!(loaded.runtimes.get("uv").unwrap().is_installed());
        assert!(loaded.runtimes.get("uvx").unwrap().is_installed());
    }

    #[test]
    fn test_alias_lookup() {
        let temp_dir = TempDir::new().unwrap();
        let mut index = RuntimeIndex::with_base_dir(temp_dir.path());

        let mut data = IndexData::default();

        data.runtimes.insert(
            "node".to_string(),
            RuntimeIndexEntry {
                provider: "node".to_string(),
                executable: "node".to_string(),
                store_name: "node".to_string(),
                bundled_with: None,
                command_prefix: vec![],
                executable_path: Some("bin/node".to_string()),
                installed_versions: vec!["20.0.0".to_string()],
            },
        );

        data.aliases
            .insert("nodejs".to_string(), "node".to_string());

        index.save(&data, None).unwrap();

        // Lookup by alias
        let entry = index.get("nodejs").unwrap();
        assert_eq!(entry.executable, "node");
    }

    #[test]
    fn test_version_sorting() {
        assert_eq!(version_cmp("1.0.0", "2.0.0"), std::cmp::Ordering::Less);
        assert_eq!(version_cmp("2.0.0", "1.0.0"), std::cmp::Ordering::Greater);
        assert_eq!(version_cmp("1.0.0", "1.0.0"), std::cmp::Ordering::Equal);
        assert_eq!(version_cmp("1.10.0", "1.9.0"), std::cmp::Ordering::Greater);
        assert_eq!(version_cmp("v1.0.0", "1.0.0"), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_metadata_validity() {
        let metadata = IndexMetadata::new(10, 5, Duration::from_secs(3600));
        assert!(metadata.is_valid());
        assert_eq!(metadata.schema_version, RUNTIME_INDEX_SCHEMA_VERSION);
    }

    #[test]
    fn test_bundled_runtime_version_sync() {
        let temp_dir = TempDir::new().unwrap();
        let mut index = RuntimeIndex::with_base_dir(temp_dir.path());

        // Initial data with empty versions
        let mut data = IndexData::default();

        data.runtimes.insert(
            "uv".to_string(),
            RuntimeIndexEntry {
                provider: "uv".to_string(),
                executable: "uv".to_string(),
                store_name: "uv".to_string(),
                bundled_with: None,
                command_prefix: vec![],
                executable_path: None,
                installed_versions: vec![],
            },
        );

        data.runtimes.insert(
            "uvx".to_string(),
            RuntimeIndexEntry {
                provider: "uv".to_string(),
                executable: "uvx".to_string(),
                store_name: "uv".to_string(),
                bundled_with: Some("uv".to_string()),
                command_prefix: vec![],
                executable_path: None,
                installed_versions: vec![],
            },
        );

        index.save(&data, None).unwrap();

        // Install uv 1.0.0 - should also update uvx
        index.mark_installed("uv", "1.0.0").unwrap();

        // Check uv is installed
        {
            let uv = index.get("uv").unwrap();
            assert!(uv.is_installed());
        }

        // Check uvx is also installed (bundled with uv)
        {
            let uvx = index.get("uvx").unwrap();
            assert!(uvx.is_installed());
            assert_eq!(uvx.latest_version(), Some("1.0.0"));
        }
    }
}
