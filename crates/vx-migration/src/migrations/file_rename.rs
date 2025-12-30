//! Migration for renaming config files.

use crate::context::MigrationContext;
use crate::error::MigrationResult;
use crate::traits::Migration;
use crate::types::{
    Change, MigrationCategory, MigrationMetadata, MigrationPriority, MigrationStepResult,
};
use crate::version::{Version, VersionRange};
use async_trait::async_trait;
use std::any::Any;

/// Migration for renaming vx.toml to vx.toml
pub struct FileRenameMigration;

impl FileRenameMigration {
    /// Create a new migration
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileRenameMigration {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Migration for FileRenameMigration {
    fn metadata(&self) -> MigrationMetadata {
        MigrationMetadata::new(
            "file-rename",
            "Config File Rename",
            "Renames vx.toml to vx.toml",
        )
        .with_versions(VersionRange::any(), Version::new(2, 0, 0))
        .with_category(MigrationCategory::FileStructure)
        .with_priority(MigrationPriority::Critical)
        .reversible()
    }

    async fn check(&self, ctx: &MigrationContext) -> MigrationResult<bool> {
        // Check if vx.toml exists and vx.toml doesn't
        let old_exists = ctx.file_exists("vx.toml").await;
        let new_exists = ctx.file_exists("vx.toml").await;

        Ok(old_exists && !new_exists)
    }

    async fn migrate(&self, ctx: &mut MigrationContext) -> MigrationResult<MigrationStepResult> {
        let mut changes = Vec::new();
        let warnings = Vec::new();

        if ctx.file_exists("vx.toml").await && !ctx.file_exists("vx.toml").await {
            ctx.rename_file("vx.toml", "vx.toml").await?;
            changes.push(Change::rename("vx.toml", "vx.toml"));
        }

        Ok(MigrationStepResult {
            success: true,
            changes,
            warnings,
            duration: std::time::Duration::ZERO,
        })
    }

    async fn rollback(&self, ctx: &mut MigrationContext) -> MigrationResult<()> {
        if ctx.file_exists("vx.toml").await && !ctx.file_exists("vx.toml").await {
            ctx.rename_file("vx.toml", "vx.toml").await?;
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MigrationOptions;
    use tempfile::TempDir;

    // Note: Currently CONFIG_FILE_NAME and CONFIG_FILE_NAME_LEGACY are both "vx.toml",
    // so this migration is effectively a no-op. These tests verify the current behavior
    // where the migration correctly identifies that no rename is needed.

    #[tokio::test]
    async fn test_check_same_filename_no_rename_needed() {
        // When old and new filenames are the same (both vx.toml),
        // check should return false since no rename is needed
        let temp = TempDir::new().unwrap();
        tokio::fs::write(temp.path().join("vx.toml"), "[tools]")
            .await
            .unwrap();

        let ctx = MigrationContext::new(temp.path(), MigrationOptions::default());
        let migration = FileRenameMigration::new();

        // Since old_exists && !new_exists is always false when filenames are identical,
        // the migration should report no rename needed
        assert!(!migration.check(&ctx).await.unwrap());
    }

    #[tokio::test]
    async fn test_check_no_config_file() {
        let temp = TempDir::new().unwrap();

        let ctx = MigrationContext::new(temp.path(), MigrationOptions::default());
        let migration = FileRenameMigration::new();

        // No config file exists, so no rename needed
        assert!(!migration.check(&ctx).await.unwrap());
    }

    #[tokio::test]
    async fn test_migrate_no_changes_when_same_filename() {
        let temp = TempDir::new().unwrap();
        tokio::fs::write(temp.path().join("vx.toml"), "[tools]\nnode = \"18\"")
            .await
            .unwrap();

        let mut ctx = MigrationContext::new(temp.path(), MigrationOptions::default());
        let migration = FileRenameMigration::new();

        let result = migration.migrate(&mut ctx).await.unwrap();
        assert!(result.success);
        // No changes since old and new filenames are the same
        assert_eq!(result.changes.len(), 0);

        // File should still exist
        assert!(temp.path().join("vx.toml").exists());
    }

    #[tokio::test]
    async fn test_migrate_dry_run() {
        let temp = TempDir::new().unwrap();
        tokio::fs::write(temp.path().join("vx.toml"), "[tools]")
            .await
            .unwrap();

        let mut ctx = MigrationContext::new(temp.path(), MigrationOptions::dry_run());
        let migration = FileRenameMigration::new();

        let result = migration.migrate(&mut ctx).await.unwrap();
        assert!(result.success);

        // File should still exist (no rename happened)
        assert!(temp.path().join("vx.toml").exists());
    }

    #[tokio::test]
    async fn test_rollback_no_op() {
        let temp = TempDir::new().unwrap();
        tokio::fs::write(temp.path().join("vx.toml"), "[runtimes]")
            .await
            .unwrap();

        let mut ctx = MigrationContext::new(temp.path(), MigrationOptions::default());
        let migration = FileRenameMigration::new();

        // Rollback should succeed (no-op when filenames are same)
        migration.rollback(&mut ctx).await.unwrap();

        // File should still exist
        assert!(temp.path().join("vx.toml").exists());
    }

    #[tokio::test]
    async fn test_metadata() {
        let migration = FileRenameMigration::new();
        let metadata = migration.metadata();

        assert_eq!(metadata.id, "file-rename");
        assert!(metadata.reversible);
    }
}
