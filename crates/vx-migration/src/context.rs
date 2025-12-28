//! Migration context for sharing state between migrations.

use crate::error::{MigrationError, MigrationResult};
use crate::types::MigrationOptions;
use crate::version::Version;
use std::any::Any;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Migration context for sharing state
#[derive(Debug)]
pub struct MigrationContext {
    /// Root path being migrated
    root_path: PathBuf,
    /// Migration options
    options: MigrationOptions,
    /// Detected source version
    source_version: Option<Version>,
    /// Target version
    target_version: Option<Version>,
    /// Shared state storage
    state: Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>>,
    /// Files that have been modified
    modified_files: Arc<RwLock<Vec<PathBuf>>>,
    /// Backup directory
    backup_dir: Option<PathBuf>,
}

impl MigrationContext {
    /// Create a new context
    pub fn new(root_path: impl Into<PathBuf>, options: MigrationOptions) -> Self {
        Self {
            root_path: root_path.into(),
            options,
            source_version: None,
            target_version: None,
            state: Arc::new(RwLock::new(HashMap::new())),
            modified_files: Arc::new(RwLock::new(Vec::new())),
            backup_dir: None,
        }
    }

    /// Get root path
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Get options
    pub fn options(&self) -> &MigrationOptions {
        &self.options
    }

    /// Check if dry-run mode
    pub fn is_dry_run(&self) -> bool {
        self.options.dry_run
    }

    /// Get source version
    pub fn source_version(&self) -> Option<&Version> {
        self.source_version.as_ref()
    }

    /// Set source version
    pub fn set_source_version(&mut self, version: Version) {
        self.source_version = Some(version);
    }

    /// Get target version
    pub fn target_version(&self) -> Option<&Version> {
        self.target_version.as_ref()
    }

    /// Set target version
    pub fn set_target_version(&mut self, version: Version) {
        self.target_version = Some(version);
    }

    /// Get backup directory
    pub fn backup_dir(&self) -> Option<&Path> {
        self.backup_dir.as_deref()
    }

    /// Set backup directory
    pub fn set_backup_dir(&mut self, dir: PathBuf) {
        self.backup_dir = Some(dir);
    }

    /// Store state value
    pub async fn set_state<T: Any + Send + Sync>(&self, key: impl Into<String>, value: T) {
        let mut state = self.state.write().await;
        state.insert(key.into(), Box::new(value));
    }

    /// Get state value
    pub async fn get_state<T: Any + Clone>(&self, key: &str) -> Option<T> {
        let state = self.state.read().await;
        state.get(key).and_then(|v| v.downcast_ref::<T>().cloned())
    }

    /// Check if state key exists
    pub async fn has_state(&self, key: &str) -> bool {
        let state = self.state.read().await;
        state.contains_key(key)
    }

    /// Remove state value
    pub async fn remove_state(&self, key: &str) {
        let mut state = self.state.write().await;
        state.remove(key);
    }

    /// Record a modified file
    pub async fn record_modified(&self, path: impl Into<PathBuf>) {
        let mut files = self.modified_files.write().await;
        files.push(path.into());
    }

    /// Get all modified files
    pub async fn modified_files(&self) -> Vec<PathBuf> {
        let files = self.modified_files.read().await;
        files.clone()
    }

    /// Resolve path relative to root
    pub fn resolve_path(&self, path: impl AsRef<Path>) -> PathBuf {
        self.root_path.join(path)
    }

    /// Read file content
    pub async fn read_file(&self, path: impl AsRef<Path>) -> MigrationResult<String> {
        let full_path = self.resolve_path(path);
        tokio::fs::read_to_string(&full_path)
            .await
            .map_err(|e| MigrationError::io("Failed to read file", Some(full_path), e))
    }

