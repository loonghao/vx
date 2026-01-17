//! Built-in version fetcher implementations

mod custom;
pub mod github;
pub mod jsdelivr;
pub mod npm;
pub mod pypi;

pub use custom::CustomApiFetcher;
pub use github::{GitHubReleasesConfig, GitHubReleasesFetcher};
pub use jsdelivr::{JsDelivrConfig, JsDelivrFetcher};
pub use npm::{NpmConfig, NpmFetcher};
pub use pypi::{PyPiConfig, PyPiFetcher};
