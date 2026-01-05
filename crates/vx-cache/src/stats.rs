/// Cache statistics.
#[derive(Debug, Clone, Copy, Default)]
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

/// Format byte size to human-readable string.
pub fn format_size(size: u64) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