    /// Write file content (respects dry-run)
    pub async fn write_file(
        &self,
        path: impl AsRef<Path>,
        content: impl AsRef<str>,
    ) -> MigrationResult<()> {
        let full_path = self.resolve_path(&path);

        if self.is_dry_run() {
            tracing::info!("[dry-run] Would write to: {:?}", full_path);
            return Ok(());
        }

        // Ensure parent directory exists
        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                MigrationError::io("Failed to create directory", Some(parent.to_path_buf()), e)
            })?;
        }

        tokio::fs::write(&full_path, content.as_ref())
            .await
            .map_err(|e| MigrationError::io("Failed to write file", Some(full_path.clone()), e))?;

        self.record_modified(full_path).await;
        Ok(())
    }

    /// Rename file (respects dry-run)
    pub async fn rename_file(
        &self,
        from: impl AsRef<Path>,
        to: impl AsRef<Path>,
    ) -> MigrationResult<()> {
        let from_path = self.resolve_path(&from);
        let to_path = self.resolve_path(&to);

        if self.is_dry_run() {
            tracing::info!("[dry-run] Would rename: {:?} -> {:?}", from_path, to_path);
            return Ok(());
        }

        tokio::fs::rename(&from_path, &to_path)
            .await
            .map_err(|e| MigrationError::io("Failed to rename file", Some(from_path), e))?;

        self.record_modified(to_path).await;
        Ok(())
    }

    /// Delete file (respects dry-run)
    pub async fn delete_file(&self, path: impl AsRef<Path>) -> MigrationResult<()> {
        let full_path = self.resolve_path(&path);

        if self.is_dry_run() {
            tracing::info!("[dry-run] Would delete: {:?}", full_path);
            return Ok(());
        }

        tokio::fs::remove_file(&full_path)
            .await
            .map_err(|e| MigrationError::io("Failed to delete file", Some(full_path), e))?;

        Ok(())
    }

    /// Check if file exists
    pub async fn file_exists(&self, path: impl AsRef<Path>) -> bool {
        let full_path = self.resolve_path(path);
        tokio::fs::metadata(&full_path).await.is_ok()
    }

    /// Check if path is a directory
    pub async fn is_dir(&self, path: impl AsRef<Path>) -> bool {
        let full_path = self.resolve_path(path);
        tokio::fs::metadata(&full_path)
            .await
            .map(|m| m.is_dir())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_context_state() {
        let temp = TempDir::new().unwrap();
        let ctx = MigrationContext::new(temp.path(), MigrationOptions::default());

        ctx.set_state("key1", "value1".to_string()).await;
        ctx.set_state("key2", 42i32).await;

        assert_eq!(
            ctx.get_state::<String>("key1").await,
            Some("value1".to_string())
        );
        assert_eq!(ctx.get_state::<i32>("key2").await, Some(42));
        assert_eq!(ctx.get_state::<String>("key3").await, None);

        assert!(ctx.has_state("key1").await);
        assert!(!ctx.has_state("key3").await);

        ctx.remove_state("key1").await;
        assert!(!ctx.has_state("key1").await);
    }

    #[tokio::test]
    async fn test_context_file_operations() {
        let temp = TempDir::new().unwrap();
        let ctx = MigrationContext::new(temp.path(), MigrationOptions::default());

        // Write file
        ctx.write_file("test.txt", "hello").await.unwrap();
        assert!(ctx.file_exists("test.txt").await);

        // Read file
        let content = ctx.read_file("test.txt").await.unwrap();
        assert_eq!(content, "hello");

        // Rename file
        ctx.rename_file("test.txt", "renamed.txt").await.unwrap();
        assert!(!ctx.file_exists("test.txt").await);
        assert!(ctx.file_exists("renamed.txt").await);

        // Delete file
        ctx.delete_file("renamed.txt").await.unwrap();
        assert!(!ctx.file_exists("renamed.txt").await);
    }

    #[tokio::test]
    async fn test_dry_run_mode() {
        let temp = TempDir::new().unwrap();
        let ctx = MigrationContext::new(temp.path(), MigrationOptions::dry_run());

        // Dry-run should not create file
        ctx.write_file("test.txt", "hello").await.unwrap();
        assert!(!ctx.file_exists("test.txt").await);
    }
}
