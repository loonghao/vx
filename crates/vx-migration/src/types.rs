//! Common types for the migration framework.

use crate::version::{Version, VersionRange};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

/// Migration metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationMetadata {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Source version range
    pub from_version: VersionRange,
    /// Target version
    pub to_version: Version,
    /// Migration category
    pub category: MigrationCategory,
    /// Priority
    pub priority: MigrationPriority,
    /// Whether this migration is reversible
    pub reversible: bool,
    /// Whether this is a breaking change
    pub breaking: bool,
    /// Estimated duration in milliseconds
    pub estimated_duration_ms: Option<u64>,
}

impl MigrationMetadata {
    /// Create new metadata with required fields
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            from_version: VersionRange::any(),
            to_version: Version::default(),
            category: MigrationCategory::Config,
            priority: MigrationPriority::Normal,
            reversible: false,
            breaking: false,
            estimated_duration_ms: None,
        }
    }

    /// Set version range
    pub fn with_versions(mut self, from: VersionRange, to: Version) -> Self {
        self.from_version = from;
        self.to_version = to;
        self
    }

    /// Set category
    pub fn with_category(mut self, category: MigrationCategory) -> Self {
        self.category = category;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: MigrationPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Mark as reversible
    pub fn reversible(mut self) -> Self {
        self.reversible = true;
        self
    }

    /// Mark as breaking
    pub fn breaking(mut self) -> Self {
        self.breaking = true;
        self
    }
}

/// Migration category
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum MigrationCategory {
    /// Configuration file migration
    #[default]
    Config,
    /// File structure changes
    FileStructure,
    /// Data format conversion
    Data,
    /// Schema changes
    Schema,
    /// Environment setup
    Environment,
    /// Custom category
    Custom(String),
}

/// Migration priority
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub enum MigrationPriority {
    /// Must execute first
    Critical = 0,
    /// High priority
    High = 1,
    /// Default priority
    #[default]
    Normal = 2,
    /// Low priority
    Low = 3,
    /// Cleanup tasks, execute last
    Cleanup = 4,
}

/// Migration options
#[derive(Debug, Clone, Default)]
pub struct MigrationOptions {
    /// Dry-run mode (preview only)
    pub dry_run: bool,
    /// Create backup before migration
    pub backup: bool,
    /// Backup directory
    pub backup_dir: Option<PathBuf>,
    /// Target version (None = latest)
    pub target_version: Option<Version>,
    /// Interactive mode
    pub interactive: bool,
    /// Verbose output
    pub verbose: bool,
    /// Rollback on failure
    pub rollback_on_failure: bool,
    /// Migrations to skip
    pub skip_migrations: HashSet<String>,
    /// Only run these migrations
    pub only_migrations: Option<HashSet<String>>,
}

impl MigrationOptions {
    /// Create options for dry-run
    pub fn dry_run() -> Self {
        Self {
            dry_run: true,
            verbose: true,
            ..Default::default()
        }
    }

    /// Create options with backup
    pub fn with_backup(backup_dir: Option<PathBuf>) -> Self {
        Self {
            backup: true,
            backup_dir,
            rollback_on_failure: true,
            ..Default::default()
        }
    }
}

/// Type of change
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChangeType {
    /// File created
    Created,
    /// File modified
    Modified,
    /// File deleted
    Deleted,
    /// File renamed
    Renamed,
    /// File moved
    Moved,
    /// Permission changed
    Permission,
    /// Other change
    Other(String),
}

/// A single change made during migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    /// Type of change
    pub change_type: ChangeType,
    /// Path affected
    pub path: PathBuf,
    /// Old path (for rename/move)
    pub old_path: Option<PathBuf>,
    /// Description
    pub description: Option<String>,
}

impl Change {
    /// Create a new change
    pub fn new(change_type: ChangeType, path: impl Into<PathBuf>) -> Self {
        Self {
            change_type,
            path: path.into(),
            old_path: None,
            description: None,
        }
    }

    /// Create a rename change
    pub fn rename(old_path: impl Into<PathBuf>, new_path: impl Into<PathBuf>) -> Self {
        Self {
            change_type: ChangeType::Renamed,
            path: new_path.into(),
            old_path: Some(old_path.into()),
            description: None,
        }
    }

    /// Add description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Result of a single migration step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStepResult {
    /// Whether the migration succeeded
    pub success: bool,
    /// Changes made
    pub changes: Vec<Change>,
    /// Warnings generated
    pub warnings: Vec<String>,
    /// Duration of the migration
    #[serde(with = "duration_serde")]
    pub duration: Duration,
}

impl Default for MigrationStepResult {
    fn default() -> Self {
        Self {
            success: true,
            changes: Vec::new(),
            warnings: Vec::new(),
            duration: Duration::ZERO,
        }
    }
}

impl MigrationStepResult {
    /// Create a successful result
    pub fn success() -> Self {
        Self::default()
    }

    /// Create a skipped result
    pub fn skipped() -> Self {
        Self {
            success: true,
            changes: Vec::new(),
            warnings: vec!["Migration skipped".to_string()],
            duration: Duration::ZERO,
        }
    }

    /// Add a change
    pub fn with_change(mut self, change: Change) -> Self {
        self.changes.push(change);
        self
    }

    /// Add a warning
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// Set duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}

/// Overall migration report
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MigrationReport {
    /// Whether the overall migration succeeded
    pub success: bool,
    /// Number of successful migrations
    pub successful_count: usize,
    /// Number of skipped migrations
    pub skipped_count: usize,
    /// Number of failed migrations
    pub failed_count: usize,
    /// Individual step results
    pub steps: Vec<MigrationStepReport>,
    /// Total duration
    #[serde(with = "duration_serde")]
    pub total_duration: Duration,
    /// Errors encountered
    pub errors: Vec<String>,
}

/// Report for a single migration step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStepReport {
    /// Migration ID
    pub migration_id: String,
    /// Migration name
    pub migration_name: String,
    /// Description
    pub description: String,
    /// Result
    pub result: MigrationStepResult,
    /// Whether it was skipped
    pub skipped: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Serde helper for Duration
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}
