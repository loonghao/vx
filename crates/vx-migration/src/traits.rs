//! Core traits for the migration framework.

use crate::context::MigrationContext;
use crate::error::{MigrationError, MigrationResult};
use crate::types::{MigrationMetadata, MigrationReport, MigrationStepResult};
use crate::version::{Version, VersionRange};
use async_trait::async_trait;
use std::any::Any;
use std::path::Path;

/// Migration plugin interface
#[async_trait]
pub trait Migration: Send + Sync {
    /// Return migration metadata
    fn metadata(&self) -> MigrationMetadata;

    /// Check if this migration needs to run
    async fn check(&self, ctx: &MigrationContext) -> MigrationResult<bool>;

    /// Execute the migration
    async fn migrate(&self, ctx: &mut MigrationContext) -> MigrationResult<MigrationStepResult>;

    /// Rollback the migration (optional)
    async fn rollback(&self, _ctx: &mut MigrationContext) -> MigrationResult<()> {
        Ok(()) // Default: no rollback support
    }

    /// Validate migration result (optional)
    async fn validate(&self, _ctx: &MigrationContext) -> MigrationResult<bool> {
        Ok(true)
    }

    /// Get dependencies (migration IDs that must run first)
    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }

    /// For downcasting
    fn as_any(&self) -> &dyn Any;
}

/// Migration lifecycle hooks
#[async_trait]
pub trait MigrationHook: Send + Sync {
    /// Hook name
    fn name(&self) -> &str;

    /// Called before migration starts
    async fn pre_migrate(&self, _ctx: &MigrationContext) -> MigrationResult<()> {
        Ok(())
    }

    /// Called after migration completes
    async fn post_migrate(
        &self,
        _ctx: &MigrationContext,
        _report: &MigrationReport,
    ) -> MigrationResult<()> {
        Ok(())
    }

    /// Called before each migration step
    async fn pre_step(
        &self,
        _ctx: &MigrationContext,
        _migration: &dyn Migration,
    ) -> MigrationResult<()> {
        Ok(())
    }

    /// Called after each migration step
    async fn post_step(
        &self,
        _ctx: &MigrationContext,
        _migration: &dyn Migration,
        _result: &MigrationStepResult,
    ) -> MigrationResult<()> {
        Ok(())
    }

    /// Called on error
    async fn on_error(
        &self,
        _ctx: &MigrationContext,
        _error: &MigrationError,
    ) -> MigrationResult<()> {
        Ok(())
    }

    /// Called on rollback
    async fn on_rollback(
        &self,
        _ctx: &MigrationContext,
        _migration: &dyn Migration,
    ) -> MigrationResult<()> {
        Ok(())
    }
}

/// Version detector interface
#[async_trait]
pub trait VersionDetector: Send + Sync {
    /// Detector name
    fn name(&self) -> &str;

    /// Detect version from path
    async fn detect(&self, path: &Path) -> MigrationResult<Option<Version>>;

    /// Supported version range
    fn supported_range(&self) -> VersionRange;
}

/// Content transformer interface
#[async_trait]
pub trait ContentTransformer: Send + Sync {
    /// Transformer name
    fn name(&self) -> &str;

    /// Transform content
    async fn transform(&self, content: &str) -> MigrationResult<String>;

    /// Check if transformation is needed
    fn needs_transform(&self, content: &str) -> bool;
}

/// Backup manager interface
#[async_trait]
pub trait BackupManager: Send + Sync {
    /// Create a backup
    async fn backup(&self, ctx: &MigrationContext) -> MigrationResult<String>;

    /// Restore from backup
    async fn restore(&self, ctx: &MigrationContext, backup_id: &str) -> MigrationResult<()>;

    /// List available backups
    async fn list(&self, ctx: &MigrationContext) -> MigrationResult<Vec<String>>;

    /// Delete a backup
    async fn delete(&self, backup_id: &str) -> MigrationResult<()>;
}
