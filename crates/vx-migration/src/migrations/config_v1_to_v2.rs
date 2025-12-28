//! Migration from config v1 to v2 format.

use crate::context::MigrationContext;
use crate::error::MigrationResult;
use crate::traits::Migration;
use crate::types::{
    Change, ChangeType, MigrationCategory, MigrationMetadata, MigrationPriority,
    MigrationStepResult,
};
use crate::version::{Version, VersionRange};
use async_trait::async_trait;
use std::any::Any;

/// Migration from config v1 ([tools]) to v2 ([runtimes]) format
pub struct ConfigV1ToV2Migration;

impl ConfigV1ToV2Migration {
    /// Create a new migration
    pub fn new() -> Self {
        Self
    }

    /// Transform v1 config to v2 format
    fn transform_config(&self, content: &str) -> MigrationResult<String> {
        let mut result = content.to_string();

        // Replace [tools] with [runtimes]
        result = result.replace("[tools]", "[runtimes]");

        // Replace [tools. with [runtimes.
        result = result.replace("[tools.", "[runtimes.");

        Ok(result)
    }

    /// Check if content is v1 format
    fn is_v1_format(&self, content: &str) -> bool {
        content.contains("[tools]") || content.contains("[tools.")
    }
}

impl Default for ConfigV1ToV2Migration {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Migration for ConfigV1ToV2Migration {
    fn metadata(&self) -> MigrationMetadata {
        MigrationMetadata::new(
            "config-v1-to-v2",
            "Config v1 to v2 Migration",
            "Migrates [tools] section to [runtimes] format",
        )
        .with_versions(
            VersionRange::lt(Version::new(2, 0, 0)),
            Version::new(2, 0, 0),
        )
        .with_category(MigrationCategory::Config)
        .with_priority(MigrationPriority::High)
        .reversible()
    }

    async fn check(&self, ctx: &MigrationContext) -> MigrationResult<bool> {
        let config_path = ctx.root_path().join("vx.toml");
        if !ctx.file_exists(&config_path).await {
            return Ok(false);
        }

        let content = ctx.read_file(&config_path).await?;
        Ok(self.is_v1_format(&content))
    }

    async fn migrate(&self, ctx: &mut MigrationContext) -> MigrationResult<MigrationStepResult> {
        let mut changes = Vec::new();
        let warnings = Vec::new();

        let config_path = "vx.toml";
        if ctx.file_exists(config_path).await {
            let content = ctx.read_file(config_path).await?;

            if self.is_v1_format(&content) {
                let new_content = self.transform_config(&content)?;
                ctx.write_file(config_path, &new_content).await?;

                changes.push(
                    Change::new(ChangeType::Modified, config_path)
                        .with_description("Converted [tools] to [runtimes]"),
                );
            }
        }

        Ok(MigrationStepResult {
            success: true,
            changes,
            warnings,
            duration: std::time::Duration::ZERO,
        })
    }

    async fn rollback(&self, _ctx: &mut MigrationContext) -> MigrationResult<()> {
        // Rollback would convert [runtimes] back to [tools]
        // For now, we don't implement automatic rollback
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

    #[tokio::test]
    async fn test_check_v1_config() {
        let temp = TempDir::new().unwrap();
        let config = r#"
[tools]
node = "18.0.0"
go = "1.21.0"
"#;
        tokio::fs::write(temp.path().join("vx.toml"), config)
            .await
            .unwrap();

        let ctx = MigrationContext::new(temp.path(), MigrationOptions::default());
        let migration = ConfigV1ToV2Migration::new();

        assert!(migration.check(&ctx).await.unwrap());
    }

    #[tokio::test]
    async fn test_check_v2_config() {
        let temp = TempDir::new().unwrap();
        let config = r#"
[runtimes]
node = "18.0.0"
go = "1.21.0"
"#;
        tokio::fs::write(temp.path().join("vx.toml"), config)
            .await
            .unwrap();

        let ctx = MigrationContext::new(temp.path(), MigrationOptions::default());
        let migration = ConfigV1ToV2Migration::new();

        assert!(!migration.check(&ctx).await.unwrap());
    }

    #[tokio::test]
    async fn test_migrate_v1_to_v2() {
        let temp = TempDir::new().unwrap();
        let config = r#"
[tools]
node = "18.0.0"

[tools.go]
version = "1.21.0"
"#;
        tokio::fs::write(temp.path().join("vx.toml"), config)
            .await
            .unwrap();

        let mut ctx = MigrationContext::new(temp.path(), MigrationOptions::default());
        let migration = ConfigV1ToV2Migration::new();

        let result = migration.migrate(&mut ctx).await.unwrap();
        assert!(result.success);
        assert_eq!(result.changes.len(), 1);

        let new_content = tokio::fs::read_to_string(temp.path().join("vx.toml"))
            .await
            .unwrap();
        assert!(new_content.contains("[runtimes]"));
        assert!(new_content.contains("[runtimes.go]"));
        assert!(!new_content.contains("[tools]"));
    }

    #[tokio::test]
    async fn test_migrate_dry_run() {
        let temp = TempDir::new().unwrap();
        let config = r#"
[tools]
node = "18.0.0"
"#;
        tokio::fs::write(temp.path().join("vx.toml"), config)
            .await
            .unwrap();

        let mut ctx = MigrationContext::new(temp.path(), MigrationOptions::dry_run());
        let migration = ConfigV1ToV2Migration::new();

        let result = migration.migrate(&mut ctx).await.unwrap();
        assert!(result.success);

        // File should not be modified in dry-run
        let content = tokio::fs::read_to_string(temp.path().join("vx.toml"))
            .await
            .unwrap();
        assert!(content.contains("[tools]"));
    }

    #[test]
    fn test_transform_config() {
        let migration = ConfigV1ToV2Migration::new();

        let v1 = r#"
[tools]
node = "18.0.0"

[tools.go]
version = "1.21.0"
"#;

        let v2 = migration.transform_config(v1).unwrap();
        assert!(v2.contains("[runtimes]"));
        assert!(v2.contains("[runtimes.go]"));
        assert!(!v2.contains("[tools]"));
    }
}
