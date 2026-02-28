//! Zero-dependency static metadata parser for `provider.star` files.
//!
//! This crate extracts metadata from `provider.star` files **without executing
//! the Starlark engine**.  It reads static string/list literals from well-known
//! top-level variables (e.g. `name = "node"`) and function definitions
//! (e.g. `def name(): return "node"`), as well as the `runtimes` list.
//!
//! # Why a separate crate?
//!
//! Both `vx-manifest` and `vx-starlark` need to parse provider metadata.
//! Putting the parser here avoids a circular dependency:
//!
//! ```text
//!   vx-manifest  ──▶  vx-star-metadata  ◀──  vx-starlark
//! ```
//!
//! # Design
//!
//! The parser is intentionally simple: it only handles the subset of Starlark
//! that appears in vx provider files (string literals, list literals, dict
//! literals with string keys/values, and `runtime_def()` / `bundled_runtime_def()`
//! function calls).  Dynamic expressions are ignored.
//!
//! # Example
//!
//! ```rust
//! use vx_star_metadata::StarMetadata;
//!
//! const STAR: &str = r#"
//! name = "node"
//! description = "Node.js JavaScript runtime"
//! ecosystem = "nodejs"
//!
//! runtimes = [
//!     runtime_def("node", aliases = ["nodejs"]),
//!     bundled_runtime_def("npm", bundled_with = "node"),
//! ]
//! "#;
//!
//! let meta = StarMetadata::parse(STAR);
//! assert_eq!(meta.name, Some("node".to_string()));
//! assert_eq!(meta.runtimes.len(), 2);
//! assert_eq!(meta.runtimes[1].bundled_with, Some("node".to_string()));
//! ```

mod parser;

pub use parser::{StarMetadata, StarRuntimeMeta};
