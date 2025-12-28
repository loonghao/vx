//! Migration registry for managing migrations.

use crate::error::{MigrationError, MigrationResult};
use crate::traits::{Migration, MigrationHook};
use crate::types::MigrationMetadata;
use std::collections::HashMap;
use std::sync::Arc;

/// Registry for migrations and hooks
pub struct MigrationRegistry {
    /// Registered migrations
    migrations: HashMap<String, Arc<dyn Migration>>,
    /// Registered hooks
    hooks: Vec<Arc<dyn MigrationHook>>,
}

impl Default for MigrationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MigrationRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            migrations: HashMap::new(),
            hooks: Vec::new(),
        }
    }

    /// Register a migration
    pub fn register<M: Migration + 'static>(&mut self, migration: M) -> MigrationResult<()> {
        let id = migration.metadata().id;
        if self.migrations.contains_key(&id) {
            return Err(MigrationError::Other(format!(
                "Migration '{id}' already registered"
            )));
        }
        self.migrations.insert(id, Arc::new(migration));
        Ok(())
    }

    /// Register a hook
    pub fn register_hook<H: MigrationHook + 'static>(&mut self, hook: H) {
        self.hooks.push(Arc::new(hook));
    }

    /// Get a migration by ID
    pub fn get(&self, id: &str) -> Option<Arc<dyn Migration>> {
        self.migrations.get(id).cloned()
    }

    /// Get all migrations
    pub fn all(&self) -> Vec<Arc<dyn Migration>> {
        self.migrations.values().cloned().collect()
    }

    /// Get all migration metadata
    pub fn metadata(&self) -> Vec<MigrationMetadata> {
        self.migrations.values().map(|m| m.metadata()).collect()
    }

    /// Get migrations sorted by priority and dependencies
    pub fn sorted(&self) -> MigrationResult<Vec<Arc<dyn Migration>>> {
        let mut sorted = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut visiting = std::collections::HashSet::new();

        for id in self.migrations.keys() {
            self.visit_migration(id, &mut sorted, &mut visited, &mut visiting)?;
        }

        // Sort by priority
        sorted.sort_by_key(|m| m.metadata().priority);

        Ok(sorted)
    }

    /// Topological sort helper
    fn visit_migration(
        &self,
        id: &str,
        sorted: &mut Vec<Arc<dyn Migration>>,
        visited: &mut std::collections::HashSet<String>,
        visiting: &mut std::collections::HashSet<String>,
    ) -> MigrationResult<()> {
        if visited.contains(id) {
            return Ok(());
        }

        if visiting.contains(id) {
            return Err(MigrationError::dependency(format!(
                "Circular dependency detected: {}",
                id
            )));
        }

        let migration = self
            .migrations
            .get(id)
            .ok_or_else(|| MigrationError::NotFound(id.to_string()))?;

        visiting.insert(id.to_string());

        for dep in migration.dependencies() {
            self.visit_migration(dep, sorted, visited, visiting)?;
        }

        visiting.remove(id);
        visited.insert(id.to_string());
        sorted.push(migration.clone());

        Ok(())
    }

    /// Get all hooks
    pub fn hooks(&self) -> &[Arc<dyn MigrationHook>] {
        &self.hooks
    }

    /// Check if a migration is registered
    pub fn contains(&self, id: &str) -> bool {
        self.migrations.contains_key(id)
    }

    /// Get migration count
    pub fn len(&self) -> usize {
        self.migrations.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.migrations.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::MigrationContext;
    use crate::types::{MigrationMetadata, MigrationStepResult};
    use crate::version::VersionRange;
    use async_trait::async_trait;
    use std::any::Any;

    struct TestMigration {
        id: String,
        deps: Vec<String>,
    }

    impl TestMigration {
        fn new(id: &str) -> Self {
            Self {
                id: id.to_string(),
                deps: vec![],
            }
        }

        fn with_deps(id: &str, deps: Vec<&str>) -> Self {
            Self {
                id: id.to_string(),
                deps: deps.into_iter().map(String::from).collect(),
            }
        }
    }

    #[async_trait]
    impl Migration for TestMigration {
        fn metadata(&self) -> MigrationMetadata {
            MigrationMetadata::new(&self.id, &self.id, "Test migration")
                .with_versions(VersionRange::any(), crate::version::Version::new(1, 0, 0))
        }

        async fn check(&self, _ctx: &MigrationContext) -> MigrationResult<bool> {
            Ok(true)
        }

        async fn migrate(
            &self,
            _ctx: &mut MigrationContext,
        ) -> MigrationResult<MigrationStepResult> {
            Ok(MigrationStepResult::success())
        }

        fn dependencies(&self) -> Vec<&str> {
            self.deps.iter().map(|s| s.as_str()).collect()
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn test_register_migration() {
        let mut registry = MigrationRegistry::new();
        registry.register(TestMigration::new("test1")).unwrap();
        registry.register(TestMigration::new("test2")).unwrap();

        assert!(registry.contains("test1"));
        assert!(registry.contains("test2"));
        assert!(!registry.contains("test3"));
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_duplicate_registration() {
        let mut registry = MigrationRegistry::new();
        registry.register(TestMigration::new("test1")).unwrap();
        assert!(registry.register(TestMigration::new("test1")).is_err());
    }

    #[test]
    fn test_dependency_sorting() {
        let mut registry = MigrationRegistry::new();
        registry
            .register(TestMigration::with_deps("c", vec!["b"]))
            .unwrap();
        registry
            .register(TestMigration::with_deps("b", vec!["a"]))
            .unwrap();
        registry.register(TestMigration::new("a")).unwrap();

        let sorted = registry.sorted().unwrap();
        let ids: Vec<_> = sorted.iter().map(|m| m.metadata().id.clone()).collect();

        // a should come before b, b before c
        let pos_a = ids.iter().position(|id| id == "a").unwrap();
        let pos_b = ids.iter().position(|id| id == "b").unwrap();
        let pos_c = ids.iter().position(|id| id == "c").unwrap();

        assert!(pos_a < pos_b);
        assert!(pos_b < pos_c);
    }

    #[test]
    fn test_circular_dependency() {
        let mut registry = MigrationRegistry::new();
        registry
            .register(TestMigration::with_deps("a", vec!["b"]))
            .unwrap();
        registry
            .register(TestMigration::with_deps("b", vec!["a"]))
            .unwrap();

        assert!(registry.sorted().is_err());
    }
}
