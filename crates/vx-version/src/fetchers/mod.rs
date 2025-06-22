//! Version fetchers for different tools and sources

pub mod github;
pub mod go;
pub mod node;
pub mod turbo_cdn;

// Re-export all fetchers for convenience
pub use github::GitHubVersionFetcher;
pub use go::GoVersionFetcher;
pub use node::NodeVersionFetcher;
pub use turbo_cdn::TurboCdnVersionFetcher;