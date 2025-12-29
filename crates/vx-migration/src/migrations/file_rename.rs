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
    use crate::types::{ChangeType, MigrationOptions};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_check_needs_rename() {
        let temp = TempDir::new().unwrap();
        tokio::fs::write(temp.path().join("vx.toml"), "[tools]")
            .await
            .unwrap();

        let ctx = MigrationContext::new(temp.path(), MigrationOptions::default());
        let migration = FileRenameMigration::new();

        assert!(migration.check(&ctx).await.unwrap());
    }

    #[tokio::test]
    async fn test_check_already_renamed() {
        let temp = TempDir::new().unwrap();
        tokio::fs::write(temp.path().join("vx.toml"), "[runtimes]")
            .await
            .unwrap();

        let ctx = MigrationContext::new(temp.path(), MigrationOptions::default());
        let migration = FileRenameMigration::new();

        assert!(!migration.check(&ctx).await.unwrap());
    }

    #[tokio::test]
    async fn test_check_both_exist() {
        let temp = TempDir::new().unwrap();
        tokio::fs::write(temp.path().join("vx.toml"), "[tools]")
            .await
            .unwrap();
        tokio::fs::write(temp.path().join("vx.toml"), "[runtimes]")
            .await
            .unwrap();

        let ctx = MigrationContext::new(temp.path(), MigrationOptions::default());
        let migration = FileRenameMigration::new();

        // Should not rename if both exist
        assert!(!migration.check(&ctx).await.unwrap());
    }

    #[tokio::test]
    async fn test_migrate_rename() {
        let temp = TempDir::new().unwrap();
        tokio::fs::write(temp.path().join("vx.toml"), "[tools]\nnode = \"18\"")
            .await
            .unwrap();

        let mut ctx = MigrationContext::new(temp.path(), MigrationOptions::default());
        let migration = FileRenameMigration::new();

        let result = migration.migrate(&mut ctx).await.unwrap();
        assert!(result.success);
        assert_eq!(result.changes.len(), 1);
        assert_eq!(result.changes[0].change_type, ChangeType::Renamed);

        assert!(!temp.path().join("vx.toml").exists());
        assert!(temp.path().join("vx.toml").exists());

        let content = tokio::fs::read_to_string(temp.path().join("vx.toml"))
            .await
            .unwrap();
        assert!(content.contains("[tools]"));
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

        // File should not be renamed in dry-run
        assert!(temp.path().join("vx.toml").exists());
        assert!(!temp.path().join("vx.toml").exists());
    }

    #[tokio::test]
    async fn test_rollback() {
        let temp = TempDir::new().unwrap();
        tokio::fs::write(temp.path().join("vx.toml"), "[runtimes]")
            .await
            .unwrap();

        let mut ctx = MigrationContext::new(temp.path(), MigrationOptions::default());
        let migration = FileRenameMigration::new();

        migration.rollback(&mut ctx).await.unwrap();

        assert!(!temp.path().join("vx.toml").exists());
        assert!(temp.path().join("vx.toml").exists());
    }
}
