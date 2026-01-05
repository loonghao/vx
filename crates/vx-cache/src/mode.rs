use serde::{Deserialize, Serialize};

/// Cache refresh mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CacheMode {
    /// Use cache if valid, otherwise fetch (default)
    #[default]
    Normal,
    /// Force refresh, ignore cache
    Refresh,
    /// Use cache only, fail if not available (offline mode)
    Offline,
    /// Skip cache entirely (for CI or testing)
    NoCache,
}
