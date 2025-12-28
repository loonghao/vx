//! Migration framework tests

use rstest::rstest;
use tempfile::TempDir;
use vx_migration::migrations::{create_default_engine, ConfigV1ToV2Migration, FileRenameMigration};
use vx_migration::prelude::*;
use vx_migration::traits::VersionDetector;
use vx_migration::version::{Version, VersionRange};

mod version_tests {
    use super::*;

    #[test]
    fn test_version_parse() {
        let v: Version = "1.2.3".parse().unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_version_with_v_prefix() {
        let v: Version = "v1.2.3".parse().unwrap();
        assert_eq!(v.major, 1);
    }

    #[test]
    fn test_version_prerelease() {
        let v: Version = "1.0.0-alpha.1".parse().unwrap();
        assert_eq!(v.pre, Some("alpha.1".to_string()));
    }

    #[rstest]
    #[case("1.0.0", "2.0.0", std::cmp::Ordering::Less)]
    #[case("1.1.0", "1.0.0", std::cmp::Ordering::Greater)]
    #[case("1.0.0", "1.0.0", std::cmp::Ordering::Equal)]
    #[case("1.0.0-alpha", "1.0.0", std::cmp::Ordering::Less)]
    fn test_version_ordering(
        #[case] a: &str,
        #[case] b: &str,
        #[case] expected: std::cmp::Ordering,
    ) {
        let va: Version = a.parse().unwrap();
        let vb: Version = b.parse().unwrap();
        assert_eq!(va.cmp(&vb), expected);
    }

    #[test]
    fn test_version_range_any() {
        let range = VersionRange::any();
        assert!(range.matches(&Version::new(0, 0, 1)));
        assert!(range.matches(&Version::new(100, 0, 0)));
    }

    #[test]
    fn test_version_range_exact() {
        let range = VersionRange::exact(Version::new(1, 0, 0));
        assert!(range.matches(&Version::new(1, 0, 0)));
        assert!(!range.matches(&Version::new(1, 0, 1)));
    }

    #[test]
    fn test_version_range_gte() {
        let range = VersionRange::gte(Version::new(2, 0, 0));
        assert!(!range.matches(&Version::new(1, 9, 9)));
        assert!(range.matches(&Version::new(2, 0, 0)));
        assert!(range.matches(&Version::new(3, 0, 0)));
    }

    #[test]
    fn test_version_range_lt() {
        let range = VersionRange::lt(Version::new(2, 0, 0));
        assert!(range.matches(&Version::new(1, 9, 9)));
        assert!(!range.matches(&Version::new(2, 0, 0)));
    }
}

mod context_tests {
    use super::*;

    #[tokio::test]
    async fn test_context_state() {
        let temp = TempDir::new().unwrap();
        let ctx = MigrationContext::new(temp.path(), MigrationOptions::default());

        ctx.set_state("key", "value".to_string()).await;
        assert_eq!(
            ctx.get_state::<String>("key").await,
            Some("value".to_string())
        );
    }

    #[tokio::test]
    async fn test_context_file_operations() {
        let temp = TempDir::new().unwrap();
        let ctx = MigrationContext::new(temp.path(), MigrationOptions::default());

        ctx.write_file("test.txt", "hello").await.unwrap();
        assert!(ctx.file_exists("test.txt").await);

        let content = ctx.read_file("test.txt").await.unwrap();
        assert_eq!(content, "hello");
    }

    #[tokio::test]
    async fn test_context_dry_run() {
        let temp = TempDir::new().unwrap();
        let ctx = MigrationContext::new(temp.path(), MigrationOptions::dry_run());

        ctx.write_file("test.txt", "hello").await.unwrap();
        assert!(!ctx.file_exists("test.txt").await);
    }
}

mod registry_tests {
    use super::*;

    #[test]
    fn test_registry_register() {
        let mut registry = MigrationRegistry::new();
        registry.register(FileRenameMigration::new()).unwrap();
        assert!(registry.contains("file-rename"));
    }

    #[test]
    fn test_registry_duplicate() {
        let mut registry = MigrationRegistry::new();
        registry.register(FileRenameMigration::new()).unwrap();
        assert!(registry.register(FileRenameMigration::new()).is_err());
    }
}

mod engine_tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_check() {
        let temp = TempDir::new().unwrap();
        tokio::fs::write(temp.path().join(".vx.toml"), "[tools]\nnode = \"18\"")
            .await
            .unwrap();

        let engine = create_default_engine();
        let needed = engine.check(temp.path()).await.unwrap();

        assert!(!needed.is_empty());
    }

