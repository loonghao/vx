//! Migration engine - the main entry point for running migrations.

use crate::context::MigrationContext;
use crate::error::{MigrationError, MigrationResult};
use crate::registry::MigrationRegistry;
use crate::traits::{Migration, MigrationHook};
use crate::types::{
    MigrationMetadata, MigrationOptions, MigrationReport, MigrationStepReport, MigrationStepResult,
};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

/// Migration engine for executing migrations
pub struct MigrationEngine {
    registry: MigrationRegistry,
}

impl Default for MigrationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl MigrationEngine {
    /// Create a new engine
    pub fn new() -> Self {
        Self {
            registry: MigrationRegistry::new(),
        }
    }

    /// Register a migration (builder pattern)
    pub fn register<M: Migration + 'static>(mut self, migration: M) -> Self {
        let _ = self.registry.register(migration);
        self
    }

    /// Register a hook (builder pattern)
    pub fn register_hook<H: MigrationHook + 'static>(mut self, hook: H) -> Self {
        self.registry.register_hook(hook);
        self
    }

    /// Add a migration to the registry
    pub fn add_migration<M: Migration + 'static>(&mut self, migration: M) -> MigrationResult<()> {
        self.registry.register(migration)
    }

    /// Add a hook to the registry
    pub fn add_hook<H: MigrationHook + 'static>(&mut self, hook: H) {
        self.registry.register_hook(hook)
    }

    /// Get all migration metadata
    pub fn migrations(&self) -> Vec<MigrationMetadata> {
        self.registry.metadata()
    }

    /// Check which migrations need to run
    pub async fn check(&self, path: &Path) -> MigrationResult<Vec<MigrationMetadata>> {
        let ctx = MigrationContext::new(path, MigrationOptions::default());
        let migrations = self.registry.sorted()?;

        let mut needed = Vec::new();
        for migration in migrations {
            if migration.check(&ctx).await? {
                needed.push(migration.metadata());
            }
        }

        Ok(needed)
    }

    /// Execute migrations
    pub async fn migrate(
        &self,
        path: &Path,
        options: &MigrationOptions,
    ) -> MigrationResult<MigrationReport> {
        let start = Instant::now();
        let mut ctx = MigrationContext::new(path, options.clone());
        let mut report = MigrationReport::default();

        // Get sorted migrations
        let migrations = self.registry.sorted()?;
        let hooks = self.registry.hooks();

        // Pre-migrate hooks
        for hook in hooks {
            hook.pre_migrate(&ctx).await.map_err(|e| {
                MigrationError::hook(hook.name(), format!("pre_migrate failed: {}", e))
            })?;
        }

        // Execute each migration
        for migration in migrations {
            let metadata = migration.metadata();

            // Check if migration should be skipped
            if options.skip_migrations.contains(&metadata.id) {
                report.skipped_count += 1;
                report.steps.push(MigrationStepReport {
                    migration_id: metadata.id.clone(),
                    migration_name: metadata.name.clone(),
                    description: metadata.description.clone(),
                    result: MigrationStepResult::skipped(),
                    skipped: true,
                    error: None,
                });
                continue;
            }

            // Check if only specific migrations should run
            if let Some(only) = &options.only_migrations
                && !only.contains(&metadata.id)
            {
                report.skipped_count += 1;
                continue;
            }

            // Check if migration needs to run
            match migration.check(&ctx).await {
                Ok(true) => {}
                Ok(false) => {
                    report.skipped_count += 1;
                    report.steps.push(MigrationStepReport {
                        migration_id: metadata.id.clone(),
                        migration_name: metadata.name.clone(),
                        description: metadata.description.clone(),
                        result: MigrationStepResult::skipped(),
                        skipped: true,
                        error: None,
                    });
                    continue;
                }
                Err(e) => {
                    report.failed_count += 1;
                    report
                        .errors
                        .push(format!("Check failed for {}: {}", metadata.id, e));
                    if options.rollback_on_failure {
                        self.rollback_completed(&mut ctx, &report, hooks).await?;
                    }
                    report.success = false;
                    report.total_duration = start.elapsed();
                    return Ok(report);
                }
            }

            // Pre-step hooks
            for hook in hooks {
                if let Err(e) = hook.pre_step(&ctx, migration.as_ref()).await {
                    tracing::warn!("Hook {} pre_step failed: {}", hook.name(), e);
                }
            }

            // Execute migration
            let step_start = Instant::now();
            let step_result = match migration.migrate(&mut ctx).await {
                Ok(mut result) => {
                    result.duration = step_start.elapsed();
                    report.successful_count += 1;

                    // Validate if not dry-run
                    if !options.dry_run
                        && let Err(e) = migration.validate(&ctx).await
                    {
                        result.warnings.push(format!("Validation warning: {}", e));
                    }

                    result
                }
                Err(e) => {
                    report.failed_count += 1;
                    let error_msg = format!("Migration {} failed: {}", metadata.id, e);
                    report.errors.push(error_msg.clone());

                    // Call error hooks
                    for hook in hooks {
                        let _ = hook.on_error(&ctx, &e).await;
                    }

                    if options.rollback_on_failure {
                        self.rollback_completed(&mut ctx, &report, hooks).await?;
                    }

                    report.steps.push(MigrationStepReport {
                        migration_id: metadata.id.clone(),
                        migration_name: metadata.name.clone(),
                        description: metadata.description.clone(),
                        result: MigrationStepResult::default(),
                        skipped: false,
                        error: Some(error_msg),
                    });

                    report.success = false;
                    report.total_duration = start.elapsed();
                    return Ok(report);
                }
            };

            // Post-step hooks
            for hook in hooks {
                if let Err(e) = hook.post_step(&ctx, migration.as_ref(), &step_result).await {
                    tracing::warn!("Hook {} post_step failed: {}", hook.name(), e);
                }
            }

            report.steps.push(MigrationStepReport {
                migration_id: metadata.id.clone(),
                migration_name: metadata.name.clone(),
                description: metadata.description.clone(),
                result: step_result,
                skipped: false,
                error: None,
            });
        }

        // Post-migrate hooks
        for hook in hooks {
            if let Err(e) = hook.post_migrate(&ctx, &report).await {
                tracing::warn!("Hook {} post_migrate failed: {}", hook.name(), e);
            }
        }

        report.success = report.failed_count == 0;
        report.total_duration = start.elapsed();

        Ok(report)
    }

    /// Rollback completed migrations
    async fn rollback_completed(
        &self,
        ctx: &mut MigrationContext,
        report: &MigrationReport,
        hooks: &[Arc<dyn MigrationHook>],
    ) -> MigrationResult<()> {
        tracing::info!(
            "Rolling back {} completed migrations",
            report.successful_count
        );

        // Rollback in reverse order
        for step in report.steps.iter().rev() {
            if step.skipped || step.error.is_some() {
                continue;
            }

            if let Some(migration) = self.registry.get(&step.migration_id)
                && migration.metadata().reversible
            {
                // Call rollback hooks
                for hook in hooks {
                    let _ = hook.on_rollback(ctx, migration.as_ref()).await;
                }

                if let Err(e) = migration.rollback(ctx).await {
                    tracing::error!("Rollback failed for {}: {}", step.migration_id, e);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Change, ChangeType};
    use crate::version::{Version, VersionRange};
    use async_trait::async_trait;
    use std::any::Any;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tempfile::TempDir;

    struct CountingMigration {
        id: String,
        counter: Arc<AtomicUsize>,
        should_run: bool,
    }

    impl CountingMigration {
        fn new(id: &str, counter: Arc<AtomicUsize>, should_run: bool) -> Self {
            Self {
                id: id.to_string(),
                counter,
                should_run,
            }
        }
    }

    #[async_trait]
    impl Migration for CountingMigration {
        fn metadata(&self) -> MigrationMetadata {
            MigrationMetadata::new(&self.id, &self.id, "Test")
                .with_versions(VersionRange::any(), Version::new(1, 0, 0))
        }

        async fn check(&self, _ctx: &MigrationContext) -> MigrationResult<bool> {
            Ok(self.should_run)
        }

        async fn migrate(
            &self,
            _ctx: &mut MigrationContext,
        ) -> MigrationResult<MigrationStepResult> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(MigrationStepResult::success()
                .with_change(Change::new(ChangeType::Modified, "test.txt")))
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[tokio::test]
    async fn test_engine_migrate() {
        let temp = TempDir::new().unwrap();
        let counter = Arc::new(AtomicUsize::new(0));

        let engine = MigrationEngine::new()
            .register(CountingMigration::new("m1", counter.clone(), true))
            .register(CountingMigration::new("m2", counter.clone(), true))
            .register(CountingMigration::new("m3", counter.clone(), false));

        let result = engine
            .migrate(temp.path(), &MigrationOptions::default())
            .await
            .unwrap();

        assert!(result.success);
        assert_eq!(result.successful_count, 2);
        assert_eq!(result.skipped_count, 1);
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_engine_dry_run() {
        let temp = TempDir::new().unwrap();
        let counter = Arc::new(AtomicUsize::new(0));

        let engine =
            MigrationEngine::new().register(CountingMigration::new("m1", counter.clone(), true));

        let result = engine
            .migrate(temp.path(), &MigrationOptions::dry_run())
            .await
            .unwrap();

        assert!(result.success);
        assert_eq!(result.successful_count, 1);
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Still runs in dry-run
    }

    #[tokio::test]
    async fn test_engine_skip_migration() {
        let temp = TempDir::new().unwrap();
        let counter = Arc::new(AtomicUsize::new(0));

        let engine = MigrationEngine::new()
            .register(CountingMigration::new("m1", counter.clone(), true))
            .register(CountingMigration::new("m2", counter.clone(), true));

        let mut options = MigrationOptions::default();
        options.skip_migrations.insert("m1".to_string());

        let result = engine.migrate(temp.path(), &options).await.unwrap();

        assert!(result.success);
        assert_eq!(result.successful_count, 1);
        assert_eq!(result.skipped_count, 1);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
