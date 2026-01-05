use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::path::Path;

/// Write bytes to `dest` using a best-effort atomic replace (tmp + rename).
///
/// On Windows, replacing an existing file with `rename` may fail; we remove the destination first.
pub fn atomic_write_bytes(dest: &Path, data: &[u8]) -> Result<()> {
    let tmp = dest.with_extension("tmp");
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&tmp, data)?;

    if dest.exists() {
        let _ = std::fs::remove_file(dest);
    }
    std::fs::rename(&tmp, dest)?;
    Ok(())
}

/// Convenience wrapper for UTF-8 strings.
pub fn atomic_write_string(dest: &Path, s: &str) -> Result<()> {
    atomic_write_bytes(dest, s.as_bytes())
}

/// Read JSON file into T.
pub fn read_json_file<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let content = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

/// Write JSON file using `atomic_write_string`.
pub fn write_json_file<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let s = serde_json::to_string_pretty(value)?;
    atomic_write_string(path, &s)
}
