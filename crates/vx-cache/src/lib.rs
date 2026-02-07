//! vx-cache
//!
//! Shared cache utilities used across vx crates.
//!
//! This crate provides:
//! - **Version cache**: High-performance bincode-based version list caching
//! - **Download cache**: Content-addressable storage for downloaded files
//! - **File utilities**: Atomic file operations
//! - **Cache statistics**: Size and count tracking

pub mod download;
pub mod exec_path;
pub mod file;
pub mod mode;
pub mod stats;
pub mod time;

pub use download::{CacheLookupResult, DownloadCache, DownloadCacheMetadata, DownloadCacheStats};
pub use exec_path::ExecPathCache;
pub use file::{atomic_write_bytes, atomic_write_string, read_json_file, write_json_file};
pub use mode::CacheMode;
pub use stats::{format_size, CacheStats};
pub use time::now_epoch_secs;
