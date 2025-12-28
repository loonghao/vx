//! Built-in migrations for vx.

mod config_v1_to_v2;
mod file_rename;
mod version_detector;

pub use config_v1_to_v2::ConfigV1ToV2Migration;
pub use file_rename::FileRenameMigration;
pub use version_detector::VxVersionDetector;

use crate::engine::MigrationEngine;

/// Create a default migration engine with all built-in migrations
pub fn create_default_engine() -> MigrationEngine {
    MigrationEngine::new()
        .register(FileRenameMigration::new())
        .register(ConfigV1ToV2Migration::new())
}
