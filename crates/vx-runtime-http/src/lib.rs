//! VX Runtime HTTP - Real HTTP client, download manager and installer implementations
//!
//! This crate provides the production ("heavy") implementations of the abstract traits
//! defined in `vx-runtime`. It contains:
//!
//! - `RealHttpClient`: HTTP client using reqwest with CDN acceleration, retry logic
//! - `RealInstaller`: Archive downloader and extractor (tar, zip, 7z, msi, pkg)
//! - `create_runtime_context()`: Factory function for production RuntimeContext
//!
//! # Architecture (RFC 0032)
//!
//! This crate is part of the functional-domain split of vx-runtime:
//! - `vx-runtime`: Lightweight interface layer (traits + types, ~10s compile)
//! - `vx-runtime-http`: HTTP/download/install implementations (this crate, ~25-35s compile)
//! - `vx-runtime-archive`: Archive extraction utilities (~30-40s compile)
//!
//! Only `vx-cli` needs to depend on this crate. Providers only need `vx-runtime`.

mod context;
mod http_client;
mod installer;

pub use context::{create_runtime_context, create_runtime_context_with_base};
pub use http_client::RealHttpClient;
pub use installer::RealInstaller;

// Re-export region utilities from vx-runtime (avoid code duplication)
pub use vx_runtime::region::{
    self, Region, detect_region, is_china_environment, is_ci_environment,
};
