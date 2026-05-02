//! High-performance version cache using bincode serialization
//!
//! Re-exported as [`VersionCache`].
//!
//! This module provides a fast, compact cache for version information.
//!
//! ## Cache Directory Structure
//!
//! ```text
//! ~/.vx/cache/
//! └── versions_v2/
//!     ├── bun.meta      # Small metadata file (bincode)
//!     ├── bun.data      # Compact version data (bincode)
//!     ├── node.meta
//!     ├── node.data
//!     └── go.jsonval    # JSON API response (JSON text format)
//! ```

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Current cache schema version
pub const CACHE_SCHEMA_VERSION: u32 = 2;

/// Default cache TTL (24 hours)
pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);

/// Compact version info - only essential fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompactVersion {
    pub version: String,
    pub prerelease: bool,
    pub published_at: u64,
}

impl CompactVersion {
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            prerelease: false,
            published_at: 0,
        }
    }

    pub fn with_prerelease(mut self, prerelease: bool) -> Self {
        self.prerelease = prerelease;
        self
    }

    pub fn with_published_at(mut self, timestamp: u64) -> Self {
        self.published_at = timestamp;
        self
    }
}

/// Cache metadata - stored separately for quick validity checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    pub schema_version: u32,
    pub created_at: u64,
    pub ttl_secs: u64,
    pub version_count: u32,
    pub source_url: Option<String>,
    pub etag: Option<String>,
}

impl CacheMetadata {
    pub fn new(version_count: usize, ttl: Duration) -> Self {
        Self {
            schema_version: CACHE_SCHEMA_VERSION,
            created_at: now_epoch_secs(),
            ttl_secs: ttl.as_secs(),
            version_count: version_count as u32,
            source_url: None,
            etag: None,
        }
    }

    pub fn with_source_url(mut self, url: impl Into<String>) -> Self {
        self.source_url = Some(url.into());
        self
    }

    pub fn with_etag(mut self, etag: impl Into<String>) -> Self {
        self.etag = Some(etag.into());
        self
    }

    pub fn is_valid(&self) -> bool {
        if self.schema_version != CACHE_SCHEMA_VERSION {
            return false;
        }
        let now = now_epoch_secs();
        now.saturating_sub(self.created_at) < self.ttl_secs
    }

    pub fn age(&self) -> u64 {
        now_epoch_secs().saturating_sub(self.created_at)
    }

    pub fn remaining_ttl(&self) -> u64 {
        let elapsed = self.age();
        self.ttl_secs.saturating_sub(elapsed)
    }
}

/// Cached version data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheData {
    pub versions: Vec<CompactVersion>,
}

/// Cache mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CacheMode {
    #[default]
    Normal,
    Refresh,
    Offline,
    NoCache,
}

impl From<vx_cache::CacheMode> for CacheMode {
    fn from(mode: vx_cache::CacheMode) -> Self {
        match mode {
            vx_cache::CacheMode::Normal => CacheMode::Normal,
            vx_cache::CacheMode::Refresh => CacheMode::Refresh,
            vx_cache::CacheMode::Offline => CacheMode::Offline,
            vx_cache::CacheMode::NoCache => CacheMode::NoCache,
        }
    }
}

/// High-performance version cache
#[derive(Debug, Clone)]
pub struct VersionCache {
    cache_dir: PathBuf,
    ttl: Duration,
    mode: CacheMode,
}