    #[tokio::test]
    async fn test_engine_migrate() {
        let temp = TempDir::new().unwrap();
        tokio::fs::write(temp.path().join(".vx.toml"), "[tools]\nnode = \"18\"")
            .await
            .unwrap();

        let engine = create_default_engine();
        let result = engine
            .migrate(temp.path(), &MigrationOptions::default())
            .await
            .unwrap();

        assert!(result.success);
        assert!(temp.path().join("vx.toml").exists());
        assert!(!temp.path().join(".vx.toml").exists());

        let content = tokio::fs::read_to_string(temp.path().join("vx.toml"))
            .await
            .unwrap();
        assert!(content.contains("[runtimes]"));
    }

    #[tokio::test]
    async fn test_engine_dry_run() {
        let temp = TempDir::new().unwrap();
        tokio::fs::write(temp.path().join(".vx.toml"), "[tools]\nnode = \"18\"")
            .await
            .unwrap();

        let engine = create_default_engine();
        let result = engine
            .migrate(temp.path(), &MigrationOptions::dry_run())
            .await
            .unwrap();

        assert!(result.success);
        // Files should not be changed in dry-run
        assert!(temp.path().join(".vx.toml").exists());
        assert!(!temp.path().join("vx.toml").exists());
    }

    #[tokio::test]
    async fn test_engine_skip_migration() {
        let temp = TempDir::new().unwrap();
        tokio::fs::write(temp.path().join(".vx.toml"), "[tools]\nnode = \"18\"")
            .await
            .unwrap();

        let engine = create_default_engine();
        let mut options = MigrationOptions::default();
        options.skip_migrations.insert("file-rename".to_string());

        let result = engine.migrate(temp.path(), &options).await.unwrap();

        assert!(result.success);
        // file-rename was skipped, so .vx.toml should still exist
        assert!(temp.path().join(".vx.toml").exists());
    }
}

mod migration_tests {
    use super::*;
    use vx_migration::migrations::VxVersionDetector;

    #[tokio::test]
    async fn test_file_rename_migration() {
        let temp = TempDir::new().unwrap();
        tokio::fs::write(temp.path().join(".vx.toml"), "[tools]")
            .await
            .unwrap();

        let mut ctx = MigrationContext::new(temp.path(), MigrationOptions::default());
        let migration = FileRenameMigration::new();

        assert!(migration.check(&ctx).await.unwrap());

        let result = migration.migrate(&mut ctx).await.unwrap();
        assert!(result.success);
        assert!(!temp.path().join(".vx.toml").exists());
        assert!(temp.path().join("vx.toml").exists());
    }

    #[tokio::test]
    async fn test_config_v1_to_v2_migration() {
        let temp = TempDir::new().unwrap();
        tokio::fs::write(
            temp.path().join("vx.toml"),
            "[tools]\nnode = \"18\"\n\n[tools.go]\nversion = \"1.21\"",
        )
        .await
        .unwrap();

        let mut ctx = MigrationContext::new(temp.path(), MigrationOptions::default());
        let migration = ConfigV1ToV2Migration::new();

        assert!(migration.check(&ctx).await.unwrap());

        let result = migration.migrate(&mut ctx).await.unwrap();
        assert!(result.success);

        let content = tokio::fs::read_to_string(temp.path().join("vx.toml"))
            .await
            .unwrap();
        assert!(content.contains("[runtimes]"));
        assert!(content.contains("[runtimes.go]"));
    }

    #[tokio::test]
    async fn test_version_detector() {
        let temp = TempDir::new().unwrap();
        tokio::fs::write(temp.path().join("vx.toml"), "[tools]\nnode = \"18\"")
            .await
            .unwrap();

        let detector = VxVersionDetector::new();
        let version = detector.detect(temp.path()).await.unwrap();

        assert_eq!(version, Some(Version::new(1, 0, 0)));
    }
}

mod history_tests {
    use super::*;

    #[tokio::test]
    async fn test_history_save_load() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("history.json");

        let mut history = MigrationHistory::new();
        history.add_entry(MigrationHistoryEntry::new("test-migration"));

        history.save(&path).await.unwrap();

        let loaded = MigrationHistory::load(&path).await.unwrap();
        assert_eq!(loaded.entries.len(), 1);
    }

    #[tokio::test]
    async fn test_history_is_completed() {
        use vx_migration::history::MigrationStatus;

        let mut history = MigrationHistory::new();
        history.add_entry(MigrationHistoryEntry::new("m1").with_status(MigrationStatus::Completed));
        history.add_entry(MigrationHistoryEntry::new("m2").with_status(MigrationStatus::Failed));

        assert!(history.is_completed("m1"));
        assert!(!history.is_completed("m2"));
    }
}
