//! VX Version Fetcher - Unified version fetching abstraction
//!
//! This crate provides a unified interface for fetching version information
//! from various data sources (jsDelivr CDN, npm registry, PyPI, etc.).
//!
//! # Architecture
//!
//! ```text
//! VersionFetcherBuilder
//!     ├── jsdelivr("owner", "repo")     -> JsDelivrFetcher
//!     ├── npm("package")                -> NpmFetcher
//!     ├── pypi("package")               -> PyPiFetcher
//!     ├── github_releases("owner","repo") -> GitHubReleasesFetcher
//!     └── custom_api("url", parser)     -> CustomApiFetcher
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use vx_version_fetcher::VersionFetcherBuilder;
//!
//! // Fetch versions from jsDelivr (GitHub proxy)
//! let versions = VersionFetcherBuilder::jsdelivr("helm", "helm")
//!     .strip_prefix("v")
//!     .skip_prereleases()
//!     .limit(50)
//!     .build()
//!     .fetch(ctx)
//!     .await?;
//!
//! // Fetch versions from npm registry
//! let versions = VersionFetcherBuilder::npm("pnpm")
//!     .skip_prereleases()
//!     .build()
//!     .fetch(ctx)
//!     .await?;
//! ```

pub mod builder;
pub mod error;
pub mod fetcher;
pub mod fetchers;
pub mod utils;

// Re-exports
pub use builder::VersionFetcherBuilder;
pub use error::{FetchError, FetchResult};
pub use fetcher::VersionFetcher;
pub use fetchers::{
    CustomApiFetcher, GitHubReleasesConfig, GitHubReleasesFetcher, JsDelivrConfig, JsDelivrFetcher,
    NpmConfig, NpmFetcher, PyPiConfig, PyPiFetcher,
};
pub use utils::version_utils;

// Re-export VersionInfo from vx-runtime for convenience
pub use vx_runtime::VersionInfo;