impl VersionCache {
    pub fn new(base_cache_dir: PathBuf) -> Self {
        Self {
            cache_dir: base_cache_dir.join("versions_v2"),
            ttl: DEFAULT_CACHE_TTL,
            mode: CacheMode::Normal,
        }
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    pub fn with_mode(mut self, mode: CacheMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn mode(&self) -> CacheMode {
        self.mode
    }

    fn meta_path(&self, tool_name: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.meta", tool_name))
    }

    fn data_path(&self, tool_name: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.data", tool_name))
    }

    fn json_value_data_path(&self, tool_name: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.jsonval", tool_name))
    }

    pub fn get_metadata(&self, tool_name: &str) -> Option<CacheMetadata> {
        if self.mode == CacheMode::NoCache || self.mode == CacheMode::Refresh {
            return None;
        }

        let meta_path = self.meta_path(tool_name);
        if !meta_path.exists() {
            return None;
        }

        let file = std::fs::File::open(&meta_path).ok()?;
        let mut reader = BufReader::new(file);
        bincode::serde::decode_from_std_read(&mut reader, bincode::config::standard()).ok()
    }

    pub fn is_valid(&self, tool_name: &str) -> bool {
        match self.mode {
            CacheMode::NoCache | CacheMode::Refresh => false,
            CacheMode::Offline => self.get_metadata(tool_name).is_some(),
            CacheMode::Normal => self
                .get_metadata(tool_name)
                .map(|m| m.is_valid())
                .unwrap_or(false),
        }
    }

    pub fn get(&self, tool_name: &str) -> Option<Vec<CompactVersion>> {
        if self.mode == CacheMode::NoCache || self.mode == CacheMode::Refresh {
            return None;
        }

        let metadata = self.get_metadata(tool_name)?;

        if self.mode == CacheMode::Normal && !metadata.is_valid() {
            self.clear(tool_name).ok();
            return None;
        }

        let data_path = self.data_path(tool_name);
        if !data_path.exists() {
            return None;
        }

        let file = std::fs::File::open(&data_path).ok()?;
        let mut reader = BufReader::new(file);
        let data: CacheData =
            bincode::serde::decode_from_std_read(&mut reader, bincode::config::standard()).ok()?;

        Some(data.versions)
    }

    pub fn get_stale(&self, tool_name: &str) -> Option<Vec<CompactVersion>> {
        if self.mode == CacheMode::NoCache {
            return None;
        }

        let data_path = self.data_path(tool_name);
        if !data_path.exists() {
            return None;
        }

        let metadata = {
            let meta_path = self.meta_path(tool_name);
            if !meta_path.exists() {
                return None;
            }
            let file = std::fs::File::open(&meta_path).ok()?;
            let mut reader = BufReader::new(file);
            let meta: CacheMetadata =
                bincode::serde::decode_from_std_read(&mut reader, bincode::config::standard())
                    .ok()?;
            if meta.schema_version != CACHE_SCHEMA_VERSION {
                return None;
            }
            meta
        };

        tracing::debug!(
            "Using stale cache for {} (age: {}s, ttl: {}s)",
            tool_name,
            metadata.age(),
            metadata.ttl_secs
        );

        let file = std::fs::File::open(&data_path).ok()?;
        let mut reader = BufReader::new(file);
        let data: CacheData =
            bincode::serde::decode_from_std_read(&mut reader, bincode::config::standard()).ok()?;

        Some(data.versions)
    }

    pub fn get_json(&self, tool_name: &str) -> Option<serde_json::Value> {
        if self.mode == CacheMode::NoCache || self.mode == CacheMode::Refresh {
            return None;
        }

        let metadata = self.get_metadata(tool_name)?;

        if self.mode == CacheMode::Normal && !metadata.is_valid() {
            self.clear(tool_name).ok();
            return None;
        }

        let json_path = self.json_value_data_path(tool_name);
        if !json_path.exists() {
            return None;
        }

        let file = std::fs::File::open(&json_path).ok()?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).ok()
    }

    pub fn get_stale_json(&self, tool_name: &str) -> Option<serde_json::Value> {
        if self.mode == CacheMode::NoCache {
            return None;
        }

        let json_path = self.json_value_data_path(tool_name);
        if !json_path.exists() {
            return None;
        }

        let metadata = {
            let meta_path = self.meta_path(tool_name);
            if !meta_path.exists() {
                return None;
            }
            let file = std::fs::File::open(&meta_path).ok()?;
            let mut reader = BufReader::new(file);
            let meta: CacheMetadata =
                bincode::serde::decode_from_std_read(&mut reader, bincode::config::standard())
                    .ok()?;
            if meta.schema_version != CACHE_SCHEMA_VERSION {
                return None;
            }
            meta
        };

        tracing::debug!(
            "Using stale JSON cache for {} (age: {}s, ttl: {}s)",
            tool_name,
            metadata.age(),
            metadata.ttl_secs
        );

        let file = std::fs::File::open(&json_path).ok()?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).ok()
    }

    pub fn set(&self, tool_name: &str, versions: Vec<CompactVersion>) -> Result<()> {
        self.set_with_options(tool_name, versions, None, None)
    }

