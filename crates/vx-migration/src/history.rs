//! Migration history tracking.

use crate::error::{MigrationError, MigrationResult};
use crate::types::{Change, MigrationStepResult};
use crate::version::Version;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Migration history
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MigrationHistory {
    /// History format version
    pub version: String,
    /// History entries
    pub entries: Vec<MigrationHistoryEntry>,
}

/// A single history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationHistoryEntry {
    /// Unique ID
    pub id: String,
    /// Migration ID
    pub migration_id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Source version
    pub from_version: Option<String>,
    /// Target version
    pub to_version: Option<String>,
    /// Status
    pub status: MigrationStatus,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Changes made
    pub changes: Vec<Change>,
    /// Machine identifier
    pub machine_id: String,
    /// Error message if failed
    pub error: Option<String>,
}

/// Migration status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MigrationStatus {
    /// Migration completed successfully
    Completed,
    /// Migration failed
    Failed,
    /// Migration was rolled back
    RolledBack,
    /// Migration was skipped
    Skipped,
}

impl MigrationHistory {
    /// Create a new history
    pub fn new() -> Self {
        Self {
            version: "1.0.0".to_string(),
            entries: Vec::new(),
        }
    }

    /// Load history from file
    pub async fn load(path: &Path) -> MigrationResult<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let content = tokio::fs::read_to_string(path).await.map_err(|e| {
            MigrationError::io("Failed to read history", Some(path.to_path_buf()), e)
        })?;

        serde_json::from_str(&content)
            .map_err(|e| MigrationError::Serialization(format!("Failed to parse history: {}", e)))
    }

    /// Save history to file
    pub async fn save(&self, path: &Path) -> MigrationResult<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                MigrationError::io("Failed to create directory", Some(parent.to_path_buf()), e)
            })?;
        }

        let content = serde_json::to_string_pretty(self).map_err(|e| {
            MigrationError::Serialization(format!("Failed to serialize history: {}", e))
        })?;

        tokio::fs::write(path, content).await.map_err(|e| {
            MigrationError::io("Failed to write history", Some(path.to_path_buf()), e)
        })?;

        Ok(())
    }

    /// Add an entry
    pub fn add_entry(&mut self, entry: MigrationHistoryEntry) {
        self.entries.push(entry);
    }

    /// Get entries for a specific migration
    pub fn get_entries(&self, migration_id: &str) -> Vec<&MigrationHistoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.migration_id == migration_id)
            .collect()
    }

    /// Check if a migration has been completed
    pub fn is_completed(&self, migration_id: &str) -> bool {
        self.entries
            .iter()
            .any(|e| e.migration_id == migration_id && e.status == MigrationStatus::Completed)
    }

    /// Get the last entry
    pub fn last_entry(&self) -> Option<&MigrationHistoryEntry> {
        self.entries.last()
    }

    /// Get entries count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get default history path
    pub fn default_path() -> PathBuf {
        vx_paths::VxPaths::default()
            .base_dir
            .join("migration-history.json")
    }
}

impl MigrationHistoryEntry {
    /// Create a new entry
    pub fn new(migration_id: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            migration_id: migration_id.into(),
            timestamp: Utc::now(),
            from_version: None,
            to_version: None,
            status: MigrationStatus::Completed,
            duration_ms: 0,
            changes: Vec::new(),
            machine_id: hostname::get()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_else(|_| "unknown".to_string()),
            error: None,
        }
    }

    /// Set versions
    pub fn with_versions(mut self, from: Option<&Version>, to: Option<&Version>) -> Self {
        self.from_version = from.map(|v| v.to_string());
        self.to_version = to.map(|v| v.to_string());
        self
    }

    /// Set status
    pub fn with_status(mut self, status: MigrationStatus) -> Self {
        self.status = status;
        self
    }

    /// Set from step result
    pub fn from_result(mut self, result: &MigrationStepResult) -> Self {
        self.duration_ms = result.duration.as_millis() as u64;
        self.changes = result.changes.clone();
        self.status = if result.success {
            MigrationStatus::Completed
        } else {
            MigrationStatus::Failed
        };
        self
    }

    /// Set error
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self.status = MigrationStatus::Failed;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_history_save_load() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("history.json");

        let mut history = MigrationHistory::new();
        history.add_entry(MigrationHistoryEntry::new("test-migration"));

        history.save(&path).await.unwrap();

        let loaded = MigrationHistory::load(&path).await.unwrap();
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].migration_id, "test-migration");
    }

    #[tokio::test]
    async fn test_history_queries() {
        let mut history = MigrationHistory::new();
        history.add_entry(MigrationHistoryEntry::new("m1").with_status(MigrationStatus::Completed));
        history.add_entry(MigrationHistoryEntry::new("m2").with_status(MigrationStatus::Failed));
        history.add_entry(MigrationHistoryEntry::new("m1").with_status(MigrationStatus::Completed));

        assert!(history.is_completed("m1"));
        assert!(!history.is_completed("m2"));
        assert!(!history.is_completed("m3"));

        assert_eq!(history.get_entries("m1").len(), 2);
        assert_eq!(history.get_entries("m2").len(), 1);
    }
}
