//! vx-cache
//!
//! Shared cache utilities used across vx crates.
//!
//! This crate intentionally stays small and dependency-light.

pub mod file;
pub mod mode;
pub mod stats;
pub mod time;

pub use file::{atomic_write_bytes, atomic_write_string, read_json_file, write_json_file};
pub use mode::CacheMode;
pub use stats::{format_size, CacheStats};
pub use time::now_epoch_secs;