    pub fn set_with_options(
        &self,
        tool_name: &str,
        versions: Vec<CompactVersion>,
        source_url: Option<&str>,
        etag: Option<&str>,
    ) -> Result<()> {
        if self.mode == CacheMode::NoCache {
            return Ok(());
        }

        std::fs::create_dir_all(&self.cache_dir)?;

        let mut metadata = CacheMetadata::new(versions.len(), self.ttl);
        if let Some(url) = source_url {
            metadata = metadata.with_source_url(url);
        }
        if let Some(tag) = etag {
            metadata = metadata.with_etag(tag);
        }

        let data = CacheData { versions };

        let meta_path = self.meta_path(tool_name);
        let meta_tmp = meta_path.with_extension("meta.tmp");
        {
            let file = std::fs::File::create(&meta_tmp)?;
            let mut writer = BufWriter::new(file);
            bincode::serde::encode_into_std_write(
                &metadata,
                &mut writer,
                bincode::config::standard(),
            )?;
        }
        std::fs::rename(&meta_tmp, &meta_path)?;

        let data_path = self.data_path(tool_name);
        let data_tmp = data_path.with_extension("data.tmp");
        {
            let file = std::fs::File::create(&data_tmp)?;
            let mut writer = BufWriter::new(file);
            bincode::serde::encode_into_std_write(&data, &mut writer, bincode::config::standard())?;
        }
        std::fs::rename(&data_tmp, &data_path)?;

        tracing::debug!(
            "Cached {} versions for {} ({} bytes)",
            data.versions.len(),
            tool_name,
            std::fs::metadata(&data_path).map(|m| m.len()).unwrap_or(0)
        );

        Ok(())
    }

    pub fn set_json(&self, tool_name: &str, data: serde_json::Value) -> Result<()> {
        self.set_json_with_options(tool_name, data, None, None)
    }

    pub fn set_json_with_options(
        &self,
        tool_name: &str,
        data: serde_json::Value,
        source_url: Option<&str>,
        etag: Option<&str>,
    ) -> Result<()> {
        if self.mode == CacheMode::NoCache {
            return Ok(());
        }

        std::fs::create_dir_all(&self.cache_dir)?;

        let version_count = data.as_array().map(|a| a.len()).unwrap_or(1);

        let mut metadata = CacheMetadata::new(version_count, self.ttl);
        if let Some(url) = source_url {
            metadata = metadata.with_source_url(url);
        }
        if let Some(tag) = etag {
            metadata = metadata.with_etag(tag);
        }

        let meta_path = self.meta_path(tool_name);
        let meta_tmp = meta_path.with_extension("meta.tmp");
        {
            let file = std::fs::File::create(&meta_tmp)?;
            let mut writer = BufWriter::new(file);
            bincode::serde::encode_into_std_write(
                &metadata,
                &mut writer,
                bincode::config::standard(),
            )?;
        }
        std::fs::rename(&meta_tmp, &meta_path)?;

        let json_path = self.json_value_data_path(tool_name);
        let json_tmp = json_path.with_extension("jsonval.tmp");
        {
            let file = std::fs::File::create(&json_tmp)?;
            let writer = BufWriter::new(file);
            serde_json::to_writer(writer, &data)?;
        }
        std::fs::rename(&json_tmp, &json_path)?;

        tracing::debug!(
            "Cached JSON Value for {} ({} bytes)",
            tool_name,
            std::fs::metadata(&json_path).map(|m| m.len()).unwrap_or(0)
        );

        Ok(())
    }

    pub fn clear(&self, tool_name: &str) -> Result<()> {
        let meta_path = self.meta_path(tool_name);
        let data_path = self.data_path(tool_name);
        let json_path = self.json_value_data_path(tool_name);

        if meta_path.exists() {
            std::fs::remove_file(&meta_path)?;
        }
        if data_path.exists() {
            std::fs::remove_file(&data_path)?;
        }
        if json_path.exists() {
            std::fs::remove_file(&json_path)?;
        }

        Ok(())
    }

    pub fn clear_all(&self) -> Result<()> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    pub fn stats(&self) -> Result<CacheStats> {
        let mut stats = CacheStats::default();

        if !self.cache_dir.exists() {
            return Ok(stats);
        }

        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|e| e == "meta") {
                stats.total_entries += 1;

                if let Ok(file) = std::fs::File::open(&path) {
                    let mut reader = BufReader::new(file);
                    if let Ok(meta) = bincode::serde::decode_from_std_read::<CacheMetadata, _, _>(
                        &mut reader,
                        bincode::config::standard(),
                    ) {
                        if meta.is_valid() {
                            stats.valid_entries += 1;
                        } else {
                            stats.expired_entries += 1;
                        }
                    }
                }
            }

            if let Ok(metadata) = path.metadata() {
                stats.total_size_bytes += metadata.len();
            }
        }

        Ok(stats)
    }

    pub fn prune(&self) -> Result<usize> {
        let mut pruned = 0;

        if !self.cache_dir.exists() {
            return Ok(pruned);
        }

        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|e| e == "meta") {
                let should_prune = if let Ok(file) = std::fs::File::open(&path) {
                    let mut reader = BufReader::new(file);
                    if let Ok(meta) = bincode::serde::decode_from_std_read::<CacheMetadata, _, _>(
                        &mut reader,
                        bincode::config::standard(),
                    ) {
                        !meta.is_valid()
                    } else {
                        true
                    }
                } else {
                    false
                };

                if should_prune && let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let _ = self.clear(stem);
                    pruned += 1;
                }
            }
        }

        Ok(pruned)
    }

    pub fn get_entry(&self, tool_name: &str) -> Option<CacheEntry> {
        let meta_path = self.meta_path(tool_name);

        if !meta_path.exists() {
            return None;
        }

        let file = std::fs::File::open(&meta_path).ok()?;
        let mut reader = BufReader::new(file);
        let meta: CacheMetadata =
            bincode::serde::decode_from_std_read(&mut reader, bincode::config::standard()).ok()?;

        let is_valid = match self.mode {
            CacheMode::Normal => meta.is_valid(),
            CacheMode::Refresh => false,
            CacheMode::Offline | CacheMode::NoCache => true,
        };

        Some(CacheEntry {
            tool_name: tool_name.to_string(),
            version_count: meta.version_count as usize,
            source_url: meta.source_url,
            cached_at: meta.created_at,
            expires_at: meta.created_at + meta.ttl_secs,
            is_valid,
        })
    }
}

/// Cache statistics
#[derive(Debug, Default)]
pub struct CacheStats {
    pub total_entries: usize,
    pub valid_entries: usize,
    pub expired_entries: usize,
    pub total_size_bytes: u64,
}

impl CacheStats {
    pub fn formatted_size(&self) -> String {
        format_size(self.total_size_bytes)
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Cache entry information
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub tool_name: String,
    pub version_count: usize,
    pub source_url: Option<String>,
    pub cached_at: u64,
    pub expires_at: u64,
    pub is_valid: bool,
}

fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Convert GitHub release to compact version
pub fn github_release_to_compact(release: &serde_json::Value) -> Option<CompactVersion> {
    let tag_name = release.get("tag_name")?.as_str()?;
    let prerelease = release
        .get("prerelease")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let published_at = release
        .get("published_at")
        .and_then(|v| v.as_str())
        .and_then(parse_iso8601_timestamp)
        .unwrap_or(0);

    let version = tag_name.strip_prefix('v').unwrap_or(tag_name);

    Some(
        CompactVersion::new(version)
            .with_prerelease(prerelease)
            .with_published_at(published_at),
    )
}

fn parse_iso8601_timestamp(s: &str) -> Option<u64> {
    let s = s.trim_end_matches('Z');
    let parts: Vec<&str> = s.split('T').collect();
    if parts.len() != 2 {
        return None;
    }

    let date_parts: Vec<u32> = parts[0].split('-').filter_map(|p| p.parse().ok()).collect();
    let time_parts: Vec<u32> = parts[1].split(':').filter_map(|p| p.parse().ok()).collect();

    if date_parts.len() != 3 || time_parts.len() != 3 {
        return None;
    }

    let year = date_parts[0];
    let month = date_parts[1];
    let day = date_parts[2];
    let hour = time_parts[0];
    let minute = time_parts[1];
    let second = time_parts[2];

    let years_since_1970 = year.saturating_sub(1970);
    let leap_years = (years_since_1970 + 1) / 4;
    let days_from_years = years_since_1970 * 365 + leap_years;

    let days_in_months = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let days_from_months = days_in_months
        .get(month.saturating_sub(1) as usize)
        .copied()
        .unwrap_or(0);

    let total_days = days_from_years + days_from_months + day.saturating_sub(1);
    let total_seconds =
        (total_days as u64) * 86400 + (hour as u64) * 3600 + (minute as u64) * 60 + (second as u64);

    Some(total_seconds)
}
